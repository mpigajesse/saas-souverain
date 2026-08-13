use rusqlite::{params, Connection, Result as SqlResult};
use std::path::Path;
use std::sync::{Arc, Mutex};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OfflineError {
    #[error("Erreur de base SQLite locale : {0}")]
    DatabaseError(#[from] rusqlite::Error),

    #[error("Document non trouvé dans le cache local pour la collection '{collection}', id '{id}'")]
    NotFound { collection: String, id: String },

    #[error("Mode Lecture Seule Actif : Écritures bloquées en mode hors-ligne")]
    ReadOnlyViolation,

    #[error("Erreur de connexion gRPC : {0}")]
    GrpcFailure(String),
}

/// Mode de fonctionnement dynamique du client
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationMode {
    /// Connexion gRPC active : Lectures et écritures autorisées
    NominalReadWrite,
    /// Dégradation gracieuse : Réseau/Quorum indisponible, écritures bloquées, lectures sur cache local
    GracefulReadOnly,
}

/// Gestionnaire du cache local SQLite et du mode hors-ligne
pub struct OfflineCacheManager {
    conn: Arc<Mutex<Connection>>,
    mode: Arc<Mutex<OperationMode>>,
}

impl OfflineCacheManager {
    /// Initialise la base SQLite locale de cache (en fichier ou en mémoire :memory:)
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self, OfflineError> {
        let conn = Connection::open(db_path)?;
        
        // Initialisation de la table de cache si elle n'existe pas
        conn.execute(
            "CREATE TABLE IF NOT EXISTS offline_cache (
                collection TEXT NOT NULL,
                record_id TEXT NOT NULL,
                encrypted_payload BLOB NOT NULL,
                updated_at INTEGER NOT NULL,
                PRIMARY KEY (collection, record_id)
            )",
            [],
        )?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            mode: Arc::new(Mutex::new(OperationMode::NominalReadWrite)),
        })
    }

    /// Récupère le mode de fonctionnement actuel
    pub fn get_mode(&self) -> OperationMode {
        *self.mode.lock().unwrap()
    }

    /// Bascule manuelle ou automatique du mode opératoire
    pub fn set_mode(&self, new_mode: OperationMode) {
        let mut mode_lock = self.mode.lock().unwrap();
        *mode_lock = new_mode;
    }

    /// Intercepte un statut gRPC et bascule en GracefulReadOnly en cas de panne réseau
    pub fn handle_grpc_result<T>(&self, result: Result<T, tonic::Status>) -> Result<T, OfflineError> {
        match result {
            Ok(val) => {
                // Tout va bien, on s'assure qu'on est en mode nominal
                self.set_mode(OperationMode::NominalReadWrite);
                Ok(val)
            }
            Err(status) => {
                // Si la défaillance est réseau (Unavailable, Unknown, DeadlineExceeded)
                match status.code() {
                    tonic::Code::Unavailable | tonic::Code::Unknown | tonic::Code::DeadlineExceeded => {
                        self.set_mode(OperationMode::GracefulReadOnly);
                        Err(OfflineError::GrpcFailure(format!(
                            "Bascule en Mode Read-Only (Réseau Indisponible : {})",
                            status.message()
                        )))
                    }
                    _ => Err(OfflineError::GrpcFailure(status.message().into())),
                }
            }
        }
    }

    /// Sauvegarde ou met à jour un enregistrement chiffré dans le cache local
    pub fn save_record(&self, collection: &str, record_id: &str, encrypted_payload: &[u8]) -> Result<(), OfflineError> {
        let mode = self.get_mode();
        if mode == OperationMode::GracefulReadOnly {
            return Err(OfflineError::ReadOnlyViolation);
        }

        let conn = self.conn.lock().unwrap();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        conn.execute(
            "INSERT INTO offline_cache (collection, record_id, encrypted_payload, updated_at)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(collection, record_id) DO UPDATE SET
                encrypted_payload = excluded.encrypted_payload,
                updated_at = excluded.updated_at",
            params![collection, record_id, encrypted_payload, now],
        )?;

        Ok(())
    }

    /// Lit un enregistrement depuis le cache local (Disponible aussi bien en mode Nominal qu'en Read-Only)
    pub fn read_record(&self, collection: &str, record_id: &str) -> Result<Vec<u8>, OfflineError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT encrypted_payload FROM offline_cache WHERE collection = ?1 AND record_id = ?2",
        )?;

        let mut rows = stmt.query(params![collection, record_id])?;

        if let Some(row) = rows.next()? {
            let payload: Vec<u8> = row.get(0)?;
            Ok(payload)
        } else {
            Err(OfflineError::NotFound {
                collection: collection.into(),
                id: record_id.into(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offline_cache_crud() {
        let manager = OfflineCacheManager::new(":memory:").unwrap();
        
        let payload = vec![0xDE, 0xAD, 0xBE, 0xEF];
        assert!(manager.save_record("stocks", "PROD-123", &payload).is_ok());

        let read_back = manager.read_record("stocks", "PROD-123").unwrap();
        assert_eq!(read_back, payload);
    }

    #[test]
    fn test_graceful_read_only_switch() {
        let manager = OfflineCacheManager::new(":memory:").unwrap();
        let payload = vec![1, 2, 3, 4];
        manager.save_record("invoices", "INV-001", &payload).unwrap();

        // Simulation d'une panne réseau gRPC
        let grpc_err = Err(tonic::Status::unavailable("Connexion gRPC coupée"));
        let res: Result<(), _> = manager.handle_grpc_result(grpc_err);
        assert!(res.is_err());

        // Vérification que le mode est passé en GracefulReadOnly
        assert_eq!(manager.get_mode(), OperationMode::GracefulReadOnly);

        // En mode ReadOnly, la lecture du cache FONCTIONNE
        assert_eq!(manager.read_record("invoices", "INV-001").unwrap(), payload);

        // Mais les nouvelles écritures sont BLOQUÉES
        let write_res = manager.save_record("invoices", "INV-002", &payload);
        assert!(matches!(write_res, Err(OfflineError::ReadOnlyViolation)));
    }
}
