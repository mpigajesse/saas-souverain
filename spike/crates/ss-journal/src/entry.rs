use std::fmt;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Une opération métier sérialisée dans le journal immuable.
#[derive(Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    pub index: u64,
    pub epoch: u64,
    pub node_id: Uuid,
    pub written_at: DateTime<Utc>,
    pub op_type: String,
    pub payload: Vec<u8>,
}

/// Audit d'herméticité : Implémentation masquée du trait Debug.
impl fmt::Debug for JournalEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JournalEntry")
            .field("index", &self.index)
            .field("epoch", &self.epoch)
            .field("node_id", &self.node_id)
            .field("written_at", &self.written_at)
            .field("op_type", &self.op_type)
            .field("payload", &format_args!("<{} bytes>", self.payload.len()))
            .finish()
    }
}