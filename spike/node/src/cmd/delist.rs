use std::path::Path;

use anyhow::{anyhow, Context, Result};
use hex;
use rand::RngCore;
use ss_crypto::{Dek, DeviceKeyPair};
use ss_journal::Journal;

use crate::config::NodeConfig;

/// De-enroll a device from this cluster and rotate the DEK.
///
/// Proof: even if the de-listed node's disk is seized, it cannot decrypt
/// future data because the DEK has been rotated and only active nodes
/// receive the new wrapped DEK.
///
/// Spike implementation: rotates DEK for this node only (the de-listed node's
/// wrapped DEK is simply discarded — in production each remaining device would
/// receive its own wrapped copy of the new DEK).
pub async fn run(device_id_str: &str, config_path: &Path) -> Result<()> {
    let mut config = NodeConfig::load(config_path)?;
    config.apply_env_overrides();

    // 1. Verify the de-listed device is not *this* node
    let target = device_id_str
        .parse::<uuid::Uuid>()
        .with_context(|| format!("Invalid device UUID: {device_id_str}"))?;

    if target == config.node_id {
        return Err(anyhow!(
            "Cannot de-enroll this node from itself. Use another authorised node."
        ));
    }

    println!("[delist] De-enrolling device {} ...", target);

    // 2. Open the current DEK using this node's private key
    let sealed_hex = config
        .sealed_dek_hex
        .as_ref()
        .ok_or_else(|| anyhow!("This node is not enrolled — no sealed DEK."))?;

    let secret_bytes = hex::decode(&config.secret_key_hex)
        .context("Invalid secret_key_hex in config")?;
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&secret_bytes);
    let keypair = DeviceKeyPair::from_secret_bytes(arr);

    let sealed = hex::decode(sealed_hex).context("Invalid sealed_dek_hex in config")?;
    let old_dek = keypair
        .open_sealed_dek(&sealed)
        .context("Failed to decrypt current DEK")?;

    println!("[delist] Current DEK decrypted.");

    // 3. Generate a new DEK
    let mut new_key_bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut new_key_bytes);
    let new_dek = Dek::from_bytes(new_key_bytes);

    println!("[delist] New DEK generated.");

    // 4. Write a rotation entry in the journal (encrypted with old DEK)
    //    This creates an audit trail: "DEK rotated, device X de-listed."
    let journal_path = config_path
        .parent()
        .unwrap_or(Path::new("."))
        .join("journal.bin");

    let mut journal = Journal::open(&journal_path, old_dek)
        .context("Cannot open journal")?;

    let payload = format!("dek_rotation:delist:{}", target).into_bytes();
    let idx = journal
        .append(config.epoch, config.node_id, "delist.dek_rotation", payload)
        .context("Failed to write rotation entry to journal")?;

    println!("[delist] Journal rotation entry written at index {}.", idx);

    // 5. Wrap the new DEK for this node (only active nodes keep the new DEK)
    let new_sealed = keypair.public.seal_dek(&new_dek)?;
    config.sealed_dek_hex = Some(hex::encode(&new_sealed));
    config.save(config_path)?;

    println!("[delist] New DEK wrapped and saved for this node.");
    println!("[delist] De-listed device {} will be unable to decrypt future data.", target);

    // 6. Check cluster size after de-listing
    // In the spike we don't maintain a local roster — check via relay.
    if let Some(tenant_id) = config.tenant_id {
        let relay = crate::relay_client::RelayClient::new(&config.relay_url);
        match relay.get_peers(tenant_id).await {
            Ok(peers) => {
                let remaining = peers.len();
                if remaining < 3 {
                    println!(
                        "[delist] ALERT: Cluster now has {} node(s) — automatic failover requires ≥ 3. \
                         Only manual failover is available.",
                        remaining
                    );
                } else {
                    println!("[delist] Cluster has {} node(s) — automatic failover still possible.", remaining);
                }
            }
            Err(e) => println!("[delist] Relay unreachable: {} — cannot confirm cluster size.", e),
        }
    }

    println!("[delist] DEK rotation complete. Re-enroll remaining nodes to receive the new DEK.");
    Ok(())
}
