use std::fmt;
use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    XChaCha20Poly1305, Key, XNonce,
};
use zeroize::{Zeroize, ZeroizeOnDrop};
use crate::CryptoError;

const NONCE_LEN: usize = 24; // Longueur du nonce XChaCha20 en octets

/// Clé symétrique 256 bits dédiée au chiffrement des données.
/// Zéroïsée automatiquement en mémoire RAM à la libération du scope (ZeroizeOnDrop).
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct Dek([u8; 32]);

/// Audit d'herméticité : Implémentation masquée du trait Debug.
/// Empêche toute fuite accidentelle de la clé dans les logs ou les traces d'erreurs.
impl fmt::Debug for Dek {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Dek([REDACTED])")
    }
}

impl Dek {
    /// Génère une nouvelle clé DEK de 256 bits de manière aléatoire et sécurisée via OsRng.
    pub fn generate() -> Self {
        let key = XChaCha20Poly1305::generate_key(&mut OsRng);
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&key);
        Self(bytes)
    }

    /// Recrée une DEK à partir d'un tableau d'octets (ex: après ouverture d'un Sealed Box ou KEK).
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Accès aux 32 octets de la clé.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Chiffre `plaintext` avec l'algorithme XChaCha20-Poly1305.
    /// Retourne un vecteur binaire formaté ainsi : `[nonce (24 octets) ‖ ciphertext]`
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let cipher = XChaCha20Poly1305::new(Key::from_slice(&self.0));
        let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
        let ciphertext = cipher
            .encrypt(&nonce, plaintext)
            .map_err(|_| CryptoError::EncryptionFailed)?;

        let mut output = Vec::with_capacity(NONCE_LEN + ciphertext.len());
        output.extend_from_slice(&nonce);
        output.extend_from_slice(&ciphertext);
        Ok(output)
    }

    /// Déchiffre un blob binaire produit par `encrypt`.
    /// Sépare les 24 premiers octets (nonce) du reste du ciphertext avant de déchiffrer.
    pub fn decrypt(&self, blob: &[u8]) -> Result<Vec<u8>, CryptoError> {
        if blob.len() < NONCE_LEN {
            return Err(CryptoError::InvalidCiphertext);
        }
        let (nonce_bytes, ciphertext) = blob.split_at(NONCE_LEN);
        let nonce = XNonce::from_slice(nonce_bytes);
        let cipher = XChaCha20Poly1305::new(Key::from_slice(&self.0));
        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| CryptoError::DecryptionFailed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_empty() {
        let dek = Dek::generate();
        let ct = dek.encrypt(b"").unwrap();
        let pt = dek.decrypt(&ct).unwrap();
        assert_eq!(pt, b"");
    }

    #[test]
    fn roundtrip_data() {
        let dek = Dek::generate();
        let msg = b"donnees-metier-sensibles";
        let ct = dek.encrypt(msg).unwrap();
        assert_ne!(ct, msg as &[u8]);
        let pt = dek.decrypt(&ct).unwrap();
        assert_eq!(pt, msg);
    }

    #[test]
    fn wrong_key_fails() {
        let dek1 = Dek::generate();
        let dek2 = Dek::generate();
        let ct = dek1.encrypt(b"secret").unwrap();
        assert!(dek2.decrypt(&ct).is_err());
    }

    #[test]
    fn tampered_ciphertext_fails() {
        let dek = Dek::generate();
        let mut ct = dek.encrypt(b"secret").unwrap();
        ct[30] ^= 0xFF; // corrompre 1 bit
        assert!(dek.decrypt(&ct).is_err());
    }

    #[test]
    fn two_encryptions_differ() {
        let dek = Dek::generate();
        let ct1 = dek.encrypt(b"hello").unwrap();
        let ct2 = dek.encrypt(b"hello").unwrap();
        assert_ne!(ct1, ct2);
    }

    #[test]
    fn debug_hermeticity() {
        let dek = Dek::generate();
        let output = format!("{:?}", dek);
        assert_eq!(output, "Dek([REDACTED])");
    }
}