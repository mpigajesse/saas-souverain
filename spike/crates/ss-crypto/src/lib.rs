mod error;
mod dek;
mod ak;
mod crl;
mod shamir;
mod device_key;
mod recovery;

pub use error::CryptoError;
pub use dek::Dek;
pub use ak::{AccessKeyPair, AccessPublicKey, SecretAccessKey};
pub use crl::CrlRegistry;
pub use shamir::{Kek, ShamirShare, split_kek, reconstruct_kek, split_secret, reconstruct_secret, TAILLE_SECRET};
pub use device_key::{DeviceKeyPair, DevicePublicKey};
pub use recovery::{
    derive_recovery_key, derive_recovery_key_avec, deriver_kek, empreinte_dek, generate_salt,
    paire_de_recuperation, somme_controle_part, ParametresArgon2,
};