//! Récupération de la DEK après perte totale (refonte V3, étape 8, ADR-010).
//!
//! Deux facteurs, tous deux exigés : la passphrase du gérant et 2 parts papier
//! sur 3. Toute la cryptographie vient de `ss-crypto` (Argon2id, BLAKE2b,
//! X25519, vsss-rs) ; ce module ne fait que l'assembler, la stocker et la
//! présenter :
//!
//! - **blob** : la DEK scellée pour la clé publique de récupération, avec ce
//!   qu'il faut pour la rouvrir (sel, paramètres Argon2id) et une empreinte de
//!   la DEK. Déposé sur le relais (clé `recuperation`) et imprimé ;
//! - **parts** : 33 octets chacune, en base32 avec une somme de contrôle liée
//!   au jeu (le sel), pour détecter une faute de frappe dès la saisie ;
//! - **état** : table `recuperation` de la base `amane`, répliquée sur tous les
//!   serveurs (configurée, reportée ou absente).
//!
//! La passphrase, les parts et la KEK ne sont jamais stockées ni transmises.

use std::time::Duration;

use anyhow::{anyhow, bail, Context, Result};
use chrono::{DateTime, Utc};
use data_encoding::BASE32_NOPAD;
use rand::RngCore;
use sqlx::PgPool;
use ss_crypto::{
    deriver_kek, derive_recovery_key_avec, empreinte_dek, generate_salt, paire_de_recuperation,
    reconstruct_secret, somme_controle_part, split_secret, Dek, DevicePublicKey, ParametresArgon2,
    ShamirShare,
};
use uuid::Uuid;
use zeroize::Zeroizing;

/// Clé du blob de récupération sur le relais.
pub const CLE_BLOB: &str = "recuperation";
/// Longueur minimale de la passphrase du gérant.
pub const PASSPHRASE_MIN: usize = 12;
/// Version du format binaire du blob.
const VERSION_BLOB: u8 = 1;
/// Taille d'une DEK scellée (clé éphémère 32 + DEK 32 + tag 16).
const TAILLE_SCELLEE: usize = 80;
/// Jeu fictif pour la somme de contrôle du bloc imprimé.
const JEU_BLOC: &[u8] = b"bloc";

// ── Blob ──────────────────────────────────────────────────────────────────────

/// Blob de récupération : tout ce qu'il faut, avec la passphrase et deux
/// parts, pour rouvrir la DEK. Rien n'y est lisible sans les deux facteurs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Blob {
    pub sel: [u8; 16],
    pub argon2: ParametresArgon2,
    pub empreinte: [u8; 16],
    pub dek_scellee: Vec<u8>,
}

impl Blob {
    /// Scelle la DEK pour la clé publique de récupération.
    pub fn sceller(dek: &Dek, cle_publique: &[u8; 32], sel: [u8; 16], argon2: ParametresArgon2) -> Result<Self> {
        let dek_scellee = DevicePublicKey::from_bytes(*cle_publique)
            .seal_dek(dek)
            .map_err(|e| anyhow!("scellement de la DEK impossible : {e:?}"))?;
        let empreinte = empreinte_dek(dek).map_err(|e| anyhow!("empreinte : {e:?}"))?;
        Ok(Self { sel, argon2, empreinte, dek_scellee })
    }

