use std::path::Path;

use anyhow::Result;

use crate::config::NodeConfig;

/// Affiche l'état du nœud local.
pub async fn run(config_path: &Path) -> Result<()> {
    if !config_path.exists() {
        println!("Noeud non initialisé. Lancez : ss-node init");
        return Ok(());
    }

    let config = NodeConfig::load(config_path)?;

    println!("=== Statut du noeud ===");
    println!("  ID        : {}", config.node_id);
    println!("  Port      : {}", config.port);
    println!(
        "  Enrôlé    : {}",
        if config.sealed_dek_hex.is_some() { "oui" } else { "non" }
    );
    println!(
        "  Récupérat.: {}",
        if config.recovery_blob_hex.is_some() { "configurée" } else { "non configurée" }
    );
    Ok(())
}
