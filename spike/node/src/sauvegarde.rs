//! Sauvegardes chiffrées (refonte V3, étape 4, ADR-003 / ADR-008).
//!
//! Le primaire exporte chaque base (`pg_dump`), chiffre l'export avec la DEK
//! du cluster (`ss_crypto::Dek`, aucune primitive maison) et le dépose sur le
//! relais zero-knowledge, qui ne stocke qu'un blob opaque. La DEK ne quitte
//! jamais le cluster : sans elle, le blob est illisible pour le relais comme
//! pour l'éditeur.
//!
//! Rotation sur 7 jours sans liste ni suppression côté relais : la clé du
//! blob porte le jour de la semaine (`sauvegarde-<base>-<1..7>`).

use std::process::Stdio;
use std::time::Duration;

use anyhow::{anyhow, bail, Context, Result};
use chrono::{Datelike, Utc};
use ss_crypto::Dek;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use uuid::Uuid;

/// Fichier des secrets PostgreSQL de la machine (pile V3).
pub const SECRETS_PG: &str = "/amane/secrets/postgresql.env";
/// Présent si le cluster héberge une application avec base (étape 3).
pub const SECRETS_APPLICATION: &str = "/amane/secrets/application.env";

/// Où se trouvent les bases et le relais.
#[derive(Clone)]
pub struct Cible {
    /// URL du relais (ex. `http://relais:8080`).
    pub relais: String,
    /// Jeton du relais (`RELAY_AUTH_TOKEN`), vide si le relais n'en exige pas.
    pub jeton_relais: String,
    pub tenant_id: Uuid,
    /// Connexion superutilisateur au primaire, sans base : `postgres://u:p@h:port`.
    pub pg_base_url: String,
}

/// Cible construite depuis la pile V3 : relais et tenant de la configuration,
/// jeton `RELAY_AUTH_TOKEN`, primaire joint par HAProxy (`AMANE_PG_PRIMAIRE`,
/// `amane-haproxy:5000` par défaut) avec le mot de passe superutilisateur.
pub fn cible_depuis_env(relais: &str, tenant_id: Option<Uuid>) -> Result<Cible> {
    let tenant_id = tenant_id.ok_or_else(|| anyhow!("TENANT_ID inconnu : sauvegardes impossibles"))?;
    let hote = std::env::var("AMANE_PG_PRIMAIRE").unwrap_or_else(|_| "amane-haproxy:5000".into());
    Ok(Cible {
        relais: relais.to_string(),
        jeton_relais: std::env::var("RELAY_AUTH_TOKEN").unwrap_or_default(),
        tenant_id,
        pg_base_url: url_superutilisateur(SECRETS_PG, &hote)?,
    })
}

/// Bases à sauvegarder : celle du nœud, et celle de l'application hébergée
/// si le cluster en a une.
pub fn bases_a_sauvegarder() -> Vec<&'static str> {
    let mut bases = vec!["amane"];
    if std::path::Path::new(SECRETS_APPLICATION).exists() {
        bases.push("application");
    }
    bases
}

/// Clé du blob d'une base pour un jour de la semaine (1 = lundi … 7).
pub fn cle_blob(base: &str, jour: u32) -> Result<String> {
    if !(1..=7).contains(&jour) {
        bail!("jour hors de 1..7 : {jour}");
    }
    if base.is_empty() || !base.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_') {
        bail!("nom de base refusé : {base:?}");
    }
    Ok(format!("sauvegarde-{base}-{jour}"))
}

/// Jour de la semaine courant (UTC), 1 = lundi.
pub fn jour_courant() -> u32 {
    Utc::now().weekday().number_from_monday()
}

fn url_base(cible: &Cible, base: &str) -> String {
    format!("{}/{}", cible.pg_base_url.trim_end_matches('/'), base)
}

fn url_blob(cible: &Cible, cle: &str) -> String {
    format!("{}/api/blobs/{}/{}", cible.relais.trim_end_matches('/'), cible.tenant_id, cle)
}

fn client() -> Result<reqwest::Client> {
    Ok(reqwest::Client::builder().timeout(Duration::from_secs(600)).build()?)
}

/// Exporte, chiffre et dépose une base. Renvoie la clé et la taille du blob.
pub async fn sauvegarder(dek: &Dek, cible: &Cible, base: &str) -> Result<(String, usize)> {
    let cle = cle_blob(base, jour_courant())?;
    let sortie = Command::new("pg_dump")
        .args(["--format=custom", "--no-owner", "--dbname", &url_base(cible, base)])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .context("pg_dump introuvable dans l'image du nœud")?;
    if !sortie.status.success() {
        bail!("pg_dump {base} : {}", String::from_utf8_lossy(&sortie.stderr).trim());
    }
    let chiffre = dek.encrypt(&sortie.stdout).map_err(|e| anyhow!("chiffrement : {e:?}"))?;
    let taille = chiffre.len();
    let mut req = client()?.put(url_blob(cible, &cle)).body(chiffre);
    if !cible.jeton_relais.is_empty() {
        req = req.header("x-relay-token", &cible.jeton_relais);
    }
    req.send().await.context("relais injoignable")?.error_for_status().context("dépôt refusé par le relais")?;
    Ok((cle, taille))
}