    /// Format binaire : version, sel, m (u32 LE), t, p, empreinte, DEK scellée.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut o = Vec::with_capacity(1 + 16 + 6 + 16 + TAILLE_SCELLEE);
        o.push(VERSION_BLOB);
        o.extend_from_slice(&self.sel);
        o.extend_from_slice(&self.argon2.m.to_le_bytes());
        o.push(self.argon2.t as u8);
        o.push(self.argon2.p as u8);
        o.extend_from_slice(&self.empreinte);
        o.extend_from_slice(&self.dek_scellee);
        o
    }

    pub fn from_bytes(o: &[u8]) -> Result<Self> {
        if o.len() != 1 + 16 + 6 + 16 + TAILLE_SCELLEE {
            bail!("blob de récupération de taille inattendue ({} octets)", o.len());
        }
        if o[0] != VERSION_BLOB {
            bail!("version de blob de récupération inconnue : {}", o[0]);
        }
        let mut sel = [0u8; 16];
        sel.copy_from_slice(&o[1..17]);
        let m = u32::from_le_bytes([o[17], o[18], o[19], o[20]]);
        let argon2 = ParametresArgon2 { m, t: o[21] as u32, p: o[22] as u32 };
        let mut empreinte = [0u8; 16];
        empreinte.copy_from_slice(&o[23..39]);
        Ok(Self { sel, argon2, empreinte, dek_scellee: o[39..].to_vec() })
    }

    /// Identifiant du jeu de parts, imprimé sur chaque feuille.
    pub fn jeu(&self) -> String {
        identifiant_jeu(&self.sel)
    }

    /// Le blob protège-t-il cette DEK ?
    pub fn protege(&self, dek: &Dek) -> bool {
        empreinte_dek(dek).map(|e| e == self.empreinte).unwrap_or(false)
    }
}

pub fn identifiant_jeu(sel: &[u8; 16]) -> String {
    hex::encode_upper(&sel[..4])
}

// ── Texte imprimé ─────────────────────────────────────────────────────────────

/// Écrit des octets en base32, suivis d'une somme de contrôle liée au jeu,
/// par groupes de 4 caractères.
fn ecrire(jeu: &[u8], octets: &[u8]) -> String {
    let mut texte = BASE32_NOPAD.encode(octets);
    texte.push_str(&BASE32_NOPAD.encode(&somme_controle_part(jeu, octets)));
    texte
        .as_bytes()
        .chunks(4)
        .map(|g| std::str::from_utf8(g).unwrap_or_default())
        .collect::<Vec<_>>()
        .join("-")
}

/// Relit un texte saisi : tolère minuscules, espaces, tirets et les confusions
/// 0/O, 1/I, 8/B ; vérifie la somme de contrôle.
fn lire(jeu: &[u8], texte: &str) -> Result<Zeroizing<Vec<u8>>> {
    let propre: String = texte
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| match c.to_ascii_uppercase() {
            '0' => 'O',
            '1' => 'I',
            '8' => 'B',
            c => c,
        })
        .collect();
    if propre.len() <= 5 {
        bail!("saisie trop courte");
    }
    let (corps, controle) = propre.split_at(propre.len() - 5);
    let octets = Zeroizing::new(
        BASE32_NOPAD.decode(corps.as_bytes()).map_err(|_| anyhow!("caractères invalides ou saisie incomplète"))?,
    );
    let attendu = BASE32_NOPAD.encode(&somme_controle_part(jeu, &octets));
    if attendu != controle {
        bail!("somme de contrôle fausse : faute de frappe, ou part d'un autre jeu");
    }
    Ok(octets)
}

/// Texte imprimé d'une part.
pub fn ecrire_part(sel: &[u8; 16], part: &ShamirShare) -> String {
    ecrire(sel, &part.to_bytes())
}

/// Relit une part saisie, en vérifiant qu'elle appartient au jeu du blob.
pub fn lire_part(sel: &[u8; 16], texte: &str) -> Result<ShamirShare> {
    let octets = lire(sel, texte)?;
    ShamirShare::from_bytes(&octets).map_err(|_| anyhow!("part invalide"))
}

/// Texte imprimé du bloc (le blob), pour récupérer sans le relais.
pub fn ecrire_bloc(blob: &Blob) -> String {
    ecrire(JEU_BLOC, &blob.to_bytes())
}

pub fn lire_bloc(texte: &str) -> Result<Blob> {
    Blob::from_bytes(&lire(JEU_BLOC, texte)?)
}

// ── Configuration et reconstitution ───────────────────────────────────────────

