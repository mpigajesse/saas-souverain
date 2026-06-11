use anyhow::{Context, Result};
use std::path::Path;

use crate::auth;
use crate::config::NodeConfig;

pub async fn run(
    username: &str,
    full_name: &str,
    role: &str,
    password: &str,
    config_path: &Path,
) -> Result<()> {
    let config = NodeConfig::load(config_path)
        .with_context(|| format!("Impossible de lire la config : {}", config_path.display()))?;

    let pg_url = config.pg_url.as_deref().unwrap_or("postgres://pme:pme@localhost:5432/pme_db");
    let pool = sqlx::PgPool::connect(pg_url)
        .await
        .context("Connexion PostgreSQL échouée")?;

    auth::run_migrations(&pool).await.context("Migration échouée")?;

    let hash = auth::hash_password(password).context("Hachage mot de passe échoué")?;

    sqlx::query(
        "INSERT INTO users (username, full_name, role, password_hash)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (username) DO UPDATE
           SET full_name = EXCLUDED.full_name,
               role      = EXCLUDED.role,
               password_hash = EXCLUDED.password_hash",
    )
    .bind(username)
    .bind(full_name)
    .bind(role)
    .bind(&hash)
    .execute(&pool)
    .await
    .with_context(|| format!("Insertion de l'utilisateur '{}' échouée", username))?;

    println!("✓ Utilisateur créé : {} ({}) [{}]", username, full_name, role);
    Ok(())
}
