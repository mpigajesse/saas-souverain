use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConsensusError {
    #[error("nœud fencé : époque {claimed} < époque courante {current}")]
    Fenced { claimed: u64, current: u64 },
    #[error("quorum insuffisant : {have}/{need} nœuds")]
    InsufficientQuorum { have: usize, need: usize },
}