/// Ce que la console imprime une seule fois : trois parts et le bloc.
pub struct Feuille {
    pub jeu: String,
    pub parts: Vec<String>,
    pub bloc: String,
    pub blob: Blob,
    pub cle_publique: [u8; 32],
}

/// Prépare la récupération : `K_p` de la passphrase, `K_s` aléatoire découpé
/// en 3 parts (2 suffisent), clé publique de récupération, DEK scellée.
/// Calcul Argon2id coûteux : à appeler hors de la boucle asynchrone.
pub fn configurer(dek: &Dek, passphrase: &str, argon2: ParametresArgon2) -> Result<Feuille> {
    verifier_passphrase(passphrase)?;
    let sel = generate_salt();
    let kp = derive_recovery_key_avec(passphrase, &sel, argon2).map_err(|e| anyhow!("Argon2id : {e:?}"))?;
    let mut ks = Zeroizing::new([0u8; 32]);
    rand::rngs::OsRng.fill_bytes(ks.as_mut());
    let parts = split_secret(&ks, 2, 3).map_err(|e| anyhow!("partage : {e:?}"))?;
    let kek = deriver_kek(&kp, &ks).map_err(|e| anyhow!("KEK : {e:?}"))?;
    let cle_publique = *paire_de_recuperation(&kek).public.as_bytes();
    let blob = Blob::sceller(dek, &cle_publique, sel, argon2)?;
    Ok(Feuille {
        jeu: blob.jeu(),
        parts: parts.iter().map(|p| ecrire_part(&sel, p)).collect(),
        bloc: ecrire_bloc(&blob),
        blob,
        cle_publique,
    })
}

pub fn verifier_passphrase(passphrase: &str) -> Result<()> {
    if passphrase.chars().count() < PASSPHRASE_MIN {
        bail!("la passphrase doit compter au moins {PASSPHRASE_MIN} caractères");
    }
    Ok(())
}

/// Reconstitue la DEK à partir du blob, de la passphrase et de deux parts.
pub fn recuperer(blob: &Blob, passphrase: &str, parts: &[ShamirShare]) -> Result<Dek> {
    let kp = derive_recovery_key_avec(passphrase, &blob.sel, blob.argon2).map_err(|e| anyhow!("Argon2id : {e:?}"))?;
    let ks = reconstruct_secret(parts).map_err(|_| anyhow!("parts insuffisantes ou en double"))?;
    let kek = deriver_kek(&kp, &ks).map_err(|e| anyhow!("KEK : {e:?}"))?;
    let dek = paire_de_recuperation(&kek)
        .open_sealed_dek(&blob.dek_scellee)
        .map_err(|_| anyhow!("passphrase ou parts incorrectes : la DEK ne s'ouvre pas"))?;
    if !blob.protege(&dek) {
        bail!("la DEK reconstituée ne correspond pas à l'empreinte du blob");
    }
    Ok(dek)
}

// ── État dans la base `amane` ─────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Etat {
    NonConfiguree,
    Reportee { par: String, le: DateTime<Utc> },
    Configuree { par: String, le: DateTime<Utc>, jeu: String, cle_publique: [u8; 32], sel: [u8; 16], argon2: ParametresArgon2 },
}

impl Etat {
    /// Valeur remontée au portail.
    pub fn code(&self) -> &'static str {
        match self {
            Etat::NonConfiguree => "non_configuree",
            Etat::Reportee { .. } => "reportee",
            Etat::Configuree { .. } => "configuree",
        }
    }
}

pub async fn migrer(pool: &PgPool) -> Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS recuperation (
            id           SMALLINT    PRIMARY KEY DEFAULT 1 CHECK (id = 1),
            etat         VARCHAR(12) NOT NULL CHECK (etat IN ('configuree', 'reportee')),
            cle_publique TEXT,
            sel          TEXT,
            argon2_m     INTEGER,
            argon2_t     INTEGER,
            argon2_p     INTEGER,
            par          TEXT        NOT NULL,
            le           TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
    )
    .execute(pool)
    .await?;
    Ok(())
}

