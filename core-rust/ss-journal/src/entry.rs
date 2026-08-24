use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Une opération métier sérialisée dans le journal.
/// `payload` est du JSON arbitraire (bytes CBOR dans le fichier).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    pub index: u64,             // position dans le journal (0-based, monotone)
    pub epoch: u64,             // jeton d'époque du nœud actif qui a écrit
    pub node_id: Uuid,          // UUID du nœud auteur
    pub written_at: DateTime<Utc>,
    pub op_type: String,        // ex. "stock.add", "facture.create"
    pub payload: Vec<u8>,       // données métier sérialisées (format opaque pour ss-journal)
}
