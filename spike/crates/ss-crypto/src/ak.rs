use std::fmt;
use x25519_dalek::{PublicKey as X25519Public, StaticSecret};
use blake2::{Blake2b512, Digest};
use rand::rngs::OsRng;
use zeroize::{Zeroize, ZeroizeOnDrop};
use serde::{Serialize, Deserialize};

/// Clé publique d'accès d'un appareil (256 bits / 32 octets X25519).
/// Sérialisable pour l'annuaire d'enrôlement et le registre de révocation (CRL).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Zeroize)]
pub struct AccessPublicKey(pub [u8; 32]);

impl AccessPublicKey {
    /// Recrée une clé publique d'accès depuis un tableau de 32 octets.
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Extrait les 32 octets bruts de la clé publique d'accès.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// Paire de clés d'accès X25519 d'un appareil (AK).
/// La clé secrète est automatiquement zéroïsée en mémoire RAM lors de la libération (ZeroizeOnDrop).
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct AccessKeyPair {
    secret_bytes: [u8; 32],
    pub public: AccessPublicKey,
}

/// Audit d'herméticité : Implémentation masquée du trait Debug.
/// Empêche la fuite de la clé secrète d'accès dans les logs ou les traces.
impl fmt::Debug for AccessKeyPair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AccessKeyPair([REDACTED])")
    }
}

impl AccessKeyPair {
    /// Génère une nouvelle paire de clés d'accès AK (X25519) via OsRng.
    pub fn generate() -> Self {
        let secret = StaticSecret::random_from_rng(OsRng);
        let public = X25519Public::from(&secret);
        Self {
            secret_bytes: secret.to_bytes(),
            public: AccessPublicKey(public.to_bytes()),
        }
    }

    /// Reconstruit une paire AK depuis les octets de la clé secrète.
    pub fn from_secret_bytes(bytes: [u8; 32]) -> Self {
        let secret = StaticSecret::from(bytes);
        let public = X25519Public::from(&secret);
        Self {
            secret_bytes: bytes,
            public: AccessPublicKey(public.to_bytes()),
        }
    }

    /// Retourne la clé secrète d'accès sous forme d'octets.
    pub fn secret_bytes(&self) -> [u8; 32] {
        self.secret_bytes
    }

    /// Calcule un secret dérivé Diffie-Hellman (ECDH + BLAKE2b-512 KDF) avec la clé publique d'un autre nœud.
    /// Applique un hachage KDF (BLAKE2b-512) sur la sortie X25519 brute et les clés publiques pour garantir une entropie uniforme.
    pub fn diffie_hellman(&self, peer_public: &AccessPublicKey) -> [u8; 32] {
        let my_secret = StaticSecret::from(self.secret_bytes);
        let peer_pub = X25519Public::from(peer_public.0);
        let shared = my_secret.diffie_hellman(&peer_pub);

        // Dérivation KDF via BLAKE2b-512
        let mut hasher = Blake2b512::new();
        hasher.update(shared.as_bytes());
        hasher.update(self.public.as_bytes());
        hasher.update(peer_public.as_bytes());
        let digest = hasher.finalize();

        let mut derived_key = [0u8; 32];
        derived_key.copy_from_slice(&digest[0..32]);
        derived_key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_access_keypair_generation() {
        let kp = AccessKeyPair::generate();
        assert_ne!(kp.public.as_bytes(), &[0u8; 32]);
    }

    #[test]
    fn test_ecdh_kdf_shared_secret() {
        let node_a = AccessKeyPair::generate();
        let node_b = AccessKeyPair::generate();

        // Calcul ECDH dérivé KDF symétrique
        let shared_a = node_a.diffie_hellman(&node_b.public);
        let shared_b = node_b.diffie_hellman(&node_a.public);

        assert_eq!(shared_a, shared_b);
        assert_ne!(shared_a, [0u8; 32]);
    }

    #[test]
    fn test_debug_hermeticity() {
        let kp = AccessKeyPair::generate();
        let debug_output = format!("{:?}", kp);
        assert_eq!(debug_output, "AccessKeyPair([REDACTED])");
    }
}