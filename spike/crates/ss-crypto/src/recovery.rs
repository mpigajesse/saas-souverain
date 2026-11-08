//! # Récupération de la DEK après perte totale (ADR-010)
//!
//! Deux facteurs, tous deux exigés :
//!
//! - `K_p = Argon2id(passphrase du gérant, sel)` ;
//! - `K_s`, 32 octets aléatoires découpés en parts papier (Shamir, 2 sur 3).
//!
//! `KEK = BLAKE2b-256` à clé `K_s` sur une étiquette et `K_p`. Une paire X25519
//! de récupération est dérivée de la KEK ; la DEK est scellée pour sa clé
//! publique (`DevicePublicKey::seal_dek`). Tout serveur qui détient la DEK peut
//! donc resceller le blob sans la passphrase, alors que l'ouvrir exige les deux
//! facteurs. Aucune primitive n'est écrite ici : Argon2id, BLAKE2b et X25519
//! viennent de RustCrypto et de dalek, le partage de vsss-rs.

use argon2::{Argon2, Params};
use blake2::digest::consts::{U16, U32};
use blake2::digest::{FixedOutput, KeyInit, Mac, Update};
use blake2::{Blake2b, Blake2bMac};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

use crate::{CryptoError, Dek, DeviceKeyPair, Kek};

const KEY_LEN: usize = 32;

/// Étiquettes de séparation des usages de BLAKE2b.
const ETIQUETTE_KEK: &[u8] = b"amane/recuperation/kek/v1";
const ETIQUETTE_EMPREINTE: &[u8] = b"amane/recuperation/empreinte-dek/v1";
const ETIQUETTE_CONTROLE: &[u8] = b"amane/recuperation/controle-part/v1";

/// Paramètres Argon2id, inscrits dans le blob de récupération pour pouvoir
/// évoluer sans rendre les anciens blobs illisibles.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParametresArgon2 {
    /// Mémoire en Kio.
    pub m: u32,
    /// Nombre de passes.
    pub t: u32,
    /// Nombre de voies.
    pub p: u32,
}

impl ParametresArgon2 {
    /// Deuxième recommandation de RFC 9106 (§ 4) : 64 Mio, 3 passes, 4 voies.
    pub const RFC9106: Self = Self { m: 65536, t: 3, p: 4 };
}

/// Dérive 32 octets d'une passphrase et d'un sel avec les paramètres par
/// défaut (RFC 9106). `salt` : 16 octets aléatoires, conservés avec le blob.
pub fn derive_recovery_key(passphrase: &str, salt: &[u8; 16]) -> Result<[u8; KEY_LEN], CryptoError> {
    derive_recovery_key_avec(passphrase, salt, ParametresArgon2::RFC9106).map(|k| *k)
}

/// Dérive `K_p` avec des paramètres explicites (ceux lus dans un blob).
pub fn derive_recovery_key_avec(
    passphrase: &str,
    salt: &[u8; 16],
    parametres: ParametresArgon2,
) -> Result<Zeroizing<[u8; KEY_LEN]>, CryptoError> {
    let params = Params::new(parametres.m, parametres.t, parametres.p, Some(KEY_LEN))
        .map_err(|e| CryptoError::Argon2(format!("{e:?}")))?;
    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);
    let mut output = Zeroizing::new([0u8; KEY_LEN]);
    argon2
        .hash_password_into(passphrase.as_bytes(), salt.as_slice(), output.as_mut())
        .map_err(|e| CryptoError::Argon2(format!("{e:?}")))?;
    Ok(output)
}

/// Génère un sel aléatoire de 16 octets.
pub fn generate_salt() -> [u8; 16] {
    let mut salt = [0u8; 16];
    OsRng.fill_bytes(&mut salt);
    salt
}

