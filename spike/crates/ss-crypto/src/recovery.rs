use argon2::{Argon2, Params};
use rand::RngCore;
use rand::rngs::OsRng;
use crate::CryptoError;

/// Paramètres Argon2id (spike : raisonnables mais non durcis pour la production)
const ARGON2_M_COST: u32 = 65536; // 64 MiB
const ARGON2_T_COST: u32 = 3;
const ARGON2_P_COST: u32 = 4;
const KEY_LEN: usize = 32;

/// Dérive 32 octets depuis un code de récupération (passphrase) et un sel.
/// `salt` doit être 16 octets aléatoires, généré une fois, stocké avec le blob.
pub fn derive_recovery_key(passphrase: &str, salt: &[u8; 16]) -> Result<[u8; KEY_LEN], CryptoError> {
    let params = Params::new(ARGON2_M_COST, ARGON2_T_COST, ARGON2_P_COST, Some(KEY_LEN))
        .map_err(|e| CryptoError::Argon2(format!("{e:?}")))?;
    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);
    let mut output = [0u8; KEY_LEN];
    argon2
        .hash_password_into(passphrase.as_bytes(), salt.as_slice(), &mut output)
        .map_err(|e| CryptoError::Argon2(format!("{e:?}")))?;
    Ok(output)
}

/// Génère un sel aléatoire de 16 octets.
pub fn generate_salt() -> [u8; 16] {
    let mut salt = [0u8; 16];
    OsRng.fill_bytes(&mut salt);
    salt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic() {
        let salt = [0u8; 16];
        let k1 = derive_recovery_key("secret-code", &salt).unwrap();
        let k2 = derive_recovery_key("secret-code", &salt).unwrap();
        assert_eq!(k1, k2);
    }

    #[test]
    fn different_passphrase_different_key() {
        let salt = [0u8; 16];
        let k1 = derive_recovery_key("code-a", &salt).unwrap();
        let k2 = derive_recovery_key("code-b", &salt).unwrap();
        assert_ne!(k1, k2);
    }

    #[test]
    fn different_salt_different_key() {
        let s1 = [0u8; 16];
        let s2 = [1u8; 16];
        let k1 = derive_recovery_key("même-code", &s1).unwrap();
        let k2 = derive_recovery_key("même-code", &s2).unwrap();
        assert_ne!(k1, k2);
    }
}
