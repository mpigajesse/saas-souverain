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
//!
//! Chaque blob est lié à son nom (étape 9, ADR-011) : il commence par un
//! en-tête en clair `amane-sauvegarde/1 <tenant> <base> <date>`, que le
//! chiffrement authentifie comme donnée associée. La restauration refuse un
//! blob déplacé (autre jour, autre base, autre tenant) au lieu de rendre
//! silencieusement d'anciennes données, et affiche la date de la sauvegarde.

use std::process::Stdio;
use std::time::Duration;

use anyhow::{anyhow, bail, Context, Result};
use chrono::{Datelike, NaiveDate, Utc};
use ss_crypto::Dek;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use uuid::Uuid;

/// Fichier des secrets PostgreSQL de la machine (pile V3).
pub const SECRETS_PG: &str = "/amane/secrets/postgresql.env";
/// Variable posée par la pile quand l'application hébergée a une base
/// (étape 3) : son nom. Le nœud ne lit pas les identifiants de l'application,
/// réservés à son conteneur (volume `secrets-application`).
pub const VARIABLE_BASE_APPLICATION: &str = "AMANE_APP_BASE";

/// Où se trouvent les bases et le relais.
#[derive(Clone)]
pub struct Cible {
    /// URL du relais (ex. `http://relais:8080`).
    pub relais: String,
    /// Jeton du relais propre au tenant (`RELAY_AUTH_TOKEN`, ADR-011), vide
    /// si le relais est en mode ouvert.
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
        jeton_relais: crate::relay_client::jeton_depuis_env(),
        tenant_id,
        pg_base_url: url_superutilisateur(SECRETS_PG, &hote)?,
    })
}

/// Bases à sauvegarder : celle du nœud, et celle de l'application hébergée
/// si le cluster en a une.
pub fn bases_a_sauvegarder() -> Vec<&'static str> {
    bases_pour(std::env::var(VARIABLE_BASE_APPLICATION).ok().as_deref())
}

