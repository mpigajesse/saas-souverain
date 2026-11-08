//! # Partage de Shamir et KEK (ADR-010)
//!
//! Le partage lui-même est délégué à la bibliothèque **vsss-rs** (partage
//! octet par octet sur GF(256)) : aucune arithmétique de corps fini n'est
//! écrite ici (règle n° 1 du dépôt). L'ancienne implémentation maison, dont les
//! branchements dépendaient des octets secrets, a été retirée le 2026-11-04.
//!
//! Une part ne révèle rien du secret ; il en faut `seuil` pour le reconstituer.
//! Le partage n'authentifie pas le secret : des parts d'un autre jeu donnent un
//! résultat faux, sans erreur. L'appelant doit donc vérifier le secret
//! reconstitué (ici, par le déchiffrement authentifié de la DEK).

use std::fmt;

use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use vsss_rs::Gf256;
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

use crate::{CryptoError, Dek};

/// Taille d'un secret partagé (une clé de 256 bits).
pub const TAILLE_SECRET: usize = 32;

/// Clé d'Enveloppe de Clé (Key Encryption Key - KEK 256 bits).
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct Kek(pub [u8; 32]);

impl fmt::Debug for Kek {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Kek([REDACTED])")
    }
}

impl Kek {
    /// Génère une KEK aléatoire de 256 bits via OsRng.
    pub fn generate() -> Self {
        let mut bytes = [0u8; 32];
        OsRng.fill_bytes(&mut bytes);
        Self(bytes)
    }

    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Emballe une DEK sous la KEK (XChaCha20-Poly1305 de `Dek::encrypt`).
    pub fn wrap_dek(&self, dek: &Dek) -> Result<Vec<u8>, CryptoError> {
        let kek_dek = Dek::from_bytes(self.0);
        kek_dek.encrypt(dek.as_bytes())
    }

    /// Déballe une DEK depuis une enveloppe KEK.
    pub fn unwrap_dek(&self, wrapped_blob: &[u8]) -> Result<Dek, CryptoError> {
        let kek_dek = Dek::from_bytes(self.0);
        let pt = Zeroizing::new(kek_dek.decrypt(wrapped_blob)?);
        if pt.len() != 32 {
            return Err(CryptoError::DecryptionFailed);
        }
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&pt);
        Ok(Dek::from_bytes(bytes))
    }
}

/// Une part d'un secret de 32 octets : son identifiant (abscisse, jamais 0)
/// et 32 octets d'ordonnée.
#[derive(Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct ShamirShare {
    pub id: u8,
    pub data: [u8; 32],
}

impl fmt::Debug for ShamirShare {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ShamirShare(id: {}, data: [REDACTED])", self.id)
    }
}

impl ShamirShare {
    /// Forme sérialisée de vsss-rs : identifiant puis données (33 octets).
    pub fn to_bytes(&self) -> Zeroizing<Vec<u8>> {
        let mut octets = Zeroizing::new(Vec::with_capacity(1 + TAILLE_SECRET));
        octets.push(self.id);
        octets.extend_from_slice(&self.data);
        octets
    }

    /// Relit une part de 33 octets (identifiant non nul puis données).
    pub fn from_bytes(octets: &[u8]) -> Result<Self, CryptoError> {
        if octets.len() != 1 + TAILLE_SECRET || octets[0] == 0 {
            return Err(CryptoError::InvalidShamirShare);
        }
        let mut data = [0u8; 32];
        data.copy_from_slice(&octets[1..]);
        Ok(Self { id: octets[0], data })
    }
}

/// Aléa du système au format attendu par vsss-rs (rand_core 0.10).
fn alea() -> rand_core_010::UnwrapErr<getrandom_04::SysRng> {
    rand_core_010::UnwrapErr(getrandom_04::SysRng)
}

/// Découpe un secret de 32 octets en `nombre` parts, dont `seuil` suffisent.
pub fn split_secret(secret: &[u8; 32], seuil: u8, nombre: u8) -> Result<Vec<ShamirShare>, CryptoError> {
    if seuil < 2 || seuil > nombre {
        return Err(CryptoError::InvalidShamirShare);
    }
    let parts = Gf256::split_bytes(seuil as usize, nombre as usize, secret, &mut alea())
        .map_err(|_| CryptoError::InvalidShamirShare)?;
    parts
        .into_iter()
        .map(|p| {
            let p = Zeroizing::new(p);
            ShamirShare::from_bytes(&p)
        })
        .collect()
}

