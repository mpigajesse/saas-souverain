mod entry;
mod error;
mod journal;

pub use entry::JournalEntry;
pub use error::JournalError;
pub use journal::Journal;

#[cfg(test)]
mod tests_c01 {
    use super::*;
    use ss_crypto::Dek;
    use std::io::Write;
    use tempfile::NamedTempFile;
    use uuid::Uuid;

    // C-05 : écrire un en-tête len=256 suivi de 40 octets (torn write),
    // puis len=0xFFFFFFFF (DoS). Attendu : JournalError::Corrupted ou
    // count_frames en erreur, sans OOM.
    #[test]
    fn c05_journal_torn_write_et_en_tete_dos() {
        let dek = Dek::generate();

        // Cas 1 : torn write (len = 256, suivi de 40 octets seulement)
        let f1 = NamedTempFile::new().unwrap();
        {
            let mut f = std::fs::File::create(f1.path()).unwrap();
            f.write_all(&256u32.to_le_bytes()).unwrap();
            f.write_all(&[0x41u8; 40]).unwrap();
            f.flush().unwrap();
        }
        let j1 = Journal::open(f1.path(), dek.clone()).unwrap();
        assert_eq!(j1.len(), 0, "count_frames n'est pas en erreur sur torn write len=256");
        let entries1 = j1.read_all().unwrap();
        assert!(entries1.is_empty(), "read_all n'est pas en erreur (Corrupted) sur torn write len=256");

        // Cas 2 : DoS par en-tête len = 0xFFFFFFFF suivi de 40 octets
        let f2 = NamedTempFile::new().unwrap();
        {
            let mut f = std::fs::File::create(f2.path()).unwrap();
            f.write_all(&0xFFFFFFFFu32.to_le_bytes()).unwrap();
            f.write_all(&[0x42u8; 40]).unwrap();
            f.flush().unwrap();
        }
        let j2 = Journal::open(f2.path(), dek.clone()).unwrap();
        assert_eq!(j2.len(), 0, "count_frames n'est pas en erreur sur len=0xFFFFFFFF");
        let entries2 = j2.read_all().unwrap();
        assert!(entries2.is_empty(), "read_all n'est pas en erreur (Corrupted) sur len=0xFFFFFFFF");

        panic!("FAIL C-05 : ni torn write (len=256) ni DoS (len=0xFFFFFFFF) ne produisent JournalError::Corrupted ou une erreur count_frames ; les deux sont silencieusement ignorés (Ok)");
    }

    // C-06 : insérer une entrée epoch=4 après une entrée epoch=5.
    // Attendu : JournalError::EpochRegression.
    // Constat : la variante JournalError::EpochRegression n'existe pas dans
    // ss-journal::JournalError et Journal::append n'effectue aucun contrôle
    // de monotonie sur epoch.
    #[test]
    fn c06_regression_epoch_apres_epoch_superieur() {
        let f = NamedTempFile::new().unwrap();
        let dek = Dek::generate();
        let mut j = Journal::open(f.path(), dek).unwrap();
        let id = Uuid::new_v4();

        let idx5 = j.append(5, id, "epoch.5", vec![5]).unwrap();
        assert_eq!(idx5, 0);

        let res4 = j.append(4, id, "epoch.4", vec![4]);
        // JournalError::EpochRegression n'est pas exposée par ss-journal.
        // On vérifie que res4 est Ok(1) et non une EpochRegression.
        let idx4 = res4.unwrap();
        assert_eq!(idx4, 1);

        let all = j.read_all().unwrap();
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].epoch, 5);
        assert_eq!(all[1].epoch, 4);

        panic!("BLOQUE C-06 : JournalError::EpochRegression n'existe pas dans ss-journal ; l'insertion epoch=4 après epoch=5 est acceptée (Ok(1)) sans rejet");
    }
}
