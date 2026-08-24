use thiserror::Error;

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("chiffrement échoué")]
    EncryptionFailed,
    #[error("déchiffrement échoué — données corrompues ou mauvaise clé")]
    DecryptionFailed,
    #[error("ciphertext trop court")]
    InvalidCiphertext,
    #[error("clé publique invalide")]
    InvalidPublicKey,
    #[error("sealed box invalide")]
    InvalidSealedBox,
    #[error("erreur Argon2 : {0}")]
    Argon2(String),
}