type Ligne = (String, Option<String>, Option<String>, Option<i32>, Option<i32>, Option<i32>, String, DateTime<Utc>);

pub async fn etat(pool: &PgPool) -> Result<Etat> {
    let ligne: Option<Ligne> = sqlx::query_as(
        "SELECT etat, cle_publique, sel, argon2_m, argon2_t, argon2_p, par, le FROM recuperation WHERE id = 1",
    )
    .fetch_optional(pool)
    .await?;
    let Some((etat, cle, sel, m, t, p, par, le)) = ligne else {
        return Ok(Etat::NonConfiguree);
    };
    if etat == "reportee" {
        return Ok(Etat::Reportee { par, le });
    }
    let cle_publique: [u8; 32] = hex::decode(cle.unwrap_or_default())?
        .try_into()
        .map_err(|_| anyhow!("clé publique de récupération invalide"))?;
    let sel: [u8; 16] = hex::decode(sel.unwrap_or_default())?
        .try_into()
        .map_err(|_| anyhow!("sel de récupération invalide"))?;
    let argon2 = ParametresArgon2 {
        m: m.unwrap_or_default() as u32,
        t: t.unwrap_or_default() as u32,
        p: p.unwrap_or_default() as u32,
    };
    Ok(Etat::Configuree { par, le, jeu: identifiant_jeu(&sel), cle_publique, sel, argon2 })
}

pub async fn enregistrer_configuration(pool: &PgPool, feuille: &Feuille, par: &str) -> Result<()> {
    sqlx::query(
        "INSERT INTO recuperation (id, etat, cle_publique, sel, argon2_m, argon2_t, argon2_p, par, le)
         VALUES (1, 'configuree', $1, $2, $3, $4, $5, $6, NOW())
         ON CONFLICT (id) DO UPDATE SET etat = EXCLUDED.etat, cle_publique = EXCLUDED.cle_publique,
           sel = EXCLUDED.sel, argon2_m = EXCLUDED.argon2_m, argon2_t = EXCLUDED.argon2_t,
           argon2_p = EXCLUDED.argon2_p, par = EXCLUDED.par, le = EXCLUDED.le",
    )
    .bind(hex::encode(feuille.cle_publique))
    .bind(hex::encode(feuille.blob.sel))
    .bind(feuille.blob.argon2.m as i32)
    .bind(feuille.blob.argon2.t as i32)
    .bind(feuille.blob.argon2.p as i32)
    .bind(par)
    .execute(pool)
    .await?;
    Ok(())
}

/// Consigne le report. Sans effet si la récupération est déjà configurée.
pub async fn reporter(pool: &PgPool, par: &str) -> Result<()> {
    sqlx::query(
        "INSERT INTO recuperation (id, etat, par, le) VALUES (1, 'reportee', $1, NOW())
         ON CONFLICT (id) DO UPDATE SET par = EXCLUDED.par, le = EXCLUDED.le
         WHERE recuperation.etat = 'reportee'",
    )
    .bind(par)
    .execute(pool)
    .await?;
    Ok(())
}

// ── Relais ────────────────────────────────────────────────────────────────────

/// Où déposer le blob : relais, jeton éventuel et tenant.
#[derive(Clone, Debug)]
pub struct Relais {
    pub url: String,
    pub jeton: String,
    pub tenant_id: Uuid,
}

impl Relais {
    pub fn depuis_env(url: &str, tenant_id: Option<Uuid>) -> Option<Self> {
        Some(Self {
            url: url.to_string(),
            jeton: std::env::var("RELAY_AUTH_TOKEN").unwrap_or_default(),
            tenant_id: tenant_id?,
        })
    }

    /// Même relais, jeton et tenant que les sauvegardes.
    pub fn depuis_cible(cible: &crate::sauvegarde::Cible) -> Self {
        Self { url: cible.relais.clone(), jeton: cible.jeton_relais.clone(), tenant_id: cible.tenant_id }
    }