/// Reconstitue un secret de 32 octets à partir d'au moins `seuil` parts
/// distinctes. Le résultat n'est pas authentifié (voir l'en-tête du module).
pub fn reconstruct_secret(parts: &[ShamirShare]) -> Result<Zeroizing<[u8; 32]>, CryptoError> {
    if parts.len() < 2 {
        return Err(CryptoError::ShamirReconstructionFailed);
    }
    for (i, a) in parts.iter().enumerate() {
        if a.id == 0 || parts[..i].iter().any(|b| b.id == a.id) {
            return Err(CryptoError::InvalidShamirShare);
        }
    }
    let brutes: Vec<Vec<u8>> = parts.iter().map(|p| p.to_bytes().to_vec()).collect();
    let resultat = Gf256::combine_bytes(&brutes);
    let mut brutes = brutes;
    brutes.iter_mut().for_each(|b| b.zeroize());
    let secret = Zeroizing::new(resultat.map_err(|_| CryptoError::ShamirReconstructionFailed)?);
    if secret.len() != TAILLE_SECRET {
        return Err(CryptoError::ShamirReconstructionFailed);
    }
    let mut octets = Zeroizing::new([0u8; 32]);
    octets.copy_from_slice(&secret);
    Ok(octets)
}

/// Découpe une KEK en 3 parts dont 2 suffisent.
pub fn split_kek(kek: &Kek) -> Result<(ShamirShare, ShamirShare, ShamirShare), CryptoError> {
    let mut parts = split_secret(kek.as_bytes(), 2, 3)?;
    let s3 = parts.pop().ok_or(CryptoError::InvalidShamirShare)?;
    let s2 = parts.pop().ok_or(CryptoError::InvalidShamirShare)?;
    let s1 = parts.pop().ok_or(CryptoError::InvalidShamirShare)?;
    Ok((s1, s2, s3))
}

/// Reconstitue une KEK à partir de 2 parts sur 3.
pub fn reconstruct_kek(s1: &ShamirShare, s2: &ShamirShare) -> Result<Kek, CryptoError> {
    let octets = reconstruct_secret(&[s1.clone(), s2.clone()])?;
    Ok(Kek(*octets))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kek_wrap_unwrap_dek() {
        let kek = Kek::generate();
        let dek = Dek::generate();
        let wrapped = kek.wrap_dek(&dek).unwrap();
        let unwrapped = kek.unwrap_dek(&wrapped).unwrap();
        assert_eq!(dek.as_bytes(), unwrapped.as_bytes());
    }

    #[test]
    fn test_shamir_split_and_reconstruct_all_combinations() {
        let kek = Kek::generate();
        let (s1, s2, s3) = split_kek(&kek).unwrap();
        assert_eq!([s1.id, s2.id, s3.id], [1, 2, 3]);
        for (a, b) in [(&s1, &s2), (&s1, &s3), (&s2, &s3), (&s3, &s1)] {
            assert_eq!(kek.as_bytes(), reconstruct_kek(a, b).unwrap().as_bytes());
        }
    }

    #[test]
    fn test_seuil_et_nombre_parametrables() {
        let secret = [42u8; 32];
        let parts = split_secret(&secret, 3, 5).unwrap();
        assert_eq!(parts.len(), 5);
        let trois = [parts[0].clone(), parts[2].clone(), parts[4].clone()];
        assert_eq!(*reconstruct_secret(&trois).unwrap(), secret);
        // Deux parts d'un seuil de 3 ne redonnent pas le secret.
        let deux = [parts[0].clone(), parts[1].clone()];
        assert!(reconstruct_secret(&deux).map(|s| *s != secret).unwrap_or(true));
    }

    #[test]
    fn test_parts_invalides_refusees() {
        let kek = Kek::generate();
        let (s1, _, _) = split_kek(&kek).unwrap();
        assert!(reconstruct_kek(&s1, &s1).is_err(), "deux fois la même part");
        assert!(reconstruct_secret(&[s1.clone()]).is_err(), "une seule part");
        assert!(split_secret(&[0u8; 32], 1, 3).is_err(), "seuil de 1");
        assert!(split_secret(&[0u8; 32], 4, 3).is_err(), "seuil au-delà du nombre");
        assert!(ShamirShare::from_bytes(&[0u8; 33]).is_err(), "identifiant nul");
        assert!(ShamirShare::from_bytes(&[1u8; 32]).is_err(), "longueur fausse");
    }

    #[test]
    fn test_une_part_ne_revele_pas_le_secret() {
        // Deux partages du même secret donnent des parts différentes.
        let secret = [9u8; 32];
        let a = split_secret(&secret, 2, 3).unwrap();
        let b = split_secret(&secret, 2, 3).unwrap();
        assert_ne!(a[0].data, b[0].data);
        assert_ne!(a[0].data, secret);
    }

    #[test]
    fn test_aller_retour_en_octets() {
        let (s1, _, _) = split_kek(&Kek::generate()).unwrap();
        let relue = ShamirShare::from_bytes(&s1.to_bytes()).unwrap();
        assert_eq!((relue.id, relue.data), (s1.id, s1.data));
    }

    #[test]
    fn test_debug_hermeticity() {
        let kek = Kek::generate();
        assert_eq!(format!("{:?}", kek), "Kek([REDACTED])");
        let (s1, _, _) = split_kek(&kek).unwrap();
        assert_eq!(format!("{:?}", s1), "ShamirShare(id: 1, data: [REDACTED])");
    }
}
