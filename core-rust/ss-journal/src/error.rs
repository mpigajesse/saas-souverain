use thiserror::Error;

#[derive(Debug, Error)]
pub enum JournalError {
    #[error("IO : {0}")]
    Io(#[from] std::io::Error),
    #[error("déchiffrement : {0}")]
    Crypto(#[from] ss_crypto::CryptoError),
    #[error("désérialisation CBOR")]
    Cbor,
    #[error("journal corrompu à l'index {index}")]
    Corrupted { index: u64 },
}
