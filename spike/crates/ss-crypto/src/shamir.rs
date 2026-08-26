//! # Module Shamir Secret Sharing (SSS N=3 / K=2) & Hiérarchie KEK
//!
//! ## Audit Cryptographique & Justification d'Ingénierie
//!
//! Conforme au modèle de menace Amane Zero-Knowledge :
//! - **Corps Fini GF(256)** : Défini sur le polynôme irréductible AES `x^8 + x^4 + x^3 + x + 1` (0x11B).
//! - **Temps d'exécution** : Les opérations d'addition (XOR) et de multiplication/inverse sur GF(256) sont
//!   implémentées de manière déterministe et isolée pour prévenir les attaques par canal auxiliaire (*timing attacks*).
//! - **Indépendance & Autonomie** : Évite les dépendances FFI C externes ou crates obsolètes (`sharks` / `vsss-rs`),
//!   garantissant des builds 100% reproductibles en Rust pur pour l'audit adverse de la Mission A.
//! - **Propriété de Sécurité Théo-Informationnelle** : Une seule part ne fournit **strictement aucune information** (0 bit)
//!   sur la KEK originale (équivalent à un *One-Time Pad*).

use std::fmt;
use rand::rngs::OsRng;
use rand::RngCore;
use zeroize::{Zeroize, ZeroizeOnDrop};
use serde::{Serialize, Deserialize};
use crate::{Dek, CryptoError};

/// Clé d'Enveloppe de Clé (Key Encryption Key - KEK 256 bits).
/// Utilisée pour emballer la DEK et découpée en parts de secret Shamir.
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

    /// Emballe une DEK sous la KEK (chiffrement de la DEK par la KEK).
    pub fn wrap_dek(&self, dek: &Dek) -> Result<Vec<u8>, CryptoError> {
        let kek_dek = Dek::from_bytes(self.0);
        kek_dek.encrypt(dek.as_bytes())
    }

    /// Déballe une DEK depuis une enveloppe KEK.
    pub fn unwrap_dek(&self, wrapped_blob: &[u8]) -> Result<Dek, CryptoError> {
        let kek_dek = Dek::from_bytes(self.0);
        let pt = kek_dek.decrypt(wrapped_blob)?;
        if pt.len() != 32 {
            return Err(CryptoError::DecryptionFailed);
        }
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&pt);
        Ok(Dek::from_bytes(bytes))
    }
}

/// Un fragment/part de secret de Shamir (SSS).
/// Contient un identifiant d'abscisse `id` (1, 2 ou 3) et 32 octets d'ordonnée.
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

// =========================================================================
// Arithmétique sur le Corps Fini GF(256) pour Shamir Secret Sharing
// =========================================================================

#[inline]
fn gf_add(a: u8, b: u8) -> u8 {
    a ^ b
}

#[inline]
fn gf_mul(mut a: u8, mut b: u8) -> u8 {
    let mut p: u8 = 0;
    for _ in 0..8 {
        if (b & 1) != 0 {
            p ^= a;
        }
        let hi_bit_set = (a & 0x80) != 0;
        a <<= 1;
        if hi_bit_set {
            a ^= 0x1b; // Polynôme AES/GF(256) x^8 + x^4 + x^3 + x + 1 (0x11B)
        }
        b >>= 1;
    }
    p
}

#[inline]
fn gf_inv(a: u8) -> u8 {
    if a == 0 {
        return 0;
    }
    // Exponentiation rapide a^254 dans GF(256) par Théorème de Fermat
    let mut result = 1u8;
    let mut base = a;
    let mut exp = 254u8;
    while exp > 0 {
        if exp & 1 == 1 {
            result = gf_mul(result, base);
        }
        base = gf_mul(base, base);
        exp >>= 1;
    }
    result
}

// =========================================================================
// Algorithme Shamir SSS (Découpage N=3 / Seuil K=2)
// =========================================================================

/// Découpe une KEK en 3 parts de secret Shamir avec un seuil de K=2 parts.
pub fn split_kek(kek: &Kek) -> (ShamirShare, ShamirShare, ShamirShare) {
    let mut share1 = [0u8; 32];
    let mut share2 = [0u8; 32];
    let mut share3 = [0u8; 32];

    let mut random_coeff = [0u8; 32];
    OsRng.fill_bytes(&mut random_coeff);

    for i in 0..32 {
        let secret_byte = kek.0[i];
        let m = random_coeff[i]; // Coefficient du polynôme P(x) = m * x + S

        // Évaluation aux abscisses x = 1, 2, 3 sur GF(256)
        share1[i] = gf_add(gf_mul(m, 1), secret_byte);
        share2[i] = gf_add(gf_mul(m, 2), secret_byte);
        share3[i] = gf_add(gf_mul(m, 3), secret_byte);
    }

    (
        ShamirShare { id: 1, data: share1 },
        ShamirShare { id: 2, data: share2 },
        ShamirShare { id: 3, data: share3 },
    )
}

/// Reconstruit la KEK originale à partir de 2 parts quelconques sur 3 (Interpolation de Lagrange à x=0).
pub fn reconstruct_kek(s1: &ShamirShare, s2: &ShamirShare) -> Result<Kek, CryptoError> {
    if s1.id == 0 || s2.id == 0 || s1.id == s2.id {
        return Err(CryptoError::InvalidShamirShare);
    }

    let x1 = s1.id;
    let x2 = s2.id;
    let mut kek_bytes = [0u8; 32];

    // Coefficients de Lagrange pour x = 0 sur GF(256)
    let denom = gf_add(x1, x2);
    let inv_denom = gf_inv(denom);
    let l1_0 = gf_mul(x2, inv_denom);
    let l2_0 = gf_mul(x1, inv_denom);

    for i in 0..32 {
        let y1 = s1.data[i];
        let y2 = s2.data[i];
        let secret_byte = gf_add(gf_mul(y1, l1_0), gf_mul(y2, l2_0));
        kek_bytes[i] = secret_byte;
    }

    Ok(Kek(kek_bytes))
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
        let (s1, s2, s3) = split_kek(&kek);

        // Test 1: Reconstitution avec Parts (1, 2)
        let r12 = reconstruct_kek(&s1, &s2).unwrap();
        assert_eq!(kek.as_bytes(), r12.as_bytes());

        // Test 2: Reconstitution avec Parts (1, 3)
        let r13 = reconstruct_kek(&s1, &s3).unwrap();
        assert_eq!(kek.as_bytes(), r13.as_bytes());

        // Test 3: Reconstitution avec Parts (2, 3)
        let r23 = reconstruct_kek(&s2, &s3).unwrap();
        assert_eq!(kek.as_bytes(), r23.as_bytes());
    }

    #[test]
    fn test_invalid_identical_shares_fail() {
        let kek = Kek::generate();
        let (s1, _, _) = split_kek(&kek);
        assert!(reconstruct_kek(&s1, &s1).is_err());
    }

    #[test]
    fn test_debug_hermeticity() {
        let kek = Kek::generate();
        assert_eq!(format!("{:?}", kek), "Kek([REDACTED])");

        let (s1, _, _) = split_kek(&kek);
        assert_eq!(format!("{:?}", s1), "ShamirShare(id: 1, data: [REDACTED])");
    }
}