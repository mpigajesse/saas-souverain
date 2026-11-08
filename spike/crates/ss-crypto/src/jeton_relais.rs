//! Jeton du relais propre à un tenant (refonte V3, étape 9, ADR-011).
//!
//! Le portail et le relais partagent un secret (`RELAY_SECRET`) ; le jeton
//! d'un tenant est `HMAC-SHA256(secret, "amane-relais/v1/" ‖ tenant)`, en
//! hexadécimal. Le portail le remet à la pile du tenant, le relais le
//! recalcule à chaque requête : aucune base de données côté relais, et le
//! jeton d'un tenant n'ouvre pas les blobs d'un autre.
//!
//! HMAC et SHA-256 viennent de RustCrypto (`hmac`, `sha2`) ; la comparaison
//! se fait en temps constant (`Mac::verify_slice`). Le portail calcule la même
//! valeur avec `hmac` et `hashlib` de la bibliothèque standard Python.

use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Préfixe de séparation de domaine : versionne le format du jeton.
const CONTEXTE: &[u8] = b"amane-relais/v1/";

fn mac(secret: &[u8], tenant_id: &str) -> HmacSha256 {
    // HMAC accepte une clé de toute longueur : `new_from_slice` ne peut échouer.
    let mut mac = HmacSha256::new_from_slice(secret).expect("HMAC accepte toute longueur de clé");
    mac.update(CONTEXTE);
    mac.update(tenant_id.as_bytes());
    mac
}

/// Jeton du tenant `tenant_id` (UUID en minuscules avec tirets, tel que
/// `Uuid::to_string` et `str(uuid)` en Python le donnent).
pub fn jeton_relais(secret: &[u8], tenant_id: &str) -> String {
    hex::encode(mac(secret, tenant_id).finalize().into_bytes())
}

/// Vrai si `jeton` est celui du tenant. Comparaison en temps constant.
pub fn verifier_jeton_relais(secret: &[u8], tenant_id: &str, jeton: &str) -> bool {
    match hex::decode(jeton.trim()) {
        Ok(octets) => mac(secret, tenant_id).verify_slice(&octets).is_ok(),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &[u8] = b"secret-de-test-du-relais-amane-0123456789";
    const NUL: &str = "00000000-0000-0000-0000-000000000000";

    #[test]
    fn vecteur_commun_avec_le_portail() {
        // Même valeur que `tenants.tests.JetonRelaisTests` côté Django : le
        // portail et le relais doivent calculer le même jeton.
        assert_eq!(
            jeton_relais(SECRET, NUL),
            "1122cac8a5f6cce94d90f49b21d7aee4cb844f27725ab56a57ae4f80764df4a5"
        );
    }

    #[test]
    fn jeton_verifie_pour_son_seul_tenant() {
        let autre = "11111111-1111-1111-1111-111111111111";
        let jeton = jeton_relais(SECRET, NUL);
        assert!(verifier_jeton_relais(SECRET, NUL, &jeton));
        assert!(!verifier_jeton_relais(SECRET, autre, &jeton));
        assert!(!verifier_jeton_relais(b"un-autre-secret", NUL, &jeton));
        assert!(!verifier_jeton_relais(SECRET, NUL, ""));
        assert!(!verifier_jeton_relais(SECRET, NUL, "pas-de-l-hexadecimal"));
        assert!(!verifier_jeton_relais(SECRET, NUL, &jeton[..32]));
    }
}
