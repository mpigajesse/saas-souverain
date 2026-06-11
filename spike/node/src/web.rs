use std::sync::Arc;

use axum::{
    extract::{Request, State},
    middleware::{self, Next},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
    Router,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::{self, AppState};
use crate::auth::routes as auth_routes;
use crate::stock::routes as stock_routes;

// ── Auth middleware ───────────────────────────────────────────────────────────

async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> Response {
    if let Some(user) = resolve_user(&state, req.headers()).await {
        req.extensions_mut().insert(user);
        return next.run(req).await;
    }
    Redirect::to("/login").into_response()
}

async fn admin_middleware(
    State(state): State<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> Response {
    match resolve_user(&state, req.headers()).await {
        Some(user) if user.role == "admin" => {
            req.extensions_mut().insert(user);
            next.run(req).await
        }
        Some(_) => Redirect::to("/").into_response(),
        None => Redirect::to("/login").into_response(),
    }
}

async fn resolve_user(
    state: &Arc<AppState>,
    headers: &axum::http::HeaderMap,
) -> Option<crate::auth::User> {
    let cookies = headers
        .get(axum::http::header::COOKIE)?
        .to_str()
        .ok()?;
    let session_id: Uuid = cookies
        .split(';')
        .find_map(|part| part.trim().strip_prefix("ss_session=").map(str::trim))
        .and_then(|v| v.parse().ok())?;
    auth::get_session_user(&state.pool, session_id).await
}

// ── Router ────────────────────────────────────────────────────────────────────

pub async fn serve(port: u16, node_id: Uuid, pool: PgPool, tenant_name: String) {
    let state = Arc::new(AppState { pool, node_id, tenant_name, web_port: port });

    // Public routes — no auth required
    let public_routes = Router::new()
        .route("/login", get(auth_routes::login_page).post(auth_routes::login_post));

    // Authenticated routes — any logged-in user (employee or admin)
    // Logout is here so that a valid session is required (expired → already logged out)
    let user_routes = Router::new()
        .route("/logout", post(auth_routes::logout))
        .route("/", get(stock_routes::dashboard))
        .route("/articles", get(stock_routes::articles_list))
        .route(
            "/articles/nouveau",
            get(stock_routes::article_form).post(stock_routes::article_create),
        )
        .route("/articles/:id/supprimer", post(stock_routes::article_delete))
        .route(
            "/mouvements",
            get(stock_routes::mouvements_page).post(stock_routes::mouvement_create),
        )
        .route_layer(middleware::from_fn_with_state(state.clone(), auth_middleware));

    // Admin-only routes
    let admin_routes = Router::new()
        .route("/admin", get(auth_routes::admin_dashboard))
        .route("/admin/utilisateurs", get(auth_routes::users_list))
        .route(
            "/admin/utilisateurs/nouveau",
            get(auth_routes::user_form).post(auth_routes::user_create),
        )
        .route(
            "/admin/utilisateurs/:id/toggle",
            post(auth_routes::user_toggle),
        )
        .route(
            "/admin/utilisateurs/:id/mdp",
            get(auth_routes::user_mdp_form).post(auth_routes::user_mdp_change),
        )
        .route_layer(middleware::from_fn_with_state(state.clone(), admin_middleware));

    let app = public_routes
        .merge(user_routes)
        .merge(admin_routes)
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(l) => {
            println!("  Web      : interface disponible sur http://0.0.0.0:{}", port);
            l
        }
        Err(e) => {
            println!("  Web      : impossible de démarrer sur :{} — {}", port, e);
            return;
        }
    };

    if let Err(e) = axum::serve(listener, app).await {
        println!("  Web      : erreur serveur — {}", e);
    }
}
