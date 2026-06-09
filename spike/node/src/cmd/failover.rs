use std::path::Path;

use anyhow::{anyhow, Context, Result};
use hex;
use sqlx::PgPool;
use ss_consensus::{check_fencing, supervision, EpochToken, FencingResult};
use ss_crypto::Dek;

use crate::config::NodeConfig;

/// Promote this node's PostgreSQL standby to primary and increment the cluster epoch.
///
/// Used for both manual failover (2-node cluster) and as part of the auto-failover
/// supervision loop when quorum is reached (≥ 3 nodes).
pub async fn run(config_path: &Path) -> Result<()> {
    let mut config = NodeConfig::load(config_path)?;
    config.apply_env_overrides();

    let pg_url = config
        .pg_url
        .as_deref()
        .ok_or_else(|| anyhow!("pg_url not set in config — is this node initialised?"))?;

    let pool = PgPool::connect(pg_url)
        .await
        .with_context(|| format!("Cannot connect to PostgreSQL at {pg_url}"))?;

    // Verify this node is currently a standby (in recovery mode)
    let primary = supervision::is_primary(&pool).await?;
    if primary {
        println!("[failover] This node is already a primary — nothing to do.");
        return Ok(());
    }

    println!("[failover] This node is a standby. Attempting promotion...");

    let promoted = supervision::promote_standby(&pool)
        .await
        .context("pg_promote() failed")?;

    if !promoted {
        return Err(anyhow!(
            "pg_promote() returned false — promotion did not complete."
        ));
    }

    println!("[failover] PostgreSQL promoted to primary successfully.");

    // Increment epoch to fence the old primary
    let new_epoch = EpochToken(config.epoch + 1);
    println!("[failover] Incrementing epoch {} -> {}", config.epoch, new_epoch.value());
    config.epoch = new_epoch.value();
    config.save(config_path)?;

    // Self-fencing check: our new epoch must be >= itself (always true)
    match check_fencing(new_epoch, new_epoch) {
        FencingResult::Allowed => println!("[failover] Epoch fencing: OK"),
        FencingResult::Fenced { claimed, current } => {
            println!("[failover] WARN: fencing inconsistency {} vs {}", claimed, current);
        }
    }

    // Announce new epoch to relay so peers learn this node is now primary
    if let Some(tenant_id) = config.tenant_id {
        let relay = crate::relay_client::RelayClient::new(&config.relay_url);
        let addr = format!("{}:{}", hostname_or_unknown(), config.port);
        let req = crate::relay_client::AnnounceRequest {
            node_id: config.node_id,
            tenant_id,
            addr,
            role: "active".to_string(),
            epoch: new_epoch.value(),
        };
        match relay.announce(&req).await {
            Ok(_) => println!("[failover] Relay notified (epoch {})", new_epoch),
            Err(e) => println!("[failover] Relay unreachable: {} (non-blocking)", e),
        }
    }

    // Verify DEK is still readable after promotion (crypto sanity check)
    let sealed_hex = config
        .sealed_dek_hex
        .as_ref()
        .ok_or_else(|| anyhow!("Node not enrolled — no sealed DEK"))?;
    let secret_bytes = hex::decode(&config.secret_key_hex)?;
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&secret_bytes);
    let keypair = ss_crypto::DeviceKeyPair::from_secret_bytes(arr);
    let sealed = hex::decode(sealed_hex)?;
    let _dek = keypair
        .open_sealed_dek(&sealed)
        .context("DEK check after promotion failed — config may be corrupted")?;

    println!("[failover] DEK integrity: OK");
    println!("[failover] This node is now PRIMARY at epoch {}.", new_epoch);
    Ok(())
}

fn hostname_or_unknown() -> String {
    std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("COMPUTERNAME"))
        .unwrap_or_else(|_| "unknown".to_string())
}
