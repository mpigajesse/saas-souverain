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
    /// Dossier des blobs (`BLOBS_DIR`).
    blobs_dir: Arc<std::path::PathBuf>,
    /// Secret partagé avec le portail (`RELAY_SECRET`), dont dérive le jeton
    /// de chaque tenant (ADR-011). `None` : mode ouvert, réservé au
    /// laboratoire et demandé explicitement (`RELAY_SANS_AUTHENTIFICATION=1`).
    secret: Option<Arc<Vec<u8>>>,
}

type Registry = Arc<Mutex<HashMap<Uuid, Vec<NodeEntry>>>>;

// ---------------------------------------------------------------------------
// Authentification : un jeton par tenant (refonte V3, étape 9, ADR-011)
// ---------------------------------------------------------------------------

/// Longueur minimale du secret : 32 caractères (`openssl rand -hex 32` en donne 64).
const SECRET_MIN: usize = 32;

const REFUS: &str = "Jeton du tenant manquant ou invalide";

/// Configuration de l'authentification lue au démarrage. Un relais sans
/// secret refuse de démarrer, sauf mode ouvert demandé explicitement : jamais
/// de relais ouvert par oubli.
fn secret_depuis(secret: Option<String>, sans_authentification: bool) -> Result<Option<Vec<u8>>, String> {
    match secret.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()) {
        Some(s) if s.len() < SECRET_MIN => {
            Err(format!("RELAY_SECRET trop court ({} caractères, {SECRET_MIN} au moins)", s.len()))
        }
        Some(s) => Ok(Some(s.into_bytes())),
        None if sans_authentification => Ok(None),
        None => Err("RELAY_SECRET absent : le relais refuse de démarrer sans authentification. \
                     Définir RELAY_SECRET (le même que celui du portail), ou RELAY_SANS_AUTHENTIFICATION=1 \
                     pour un laboratoire"
            .to_string()),
    }
}

/// Vrai si la requête porte le jeton du tenant (en-tête `x-relay-token`), ou
/// si le relais est en mode ouvert. Le jeton est recalculé à partir du secret
/// (HMAC, `ss_crypto::verifier_jeton_relais`) : aucune base côté relais.
fn autorise(state: &AppState, headers: &HeaderMap, tenant_id: &Uuid) -> bool {
    let Some(secret) = &state.secret else { return true };
    headers
        .get("x-relay-token")
        .and_then(|v| v.to_str().ok())
        .map(|jeton| ss_crypto::verifier_jeton_relais(secret, &tenant_id.to_string(), jeton))
        .unwrap_or(false)
}

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
    headers: HeaderMap,
    Json(req): Json<AnnounceRequest>,
) -> Result<Json<AnnounceResponse>, (StatusCode, Json<AnnounceResponse>)> {
    // La topologie d'un tenant ne s'écrit qu'avec son jeton : sinon
    // n'importe qui y glisserait de fausses adresses.
    if !autorise(&state, &headers, &req.tenant_id) {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(AnnounceResponse { status: "error".to_string(), message: REFUS.to_string() }),
        ));
    }
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
    headers: HeaderMap,
    Query(params): Query<NodesQuery>,
) -> Result<Json<NodesResponse>, (StatusCode, Json<NodesResponse>)> {
    if !autorise(&state, &headers, &params.tenant_id) {
        return Err((StatusCode::UNAUTHORIZED, Json(NodesResponse { status: "error".to_string(), nodes: vec![] })));
    }
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

fn blob_path(blobs_dir: &std::path::Path, tenant_id: &Uuid, key: &str) -> Option<std::path::PathBuf> {
    let safe_key = key.replace(['/', '\\'], "_").replace("..", "_");
    if safe_key.is_empty() || safe_key.len() > 128 {
        return None;
    }
    Some(blobs_dir.join(tenant_id.to_string()).join(&safe_key))
}

/// Chemin du blob si la requête est recevable : tenant valide, jeton de ce
/// tenant, clé acceptable. Sinon, la réponse d'erreur à renvoyer.
fn blob_autorise(
    state: &AppState,
    headers: &HeaderMap,
    tenant_id: &str,
    key: &str,
) -> Result<std::path::PathBuf, axum::response::Response> {
    let tenant = tenant_id
        .parse::<Uuid>()
        .map_err(|_| (StatusCode::BAD_REQUEST, "Parametres invalides").into_response())?;
    if !autorise(state, headers, &tenant) {
        return Err((StatusCode::UNAUTHORIZED, REFUS).into_response());
    }
    blob_path(&state.blobs_dir, &tenant, key)
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "Parametres invalides").into_response())
}

