use anyhow::Result;
use argon2::password_hash::{rand_core::OsRng, SaltString};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

pub mod routes;

// ── Shared application state ──────────────────────────────────────────────────

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub node_id: Uuid,
    pub tenant_name: String,
}

// ── User model ────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub full_name: String,
    pub role: String,
    pub password_hash: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

// ── Cluster info ──────────────────────────────────────────────────────────────

pub struct ClusterInfo {
    pub is_primary: bool,
    pub standbys: Vec<StandbyInfo>,
    pub primary_host: Option<String>,
    pub replication_lag: Option<String>,
}

pub struct StandbyInfo {
    pub name: String,
    pub state: String,
}

pub async fn get_cluster_info(pool: &PgPool) -> ClusterInfo {
    let is_primary = sqlx::query_scalar::<_, bool>("SELECT NOT pg_is_in_recovery()")
        .fetch_one(pool)
        .await
        .unwrap_or(false);

    if is_primary {
        let standbys = sqlx::query_as::<_, (String, String)>(
            "SELECT application_name, state FROM pg_stat_replication ORDER BY application_name",
        )
        .fetch_all(pool)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|(name, state)| StandbyInfo { name, state })
        .collect();

        ClusterInfo {
            is_primary: true,
            standbys,
            primary_host: None,
            replication_lag: None,
        }
    } else {
        let row = sqlx::query_as::<_, (Option<String>, Option<String>)>(
            "SELECT sender_host, status FROM pg_stat_wal_receiver LIMIT 1",
        )
        .fetch_optional(pool)
        .await
        .unwrap_or(None);

        let (primary_host, replication_lag) = match row {
            Some((host, status)) => (host, status),
            None => (None, None),
        };

        ClusterInfo {
            is_primary: false,
            standbys: vec![],
            primary_host,
            replication_lag,
        }
    }
}

// ── Axum extractors ───────────────────────────────────────────────────────────

// ── Migrations ────────────────────────────────────────────────────────────────

pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id            UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
            username      VARCHAR(100) NOT NULL UNIQUE,
            full_name     VARCHAR(200) NOT NULL DEFAULT '',
            role          VARCHAR(20)  NOT NULL DEFAULT 'employee'
                          CHECK (role IN ('admin','employee')),
            password_hash TEXT         NOT NULL,
            is_active     BOOLEAN      NOT NULL DEFAULT TRUE,
            created_at    TIMESTAMPTZ  NOT NULL DEFAULT NOW()
        )",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS sessions (
            id         UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
            user_id    UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            expires_at TIMESTAMPTZ NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS sessions_expires_at_idx ON sessions (expires_at)",
    )
    .execute(pool)
    .await?;

    Ok(())
}

// ── Bootstrap admin ───────────────────────────────────────────────────────────

pub async fn ensure_default_admin(pool: &PgPool) -> Result<()> {
    let (count,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM users WHERE role = 'admin'")
            .fetch_one(pool)
            .await?;

    if count > 0 {
        return Ok(());
    }

    use rand::distributions::Alphanumeric;
    use rand::Rng;
    let password: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(14)
        .map(char::from)
        .collect();

    let hash = hash_password(&password)?;
    sqlx::query(
        "INSERT INTO users (username, full_name, role, password_hash)
         VALUES ($1, $2, $3, $4)",
    )
    .bind("admin")
    .bind("Administrateur PME")
    .bind("admin")
    .bind(&hash)
    .execute(pool)
    .await?;

    println!("╔═══════════════════════════════════════════════╗");
    println!("║   PREMIER DÉMARRAGE — Compte administrateur   ║");
    println!("║                                               ║");
    println!("║   Identifiant : admin                         ║");
    println!("║   Mot de passe: {:<14}                ║", password);
    println!("║                                               ║");
    println!("║   → Changez-le dès la première connexion !   ║");
    println!("╚═══════════════════════════════════════════════╝");

    Ok(())
}

// ── Password ──────────────────────────────────────────────────────────────────

pub fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| anyhow::anyhow!("Erreur hachage : {}", e))
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    PasswordHash::new(hash)
        .ok()
        .map(|h| {
            Argon2::default()
                .verify_password(password.as_bytes(), &h)
                .is_ok()
        })
        .unwrap_or(false)
}

// ── Session helpers ───────────────────────────────────────────────────────────

pub async fn authenticate(pool: &PgPool, username: &str, password: &str) -> Option<User> {
    let user: Option<User> = sqlx::query_as::<_, User>(
        "SELECT id, username, full_name, role, password_hash, is_active, created_at
         FROM users WHERE username = $1 AND is_active = TRUE",
    )
    .bind(username)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    user.filter(|u| verify_password(password, &u.password_hash))
}

pub async fn create_session(pool: &PgPool, user_id: Uuid) -> Result<Uuid> {
    let (id,): (Uuid,) = sqlx::query_as(
        "INSERT INTO sessions (user_id, expires_at)
         VALUES ($1, NOW() + INTERVAL '8 hours')
         RETURNING id",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    Ok(id)
}

pub async fn get_session_user(pool: &PgPool, session_id: Uuid) -> Option<User> {
    sqlx::query_as::<_, User>(
        "SELECT u.id, u.username, u.full_name, u.role, u.password_hash,
                u.is_active, u.created_at
         FROM sessions s
         JOIN users u ON u.id = s.user_id
         WHERE s.id = $1
           AND s.expires_at > NOW()
           AND u.is_active = TRUE",
    )
    .bind(session_id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
}

pub async fn delete_session(pool: &PgPool, session_id: Uuid) -> Result<()> {
    sqlx::query("DELETE FROM sessions WHERE id = $1")
        .bind(session_id)
        .execute(pool)
        .await?;
    Ok(())
}

// ── User management ───────────────────────────────────────────────────────────

pub async fn create_user(
    pool: &PgPool,
    username: &str,
    full_name: &str,
    role: &str,
    password: &str,
) -> Result<Uuid> {
    let hash = hash_password(password)?;
    let (id,): (Uuid,) = sqlx::query_as(
        "INSERT INTO users (username, full_name, role, password_hash)
         VALUES ($1, $2, $3, $4) RETURNING id",
    )
    .bind(username.trim())
    .bind(full_name.trim())
    .bind(role)
    .bind(&hash)
    .fetch_one(pool)
    .await?;
    Ok(id)
}

pub async fn list_users(pool: &PgPool) -> Result<Vec<User>> {
    Ok(sqlx::query_as::<_, User>(
        "SELECT id, username, full_name, role, password_hash, is_active, created_at
         FROM users ORDER BY role DESC, username",
    )
    .fetch_all(pool)
    .await?)
}

pub async fn toggle_user(pool: &PgPool, id: Uuid) -> Result<()> {
    sqlx::query(
        "UPDATE users SET is_active = NOT is_active WHERE id = $1 AND role != 'admin'",
    )
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_password(pool: &PgPool, id: Uuid, new_password: &str) -> Result<()> {
    let hash = hash_password(new_password)?;
    sqlx::query("UPDATE users SET password_hash = $1 WHERE id = $2")
        .bind(&hash)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

// ── Cookie helpers ────────────────────────────────────────────────────────────

pub fn session_cookie_set(session_id: Uuid) -> String {
    format!(
        "ss_session={}; Path=/; HttpOnly; SameSite=Strict; Max-Age=28800",
        session_id
    )
}

pub fn session_cookie_clear() -> &'static str {
    "ss_session=; Path=/; HttpOnly; SameSite=Strict; Max-Age=0"
}
