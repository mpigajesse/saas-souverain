use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

use chrono::Utc;
use ss_crypto::Dek;
use uuid::Uuid;

use crate::{JournalEntry, JournalError};

/// Journal append-only chiffré.
///
/// Format fichier : chaque entrée = `u32 len (little-endian) ‖ blob chiffré`
/// Le blob est la sérialisation CBOR de `JournalEntry`, chiffrée avec `dek.encrypt()`.
pub struct Journal {
    path: PathBuf,
    dek: Dek,
    next_index: u64,
}

impl Journal {
    /// Ouvre (ou crée) un fichier de journal.
    /// Lit les frames existantes (sans les déchiffrer) pour déterminer le prochain index.
    pub fn open(path: impl Into<PathBuf>, dek: Dek) -> Result<Self, JournalError> {
        let path: PathBuf = path.into();

        // Compter les frames par leur entête (u32 len) sans déchiffrer.
        // Si le fichier n'existe pas encore, next_index = 0.
        let next_index = if path.exists() {
            Self::count_frames(&path)?
        } else {
            0
        };

        Ok(Self { path, dek, next_index })
    }

    /// Ajoute une entrée. Retourne son index.
    pub fn append(
        &mut self,
        epoch: u64,
        node_id: Uuid,
        op_type: &str,
        payload: Vec<u8>,
    ) -> Result<u64, JournalError> {
        let index = self.next_index;

        let entry = JournalEntry {
            index,
            epoch,
            node_id,
            written_at: Utc::now(),
            op_type: op_type.to_owned(),
            payload,
        };

        // Sérialiser en CBOR
        let mut cbor_buf = Vec::new();
        ciborium::ser::into_writer(&entry, &mut cbor_buf).map_err(|_| JournalError::Cbor)?;

        // Chiffrer avec la DEK
        let blob = self.dek.encrypt(&cbor_buf)?;

        // Écrire : u32 len (LE) ‖ blob
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        let mut writer = BufWriter::new(file);

        let len = blob.len() as u32;
        writer.write_all(&len.to_le_bytes())?;
        writer.write_all(&blob)?;
        writer.flush()?;

        self.next_index += 1;
        Ok(index)
    }

    /// Lit toutes les entrées dans l'ordre.
    pub fn read_all(&self) -> Result<Vec<JournalEntry>, JournalError> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }

        let file = std::fs::File::open(&self.path)?;
        let mut reader = BufReader::new(file);
        let mut entries = Vec::new();
        let mut frame_index: u64 = 0;

        loop {
            // Lire les 4 octets de longueur
            let mut len_buf = [0u8; 4];
            match reader.read_exact(&mut len_buf) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(JournalError::Io(e)),
            }

            let len = u32::from_le_bytes(len_buf) as usize;

            // Lire le blob chiffré
            let mut blob = vec![0u8; len];
            reader
                .read_exact(&mut blob)
                .map_err(|_| JournalError::Corrupted { index: frame_index })?;

            // Déchiffrer
            let plaintext = self.dek.decrypt(&blob)?;

            // Désérialiser le CBOR
            let entry: JournalEntry = ciborium::de::from_reader(plaintext.as_slice())
                .map_err(|_| JournalError::Corrupted { index: frame_index })?;

            entries.push(entry);
            frame_index += 1;
        }

        Ok(entries)
    }

    /// Nombre d'entrées actuellement dans le journal.
    pub fn len(&self) -> u64 {
        self.next_index
    }

    pub fn is_empty(&self) -> bool {
        self.next_index == 0
    }

    /// Compte les frames dans le fichier en lisant uniquement les entêtes de longueur.
    /// Ne déchiffre pas — permet d'ouvrir un journal même avec une DEK incorrecte
    /// (la vérification d'intégrité se produit à la lecture via `read_all`).
    fn count_frames(path: &Path) -> Result<u64, JournalError> {
        let file = std::fs::File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut count: u64 = 0;

        loop {
            let mut len_buf = [0u8; 4];
            match reader.read_exact(&mut len_buf) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(JournalError::Io(e)),
            }

            let len = u32::from_le_bytes(len_buf) as usize;

            // Sauter le blob (sans le lire en mémoire entière si on peut)
            // On utilise Read::by_ref pour consommer exactement `len` octets.
            let mut blob = vec![0u8; len];
            reader
                .read_exact(&mut blob)
                .map_err(|_| JournalError::Corrupted { index: count })?;

            count += 1;
        }

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ss_crypto::Dek;
    use tempfile::NamedTempFile;
    use uuid::Uuid;

    fn tmp_journal() -> (Journal, NamedTempFile) {
        let f = NamedTempFile::new().unwrap();
        let dek = Dek::generate();
        let j = Journal::open(f.path(), dek).unwrap();
        (j, f)
    }

    #[test]
    fn append_and_read() {
        let (mut j, _f) = tmp_journal();
        let id = Uuid::new_v4();
        let idx = j.append(1, id, "test.op", b"payload".to_vec()).unwrap();
        assert_eq!(idx, 0);
        let entries = j.read_all().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].op_type, "test.op");
        assert_eq!(entries[0].payload, b"payload");
    }

    #[test]
    fn indices_monotone() {
        let (mut j, _f) = tmp_journal();
        let id = Uuid::new_v4();
        for i in 0..5 {
            let idx = j.append(1, id, "op", vec![i]).unwrap();
            assert_eq!(idx, i as u64);
        }
        assert_eq!(j.len(), 5);
    }

    #[test]
    fn reopen_restores_state() {
        let f = NamedTempFile::new().unwrap();
        let dek = Dek::generate();
        {
            let mut j = Journal::open(f.path(), dek.clone()).unwrap();
            j.append(1, Uuid::new_v4(), "op.a", b"data-a".to_vec()).unwrap();
            j.append(1, Uuid::new_v4(), "op.b", b"data-b".to_vec()).unwrap();
        }
        // Réouvrir
        let j2 = Journal::open(f.path(), dek).unwrap();
        let entries = j2.read_all().unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[1].op_type, "op.b");
    }

    #[test]
    fn wrong_dek_fails() {
        let f = NamedTempFile::new().unwrap();
        let dek1 = Dek::generate();
        let dek2 = Dek::generate();
        {
            let mut j = Journal::open(f.path(), dek1).unwrap();
            j.append(1, Uuid::new_v4(), "op", b"secret".to_vec()).unwrap();
        }
        let j2 = Journal::open(f.path(), dek2).unwrap();
        assert!(j2.read_all().is_err());
    }
}