/// Récupère, déchiffre et restaure une base depuis le relais.
pub async fn restaurer(dek: &Dek, cible: &Cible, base: &str, jour: u32) -> Result<()> {
    let cle = cle_blob(base, jour)?;
    let mut req = client()?.get(url_blob(cible, &cle));
    if !cible.jeton_relais.is_empty() {
        req = req.header("x-relay-token", &cible.jeton_relais);
    }
    let chiffre = req
        .send()
        .await
        .context("relais injoignable")?
        .error_for_status()
        .with_context(|| format!("sauvegarde {cle} introuvable sur le relais"))?
        .bytes()
        .await?;
    let export = dek.decrypt(&chiffre).map_err(|e| anyhow!("déchiffrement impossible (mauvaise DEK ?) : {e:?}"))?;

    let mut enfant = Command::new("pg_restore")
        .args(["--clean", "--if-exists", "--no-owner", "--dbname", &url_base(cible, base)])
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("pg_restore introuvable dans l'image du nœud")?;
    let mut entree = enfant.stdin.take().ok_or_else(|| anyhow!("entrée de pg_restore indisponible"))?;
    entree.write_all(&export).await?;
    drop(entree);
    let fin = enfant.wait_with_output().await?;
    if !fin.status.success() {
        bail!("pg_restore {base} : {}", String::from_utf8_lossy(&fin.stderr).trim());
    }
    Ok(())
}

/// Connexion superutilisateur au primaire, à partir du fichier de secrets de
/// la machine (`PGPASSWORD_SUPERUSER`) : `postgres://postgres:…@amane-haproxy:5000`.
pub fn url_superutilisateur(fichier_secrets: &str, hote: &str) -> Result<String> {
    let contenu = std::fs::read_to_string(fichier_secrets)
        .with_context(|| format!("secrets {fichier_secrets} illisibles"))?;
    let mdp = contenu
        .lines()
        .filter_map(|l| l.trim().strip_prefix("export PGPASSWORD_SUPERUSER="))
        .map(|v| v.trim_matches('\'').to_string())
        .next()
        .ok_or_else(|| anyhow!("PGPASSWORD_SUPERUSER absent de {fichier_secrets}"))?;
    Ok(format!("postgres://postgres:{mdp}@{hote}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cles_tournantes() {
        assert_eq!(cle_blob("amane", 1).unwrap(), "sauvegarde-amane-1");
        assert_eq!(cle_blob("application", 7).unwrap(), "sauvegarde-application-7");
        assert!(cle_blob("amane", 0).is_err());
        assert!(cle_blob("amane", 8).is_err());
        assert!(cle_blob("../x", 1).is_err());
        assert!(cle_blob("", 1).is_err());
        assert!((1..=7).contains(&jour_courant()));
    }

    #[test]
    fn url_superutilisateur_depuis_les_secrets() {
        let f = std::env::temp_dir().join(format!("secrets-{}.env", Uuid::new_v4()));
        std::fs::write(&f, "# en-tête\nexport PGPASSWORD_SUPERUSER='abc-123'\nexport X='y'\n").unwrap();
        let url = url_superutilisateur(f.to_str().unwrap(), "amane-haproxy:5000").unwrap();
        assert_eq!(url, "postgres://postgres:abc-123@amane-haproxy:5000");
        std::fs::write(&f, "export X='y'\n").unwrap();
        assert!(url_superutilisateur(f.to_str().unwrap(), "h:1").is_err());
        let _ = std::fs::remove_file(f);
    }

    #[test]
    fn urls() {
        let c = Cible {
            relais: "http://relais:8080/".into(),
            jeton_relais: String::new(),
            tenant_id: Uuid::nil(),
            pg_base_url: "postgres://postgres:p@h:5000/".into(),
        };
        assert_eq!(url_base(&c, "amane"), "postgres://postgres:p@h:5000/amane");
        assert_eq!(
            url_blob(&c, "sauvegarde-amane-2"),
            "http://relais:8080/api/blobs/00000000-0000-0000-0000-000000000000/sauvegarde-amane-2"
        );
    }

    #[test]
    fn le_blob_depose_est_illisible_sans_la_dek() {
        // Ce que le relais stocke : un chiffré que seule la DEK du cluster ouvre.
        let dek = Dek::generate();
        let export = b"PGDMP export de la base".to_vec();
        let chiffre = dek.encrypt(&export).unwrap();
        assert!(!chiffre.windows(5).any(|w| w == b"PGDMP"));
        assert_eq!(dek.decrypt(&chiffre).unwrap(), export);
        assert!(Dek::generate().decrypt(&chiffre).is_err());
    }
}
