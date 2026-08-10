use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
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

        let next_index = if path.exists() {
            Self::count_frames(&path)?
        } else {
            0
        };

        Ok(Self { path, dek, next_index })
    }

    /// Ajoute une entrée. Retourne son index séquentiel.
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

    /// Lit toutes les entrées dans l'ordre chronologique.
    pub fn read_all(&self) -> Result<Vec<JournalEntry>, JournalError> {
        self.read_range(0, usize::MAX)
    }

    /// Lit une plage d'entrées à partir de `start_index` avec une limite optionnelle.
    pub fn read_range(&self, start_index: u64, limit: usize) -> Result<Vec<JournalEntry>, JournalError> {
        if !self.path.exists() || limit == 0 {
            return Ok(Vec::new());
        }

        let file = std::fs::File::open(&self.path)?;
        let mut reader = BufReader::new(file);
        let mut entries = Vec::new();
        let mut frame_index: u64 = 0;

        loop {
            if entries.len() >= limit {
                break;
            }

            let mut len_buf = [0u8; 4];
            match reader.read_exact(&mut len_buf) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(JournalError::Io(e)),
            }

            let len = u32::from_le_bytes(len_buf) as usize;

            if frame_index < start_index {
                // Sauter la frame sans lire tout en mémoire si on cherche un index ultérieur
                reader.seek(SeekFrom::Current(len as i64))?;
                frame_index += 1;
                continue;
            }

            let mut blob = vec![0u8; len];
            reader
                .read_exact(&mut blob)
                .map_err(|_| JournalError::Corrupted { index: frame_index })?;

            let plaintext = self.dek.decrypt(&blob)?;

            let entry: JournalEntry = ciborium::de::from_reader(plaintext.as_slice())
                .map_err(|_| JournalError::Corrupted { index: frame_index })?;

            entries.push(entry);
            frame_index += 1;
        }

        Ok(entries)
    }

    /// Nombre d'entrées actuellement enregistrées dans le journal.
    pub fn len(&self) -> u64 {
        self.next_index
    }

    pub fn is_empty(&self) -> bool {
        self.next_index == 0
    }

    /// Compte les frames dans le fichier en lisant uniquement les en-têtes de longueur (4 octets).
    /// Ne déchiffre pas les payloads — gain de performance majeur lors de l'ouverture du journal.
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

            let len = u32::from_le_bytes(len_buf) as i64;
            reader.seek(SeekFrom::Current(len))?;
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
    fn read_range_pagination() {
        let (mut j, _f) = tmp_journal();
        let id = Uuid::new_v4();
        for i in 0..10 {
            j.append(1, id, "op", vec![i as u8]).unwrap();
        }
        let page = j.read_range(3, 4).unwrap();
        assert_eq!(page.len(), 4);
        assert_eq!(page[0].index, 3);
        assert_eq!(page[3].index, 6);
    }

    #[test]
    fn debug_hermeticity() {
        let entry = JournalEntry {
            index: 0,
            epoch: 1,
            node_id: Uuid::new_v4(),
            written_at: Utc::now(),
            op_type: "stock.update".into(),
            payload: vec![1, 2, 3, 4, 5],
        };
        let debug_str = format!("{:?}", entry);
        assert!(debug_str.contains("<5 bytes>"));
        assert!(!debug_str.contains("[1, 2, 3, 4, 5]"));
    }
}