async fn blob_put(
    State(state): State<AppState>,
    Path((tenant_id, key)): Path<(String, String)>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    let path = match blob_autorise(&state, &headers, &tenant_id, &key) {
        Ok(p) => p,
        Err(r) => return r,
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
    State(state): State<AppState>,
    Path((tenant_id, key)): Path<(String, String)>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let path = match blob_autorise(&state, &headers, &tenant_id, &key) {
        Ok(p) => p,
        Err(r) => return r,
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
    State(state): State<AppState>,
    Path((tenant_id, key)): Path<(String, String)>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let path = match blob_autorise(&state, &headers, &tenant_id, &key) {
        Ok(p) => p,
        Err(r) => return r,
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

    let blobs_dir = state.blobs_dir.display().to_string();

    // Compter les tenants qui ont des blobs stockés
    let blob_tenants = std::fs::read_dir(state.blobs_dir.as_path())
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
        // Jeton propre à chaque tenant exigé (ADR-011) ; faux en mode ouvert.
        "authentification": state.secret.is_some(),
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

    let secret = match secret_depuis(
        std::env::var("RELAY_SECRET").ok(),
        std::env::var("RELAY_SANS_AUTHENTIFICATION").map(|v| v == "1").unwrap_or(false),
    ) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("ss-relay : {e}");
            std::process::exit(2);
        }
    };
    match &secret {
        Some(_) => info!("Authentification : jeton propre à chaque tenant exigé"),
        None => tracing::warn!(
            "Mode ouvert (RELAY_SANS_AUTHENTIFICATION=1) : blobs et topologie accessibles sans jeton — laboratoire uniquement"
        ),
    }

    let state = AppState {
        registry: Arc::new(Mutex::new(HashMap::new())),
        started_at: Instant::now(),
        blobs_dir: Arc::new(blobs_dir.into()),
        secret: secret.map(Arc::new),
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

    const SECRET: &[u8] = b"secret-de-test-du-relais-amane-0123456789";

    fn etat(dossier: &std::path::Path, secret: Option<&[u8]>) -> AppState {
        AppState {
            registry: Arc::new(Mutex::new(HashMap::new())),
            started_at: Instant::now(),
            blobs_dir: Arc::new(dossier.to_path_buf()),
            secret: secret.map(|s| Arc::new(s.to_vec())),
        }
    }

    fn dossier() -> std::path::PathBuf {
        std::env::temp_dir().join(format!("relais-essai-{}", Uuid::new_v4()))
    }

    fn jeton(tenant: &Uuid) -> String {
        ss_crypto::jeton_relais(SECRET, &tenant.to_string())
    }

    fn requete(methode: &str, url: &str, jeton: Option<&str>, corps: Vec<u8>) -> Request<Body> {
        let mut r = Request::builder().method(methode).uri(url);
        if let Some(j) = jeton {
            r = r.header("x-relay-token", j);
        }
        r.body(Body::from(corps)).unwrap()
    }

    async fn json(r: axum::response::Response) -> serde_json::Value {
        serde_json::from_slice(&axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap()).unwrap()
    }

    #[tokio::test]
    async fn blob_depose_relu_puis_supprime() {
        let dossier = dossier();
        let app = routeur(etat(&dossier, Some(SECRET)));
        let tenant = Uuid::new_v4();
        let j = jeton(&tenant);
        let url = format!("/api/blobs/{tenant}/sauvegarde-amane-3");
        // Plus de 2 Mo : la limite par défaut d'axum refuserait ce dépôt.
        let blob = vec![7u8; 3 * 1024 * 1024];

        let r = app.clone().oneshot(requete("PUT", &url, Some(&j), blob.clone())).await.unwrap();
        assert_eq!(r.status(), StatusCode::OK);

        let r = app.clone().oneshot(requete("GET", &url, Some(&j), vec![])).await.unwrap();
        assert_eq!(r.status(), StatusCode::OK);
        let lu = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
        assert_eq!(lu.len(), blob.len());

        let r = app.clone().oneshot(requete("DELETE", &url, Some(&j), vec![])).await.unwrap();
        assert_eq!(r.status(), StatusCode::OK);
        let r = app.oneshot(requete("GET", &url, Some(&j), vec![])).await.unwrap();
        assert_eq!(r.status(), StatusCode::NOT_FOUND);
        let _ = std::fs::remove_dir_all(dossier);
    }

    #[tokio::test]
    async fn blob_refuse_sans_le_jeton_de_son_tenant() {
        let dossier = dossier();
        let app = routeur(etat(&dossier, Some(SECRET)));
        let victime = Uuid::new_v4();
        let autre = Uuid::new_v4();
        let url = format!("/api/blobs/{victime}/recuperation");
        let r = app.clone().oneshot(requete("PUT", &url, Some(&jeton(&victime)), b"chiffre".to_vec())).await.unwrap();
        assert_eq!(r.status(), StatusCode::OK);

        // Sans jeton, avec un jeton faux, ou avec le jeton (valide) d'un autre
        // tenant : ni lecture, ni écrasement, ni suppression.
        for j in [None, Some("00".repeat(32)), Some(jeton(&autre))] {
            for methode in ["GET", "PUT", "DELETE"] {
                let r = app.clone().oneshot(requete(methode, &url, j.as_deref(), b"x".to_vec())).await.unwrap();
                assert_eq!(r.status(), StatusCode::UNAUTHORIZED, "{methode} avec {j:?}");
            }
        }
        let r = app.oneshot(requete("GET", &url, Some(&jeton(&victime)), vec![])).await.unwrap();
        let lu = axum::body::to_bytes(r.into_body(), usize::MAX).await.unwrap();
        assert_eq!(&lu[..], b"chiffre");
        let _ = std::fs::remove_dir_all(dossier);
    }

    #[tokio::test]
    async fn topologie_reservee_au_tenant() {
        let app = routeur(etat(&dossier(), Some(SECRET)));
        let tenant = Uuid::new_v4();
        let annonce = serde_json::json!({
            "node_id": Uuid::new_v4(), "tenant_id": tenant, "addr": "10.0.0.1:9001", "role": "active", "epoch": 1
        });
        let poster = |j: Option<String>| {
            let mut r = Request::post("/api/nodes/announce").header("content-type", "application/json");
            if let Some(j) = j {
                r = r.header("x-relay-token", j);
            }
            r.body(Body::from(annonce.to_string())).unwrap()
        };
        let r = app.clone().oneshot(poster(None)).await.unwrap();
        assert_eq!(r.status(), StatusCode::UNAUTHORIZED);
        let r = app.clone().oneshot(poster(Some(jeton(&Uuid::new_v4())))).await.unwrap();
        assert_eq!(r.status(), StatusCode::UNAUTHORIZED);
        let r = app.clone().oneshot(poster(Some(jeton(&tenant)))).await.unwrap();
        assert_eq!(r.status(), StatusCode::OK);

        let url = format!("/api/nodes?tenant_id={tenant}");
        let r = app.clone().oneshot(requete("GET", &url, None, vec![])).await.unwrap();
        assert_eq!(r.status(), StatusCode::UNAUTHORIZED);
        let r = app.oneshot(requete("GET", &url, Some(&jeton(&tenant)), vec![])).await.unwrap();
        assert_eq!(r.status(), StatusCode::OK);
        assert_eq!(json(r).await["nodes"].as_array().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn mode_ouvert_explicite() {
        let dossier = dossier();
        let app = routeur(etat(&dossier, None));
        let url = format!("/api/blobs/{}/x", Uuid::new_v4());
        let r = app.clone().oneshot(requete("PUT", &url, None, b"x".to_vec())).await.unwrap();
        assert_eq!(r.status(), StatusCode::OK);
        let r = app.oneshot(requete("GET", "/health", None, vec![])).await.unwrap();
        assert_eq!(json(r).await["authentification"], false);
        let _ = std::fs::remove_dir_all(dossier);
    }

    #[test]
    fn demarrage_refuse_sans_secret() {
        assert!(secret_depuis(None, false).is_err());
        assert!(secret_depuis(Some("  ".into()), false).is_err());
        assert!(secret_depuis(Some("court".into()), false).is_err());
        // Un secret trop court n'est pas rattrapé par le mode ouvert.
        assert!(secret_depuis(Some("court".into()), true).is_err());
        assert_eq!(secret_depuis(None, true).unwrap(), None);
        let s = "a".repeat(SECRET_MIN);
        assert_eq!(secret_depuis(Some(s.clone()), false).unwrap(), Some(s.into_bytes()));
    }

    #[test]
    fn chemin_de_blob_refuse_les_evasions() {
        let racine = std::path::Path::new("/data");
        let p = blob_path(racine, &Uuid::nil(), "../../etc/passwd").unwrap();
        assert!(!p.to_string_lossy().contains(".."));
        assert!(blob_path(racine, &Uuid::nil(), "").is_none());
    }

    #[tokio::test]
    async fn tenant_invalide_refuse() {
        let app = routeur(etat(&dossier(), Some(SECRET)));
        let r = app.oneshot(requete("GET", "/api/blobs/pas-un-uuid/x", None, vec![])).await.unwrap();
        assert_eq!(r.status(), StatusCode::BAD_REQUEST);
    }
}
