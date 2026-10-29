use axum::{
    body::Bytes,
    extract::{DefaultBodyLimit, Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tokio::net::TcpListener;
use tracing::info;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// État partagé
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct AppState {
    registry: Registry,
    started_at: Instant,
}

type Registry = Arc<Mutex<HashMap<Uuid, Vec<NodeEntry>>>>;

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
    State(state): State<AppState>,
    Json(req): Json<AnnounceRequest>,
) -> Result<Json<AnnounceResponse>, (StatusCode, Json<AnnounceResponse>)> {
    let now = Utc::now();

    let mut reg = state.registry.lock().map_err(|_| {
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
        info!(node_id = %req.node_id, addr = %req.addr, "Nœud mis à jour");
    } else {
        nodes.push(NodeEntry {
            node_id: req.node_id,
            tenant_id: req.tenant_id,
            addr: req.addr.clone(),
            role: req.role.clone(),
            epoch: req.epoch,
            last_seen: now,
        });
        info!(node_id = %req.node_id, addr = %req.addr, "Nouveau nœud enregistré");
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
    State(state): State<AppState>,
    Query(params): Query<NodesQuery>,
) -> Result<Json<NodesResponse>, (StatusCode, Json<NodesResponse>)> {
    let reg = state.registry.lock().map_err(|_| {
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
// ---------------------------------------------------------------------------

fn blob_path(blobs_dir: &str, tenant_id: &str, key: &str) -> Option<std::path::PathBuf> {
    let tenant_uuid = tenant_id.parse::<Uuid>().ok()?;
    let safe_key = key.replace(['/', '\\'], "_").replace("..", "_");
    if safe_key.is_empty() || safe_key.len() > 128 {
        return None;
    }
    Some(std::path::Path::new(blobs_dir).join(tenant_uuid.to_string()).join(&safe_key))
}

fn check_blob_auth(headers: &HeaderMap) -> bool {
    let expected = std::env::var("RELAY_AUTH_TOKEN").unwrap_or_default();
    if expected.is_empty() {
        return true;
    }
    headers
        .get("x-relay-token")
        .and_then(|v| v.to_str().ok())
        .map(|t| t == expected)
        .unwrap_or(false)
}

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
        None => return (StatusCode::BAD_REQUEST, "Parametres invalides").into_response(),
    };
    if let Some(parent) = path.parent() {
        if let Err(e) = tokio::fs::create_dir_all(parent).await {
            return (StatusCode::INTERNAL_SERVER_ERROR, format!("mkdir: {}", e)).into_response();
        }
    }
    if let Err(e) = tokio::fs::write(&path, &body).await {
        return (StatusCode::INTERNAL_SERVER_ERROR, format!("write: {}", e)).into_response();
    }
    info!(tenant_id = %tenant_id, key = %key, bytes = body.len(), "Blob stocke");
    (StatusCode::OK, "ok").into_response()
}

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
        None => return (StatusCode::BAD_REQUEST, "Parametres invalides").into_response(),
    };
    match tokio::fs::read(&path).await {
        Ok(data) => (
            StatusCode::OK,
            [(axum::http::header::CONTENT_TYPE, "application/octet-stream")],
            data,
        )
            .into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "Blob introuvable").into_response(),
    }
}

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
        None => return (StatusCode::BAD_REQUEST, "Parametres invalides").into_response(),
    };
    match tokio::fs::remove_file(&path).await {
        Ok(_) => (StatusCode::OK, "supprime").into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "Blob introuvable").into_response(),
    }
}

// ---------------------------------------------------------------------------
// GET /health  — retourne hostname, uptime, interfaces réseau, stockage blobs
// ---------------------------------------------------------------------------

