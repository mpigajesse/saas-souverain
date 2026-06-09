use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;
use anyhow::{Context, Result};

/// Configuration persistante d'un nœud PME.
/// Sauvegardée dans `~/.ss-node/config.toml` (ou chemin passé en argument).
#[derive(Debug, Serialize, Deserialize)]
pub struct NodeConfig {
    pub node_id: Uuid,
    /// Clé secrète X25519 en hex (32 octets)
    pub secret_key_hex: String,
    /// DEK scellée pour ce nœud (hex) — absente si pas encore enrôlé
    pub sealed_dek_hex: Option<String>,
    /// Sel Argon2id pour le code de récupération (hex 16 octets) — absent si pas encore défini
    pub recovery_salt_hex: Option<String>,
    /// DEK chiffrée sous le code de récupération (hex) — absente si pas encore défini
    pub recovery_blob_hex: Option<String>,
    /// Port d'écoute du nœud
    pub port: u16,
    /// Identifiant du cluster PME (reçu lors de l'inscription sur le SaaS éditeur)
    pub tenant_id: Option<Uuid>,
    /// URL du relais éditeur
    #[serde(default = "default_relay_url")]
    pub relay_url: String,
    /// URL de connexion PostgreSQL local (ex: postgres://postgres:pass@localhost/ss)
    pub pg_url: Option<String>,
    /// Époque courante de ce nœud (jeton de fencing monotone)
    #[serde(default = "default_epoch")]
    pub epoch: u64,
}

fn default_epoch() -> u64 {
    1
}

fn default_relay_url() -> String {
    "http://localhost:8080".to_string()
}

impl NodeConfig {
    /// Retourne le chemin par défaut vers le fichier de configuration.
    pub fn default_path() -> PathBuf {
        dirs_or_default().join("config.toml")
    }

    /// Charge la configuration depuis un fichier TOML.
    pub fn load(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Impossible de lire la config : {}", path.display()))?;
        let config: Self = toml::from_str(&content)
            .with_context(|| format!("Config TOML invalide : {}", path.display()))?;
        Ok(config)
    }

    /// Sauvegarde la configuration dans un fichier TOML.
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Impossible de créer le répertoire : {}", parent.display()))?;
        }
        let content = toml::to_string_pretty(self)
            .context("Impossible de sérialiser la config en TOML")?;
        fs::write(path, content)
            .with_context(|| format!("Impossible d'écrire la config : {}", path.display()))?;
        Ok(())
    }

    /// Applique les surcharges depuis les variables d'environnement Docker.
    /// Priorité : env var > valeur dans config.toml
    pub fn apply_env_overrides(&mut self) {
        if let Ok(val) = std::env::var("TENANT_ID") {
            if let Ok(uuid) = val.parse::<Uuid>() {
                self.tenant_id = Some(uuid);
            }
        }
        if let Ok(val) = std::env::var("RELAY_URL") {
            if !val.is_empty() {
                self.relay_url = val;
            }
        }
        if let Ok(val) = std::env::var("PG_URL") {
            if !val.is_empty() {
                self.pg_url = Some(val);
            }
        }
        // REGISTRATION_TOKEN n'est pas stocké dans NodeConfig (c'est côté SaaS).
        // Il est utilisé directement par l'API /api/devices/register/ via le SaaS Django.
    }
}

fn dirs_or_default() -> PathBuf {
    let base = std::env::var("SS_NODE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| dirs_sys_or_cwd());
    fs::create_dir_all(&base).ok();
    base
}

fn dirs_sys_or_cwd() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        std::env::var("APPDATA")
            .map(|d| PathBuf::from(d).join("ss-node"))
            .unwrap_or_else(|_| PathBuf::from(".ss-node"))
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::env::var("HOME")
            .map(|d| PathBuf::from(d).join(".ss-node"))
            .unwrap_or_else(|_| PathBuf::from(".ss-node"))
    }
}
