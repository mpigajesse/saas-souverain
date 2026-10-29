//! Enrôlement d'un serveur (refonte V3, étape 4, ADR-008) : transmission de la
//! DEK du cluster à un serveur qui ne l'a pas, par une opération HUMAINE des
//! deux côtés (règle « les opérations sur les clés restent humaines ») :
//!
//! 1. sur un serveur qui détient la DEK, un administrateur génère un code à
//!    usage unique dans la console du nœud (`/amane/admin/enrolement`) ;
//! 2. sur le nouveau serveur, il lance `ss-node enroler --depuis … --code …` ;
//! 3. le premier scelle la DEK pour la clé publique du second
//!    (`DevicePublicKey::seal_dek`) : seul le nouveau serveur peut l'ouvrir,
//!    elle ne circule jamais en clair.

use std::sync::Mutex;
use std::time::{Duration, Instant};

use anyhow::{anyhow, Context, Result};
use rand::Rng;
use serde::{Deserialize, Serialize};
use ss_crypto::{Dek, DevicePublicKey};

/// Durée de validité d'un code.
pub const VALIDITE: Duration = Duration::from_secs(15 * 60);
/// Au-delà, tous les codes en cours sont annulés (essais de devinette).
const ECHECS_MAX: u32 = 5;
/// Alphabet sans caractères ambigus (0/O, 1/I/L).
const ALPHABET: &[u8] = b"ABCDEFGHJKMNPQRSTUVWXYZ23456789";

/// Codes d'enrôlement émis par ce nœud (en mémoire : un code se consomme sur
/// le nœud qui l'a émis).
#[derive(Default)]
pub struct Invitations {
    codes: Mutex<Vec<(String, Instant)>>,
    echecs: Mutex<u32>,
}

/// Forme canonique d'un code saisi : majuscules, sans espaces ni tirets.
pub fn normaliser(code: &str) -> String {
    code.chars().filter(|c| c.is_ascii_alphanumeric()).map(|c| c.to_ascii_uppercase()).collect()
}

impl Invitations {
    /// Émet un code de 16 caractères (~79 bits), affiché par groupes de 4.
    pub fn emettre(&self) -> String {
        let mut rng = rand::rngs::OsRng;
        let brut: String = (0..16).map(|_| ALPHABET[rng.gen_range(0..ALPHABET.len())] as char).collect();
        let mut codes = self.codes.lock().unwrap();
        codes.retain(|(_, emis)| emis.elapsed() < VALIDITE);
        codes.push((brut.clone(), Instant::now()));
        brut.as_bytes().chunks(4).map(|g| std::str::from_utf8(g).unwrap()).collect::<Vec<_>>().join("-")
    }

    /// Consomme un code valide (usage unique). Après `ECHECS_MAX` codes faux,
    /// tous les codes en cours sont annulés.
    pub fn consommer(&self, code: &str) -> bool {
        let code = normaliser(code);
        let mut codes = self.codes.lock().unwrap();
        codes.retain(|(_, emis)| emis.elapsed() < VALIDITE);
        if let Some(i) = codes.iter().position(|(c, _)| *c == code) {
            codes.remove(i);
            *self.echecs.lock().unwrap() = 0;
            return true;
        }
        let mut echecs = self.echecs.lock().unwrap();
        *echecs += 1;
        if *echecs >= ECHECS_MAX {
            codes.clear();
            *echecs = 0;
        }
        false
    }
}

/// Demande envoyée par le serveur à enrôler.
#[derive(Debug, Serialize, Deserialize)]
pub struct Demande {
    pub code: String,
    /// Clé publique X25519 du nouveau serveur, en hexadécimal.
    pub cle_publique: String,
}

/// Réponse : la DEK scellée pour cette seule clé publique.
#[derive(Debug, Serialize, Deserialize)]
pub struct Reponse {
    pub dek_scellee: String,
}

/// Scelle la DEK pour une clé publique fournie en hexadécimal.
pub fn sceller_pour(dek: &Dek, cle_publique_hex: &str) -> Result<String> {
    let octets: [u8; 32] = hex::decode(cle_publique_hex.trim())
        .context("clé publique : hexadécimal attendu")?
        .try_into()
        .map_err(|_| anyhow!("clé publique : 32 octets attendus"))?;
    let scellee = DevicePublicKey::from_bytes(octets)
        .seal_dek(dek)
        .map_err(|e| anyhow!("scellement impossible : {e:?}"))?;
    Ok(hex::encode(scellee))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ss_crypto::DeviceKeyPair;

    #[test]
    fn code_a_usage_unique() {
        let inv = Invitations::default();
        let code = inv.emettre();
        assert_eq!(code.len(), 19, "4 groupes de 4 séparés par des tirets");
        // Saisie tolérante (minuscules, espaces).
        assert!(inv.consommer(&code.to_lowercase().replace('-', " ")));
        assert!(!inv.consommer(&code), "un code ne sert qu'une fois");
    }

    #[test]
    fn trop_d_echecs_annulent_les_codes() {
        let inv = Invitations::default();
        let code = inv.emettre();
        for _ in 0..ECHECS_MAX {
            assert!(!inv.consommer("AAAA-BBBB-CCCC-DDDD"));
        }
        assert!(!inv.consommer(&code), "le code valide a été annulé après trop d'échecs");
    }

    #[test]
    fn la_dek_scellee_ne_s_ouvre_que_chez_le_destinataire() {
        let dek = Dek::generate();
        let nouveau = DeviceKeyPair::generate();
        let scellee = sceller_pour(&dek, &hex::encode(nouveau.public.as_bytes())).unwrap();
        let recue = nouveau.open_sealed_dek(&hex::decode(&scellee).unwrap()).unwrap();
        assert_eq!(recue.as_bytes(), dek.as_bytes());
        // Une autre machine ne peut pas l'ouvrir.
        assert!(DeviceKeyPair::generate().open_sealed_dek(&hex::decode(&scellee).unwrap()).is_err());
    }

    #[test]
    fn cle_publique_invalide() {
        let dek = Dek::generate();
        assert!(sceller_pour(&dek, "pas-hex").is_err());
        assert!(sceller_pour(&dek, "abcd").is_err());
    }
}