/// BLAKE2b-256 à clé (MAC) : `cle` de 1 à 64 octets.
fn mac32(cle: &[u8], parties: &[&[u8]]) -> Result<Zeroizing<[u8; 32]>, CryptoError> {
    let mut m = <Blake2bMac<U32> as KeyInit>::new_from_slice(cle).map_err(|_| CryptoError::EncryptionFailed)?;
    for partie in parties {
        Mac::update(&mut m, partie);
    }
    let mut sortie = Zeroizing::new([0u8; 32]);
    sortie.copy_from_slice(&m.finalize().into_bytes());
    Ok(sortie)
}

/// BLAKE2b-128 à clé (MAC).
fn mac16(cle: &[u8], parties: &[&[u8]]) -> Result<[u8; 16], CryptoError> {
    let mut m = <Blake2bMac<U16> as KeyInit>::new_from_slice(cle).map_err(|_| CryptoError::EncryptionFailed)?;
    for partie in parties {
        Mac::update(&mut m, partie);
    }
    let mut sortie = [0u8; 16];
    sortie.copy_from_slice(&m.finalize().into_bytes());
    Ok(sortie)
}

/// `KEK = BLAKE2b-256(clé = K_s, étiquette ‖ K_p)` : les deux facteurs sont
/// indispensables.
pub fn deriver_kek(cle_passphrase: &[u8; 32], secret_des_parts: &[u8; 32]) -> Result<Kek, CryptoError> {
    let sortie = mac32(secret_des_parts, &[ETIQUETTE_KEK, cle_passphrase])?;
    Ok(Kek::from_bytes(*sortie))
}

/// Paire X25519 de récupération dérivée de la KEK. Sa clé publique, conservée
/// par le cluster, permet de resceller la DEK sans la passphrase.
pub fn paire_de_recuperation(kek: &Kek) -> DeviceKeyPair {
    DeviceKeyPair::from_secret_bytes(*kek.as_bytes())
}

/// Empreinte de 16 octets de la DEK (BLAKE2b à clé : ne révèle rien de la DEK).
/// Elle permet de savoir, sans déchiffrer, si un blob protège la DEK courante.
pub fn empreinte_dek(dek: &Dek) -> Result<[u8; 16], CryptoError> {
    mac16(dek.as_bytes(), &[ETIQUETTE_EMPREINTE])
}

