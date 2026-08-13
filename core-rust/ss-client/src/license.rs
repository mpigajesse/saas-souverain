use ed25519_dalek::{Signature, VerifyingKey, Verifier};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LicenseError {
    #[error("Signature cryptographique de la licence invalide ou corrompue")]
    InvalidSignature,

    #[error("Licence expirée depuis l'horodatage {expired_at} (actuel : {current})")]
    Expired { expired_at: u64, current: u64 },

    #[error("Nombre maximal de postes autorisés dépassé : max ({max}), réclamé ({claimed})")]
    NodeLimitExceeded { max: u32, claimed: u32 },

    #[error("Erreur de sérialisation JSON du jeton : {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Clé publique Éditeur invalide : {0}")]
    InvalidPublicKey(String),
}

/// Contenu du jeton de licence cryptographique émis par le SaaS Éditeur
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LicensePayload {
    pub tenant_id: String,
    pub installation_id: String,
    pub expires_at_epoch_sec: u64,
    pub max_nodes: u32,
    pub plan_tier: String, // ex: "ENTERPRISE", "STANDARD"
}

/// Structure complète d'un jeton de licence signé (Blind Relay)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedLicenseToken {
    pub payload_bytes: Vec<u8>,
    pub signature_bytes: Vec<u8>, // 64 octets Ed25519
}

pub struct LicenseValidator {
    publisher_public_key: VerifyingKey,
}

impl LicenseValidator {
    /// Initialise le vérificateur avec la clé publique Ed25519 de l'éditeur SaaS
    pub fn new(publisher_public_key_bytes: &[u8; 32]) -> Result<Self, LicenseError> {
        let key = VerifyingKey::from_bytes(publisher_public_key_bytes)
            .map_err(|e| LicenseError::InvalidPublicKey(e.to_string()))?;
        Ok(Self {
            publisher_public_key: key,
        })
    }

    /// Valide de façon 100% autonome et hors-ligne un jeton de licence signé
    pub fn validate_offline_token(
        &self,
        token: &SignedLicenseToken,
        claimed_nodes_count: u32,
        current_time_sec: u64,
    ) -> Result<LicensePayload, LicenseError> {
        // 1. Vérification de la signature cryptographique Ed25519
        if token.signature_bytes.len() != 64 {
            return Err(LicenseError::InvalidSignature);
        }
        let mut sig_arr = [0u8; 64];
        sig_arr.copy_from_slice(&token.signature_bytes);
        let signature = Signature::from_bytes(&sig_arr);

        self.publisher_public_key
            .verify(&token.payload_bytes, &signature)
            .map_err(|_| LicenseError::InvalidSignature)?;

        // 2. Décodage du payload JSON
        let payload: LicensePayload = serde_json::from_slice(&token.payload_bytes)?;

        // 3. Vérification de la date d'expiration
        if current_time_sec > payload.expires_at_epoch_sec {
            return Err(LicenseError::Expired {
                expired_at: payload.expires_at_epoch_sec,
                current: current_time_sec,
            });
        }

        // 4. Vérification du quota d'appareils autorisés
        if claimed_nodes_count > payload.max_nodes {
            return Err(LicenseError::NodeLimitExceeded {
                max: payload.max_nodes,
                claimed: claimed_nodes_count,
            });
        }

        Ok(payload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;

    #[test]
    fn test_valid_offline_license_verification() {
        // Simulation SaaS Éditeur : Génération d'une paire de clés Ed25519
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();

        // Création d'un payload de licence valide pour 30 jours
        let payload = LicensePayload {
            tenant_id: "TENANT-PME-001".into(),
            installation_id: "INST-UUID-999".into(),
            expires_at_epoch_sec: 2000000000,
            max_nodes: 5,
            plan_tier: "ENTERPRISE".into(),
        };
        let payload_bytes = serde_json::to_vec(&payload).unwrap();

        // Signature du payload par le SaaS Éditeur
        use ed25519_dalek::Signer;
        let signature = signing_key.sign(&payload_bytes);

        let token = SignedLicenseToken {
            payload_bytes,
            signature_bytes: signature.to_bytes().to_vec(),
        };

        // Côté Client (Mission B) : Validation autonome avec la clé publique de l'éditeur
        let validator = LicenseValidator::new(&verifying_key.to_bytes()).unwrap();
        let validated_payload = validator
            .validate_offline_token(&token, 3, 1700000000)
            .unwrap();

        assert_eq!(validated_payload.tenant_id, "TENANT-PME-001");
        assert_eq!(validated_payload.max_nodes, 5);
    }

    #[test]
    fn test_tampered_license_signature_fails() {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();

        let payload = LicensePayload {
            tenant_id: "TENANT-FRAUD".into(),
            installation_id: "INST-001".into(),
            expires_at_epoch_sec: 2000000000,
            max_nodes: 2,
            plan_tier: "STANDARD".into(),
        };
        let mut payload_bytes = serde_json::to_vec(&payload).unwrap();
        
        // Falsification d'un octet dans le payload
        payload_bytes[0] ^= 0xFF;

        use ed25519_dalek::Signer;
        let signature = signing_key.sign(&payload_bytes);

        let token = SignedLicenseToken {
            payload_bytes,
            signature_bytes: signature.to_bytes().to_vec(),
        };

        let validator = LicenseValidator::new(&verifying_key.to_bytes()).unwrap();
        let res = validator.validate_offline_token(&token, 2, 1700000000);
        assert!(matches!(res, Err(LicenseError::InvalidSignature)));
    }
}
