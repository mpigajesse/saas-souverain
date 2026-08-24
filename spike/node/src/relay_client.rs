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

    /// Dépose un blob CHIFFRÉ opaque sur le relais (zero-knowledge).
    /// Le relais ne peut pas le lire — il ne stocke que des bytes.
    pub async fn put_blob(&self, tenant_id: Uuid, key: &str, data: Vec<u8>) -> Result<()> {
        let url = format!("{}/api/blobs/{}/{}", self.base_url, tenant_id, key);
        let mut req = self.client.put(&url).body(data);
        // Jeton optionnel : seulement si le relais en exige un (RELAY_AUTH_TOKEN).
        if let Ok(token) = std::env::var("RELAY_AUTH_TOKEN") {
            if !token.is_empty() {
                req = req.header("x-relay-token", token);
            }
        }
        req.send()
            .await
            .with_context(|| format!("Relais injoignable : {}", url))?
            .error_for_status()
            .context("Erreur du relais lors du dépôt du blob")?;
        Ok(())
    }

    /// Vérifie l'existence d'un blob (sans en lire le contenu utile).
    /// Retourne false si le relais est injoignable.
    pub async fn blob_exists(&self, tenant_id: Uuid, key: &str) -> bool {
        let url = format!("{}/api/blobs/{}/{}", self.base_url, tenant_id, key);
        let mut req = self.client.get(&url);
        if let Ok(token) = std::env::var("RELAY_AUTH_TOKEN") {
            if !token.is_empty() {
                req = req.header("x-relay-token", token);
            }
        }
        req.send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
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