/// Somme de contrôle d'une part papier (3 octets), liée au jeu de parts : une
/// faute de frappe ou une part d'un autre jeu est détectée à la saisie. Ce
/// n'est pas une protection contre un attaquant, seulement contre l'erreur.
pub fn somme_controle_part(jeu: &[u8], part: &[u8]) -> [u8; 3] {
    let mut h = Blake2b::<U16>::default();
    Update::update(&mut h, ETIQUETTE_CONTROLE);
    Update::update(&mut h, &(jeu.len() as u32).to_le_bytes());
    Update::update(&mut h, jeu);
    Update::update(&mut h, part);
    let sortie = h.finalize_fixed();
    [sortie[0], sortie[1], sortie[2]]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{reconstruct_secret, split_secret};

    /// Paramètres réduits pour des tests rapides.
    const LEGERS: ParametresArgon2 = ParametresArgon2 { m: 1024, t: 1, p: 1 };

    #[test]
    fn deterministic() {
        let salt = [0u8; 16];
        let k1 = derive_recovery_key_avec("secret-code", &salt, LEGERS).unwrap();
        let k2 = derive_recovery_key_avec("secret-code", &salt, LEGERS).unwrap();
        assert_eq!(*k1, *k2);
    }

    #[test]
    fn different_passphrase_different_key() {
        let salt = [0u8; 16];
        let k1 = derive_recovery_key_avec("code-a", &salt, LEGERS).unwrap();
        let k2 = derive_recovery_key_avec("code-b", &salt, LEGERS).unwrap();
        assert_ne!(*k1, *k2);
    }

    #[test]
    fn different_salt_different_key() {
        let k1 = derive_recovery_key_avec("même-code", &[0u8; 16], LEGERS).unwrap();
        let k2 = derive_recovery_key_avec("même-code", &[1u8; 16], LEGERS).unwrap();
        assert_ne!(*k1, *k2);
    }

    #[test]
    fn parametres_par_defaut_rfc9106() {
        let k = derive_recovery_key("code", &[3u8; 16]).unwrap();
        let k_avec = derive_recovery_key_avec("code", &[3u8; 16], ParametresArgon2::RFC9106).unwrap();
        assert_eq!(k, *k_avec);
    }

    #[test]
    fn la_kek_exige_les_deux_facteurs() {
        let (kp, ks) = ([1u8; 32], [2u8; 32]);
        let kek = deriver_kek(&kp, &ks).unwrap();
        assert_eq!(kek.as_bytes(), deriver_kek(&kp, &ks).unwrap().as_bytes());
        assert_ne!(kek.as_bytes(), deriver_kek(&[9u8; 32], &ks).unwrap().as_bytes());
        assert_ne!(kek.as_bytes(), deriver_kek(&kp, &[9u8; 32]).unwrap().as_bytes());
    }

    #[test]
    fn parcours_complet_passphrase_et_parts() {
        // Configuration : la DEK est scellée pour la clé de récupération.
        let dek = Dek::generate();
        let sel = generate_salt();
        let kp = derive_recovery_key_avec("passphrase du gérant", &sel, LEGERS).unwrap();
        let mut ks = [0u8; 32];
        OsRng.fill_bytes(&mut ks);
        let parts = split_secret(&ks, 2, 3).unwrap();
        let publique = paire_de_recuperation(&deriver_kek(&kp, &ks).unwrap()).public.clone();
        let scellee = publique.seal_dek(&dek).unwrap();

        // Récupération : passphrase et deux parts quelconques.
        let kp2 = derive_recovery_key_avec("passphrase du gérant", &sel, LEGERS).unwrap();
        let ks2 = reconstruct_secret(&[parts[2].clone(), parts[0].clone()]).unwrap();
        let paire = paire_de_recuperation(&deriver_kek(&kp2, &ks2).unwrap());
        assert_eq!(paire.open_sealed_dek(&scellee).unwrap().as_bytes(), dek.as_bytes());

        // Mauvaise passphrase : la DEK ne s'ouvre pas.
        let faux = derive_recovery_key_avec("autre passphrase", &sel, LEGERS).unwrap();
        assert!(paire_de_recuperation(&deriver_kek(&faux, &ks2).unwrap()).open_sealed_dek(&scellee).is_err());
        // Parts d'un autre jeu : la DEK ne s'ouvre pas non plus.
        let autres = split_secret(&[5u8; 32], 2, 3).unwrap();
        let ks_faux = reconstruct_secret(&[autres[0].clone(), parts[1].clone()]).unwrap();
        assert!(paire_de_recuperation(&deriver_kek(&kp2, &ks_faux).unwrap()).open_sealed_dek(&scellee).is_err());
    }

    #[test]
    fn empreinte_stable_et_propre_a_la_dek() {
        let dek = Dek::generate();
        assert_eq!(empreinte_dek(&dek).unwrap(), empreinte_dek(&Dek::from_bytes(*dek.as_bytes())).unwrap());
        assert_ne!(empreinte_dek(&dek).unwrap(), empreinte_dek(&Dek::generate()).unwrap());
        assert_ne!(&empreinte_dek(&dek).unwrap()[..], &dek.as_bytes()[..16]);
    }

    #[test]
    fn somme_de_controle_liee_au_jeu_et_a_la_part() {
        let a = somme_controle_part(b"jeu-1", &[1, 2, 3]);
        assert_eq!(a, somme_controle_part(b"jeu-1", &[1, 2, 3]));
        assert_ne!(a, somme_controle_part(b"jeu-2", &[1, 2, 3]));
        assert_ne!(a, somme_controle_part(b"jeu-1", &[1, 2, 4]));
    }
}
