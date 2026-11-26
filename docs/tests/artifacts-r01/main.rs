//! R-03 — copier un blob chiffré de `sauvegarde-amane-1` vers
//! `sauvegarde-application-1`, puis restaurer. Attendu : déchiffrement en
//! erreur (l'AAD inclut tenant_id, nom de base, date).
//!
//! Inclut le **vrai** code de `spike/node/src/sauvegarde.rs` et
//! `relay_client.rs` (sans le modifier) et pilote le relais **local** de test
//! sur 127.0.0.1:18099.
#[path = "/home/jose/school/amane/repo/spike/node/src/relay_client.rs"]
mod relay_client;
#[path = "/home/jose/school/amane/repo/spike/node/src/sauvegarde.rs"]
mod sauvegarde;

use chrono::Datelike;
use ss_crypto::Dek;
use uuid::Uuid;

const RELAIS: &str = "http://127.0.0.1:18099";

fn jeton(secret: &[u8], tenant: &str) -> String {
    ss_crypto::jeton_relais(secret, tenant)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let secret = std::fs::read_to_string("/tmp/opencode/r01/secret.txt")?;
    let secret = secret.trim().as_bytes().to_vec();
    let dek = Dek::generate();
    let tenant = Uuid::parse_str("dddddddd-4444-4444-8444-dddddddddddd")?;
    let jet = jeton(&secret, &tenant.to_string());

    // Un « export » pg_dump factice mais réaliste (en-tête PGDMP).
    let mut export = b"PGDMP".to_vec();
    export.extend_from_slice(&vec![0x42u8; 4096]);
    let date = chrono::Utc::now().date_naive();
    let jour = date.weekday().number_from_monday();

    // 1) Emballe comme le nœud : blob lié à (tenant, "amane", date).
    let blob = sauvegarde::emballer(&dek, tenant, "amane", date, &export)?;
    println!("[1] blob amane emballe : {} octets (jour {})", blob.len(), jour);

    // 2) Dépose sous sauvegarde-amane-<jour> via le relais local.
    let cle_amane = sauvegarde::cle_blob("amane", jour)?;
    let url_amane = format!("{RELAIS}/api/blobs/{tenant}/{cle_amane}");
    let client = reqwest::Client::new();
    let r = client.put(&url_amane).header("x-relay-token", &jet).body(blob.clone()).send().await?;
    println!("[2] PUT {cle_amane} -> HTTP {}", r.status());

    // 3) Copie le blob chiffré vers sauvegarde-application-<jour> (même tenant).
    //    On relit le blob tel quel depuis le relais, puis on le dépose sous le
    //    nom de la base application — c'est exactement ce que ferait une
    //    copie de fichier côté relais.
    let relu = client.get(&url_amane).header("x-relay-token", &jet).send().await?.bytes().await?;
    let cle_app = sauvegarde::cle_blob("application", jour)?;
    let url_app = format!("{RELAIS}/api/blobs/{tenant}/{cle_app}");
    let r = client.put(&url_app).header("x-relay-token", &jet).body(relu.to_vec()).send().await?;
    println!("[3] copie vers {cle_app} -> HTTP {}", r.status());

    // 4) Restauration « application » : le nœud relit le blob et appelle
    //    deballer(dek, blob, tenant, "application", jour).
    let blob_app = client.get(&url_app).header("x-relay-token", &jet).send().await?.bytes().await?;
    let tentative = sauvegarde::deballer(&dek, &blob_app, tenant, "application", jour);
    match tentative {
        Ok((d, _)) => println!("[4] RESTAURATION INATTENDUE: OK date={d} (FAIL)"),
        Err(e) => println!("[4] restauration application REFUSEE (attendu): {e}"),
    }

    // 5) Contrôle positif : la restauration amane légitime réussit (preuve que
    //    le refus ci-dessus vient bien de la liaison, pas d'un blob invalide).
    match sauvegarde::deballer(&dek, &blob, tenant, "amane", jour) {
        Ok((d, ex)) => println!("[5] restauration amane legitime: OK date={d}, export={} octets", ex.len()),
        Err(e) => println!("[5] restauration amane INATTENDUE en erreur: {e} (FAIL)"),
    }

    // 6) Variante : en-tête falsifié pour maquiller la copie en « application ».
    //    L'AEAD authentifie l'en-tête comme AAD : le déchiffrement doit échouer.
    //    (reproduit le test unitaire en_tete_falsifie_refuse, en live)
    let blob2 = sauvegarde::emballer(&dek, tenant, "amane", date, &export)?;
    // Construit un faux en-tête "application" de même longueur que l'en-tête amane.
    let entete_amane = format!("amane-sauvegarde/1 {tenant} amane {date}\n");
    let entete_app = format!("amane-sauvegarde/1 {tenant} application {date}\n");
    // On ne peut pas réécrire sans changer la longueur ; on teste donc la
    // liaison AAD directement via decrypt_avec_contexte avec le mauvais contexte.
    let fin = blob2.iter().position(|&o| o == b'\n').unwrap();
    let (_e, chiffre) = blob2.split_at(fin + 1);
    let aad_amane = entete_amane.as_bytes();
    let aad_app = entete_app.as_bytes();
    let r_app = dek.decrypt_avec_contexte(chiffre, aad_app);
    let r_amane = dek.decrypt_avec_contexte(chiffre, aad_amane);
    println!(
        "[6] AAD amane -> {}, AAD application -> {} (longueurs entete amane={}, app={})",
        if r_amane.is_ok() { "OK" } else { "ERREUR" },
        if r_app.is_ok() { "OK" } else { "ERREUR" },
        aad_amane.len(), aad_app.len()
    );

    Ok(())
}