use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tracing::info;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Une entrée dans le registre de topologie du cluster.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeEntry {
    pub node_id: Uuid,
    pub tenant_id: Uuid,
    pub addr: String,
    /// "active" ou "passive"
    pub role: String,
    pub epoch: u64,
    pub last_seen: DateTime<Utc>,
}

/// tenant_id → liste des nœuds connus
type Registry = Arc<Mutex<HashMap<Uuid, Vec<NodeEntry>>>>;

// ---------------------------------------------------------------------------
// Requête / Réponse : POST /api/nodes/announce
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct AnnounceRequest {
    pub node_id: Uuid,
    pub tenant_id: Uuid,
    pub addr: String,
    pub role: String,
    pub epoch: u64,
}

#[derive(Debug, Serialize)]
pub struct AnnounceResponse {
    pub status: String,
    pub message: String,
}

// ---------------------------------------------------------------------------
// Requête / Réponse : GET /api/nodes
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct NodesQuery {
    pub tenant_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct NodesResponse {
    pub status: String,
    pub nodes: Vec<NodeEntry>,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// POST /api/nodes/announce
///
/// Enregistre ou met à jour le nœud dans le registre en mémoire.
/// Si un nœud portant le même `node_id` existe déjà pour ce tenant,
/// ses champs `addr`, `role`, `epoch` et `last_seen` sont mis à jour.
/// Sinon le nœud est inséré.
async fn announce(
    State(registry): State<Registry>,
    Json(req): Json<AnnounceRequest>,
) -> Result<Json<AnnounceResponse>, (StatusCode, Json<AnnounceResponse>)> {
    let now = Utc::now();

    let mut reg = registry.lock().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AnnounceResponse {
                status: "error".to_string(),
                message: "Registre temporairement indisponible".to_string(),
            }),
        )
    })?;

    let nodes = reg.entry(req.tenant_id).or_default();

    if let Some(existing) = nodes.iter_mut().find(|n| n.node_id == req.node_id) {
        existing.addr = req.addr.clone();
        existing.role = req.role.clone();
        existing.epoch = req.epoch;
        existing.last_seen = now;
        info!(
            node_id = %req.node_id,
            tenant_id = %req.tenant_id,
            addr = %req.addr,
            role = %req.role,
            epoch = req.epoch,
            "Nœud mis à jour"
        );
    } else {
        nodes.push(NodeEntry {
            node_id: req.node_id,
            tenant_id: req.tenant_id,
            addr: req.addr.clone(),
            role: req.role.clone(),
            epoch: req.epoch,
            last_seen: now,
        });
        info!(
            node_id = %req.node_id,
            tenant_id = %req.tenant_id,
            addr = %req.addr,
            role = %req.role,
            epoch = req.epoch,
            "Nouveau nœud enregistré"
        );
    }

    Ok(Json(AnnounceResponse {
        status: "ok".to_string(),
        message: "Nœud enregistré".to_string(),
    }))
}

/// GET /api/nodes?tenant_id=<UUID>
///
/// Retourne la liste des nœuds connus pour le tenant demandé.
/// Retourne une liste vide si le tenant est inconnu.
async fn get_nodes(
    State(registry): State<Registry>,
    Query(params): Query<NodesQuery>,
) -> Result<Json<NodesResponse>, (StatusCode, Json<NodesResponse>)> {
    let reg = registry.lock().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(NodesResponse {
                status: "error".to_string(),
                nodes: vec![],
            }),
        )
    })?;

    let nodes = reg
        .get(&params.tenant_id)
        .cloned()
        .unwrap_or_default();

    info!(
        tenant_id = %params.tenant_id,
        count = nodes.len(),
        "Requête topologie"
    );

    Ok(Json(NodesResponse {
        status: "ok".to_string(),
        nodes,
    }))
}

/// GET /health
async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

// ---------------------------------------------------------------------------
// Point d'entrée
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let registry: Registry = Arc::new(Mutex::new(HashMap::new()));
    let port = std::env::var("RELAY_PORT").unwrap_or_else(|_| "8080".to_string());

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/nodes/announce", post(announce))
        .route("/api/nodes", get(get_nodes))
        .with_state(registry);

    let addr = format!("0.0.0.0:{}", port);
    info!("ss-relay démarré sur {}", addr);

    let listener = TcpListener::bind(&addr).await.expect("Impossible de lier l'adresse");
    axum::serve(listener, app).await.expect("Erreur serveur");
}
