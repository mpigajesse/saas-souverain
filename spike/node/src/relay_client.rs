use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct AnnounceRequest {
    pub node_id: Uuid,
    pub tenant_id: Uuid,
    pub addr: String,
    pub role: String,
    pub epoch: u64,
}

// Désérialisé tel que le relais le publie ; seules certaines commandes en
// lisent les champs (`delist` ne compte que les pairs).
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct NodeInfo {
    pub node_id: Uuid,
    pub addr: String,
    pub role: String,
    pub epoch: u64,
    pub last_seen: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
struct AnnounceResponse {
    #[allow(dead_code)]
    status: String,
}

#[derive(Debug, Deserialize)]
struct NodesResponse {
    #[allow(dead_code)]
    status: String,
    nodes: Vec<NodeInfo>,
}

/// Jeton du relais propre au tenant, remis par le portail dans la pile
/// (`RELAY_AUTH_TOKEN`, ADR-011) ; vide si le relais est en mode ouvert.
/// Il ouvre les blobs et la topologie de ce seul tenant.
pub fn jeton_depuis_env() -> String {
    std::env::var("RELAY_AUTH_TOKEN").unwrap_or_default().trim().to_string()
}

/// Ajoute l'en-tête `x-relay-token` quand un jeton est connu.
pub fn avec_jeton(r: reqwest::RequestBuilder, jeton: &str) -> reqwest::RequestBuilder {
    if jeton.is_empty() { r } else { r.header("x-relay-token", jeton) }
}

pub struct RelayClient {
    base_url: String,
    jeton: String,
    client: reqwest::Client,
}

impl RelayClient {
    pub fn new(base_url: impl Into<String>, jeton: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            jeton: jeton.into(),
            client: reqwest::Client::new(),
        }
    }

    /// Annonce ce nœud au relais. Silencieux si le relais est injoignable (non bloquant).
    pub async fn announce(&self, req: &AnnounceRequest) -> Result<()> {
        let url = format!("{}/api/nodes/announce", self.base_url);
        avec_jeton(self.client.post(&url), &self.jeton)
            .json(req)
            .send()
            .await
            .with_context(|| format!("Relais injoignable : {}", url))?
            .error_for_status()
            .context("Erreur du relais lors de l'annonce")?;
        Ok(())
    }

    /// Récupère les pairs du cluster pour ce tenant.
    pub async fn get_peers(&self, tenant_id: Uuid) -> Result<Vec<NodeInfo>> {
        let url = format!("{}/api/nodes", self.base_url);
        let resp: NodesResponse = avec_jeton(self.client.get(&url), &self.jeton)
            .query(&[("tenant_id", tenant_id.to_string())])
            .send()
            .await
            .with_context(|| format!("Relais injoignable : {}", url))?
            .error_for_status()
            .context("Erreur du relais lors de la découverte des pairs")?
            .json()
            .await
            .context("Réponse relais invalide")?;
        Ok(resp.nodes)
    }
}
