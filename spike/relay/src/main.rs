use axum::{
    body::Bytes,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeEntry {
    pub node_id: Uuid,
    pub tenant_id: Uuid,
    pub addr: String,
    pub role: String,
    pub epoch: u64,
    pub last_seen: DateTime<Utc>,
}

type Registry = Arc<Mutex<HashMap<Uuid, Vec<NodeEntry>>>>;

// ---------------------------------------------------------------------------
// POST /api/nodes/announce
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
            node_id = %req.node_id, tenant_id = %req.tenant_id,
            addr = %req.addr, role = %req.role, epoch = req.epoch,
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
            node_id = %req.node_id, tenant_id = %req.tenant_id,
            addr = %req.addr, role = %req.role, epoch = req.epoch,
            "Nouveau nœud enregistré"
        );
    }

    Ok(Json(AnnounceResponse {
        status: "ok".to_string(),
        message: "Nœud enregistré".to_string(),
    }))
}

// ---------------------------------------------------------------------------
// GET /api/nodes?tenant_id=<UUID>
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

    let nodes = reg.get(&params.tenant_id).cloned().unwrap_or_default();

    info!(tenant_id = %params.tenant_id, count = nodes.len(), "Requête topologie");

    Ok(Json(NodesResponse {
        status: "ok".to_string(),
        nodes,
    }))
}

// ---------------------------------------------------------------------------
// Blob storage zero-knowledge
//
// PUT    /api/blobs/{tenant_id}/{key}   — stocker un blob chiffré
// GET    /api/blobs/{tenant_id}/{key}   — récupérer un blob chiffré
// DELETE /api/blobs/{tenant_id}/{key}   — supprimer un blob chiffré
//
// Le relais ne voit jamais la clé de chiffrement (DEK).
// Il stocke et restitue des bytes opaques.
// Auth : en-tête X-Relay-Token doit correspondre à RELAY_AUTH_TOKEN env var.
// ---------------------------------------------------------------------------

fn blob_path(blobs_dir: &str, tenant_id: &str, key: &str) -> Option<std::path::PathBuf> {
    // Validation simple : tenant_id = UUID, key = alphanum + tirets/points
    let tenant_uuid = tenant_id.parse::<Uuid>().ok()?;
    let safe_key = key.replace(['/', '\\'], "_").replace("..", "_");
    if safe_key.is_empty() || safe_key.len() > 128 {
        return None;
    }
    let path = std::path::Path::new(blobs_dir)
        .join(tenant_uuid.to_string())
        .join(&safe_key);
    Some(path)
}

fn check_blob_auth(headers: &HeaderMap) -> bool {
    let expected = std::env::var("RELAY_AUTH_TOKEN").unwrap_or_default();
    if expected.is_empty() {
        return true; // pas de token configuré → accès libre (spike)
    }
    headers
        .get("x-relay-token")
        .and_then(|v| v.to_str().ok())
        .map(|t| t == expected)
        .unwrap_or(false)
}

/// PUT /api/blobs/{tenant_id}/{key}
async fn blob_put(
    Path((tenant_id, key)): Path<(String, String)>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    if !check_blob_auth(&headers) {
        return (StatusCode::UNAUTHORIZED, "Token manquant ou invalide").into_response();
    }

    let blobs_dir = std::env::var("BLOBS_DIR").unwrap_or_else(|_| "/data/blobs".to_string());

    let path = match blob_path(&blobs_dir, &tenant_id, &key) {
        Some(p) => p,
        None => return (StatusCode::BAD_REQUEST, "tenant_id ou clé invalide").into_response(),
    };

    if let Some(parent) = path.parent() {
        if let Err(e) = tokio::fs::create_dir_all(parent).await {
            return (StatusCode::INTERNAL_SERVER_ERROR, format!("mkdir: {}", e)).into_response();
        }
    }

    if let Err(e) = tokio::fs::write(&path, &body).await {
        return (StatusCode::INTERNAL_SERVER_ERROR, format!("write: {}", e)).into_response();
    }

    info!(tenant_id = %tenant_id, key = %key, bytes = body.len(), "Blob stocké");
    (StatusCode::OK, "ok").into_response()
}

/// GET /api/blobs/{tenant_id}/{key}
async fn blob_get(
    Path((tenant_id, key)): Path<(String, String)>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if !check_blob_auth(&headers) {
        return (StatusCode::UNAUTHORIZED, "Token manquant ou invalide").into_response();
    }

    let blobs_dir = std::env::var("BLOBS_DIR").unwrap_or_else(|_| "/data/blobs".to_string());

    let path = match blob_path(&blobs_dir, &tenant_id, &key) {
        Some(p) => p,
        None => return (StatusCode::BAD_REQUEST, "tenant_id ou cle invalide").into_response(),
    };

    match tokio::fs::read(&path).await {
        Ok(data) => {
            info!(tenant_id = %tenant_id, key = %key, bytes = data.len(), "Blob recupere");
            (
                StatusCode::OK,
                [(axum::http::header::CONTENT_TYPE, "application/octet-stream")],
                data,
            )
                .into_response()
        }
        Err(_) => (StatusCode::NOT_FOUND, "Blob introuvable").into_response(),
    }
}

/// DELETE /api/blobs/{tenant_id}/{key}
async fn blob_delete(
    Path((tenant_id, key)): Path<(String, String)>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if !check_blob_auth(&headers) {
        return (StatusCode::UNAUTHORIZED, "Token manquant ou invalide").into_response();
    }

    let blobs_dir = std::env::var("BLOBS_DIR").unwrap_or_else(|_| "/data/blobs".to_string());

    let path = match blob_path(&blobs_dir, &tenant_id, &key) {
        Some(p) => p,
        None => return (StatusCode::BAD_REQUEST, "tenant_id ou clé invalide").into_response(),
    };

    match tokio::fs::remove_file(&path).await {
        Ok(_) => {
            info!(tenant_id = %tenant_id, key = %key, "Blob supprimé");
            (StatusCode::OK, "supprimé").into_response()
        }
        Err(_) => (StatusCode::NOT_FOUND, "Blob introuvable").into_response(),
    }
}

// ---------------------------------------------------------------------------
// GET /health
// ---------------------------------------------------------------------------

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "ss-relay",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

// ---------------------------------------------------------------------------
// Point d'entrée
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let registry: Registry = Arc::new(Mutex::new(HashMap::new()));
    let port = std::env::var("RELAY_PORT").unwrap_or_else(|_| "8080".to_string());

    let blobs_dir = std::env::var("BLOBS_DIR").unwrap_or_else(|_| "/data/blobs".to_string());
    tokio::fs::create_dir_all(&blobs_dir).await.ok();
    info!("Stockage blobs : {}", blobs_dir);

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/nodes/announce", post(announce))
        .route("/api/nodes", get(get_nodes))
        .route(
            "/api/blobs/{tenant_id}/{key}",
            get(blob_get).put(blob_put).delete(blob_delete),
        )
        .with_state(registry);

    let addr = format!("0.0.0.0:{}", port);
    info!("ss-relay démarré sur {}", addr);

    let listener = TcpListener::bind(&addr).await.expect("Impossible de lier l'adresse");
    axum::serve(listener, app).await.expect("Erreur serveur");
}