async fn health(State(state): State<AppState>) -> Json<serde_json::Value> {
    let uptime_secs = state.started_at.elapsed().as_secs();

    let hostname = std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("COMPUTERNAME"))
        .unwrap_or_else(|_| "unknown".to_string());

    let port = std::env::var("RELAY_PORT").unwrap_or_else(|_| "8080".to_string());

    let blobs_dir = std::env::var("BLOBS_DIR").unwrap_or_else(|_| "/data/blobs".to_string());

    // Compter les tenants qui ont des blobs stockés
    let blob_tenants = std::fs::read_dir(&blobs_dir)
        .map(|rd| rd.filter_map(|e| e.ok()).filter(|e| e.path().is_dir()).count())
        .unwrap_or(0);

    // Interfaces réseau depuis RELAY_NETWORKS=iface:ip:role,iface:ip:role
    // ex: RELAY_NETWORKS=ens33:192.168.1.54:editeur,ens34:192.168.100.100:interne
    let networks: Vec<serde_json::Value> = std::env::var("RELAY_NETWORKS")
        .unwrap_or_default()
        .split(',')
        .filter(|s| !s.is_empty())
        .map(|entry| {
            let parts: Vec<&str> = entry.trim().splitn(3, ':').collect();
            serde_json::json!({
                "iface": parts.first().unwrap_or(&"?"),
                "ip":    parts.get(1).unwrap_or(&"?"),
                "role":  parts.get(2).unwrap_or(&""),
            })
        })
        .collect();

    Json(serde_json::json!({
        "status":        "ok",
        "service":       "ss-relay",
        "version":       env!("CARGO_PKG_VERSION"),
        "hostname":      hostname,
        "port":          port,
        "uptime_secs":   uptime_secs,
        "blobs_dir":     blobs_dir,
        "blob_tenants":  blob_tenants,
        "networks":      networks,
        "zero_knowledge": true
    }))
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

/// Taille maximale d'un blob : les sauvegardes chiffrées des PME (étape 4)
/// dépassent largement la limite de 2 Mo appliquée par défaut par axum.
const BLOB_MAX: usize = 1024 * 1024 * 1024;

fn routeur(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/nodes/announce", post(announce))
        .route("/api/nodes", get(get_nodes))
        // Syntaxe des paramètres d'axum 0.7 (`:nom`). La forme `{nom}` est
        // celle d'axum 0.8 : en 0.7 elle était prise au pied de la lettre et
        // les routes de blobs ne répondaient jamais (refonte V3, étape 4).
        .route(
            "/api/blobs/:tenant_id/:key",
            get(blob_get)
                .put(blob_put)
                .delete(blob_delete)
                .layer(DefaultBodyLimit::max(BLOB_MAX)),
        )
        .with_state(state)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let port = std::env::var("RELAY_PORT").unwrap_or_else(|_| "8080".to_string());
    let blobs_dir = std::env::var("BLOBS_DIR").unwrap_or_else(|_| "/data/blobs".to_string());
    tokio::fs::create_dir_all(&blobs_dir).await.ok();

    let state = AppState {
        registry: Arc::new(Mutex::new(HashMap::new())),
        started_at: Instant::now(),
    };

    let app = routeur(state);

    let addr = format!("0.0.0.0:{}", port);
    info!("ss-relay sur {}", addr);

    let listener = TcpListener::bind(&addr).await.expect("Impossible de lier l'adresse");
    axum::serve(listener, app).await.expect("Erreur serveur");
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    fn etat() -> AppState {
        AppState { registry: Arc::new(Mutex::new(HashMap::new())), started_at: Instant::now() }
    }

    #[tokio::test]
    async fn blob_depose_relu_puis_supprime() {
        let dossier = std::env::temp_dir().join(format!("relais-essai-{}", Uuid::new_v4()));
        std::env::set_var("BLOBS_DIR", &dossier);
        let tenant = Uuid::new_v4();
        let url = format!("/api/blobs/{tenant}/sauvegarde-amane-3");
        // Plus de 2 Mo : la limite par défaut d'axum refuserait ce dépôt.
        let blob = vec![7u8; 3 * 1024 * 1024];

        let r = routeur(etat())
            .oneshot(Request::put(&url).body(Body::from(blob.clone())).unwrap())
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::OK);

        let r = routeur(etat()).oneshot(Request::get(&url).body(Body::empty()).unwrap()).await.unwrap();
        assert_eq!(r.status(), StatusCode::OK);
        let lu = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
        assert_eq!(lu.len(), blob.len());

        let r = routeur(etat()).oneshot(Request::delete(&url).body(Body::empty()).unwrap()).await.unwrap();
        assert_eq!(r.status(), StatusCode::OK);
        let r = routeur(etat()).oneshot(Request::get(&url).body(Body::empty()).unwrap()).await.unwrap();
        assert_eq!(r.status(), StatusCode::NOT_FOUND);
        let _ = std::fs::remove_dir_all(dossier);
    }

    #[test]
    fn chemin_de_blob_refuse_les_evasions() {
        assert!(blob_path("/data", "pas-un-uuid", "x").is_none());
        let p = blob_path("/data", &Uuid::nil().to_string(), "../../etc/passwd").unwrap();
        assert!(!p.to_string_lossy().contains(".."));
        assert!(blob_path("/data", &Uuid::nil().to_string(), "").is_none());
    }
}