    fn url_blob(&self) -> String {
        format!("{}/api/blobs/{}/{}", self.url.trim_end_matches('/'), self.tenant_id, CLE_BLOB)
    }

    fn requete(&self, r: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if self.jeton.is_empty() { r } else { r.header("x-relay-token", &self.jeton) }
    }

    fn client() -> Result<reqwest::Client> {
        Ok(reqwest::Client::builder().timeout(Duration::from_secs(15)).build()?)
    }

    pub async fn deposer(&self, blob: &Blob) -> Result<()> {
        self.requete(Self::client()?.put(self.url_blob()).body(blob.to_bytes()))
            .send()
            .await
            .context("relais injoignable")?
            .error_for_status()
            .context("dépôt du blob de récupération refusé par le relais")?;
        Ok(())
    }

    /// Lit le blob du relais ; `None` s'il n'y en a pas.
    pub async fn lire(&self) -> Result<Option<Blob>> {
        let reponse = self.requete(Self::client()?.get(self.url_blob())).send().await.context("relais injoignable")?;
        if reponse.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }
        let octets = reponse.error_for_status().context("lecture du blob de récupération refusée")?.bytes().await?;
        Ok(Some(Blob::from_bytes(&octets)?))
    }
}

// ── Garde des anciennes sauvegardes (ADR-010, décision 7) ─────────────────────

/// Un primaire ne dépose aucune sauvegarde si le relais détient un blob de
/// récupération pour une autre DEK : c'est un cluster réinstallé après une
/// perte totale, qui écraserait les sauvegardes du jour avec une base vide.
pub async fn garde_sauvegarde(dek: &Dek, relais: &Relais) -> Result<()> {
    match relais.lire().await? {
        Some(blob) if !blob.protege(dek) => bail!(
            "le relais détient la récupération d'une autre DEK : cluster réinstallé après une perte totale ? \
             Sauvegardes suspendues pour protéger les anciennes ; lancez `ss-node recuperer`"
        ),
        _ => Ok(()),
    }
}

/// Resscelle la DEK pour la clé publique enregistrée et redépose le blob : tout
/// serveur qui détient la DEK le peut, sans la passphrase (ADR-010, décision 5).
pub async fn republier(pool: &PgPool, dek: &Dek, relais: &Relais) -> Result<bool> {
    let Etat::Configuree { cle_publique, sel, argon2, .. } = etat(pool).await? else {
        return Ok(false);
    };
    relais.deposer(&Blob::sceller(dek, &cle_publique, sel, argon2)?).await?;
    Ok(true)
}

/// État remonté au portail : celui de la base, ou `dek_differente` quand les
/// sauvegardes sont suspendues par la garde.
pub async fn code_pour_portail(pool: &PgPool, dek: Option<&Dek>, relais: Option<&Relais>) -> Option<&'static str> {
    if let (Some(dek), Some(relais)) = (dek, relais) {
        if let Ok(Some(blob)) = relais.lire().await {
            if !blob.protege(dek) {
                return Some("dek_differente");
            }
        }
    }
    etat(pool).await.ok().map(|e| e.code())
}

#[cfg(test)]
mod tests {
    use super::*;

    const LEGERS: ParametresArgon2 = ParametresArgon2 { m: 1024, t: 1, p: 1 };
    const PASSE: &str = "cheval correct agrafe batterie";

    #[test]
    fn configuration_puis_recuperation_avec_deux_parts_quelconques() {
        let dek = Dek::generate();
        let f = configurer(&dek, PASSE, LEGERS).unwrap();
        assert_eq!(f.parts.len(), 3);
        let blob = lire_bloc(&f.bloc).unwrap();
        assert_eq!(blob, f.blob);
        for (a, b) in [(0, 1), (0, 2), (2, 1)] {
            let parts = [lire_part(&blob.sel, &f.parts[a]).unwrap(), lire_part(&blob.sel, &f.parts[b]).unwrap()];
            assert_eq!(recuperer(&blob, PASSE, &parts).unwrap().as_bytes(), dek.as_bytes());
        }
    }