fn bases_pour(base_application: Option<&str>) -> Vec<&'static str> {
    let mut bases = vec!["amane"];
    if base_application == Some("application") {
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

// ── Liaison du blob à son nom (ADR-011) ───────────────────────────────────────

/// Format des blobs de sauvegarde ; les blobs antérieurs à l'étape 9 n'ont
/// pas d'en-tête et sont refusés.
const FORMAT: &str = "amane-sauvegarde/1";
/// Longueur maximale de l'en-tête : format, UUID, nom de base, date.
const ENTETE_MAX: usize = 160;

fn entete(tenant_id: Uuid, base: &str, date: NaiveDate) -> String {
    format!("{FORMAT} {tenant_id} {base} {date}\n")
}

/// Blob déposé sur le relais : en-tête en clair, puis l'export chiffré par la
/// DEK avec l'en-tête pour donnée associée.
pub fn emballer(dek: &Dek, tenant_id: Uuid, base: &str, date: NaiveDate, export: &[u8]) -> Result<Vec<u8>> {
    let entete = entete(tenant_id, base, date);
    let chiffre = dek
        .encrypt_avec_contexte(export, entete.as_bytes())
        .map_err(|e| anyhow!("chiffrement : {e:?}"))?;
    let mut blob = Vec::with_capacity(entete.len() + chiffre.len());
    blob.extend_from_slice(entete.as_bytes());
    blob.extend_from_slice(&chiffre);
    Ok(blob)
}

/// Vérifie que le blob est bien la sauvegarde attendue (tenant, base, jour de
/// sa clé) et le déchiffre. Renvoie la date de la sauvegarde et l'export.
pub fn deballer(dek: &Dek, blob: &[u8], tenant_id: Uuid, base: &str, jour: u32) -> Result<(NaiveDate, Vec<u8>)> {
    let fin = blob
        .iter()
        .take(ENTETE_MAX)
        .position(|&o| o == b'\n')
        .ok_or_else(|| anyhow!("blob sans en-tête : format antérieur à l'étape 9, ou blob corrompu"))?;
    let (entete, chiffre) = blob.split_at(fin + 1);
    let texte = std::str::from_utf8(&entete[..fin]).map_err(|_| anyhow!("en-tête de sauvegarde illisible"))?;
    let champs: Vec<&str> = texte.split(' ').collect();
    let [format, tenant, base_lue, date] = champs[..] else {
        bail!("en-tête de sauvegarde mal formé : {texte:?}");
    };
    if format != FORMAT {
        bail!("format de sauvegarde inconnu : {format:?}");
    }
    if tenant != tenant_id.to_string() {
        bail!("blob d'un autre tenant ({tenant}) : refusé");
    }
    if base_lue != base {
        bail!("blob de la base {base_lue:?} déposé sous le nom de la base {base:?} : refusé");
    }
    let date: NaiveDate = date.parse().map_err(|_| anyhow!("date de sauvegarde illisible : {date:?}"))?;
    let jour_lu = date.weekday().number_from_monday();
    if jour_lu != jour {
        bail!("sauvegarde du {date} (jour {jour_lu}) déposée sous le nom du jour {jour} : refusée");
    }
    let export = dek
        .decrypt_avec_contexte(chiffre, entete)
        .map_err(|e| anyhow!("déchiffrement impossible (mauvaise DEK, ou en-tête modifié) : {e:?}"))?;
    Ok((date, export))
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
    let date = Utc::now().date_naive();
    let cle = cle_blob(base, date.weekday().number_from_monday())?;
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
    let blob = emballer(dek, cible.tenant_id, base, date, &sortie.stdout)?;
    let taille = blob.len();
    let req = client()?.put(url_blob(cible, &cle)).body(blob);
    crate::relay_client::avec_jeton(req, &cible.jeton_relais)
        .send().await.context("relais injoignable")?.error_for_status().context("dépôt refusé par le relais")?;
    Ok((cle, taille))
}

/// Récupère, vérifie, déchiffre et restaure une base depuis le relais.
/// Renvoie la date de la sauvegarde restaurée.
pub async fn restaurer(dek: &Dek, cible: &Cible, base: &str, jour: u32) -> Result<NaiveDate> {
    let cle = cle_blob(base, jour)?;
    let req = client()?.get(url_blob(cible, &cle));
    let blob = crate::relay_client::avec_jeton(req, &cible.jeton_relais)
        .send()
        .await
        .context("relais injoignable")?
        .error_for_status()
        .with_context(|| format!("sauvegarde {cle} introuvable sur le relais"))?
        .bytes()
        .await?;
    let (date, export) = deballer(dek, &blob, cible.tenant_id, base, jour)?;

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
    Ok(date)
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
    fn base_de_l_application_sauvegardee_si_la_pile_l_annonce() {
        assert_eq!(bases_pour(None), vec!["amane"]);
        assert_eq!(bases_pour(Some("application")), vec!["amane", "application"]);
        // Aucun autre nom : la variable ne choisit pas une base arbitraire.
        assert_eq!(bases_pour(Some("postgres")), vec!["amane"]);
    }

    #[test]
    fn cles_tournantes() {
        assert_eq!(cle_blob("amane", 1).unwrap(), "sauvegarde-amane-1");
        assert_eq!(cle_blob("application", 7).unwrap(), "sauvegarde-application-7");
        assert!(cle_blob("amane", 0).is_err());
        assert!(cle_blob("amane", 8).is_err());
        assert!(cle_blob("../x", 1).is_err());
        assert!(cle_blob("", 1).is_err());
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

    fn date(s: &str) -> NaiveDate {
        s.parse().unwrap()
    }

    #[test]
    fn le_blob_depose_est_illisible_sans_la_dek() {
        // Ce que le relais stocke : un chiffré que seule la DEK du cluster ouvre.
        let dek = Dek::generate();
        let tenant = Uuid::new_v4();
        let export = b"PGDMP export de la base".to_vec();
        let lundi = date("2026-11-02");
        let blob = emballer(&dek, tenant, "amane", lundi, &export).unwrap();
        assert!(!blob.windows(5).any(|w| w == b"PGDMP"));
        assert_eq!(deballer(&dek, &blob, tenant, "amane", 1).unwrap(), (lundi, export));
        assert!(deballer(&Dek::generate(), &blob, tenant, "amane", 1).is_err());
    }

    #[test]
    fn blob_deplace_refuse() {
        let dek = Dek::generate();
        let tenant = Uuid::new_v4();
        // Export du lundi 2 novembre 2026, base amane.
        let blob = emballer(&dek, tenant, "amane", date("2026-11-02"), b"PGDMP lundi").unwrap();
        // Déposé sous le nom du jeudi, d'une autre base ou d'un autre tenant.
        let jeudi = deballer(&dek, &blob, tenant, "amane", 4).unwrap_err().to_string();
        assert!(jeudi.contains("jour 1") && jeudi.contains("jour 4"), "{jeudi}");
        assert!(deballer(&dek, &blob, tenant, "application", 1).unwrap_err().to_string().contains("base"));
        assert!(deballer(&dek, &blob, Uuid::new_v4(), "amane", 1).unwrap_err().to_string().contains("tenant"));
    }

    #[test]
    fn en_tete_falsifie_refuse() {
        // Réécrire l'en-tête pour qu'il corresponde au nouveau nom ne suffit
        // pas : il est authentifié par le chiffrement.
        let dek = Dek::generate();
        let tenant = Uuid::new_v4();
        let blob = emballer(&dek, tenant, "amane", date("2026-11-02"), b"PGDMP lundi").unwrap();
        let ancien = entete(tenant, "amane", date("2026-11-02"));
        let faux = entete(tenant, "amane", date("2026-11-05"));
        assert_eq!(ancien.len(), faux.len());
        let mut falsifie = faux.into_bytes();
        falsifie.extend_from_slice(&blob[ancien.len()..]);
        let erreur = deballer(&dek, &falsifie, tenant, "amane", 4).unwrap_err().to_string();
        assert!(erreur.contains("déchiffrement impossible"), "{erreur}");
    }

    #[test]
    fn ancien_format_refuse() {
        // Blob d'avant l'étape 9 : chiffré nu, sans en-tête.
        let dek = Dek::generate();
        let ancien = dek.encrypt(&[0u8; 400]).unwrap();
        assert!(deballer(&dek, &ancien, Uuid::new_v4(), "amane", 1).is_err());
        assert!(deballer(&dek, b"", Uuid::new_v4(), "amane", 1).is_err());
    }
}
