use std::path::Path;

use anyhow::{anyhow, bail, Context, Result};
use ss_crypto::DeviceKeyPair;

use crate::config::NodeConfig;
use crate::enrolement::{Demande, Reponse};

/// `ss-node enroler --depuis <url> --code <code>` : reçoit la DEK du cluster,
/// scellée pour la clé publique de cette machine (étape 4, ADR-008).
pub async fn run(depuis: &str, code: &str, config_path: &Path) -> Result<()> {
    let mut config = NodeConfig::load(config_path)?;
    if config.sealed_dek_hex.is_some() {
        println!("Cette machine détient déjà la DEK : rien à faire.");
        return Ok(());
    }
    let secret: [u8; 32] = hex::decode(&config.secret_key_hex)
        .context("clé secrète illisible")?
        .try_into()
        .map_err(|_| anyhow!("clé secrète de taille inattendue"))?;
    let paire = DeviceKeyPair::from_secret_bytes(secret);
    let demande = Demande { code: code.to_string(), cle_publique: hex::encode(paire.public.as_bytes()) };

    let url = format!("{}/amane/api/enrolement", depuis.trim_end_matches('/'));
    let reponse = reqwest::Client::new().post(&url).json(&demande).send().await
        .with_context(|| format!("serveur {depuis} injoignable"))?;
    if !reponse.status().is_success() {
        bail!("enrôlement refusé : {} {}", reponse.status(), reponse.text().await.unwrap_or_default());
    }
    let Reponse { dek_scellee } = reponse.json().await.context("réponse illisible")?;
    // Vérification locale avant d'enregistrer : la DEK doit s'ouvrir avec la
    // clé privée de cette machine.
    paire
        .open_sealed_dek(&hex::decode(&dek_scellee).context("DEK scellée illisible")?)
        .map_err(|_| anyhow!("la DEK reçue ne s'ouvre pas avec la clé de cette machine"))?;
    config.sealed_dek_hex = Some(dek_scellee);
    config.save(config_path)?;
    println!("DEK du cluster reçue et enregistrée. Redémarrez le nœud : docker restart ss-node");
    Ok(())
}
