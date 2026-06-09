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

pub struct RelayClient {
    base_url: String,
    client: reqwest::Client,
}

impl RelayClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            client: reqwest::Client::new(),
        }
    }

    /// Annonce ce nœud au relais. Silencieux si le relais est injoignable (non bloquant).
    pub async fn announce(&self, req: &AnnounceRequest) -> Result<()> {
        let url = format!("{}/api/nodes/announce", self.base_url);
        self.client
            .post(&url)
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
        let resp: NodesResponse = self
            .client
            .get(&url)
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
