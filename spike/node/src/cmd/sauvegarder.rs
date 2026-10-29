use std::path::Path;

use anyhow::{anyhow, Result};

use crate::cmd::run::ouvrir_dek;
use crate::config::NodeConfig;
use crate::sauvegarde::{bases_a_sauvegarder, cible_depuis_env, restaurer, sauvegarder};

/// `ss-node sauvegarder` : sauvegarde immédiate, chiffrée, déposée sur le relais.
pub async fn run(config_path: &Path) -> Result<()> {
    let mut config = NodeConfig::load(config_path)?;
    config.apply_env_overrides();
    let dek = ouvrir_dek(&config)?.ok_or_else(|| anyhow!("DEK absente : enrôlez d'abord cette machine (ss-node enroler)"))?;
    let cible = cible_depuis_env(&config.relay_url, config.tenant_id)?;
    for base in bases_a_sauvegarder() {
        let (cle, taille) = sauvegarder(&dek, &cible, base).await?;
        println!("Sauvegarde {base} : {cle} ({taille} octets chiffrés) déposée sur le relais");
    }
    Ok(())
}

/// `ss-node restaurer --base <base> --jour <1..7>` : restauration depuis le relais.
pub async fn restaurer_base(base: &str, jour: u32, config_path: &Path) -> Result<()> {
    let mut config = NodeConfig::load(config_path)?;
    config.apply_env_overrides();
    let dek = ouvrir_dek(&config)?.ok_or_else(|| anyhow!("DEK absente : enrôlez d'abord cette machine (ss-node enroler)"))?;
    let cible = cible_depuis_env(&config.relay_url, config.tenant_id)?;
    restaurer(&dek, &cible, base, jour).await?;
    println!("Base {base} restaurée depuis la sauvegarde du jour {jour}");
    Ok(())
}
