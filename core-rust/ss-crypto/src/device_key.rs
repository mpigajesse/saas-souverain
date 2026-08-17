use x25519_dalek::{EphemeralSecret, PublicKey as X25519Public, StaticSecret};
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    XChaCha20Poly1305, Key, XNonce,
};
use blake2::{Blake2b512, Digest};
use rand::rngs::OsRng;
use zeroize::{Zeroize, ZeroizeOnDrop};
use serde::{Serialize, Deserialize};
use crate::{Dek, CryptoError};

const EPH_PUB_LEN: usize = 32;
/// Taille du tag Poly1305 (16 octets) + données (32 octets DEK)
const SEALED_MIN_LEN: usize = EPH_PUB_LEN + 32 + 16;

/// Clé publique d'un appareil (X25519).
#[derive(Clone, Debug, Serialize, Deserialize, Zeroize)]
pub struct DevicePublicKey(pub [u8; 32]);

impl DevicePublicKey {
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Scelle la DEK pour cet appareil. L'expéditeur reste anonyme.
    ///
    /// Algorithme sealed box :
    /// 1. Génère une paire éphémère (eph_secret, eph_public)
    /// 2. ECDH : shared = X25519(eph_secret, recipient_public)
    /// 3. Dérive nonce et clé via BLAKE2b-512 :
    ///    digest = BLAKE2b-512(shared ‖ eph_public ‖ recipient_public)
    ///    nonce  = digest[0..24]
    ///    key    = digest[32..64]
    /// 4. Chiffre la DEK (32 octets) avec XChaCha20-Poly1305(key, nonce)
    /// 5. Retourne : eph_public (32 octets) ‖ ciphertext
    pub fn seal_dek(&self, dek: &Dek) -> Result<Vec<u8>, CryptoError> {
        // Étape 1 : paire éphémère
        let eph_secret = EphemeralSecret::random_from_rng(OsRng);
        let eph_public = X25519Public::from(&eph_secret);

        // Étape 2 : ECDH
        let recipient_public = X25519Public::from(self.0);
        let shared = eph_secret.diffie_hellman(&recipient_public);

        // Étape 3 : dérivation BLAKE2b-512
        let mut hasher = Blake2b512::new();
        hasher.update(shared.as_bytes());
        hasher.update(eph_public.as_bytes());
        hasher.update(&self.0);
        let digest = hasher.finalize();

        let nonce = XNonce::from_slice(&digest[0..24]);
        let key = Key::from_slice(&digest[32..64]);

        // Étape 4 : chiffrement
        let cipher = XChaCha20Poly1305::new(key);
        let ciphertext = cipher
            .encrypt(nonce, dek.as_bytes().as_slice())
            .map_err(|_| CryptoError::EncryptionFailed)?;

        // Étape 5 : eph_public ‖ ciphertext
        let mut output = Vec::with_capacity(EPH_PUB_LEN + ciphertext.len());
        output.extend_from_slice(eph_public.as_bytes());
        output.extend_from_slice(&ciphertext);
        Ok(output)
    }
}

/// Paire de clés X25519 d'un appareil. La clé secrète est zéroïsée à la libération.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct DeviceKeyPair {
    secret: [u8; 32],
    pub public: DevicePublicKey,
}

impl DeviceKeyPair {
    /// Génère une nouvelle paire (premier lancement de l'appareil).
    pub fn generate() -> Self {
        let secret = StaticSecret::random_from_rng(OsRng);
        let public = X25519Public::from(&secret);
        Self {
            secret: secret.to_bytes(),
            public: DevicePublicKey(public.to_bytes()),
        }
    }

    /// Recrée depuis octets (lecture depuis le fichier de config chiffré).
    pub fn from_secret_bytes(bytes: [u8; 32]) -> Self {
        let secret = StaticSecret::from(bytes);
        let public = X25519Public::from(&secret);
        Self {
            secret: bytes,
            public: DevicePublicKey(public.to_bytes()),
        }
    }

    pub fn secret_bytes(&self) -> [u8; 32] {
        self.secret
    }

    /// Ouvre un sealed box produit par `DevicePublicKey::seal_dek`.
    pub fn open_sealed_dek(&self, sealed: &[u8]) -> Result<Dek, CryptoError> {
        if sealed.len() < SEALED_MIN_LEN {
            return Err(CryptoError::InvalidSealedBox);
        }

        // Extraire la clé publique éphémère (32 premiers octets)
        let eph_pub_bytes: [u8; 32] = sealed[..EPH_PUB_LEN]
            .try_into()
            .map_err(|_| CryptoError::InvalidSealedBox)?;
        let eph_public = X25519Public::from(eph_pub_bytes);
        let ciphertext = &sealed[EPH_PUB_LEN..];

        // Reconstituer la même clé de déchiffrement
        let my_secret = StaticSecret::from(self.secret);
        let shared = my_secret.diffie_hellman(&eph_public);

        let mut hasher = Blake2b512::new();
        hasher.update(shared.as_bytes());
        hasher.update(eph_public.as_bytes());
        hasher.update(&self.public.0);
        let digest = hasher.finalize();

        let nonce = XNonce::from_slice(&digest[0..24]);
        let key = Key::from_slice(&digest[32..64]);

        // Déchiffrer
        let cipher = XChaCha20Poly1305::new(key);
        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| CryptoError::DecryptionFailed)?;

        if plaintext.len() != 32 {
            return Err(CryptoError::DecryptionFailed);
        }
        let mut dek_bytes = [0u8; 32];
        dek_bytes.copy_from_slice(&plaintext);
        Ok(Dek::from_bytes(dek_bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seal_and_open() {
        let keypair = DeviceKeyPair::generate();
        let dek = Dek::generate();
        let sealed = keypair.public.seal_dek(&dek).unwrap();
        let recovered = keypair.open_sealed_dek(&sealed).unwrap();
        assert_eq!(dek.as_bytes(), recovered.as_bytes());
    }

    #[test]
    fn wrong_key_cannot_open() {
        let keypair1 = DeviceKeyPair::generate();
        let keypair2 = DeviceKeyPair::generate();
        let dek = Dek::generate();
        let sealed = keypair1.public.seal_dek(&dek).unwrap();
        assert!(keypair2.open_sealed_dek(&sealed).is_err());
    }

    #[test]
    fn two_seals_differ() {
        let keypair = DeviceKeyPair::generate();
        let dek = Dek::generate();
        let s1 = keypair.public.seal_dek(&dek).unwrap();
        let s2 = keypair.public.seal_dek(&dek).unwrap();
        assert_ne!(s1, s2); // paires éphémères différentes
    }
}
