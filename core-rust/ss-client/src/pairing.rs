use serde::{Deserialize, Serialize};
use ss_crypto::{CryptoError, Dek, DeviceKeyPair, DevicePublicKey};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PairingError {
    #[error("Jeton d'invitation d'appairage invalide ou déjà consommé")]
    InvalidInvitationToken,

    #[error("Jeton d'invitation expiré depuis {expired_at} (actuel : {current})")]
    InvitationExpired { expired_at: u64, current: u64 },

    #[error("Erreur cryptographique lors du scellement/déballage Sealed Box : {0}")]
    CryptoError(#[from] CryptoError),

    #[error("Erreur de sérialisation JSON : {0}")]
    SerializationError(#[from] serde_json::Error),
}

/// Invitation à usage unique générée par le nœud actif pour le QR Code d'appairage
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PairingInvitationToken {
    pub invitation_id: String,
    pub cluster_id: String,
    pub expires_at_epoch_sec: u64,
    pub consumed: bool,
}

/// Paquet d'enrôlement contenant la clé d'accès (AK) scellée via Sealed Box X25519 (ss-crypto)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SealedAkPayload {
    pub invitation_id: String,
    pub recipient_public_key: DevicePublicKey, // Clé publique X25519 du nouveau poste
    pub sealed_ak_bytes: Vec<u8>,              // AK du cluster scellée avec XChaCha20-Poly1305 (ss-crypto)
}

pub struct PairingManager;

impl PairingManager {
    /// Génère une invitation d'appairage à usage unique pour un nouveau poste
    pub fn create_invitation(
        cluster_id: &str,
        validity_duration_sec: u64,
        current_time_sec: u64,
    ) -> PairingInvitationToken {
        let invitation_id = format!("INV-{}", uuid_simple());
        PairingInvitationToken {
            invitation_id,
            cluster_id: cluster_id.to_string(),
            expires_at_epoch_sec: current_time_sec + validity_duration_sec,
            consumed: false,
        }
    }

    /// Scelle la clé d'accès (AK) du cluster pour le nouveau poste grâce à sa clé publique X25519 (ss-crypto)
    ///
    /// Utilisations réelles de ss-crypto :
    /// - Génération d'une paire éphémère X25519
    /// - Derivation BLAKE2b-512
    /// - Chiffrement XChaCha20-Poly1305 de la clé AK
    pub fn seal_access_key_for_node(
        invitation: &mut PairingInvitationToken,
        cluster_ak: &Dek,
        recipient_public_key: &DevicePublicKey,
        current_time_sec: u64,
    ) -> Result<SealedAkPayload, PairingError> {
        // 1. Vérification de la validité du jeton d'invitation
        if invitation.consumed {
            return Err(PairingError::InvalidInvitationToken);
        }
        if current_time_sec > invitation.expires_at_epoch_sec {
            return Err(PairingError::InvitationExpired {
                expired_at: invitation.expires_at_epoch_sec,
                current: current_time_sec,
            });
        }

        // 2. Marquer le jeton comme consommé (usage unique strict)
        invitation.consumed = true;

        // 3. VRAI scellement cryptographique via ss-crypto (DevicePublicKey::seal_dek)
        let sealed_ak_bytes = recipient_public_key.seal_dek(cluster_ak)?;

        Ok(SealedAkPayload {
            invitation_id: invitation.invitation_id.clone(),
            recipient_public_key: recipient_public_key.clone(),
            sealed_ak_bytes,
        })
    }

    /// Déballe la clé d'accès (AK) côté nouveau poste avec sa vraie paire de clés X25519 (ss-crypto)
    pub fn unseal_access_key(
        sealed_payload: &SealedAkPayload,
        recipient_keypair: &DeviceKeyPair,
    ) -> Result<Dek, PairingError> {
        // VRAI déballage cryptographique via ss-crypto (DeviceKeyPair::open_sealed_dek)
        let recovered_ak = recipient_keypair.open_sealed_dek(&sealed_payload.sealed_ak_bytes)?;
        Ok(recovered_ak)
    }
}

/// Générateur simple d'identifiant d'invitation
fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    format!("{:08x}", nanos)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_successful_pairing_workflow_real_crypto() {
        let current_time = 1700000000;
        let mut inv = PairingManager::create_invitation("CLUSTER-PME-01", 300, current_time);

        // Clé AK du cluster PME
        let cluster_ak = Dek::generate();

        // Paire de clés X25519 réelle générée pour le nouveau poste (ss-crypto)
        let recipient_keypair = DeviceKeyPair::generate();

        // 1. Scellement réel par le poste actif avec la clé publique du destinataire
        let sealed_payload = PairingManager::seal_access_key_for_node(
            &mut inv,
            &cluster_ak,
            &recipient_keypair.public,
            current_time + 10,
        )
        .unwrap();

        assert!(inv.consumed);
        // Vérification que le payload scellé n'est PAS en clair
        assert_ne!(sealed_payload.sealed_ak_bytes, cluster_ak.as_bytes().as_slice());

        // 2. Déballage réel avec la clé privée du destinataire
        let unsealed_ak = PairingManager::unseal_access_key(&sealed_payload, &recipient_keypair).unwrap();
        assert_eq!(unsealed_ak.as_bytes(), cluster_ak.as_bytes());
    }

    #[test]
    fn test_wrong_keypair_cannot_unseal() {
        let current_time = 1700000000;
        let mut inv = PairingManager::create_invitation("CLUSTER-PME-01", 300, current_time);
        let cluster_ak = Dek::generate();

        let recipient_keypair = DeviceKeyPair::generate();
        let attacker_keypair = DeviceKeyPair::generate();

        let sealed_payload = PairingManager::seal_access_key_for_node(
            &mut inv,
            &cluster_ak,
            &recipient_keypair.public,
            current_time,
        )
        .unwrap();

        // Un pirate qui essaie d'ouvrir le sealed box avec sa propre clé DOIT échouer !
        let unseal_attempt = PairingManager::unseal_access_key(&sealed_payload, &attacker_keypair);
        assert!(unseal_attempt.is_err());
    }

    #[test]
    fn test_consumed_invitation_fails() {
        let current_time = 1700000000;
        let mut inv = PairingManager::create_invitation("CLUSTER-PME-01", 300, current_time);
        let cluster_ak = Dek::generate();
        let recipient_keypair = DeviceKeyPair::generate();

        // Premier usage -> Succès
        assert!(PairingManager::seal_access_key_for_node(&mut inv, &cluster_ak, &recipient_keypair.public, current_time).is_ok());

        // Deuxième usage (Réutilisation d'invitation) -> Échec !
        let second_try = PairingManager::seal_access_key_for_node(&mut inv, &cluster_ak, &recipient_keypair.public, current_time);
        assert!(matches!(second_try, Err(PairingError::InvalidInvitationToken)));
    }
}
