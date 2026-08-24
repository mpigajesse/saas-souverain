use anyhow::Result;
use sqlx::PgPool;

/// Returns true if this PostgreSQL instance is a primary (not in recovery mode).
pub async fn is_primary(pool: &PgPool) -> Result<bool> {
    let row: (bool,) = sqlx::query_as("SELECT pg_is_in_recovery()")
        .fetch_one(pool)
        .await?;
    Ok(!row.0)
}

/// Promote a standby PostgreSQL instance to primary.
/// Calls `pg_promote()` (PostgreSQL 12+).
/// Returns true if the promotion was triggered successfully.
pub async fn promote_standby(pool: &PgPool) -> Result<bool> {
    let row: (bool,) = sqlx::query_as("SELECT pg_promote(wait := true)")
        .fetch_one(pool)
        .await?;
    Ok(row.0)
}

/// Returns true if the WAL receiver is actively streaming from a primary.
/// An empty result means the connection to the primary was lost.
pub async fn wal_receiver_active(pool: &PgPool) -> Result<bool> {
    let row: (i64,) = sqlx::query_as(
        "SELECT count(*) FROM pg_stat_wal_receiver WHERE status = 'streaming'",
    )
    .fetch_one(pool)
    .await?;
    Ok(row.0 > 0)
}

/// Returns the current PostgreSQL timeline ID (TLI).
///
/// PostgreSQL increments the timeline on every promotion (`pg_promote`), so the
/// TLI is a native, monotonic epoch — no hand-rolled consensus (décision n°6).
/// A former primary that comes back online keeps its old (lower) timeline while
/// the cluster has moved on to a higher one, which is exactly what fencing needs.
pub async fn current_timeline(pool: &PgPool) -> Result<i64> {
    // pg_control_checkpoint() expose timeline_id (integer). L'utilisateur métier
    // est superuser (bootstrap initdb) → accès autorisé. Fonctionne primaire ET standby.
    let tli: i32 = sqlx::query_scalar("SELECT timeline_id FROM pg_control_checkpoint()")
        .fetch_one(pool)
        .await?;
    Ok(tli as i64)
}

/// Check if the cluster can perform automatic failover (≥ 3 nodes required).
/// Uses the count of replication slots or connected standbys visible from the primary.
pub async fn connected_standby_count(pool: &PgPool) -> Result<i64> {
    let row: (i64,) = sqlx::query_as(
        "SELECT count(*) FROM pg_stat_replication WHERE state = 'streaming'",
    )
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

#[cfg(test)]
mod tests {
    // Integration tests require a running PostgreSQL instance — run with:
    // TEST_PG_URL=postgres://... cargo test --package ss-consensus -- --ignored
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn primary_detection() {
        let url = std::env::var("TEST_PG_URL").expect("TEST_PG_URL must be set");
        let pool = PgPool::connect(&url).await.unwrap();
        let primary = is_primary(&pool).await.unwrap();
        println!("is_primary: {}", primary);
    }
}
