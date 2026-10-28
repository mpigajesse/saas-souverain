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

/// Position WAL reçue par ce standby, en octets.
///
/// Sert à départager deux standbys candidats à la promotion : le plus avancé
/// gagne (moins de données perdues). Retourne `None` si la valeur est nulle ou
/// illisible — sur un primaire, `pg_last_wal_receive_lsn()` renvoie NULL.
pub async fn wal_receive_lsn(pool: &PgPool) -> Result<Option<u64>> {
    let row: (Option<String>,) =
        sqlx::query_as("SELECT pg_last_wal_receive_lsn()::text")
            .fetch_one(pool)
            .await?;
    Ok(row.0.as_deref().and_then(crate::parse_lsn))
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