    #[test]
    fn mauvaise_passphrase_ou_une_seule_part() {
        let dek = Dek::generate();
        let f = configurer(&dek, PASSE, LEGERS).unwrap();
        let p0 = lire_part(&f.blob.sel, &f.parts[0]).unwrap();
        let p1 = lire_part(&f.blob.sel, &f.parts[1]).unwrap();
        assert!(recuperer(&f.blob, "une autre passphrase", &[p0.clone(), p1]).is_err());
        assert!(recuperer(&f.blob, PASSE, &[p0.clone()]).is_err());
        assert!(recuperer(&f.blob, PASSE, &[p0.clone(), p0]).is_err());
    }

    #[test]
    fn saisie_tolerante_mais_controlee() {
        let dek = Dek::generate();
        let f = configurer(&dek, PASSE, LEGERS).unwrap();
        let sel = f.blob.sel;
        // Minuscules, espaces au lieu des tirets : accepté.
        let tolere = f.parts[0].to_lowercase().replace('-', " ");
        assert!(lire_part(&sel, &tolere).is_ok());
        // Un caractère changé : refusé par la somme de contrôle.
        let mut faute: Vec<char> = f.parts[0].chars().collect();
        faute[0] = if faute[0] == 'A' { 'C' } else { 'A' };
        assert!(lire_part(&sel, &faute.iter().collect::<String>()).is_err());
        // Une part d'un autre jeu : refusée.
        let autre = configurer(&dek, PASSE, LEGERS).unwrap();
        assert!(lire_part(&sel, &autre.parts[0]).is_err());
    }

    #[test]
    fn le_blob_ne_protege_que_sa_dek() {
        let dek = Dek::generate();
        let f = configurer(&dek, PASSE, LEGERS).unwrap();
        assert!(f.blob.protege(&dek));
        assert!(!f.blob.protege(&Dek::generate()));
        // Rescellé pour la même clé publique : toujours ouvrable avec les mêmes facteurs.
        let rescelle = Blob::sceller(&dek, &f.cle_publique, f.blob.sel, f.blob.argon2).unwrap();
        assert_ne!(rescelle.dek_scellee, f.blob.dek_scellee);
        let parts = [lire_part(&f.blob.sel, &f.parts[1]).unwrap(), lire_part(&f.blob.sel, &f.parts[2]).unwrap()];
        assert_eq!(recuperer(&rescelle, PASSE, &parts).unwrap().as_bytes(), dek.as_bytes());
    }

    #[test]
    fn passphrase_trop_courte_refusee() {
        assert!(configurer(&Dek::generate(), "court", LEGERS).is_err());
    }

    #[test]
    fn format_binaire_et_parametres_conserves() {
        let f = configurer(&Dek::generate(), PASSE, ParametresArgon2 { m: 2048, t: 2, p: 1 }).unwrap();
        let relu = Blob::from_bytes(&f.blob.to_bytes()).unwrap();
        assert_eq!(relu.argon2, ParametresArgon2 { m: 2048, t: 2, p: 1 });
        assert_eq!(f.blob.to_bytes().len(), 119);
        assert!(Blob::from_bytes(&f.blob.to_bytes()[..100]).is_err());
        let mut v2 = f.blob.to_bytes();
        v2[0] = 2;
        assert!(Blob::from_bytes(&v2).is_err());
    }

    #[test]
    fn rien_de_secret_dans_le_blob() {
        let dek = Dek::generate();
        let f = configurer(&dek, PASSE, LEGERS).unwrap();
        let octets = f.blob.to_bytes();
        assert!(!octets.windows(32).any(|w| w == dek.as_bytes()));
        assert!(!octets.windows(PASSE.len()).any(|w| w == PASSE.as_bytes()));
    }
}
