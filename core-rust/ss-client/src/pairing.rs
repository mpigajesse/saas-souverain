use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PairingError {
    #[error("Jeton d'invitation d'appairage invalide ou déjà consommé")]
    InvalidInvitationToken,

    #[error("Jeton d'invitation expiré depuis {expired_at} (actuel : {current})")]
    InvitationExpired { expired_at: u64, current: u64 },

    #[error("Erreur lors du scellement/déballage Sealed Box : {0}")]
    SealedBoxError(String),

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

/// Paquet d'enrôlement contenant la clé d'accès (AK) emballée via Sealed Box (Libsodium X25519)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SealedAkPayload {
    pub invitation_id: String,
    pub recipient_public_key_bytes: [u8; 32], // Clé publique X25519 du nouveau poste
    pub sealed_ak_bytes: Vec<u8>,             // AK du cluster emballée en Sealed Box
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

    /// Simule l'emballage Sealed Box de la clé d'accès (AK) à destination de la clé publique X25519 du nouveau poste
    pub fn seal_access_key_for_node(
        invitation: &mut PairingInvitationToken,
        cluster_ak_bytes: &[u8; 32],
        recipient_public_key_bytes: &[u8; 32],
        current_time_sec: u64,
    ) -> Result<SealedAkPayload, PairingError> {
        // 1. Vérification de l'état du jeton d'invitation
        if invitation.consumed {
            return Err(PairingError::InvalidInvitationToken);
        }
        if current_time_sec > invitation.expires_at_epoch_sec {
            return Err(PairingError::InvitationExpired {
                expired_at: invitation.expires_at_epoch_sec,
                current: current_time_sec,
            });
        }

        // 2. Marquer le jeton comme consommé (usage unique)
        invitation.consumed = true;

        // 3. Emballage Sealed Box (simulé / format de scellement)
        let mut dummy_sealed = vec![0xA1, 0xB2]; // Header Sealed Box
        dummy_sealed.extend_from_slice(cluster_ak_bytes);
        dummy_sealed.extend_from_slice(recipient_public_key_bytes);

        Ok(SealedAkPayload {
            invitation_id: invitation.invitation_id.clone(),
            recipient_public_key_bytes: *recipient_public_key_bytes,
            sealed_ak_bytes: dummy_sealed,
        })
    }

    /// Déballe la clé d'accès (AK) côté nouveau poste avec sa clé privée X25519
    pub fn unseal_access_key(
        sealed_payload: &SealedAkPayload,
        recipient_private_key_bytes: &[u8; 32],
    ) -> Result<[u8; 32], PairingError> {
        if sealed_payload.sealed_ak_bytes.len() < 34 {
            return Err(PairingError::SealedBoxError("Payload Sealed Box trop court".into()));
        }

        // Extraction simulée des 32 octets de l'AK du cluster
        let mut ak_out = [0u8; 32];
        ak_out.copy_from_slice(&sealed_payload.sealed_ak_bytes[2..34]);
        
        // Simuler la validation de la clé privée récipiendaire
        if recipient_private_key_bytes[0] == 0xFF {
            return Err(PairingError::SealedBoxError("Clé privée récipiendaire invalide".into()));
        }

        Ok(ak_out)
    }
}

/// Générateur d'UUID simple pour simulation d'ID d'invitation sans dépendance externe lourde
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
    fn test_successful_pairing_workflow() {
        let current_time = 1700000000;
        let mut inv = PairingManager::create_invitation("CLUSTER-PME-01", 300, current_time);

        let cluster_ak = [0x77u8; 32];
        let new_node_pubkey = [0x88u8; 32];
        let new_node_privkey = [0x11u8; 32];

        // 1. Scellement de l'AK par le poste actif
        let sealed_payload = PairingManager::seal_access_key_for_node(
            &mut inv,
            &cluster_ak,
            &new_node_pubkey,
            current_time + 10,
        )
        .unwrap();

        assert!(inv.consumed);

        // 2. Déballage de l'AK par le nouveau poste
        let unsealed_ak = PairingManager::unseal_access_key(&sealed_payload, &new_node_privkey).unwrap();
        assert_eq!(unsealed_ak, cluster_ak);
    }

    #[test]
    fn test_consumed_invitation_fails() {
        let current_time = 1700000000;
        let mut inv = PairingManager::create_invitation("CLUSTER-PME-01", 300, current_time);
        let cluster_ak = [0x77u8; 32];
        let pubkey = [0x88u8; 32];

        // Premier usage -> Succès
        assert!(PairingManager::seal_access_key_for_node(&mut inv, &cluster_ak, &pubkey, current_time).is_ok());

        // Deuxième usage (Réutilisation d'invitation) -> Échec !
        let second_try = PairingManager::seal_access_key_for_node(&mut inv, &cluster_ak, &pubkey, current_time);
        assert!(matches!(second_try, Err(PairingError::InvalidInvitationToken)));
    }
}
