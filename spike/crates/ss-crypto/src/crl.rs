use std::collections::HashSet;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use crate::{AccessPublicKey, CryptoError};

/// Registre de Révocation des Clés d'Accès (Certificate Revocation List).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CrlRegistry {
    sequence: u64,
    revoked_keys: HashSet<[u8; 32]>,
    updated_at: DateTime<Utc>,
}

impl CrlRegistry {
    pub fn new() -> Self {
        Self {
            sequence: 0,
            revoked_keys: HashSet::new(),
            updated_at: Utc::now(),
        }
    }

    pub fn revoke(&mut self, public_key: &AccessPublicKey) -> bool {
        let bytes = *public_key.as_bytes();
        if self.revoked_keys.insert(bytes) {
            self.sequence += 1;
            self.updated_at = Utc::now();
            true
        } else {
            false
        }
    }

    pub fn is_revoked(&self, public_key: &AccessPublicKey) -> bool {
        self.revoked_keys.contains(public_key.as_bytes())
    }

    pub fn verify_access(&self, public_key: &AccessPublicKey) -> Result<(), CryptoError> {
        if self.is_revoked(public_key) {
            Err(CryptoError::RevokedAccessKey)
        } else {
            Ok(())
        }
    }

    pub fn sequence(&self) -> u64 {
        self.sequence
    }

    pub fn len(&self) -> usize {
        self.revoked_keys.len()
    }

    pub fn is_empty(&self) -> bool {
        self.revoked_keys.is_empty()
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

impl Default for CrlRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AccessKeyPair;

    #[test]
    fn test_crl_revocation_flow() {
        let mut crl = CrlRegistry::new();
        let node_a = AccessKeyPair::generate();
        let node_b = AccessKeyPair::generate();

        assert!(!crl.is_revoked(&node_a.public));
        assert!(crl.revoke(&node_a.public));
        assert!(crl.is_revoked(&node_a.public));
        assert!(!crl.is_revoked(&node_b.public));
    }
}