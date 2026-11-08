//! `ss-node recuperer` : reconstitue la DEK du cluster après une perte totale
//! (refonte V3, étape 8, ADR-010), à partir de la passphrase du gérant et de
//! deux parts papier sur trois.
//!
//! Sur un serveur réinstallé, une DEK neuve a été générée au premier
//! démarrage : elle est remplacée par la DEK récupérée, scellée pour la clé de
//! cette machine, et le journal local, chiffré par la DEK neuve, est mis de
//! côté. Ensuite : `docker restart ss-node`, puis `ss-node restaurer`.

use std::io::{BufRead, IsTerminal, Write};
use std::path::Path;

use anyhow::{anyhow, bail, Context, Result};
use ss_crypto::{DeviceKeyPair, ShamirShare};
use zeroize::Zeroizing;

use crate::config::NodeConfig;
use crate::recuperation::{lire_bloc, lire_part, recuperer, Blob, Relais};

/// Lit une ligne : masquée au terminal (passphrase), visible sinon.
fn demander(question: &str, masquee: bool) -> Result<Zeroizing<String>> {
    let interactif = std::io::stdin().is_terminal();
    if interactif {
        print!("{question}");
        std::io::stdout().flush()?;
        if masquee {
            return Ok(Zeroizing::new(rpassword::read_password()?));
        }
    }
    let mut ligne = Zeroizing::new(String::new());
    std::io::stdin().lock().read_line(&mut ligne)?;
    if ligne.is_empty() && !interactif {
        bail!("entrée terminée avant la fin de la saisie");
    }
    Ok(Zeroizing::new(ligne.trim_end_matches(['\r', '\n']).to_string()))
}

/// Redemande une part tant que sa somme de contrôle est fausse (au terminal).
fn demander_part(numero: u8, blob: &Blob) -> Result<ShamirShare> {
    loop {
        let texte = demander(&format!("Part {numero} (n'importe laquelle des trois) : "), false)?;
        match lire_part(&blob.sel, &texte) {
            Ok(p) => return Ok(p),
            Err(e) if std::io::stdin().is_terminal() => println!("  {e} — recommencez."),
            Err(e) => return Err(e),
        }
    }
}

pub async fn run(bloc: bool, config_path: &Path) -> Result<()> {
    // Relais et tenant lus avec les surcharges de la pile ; la configuration
    // réécrite plus bas reste celle du fichier (comme pour `enroler`).
    let relais = {
        let mut avec_env = NodeConfig::load(config_path)?;
        avec_env.apply_env_overrides();
        Relais::depuis_env(&avec_env.relay_url, avec_env.tenant_id)
    };
    let mut config = NodeConfig::load(config_path)?;

    println!("Récupération de la DEK du cluster (passphrase du gérant et deux parts papier).");
    let blob = if bloc {
        let texte = demander("Bloc de récupération imprimé : ", false)?;
        lire_bloc(&texte)?
    } else {
        let relais = relais.ok_or_else(|| anyhow!("TENANT_ID inconnu : relais inaccessible, utilisez --bloc"))?;
        relais
            .lire()
            .await
            .context("lecture du blob sur le relais (utilisez --bloc pour saisir le bloc imprimé)")?
            .ok_or_else(|| anyhow!("aucun blob de récupération sur le relais : utilisez --bloc"))?
    };
    println!("  Jeu de parts : {}", blob.jeu());

    let passphrase = demander("Passphrase du gérant : ", true)?;
    let parts = [demander_part(1, &blob)?, demander_part(2, &blob)?];
    println!("  Reconstitution (Argon2id, quelques secondes)...");
    let dek = recuperer(&blob, &passphrase, &parts)?;

    let secret: [u8; 32] = hex::decode(&config.secret_key_hex)
        .context("clé secrète illisible")?
        .try_into()
        .map_err(|_| anyhow!("clé secrète de taille inattendue"))?;
    let paire = DeviceKeyPair::from_secret_bytes(secret);

    if let Some(actuelle) = config.sealed_dek_hex.as_ref() {
        let ouverte = paire.open_sealed_dek(&hex::decode(actuelle).context("DEK scellée illisible")?);
        if matches!(&ouverte, Ok(d) if d.as_bytes() == dek.as_bytes()) {
            println!("Cette machine détient déjà cette DEK : rien à faire.");
            return Ok(());
        }
        // Journal local chiffré par la DEK neuve : illisible avec l'ancienne.
        let journal = config_path.parent().unwrap_or(Path::new(".")).join("journal.bin");
        if journal.exists() {
            let archive = journal.with_file_name(format!(
                "journal.bin.avant-recuperation-{}",
                chrono::Utc::now().format("%Y%m%d-%H%M%S")
            ));
            std::fs::rename(&journal, &archive).context("mise de côté du journal impossible")?;
            println!("  Journal local chiffré par la DEK neuve mis de côté : {}", archive.display());
        }
    }

    let scellee = paire.public.seal_dek(&dek).map_err(|e| anyhow!("scellement impossible : {e:?}"))?;
    config.sealed_dek_hex = Some(hex::encode(scellee));
    config.save(config_path)?;
    println!("DEK du cluster récupérée et enregistrée pour cette machine.");
    println!("Ensuite :");
    println!("  docker restart ss-node");
    println!("  docker exec ss-node ss-node restaurer --base amane --jour <1..7>");
    println!("  docker exec ss-node ss-node restaurer --base application --jour <1..7>   (si une application est hébergée)");
    println!("Puis enrôlez les autres serveurs depuis la console de celui-ci.");
    Ok(())
}
