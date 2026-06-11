use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::stock::{routes, AppState};

pub async fn serve(port: u16, node_id: Uuid, pool: PgPool, tenant_name: String) {
    let state = Arc::new(AppState { pool, node_id, tenant_name });

    let app = Router::new()
        .route("/", get(routes::dashboard))
        .route("/articles", get(routes::articles_list))
        .route("/articles/nouveau", get(routes::article_form).post(routes::article_create))
        .route("/articles/:id/supprimer", post(routes::article_delete))
        .route("/mouvements", get(routes::mouvements_page).post(routes::mouvement_create))
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
