use std::io::Write;
use std::path::Path;

use anyhow::Result;
use hex;
use ss_crypto::{derive_recovery_key, generate_salt, Dek, DeviceKeyPair};
use uuid::Uuid;

use crate::config::NodeConfig;

/// Initialise ce nœud.
///
/// - `first_node = false` : génère uniquement la paire X25519. L'enrôlement
///   (réception de la DEK) se fera ensuite via `ss-node enroll`.
/// - `first_node = true`  : génère aussi la DEK, la scelle pour ce nœud, et
///   crée le blob de récupération protégé par un code saisi interactivement.
pub async fn run(first_node: bool, config_path: &Path) -> Result<()> {
    // Mode Docker : si les env vars sont présentes et que le nœud n'est pas encore init,
    // générer les clés automatiquement sans interaction.
    let is_docker_auto = std::env::var("TENANT_ID").is_ok()
        && std::env::var("REGISTRATION_TOKEN").is_ok()
        && !config_path.exists();

    if is_docker_auto {
        let keypair = DeviceKeyPair::generate();
        let node_id = Uuid::new_v4();
        let tenant_id = std::env::var("TENANT_ID")
            .ok()
            .and_then(|v| v.parse::<Uuid>().ok());
        let relay_url = std::env::var("RELAY_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());

        println!("Mode Docker détecté — initialisation automatique");
        println!("  ID      : {}", node_id);
        println!("  Clé pub : {}", hex::encode(keypair.public.as_bytes()));

        let config = NodeConfig {
            node_id,
            secret_key_hex: hex::encode(keypair.secret_bytes()),
            sealed_dek_hex: None,
            recovery_salt_hex: None,
            recovery_blob_hex: None,
            port: 9001,
            tenant_id,
            relay_url,
            pg_url: None,
            epoch: 1,
        };
        config.save(config_path)?;
        println!("Config sauvegardée : {}", config_path.display());
        println!("  Note : DEK non encore configurée. Le nœud s'annoncera au relais.");
        return Ok(());
    }

    if config_path.exists() {
        println!("Attention : config déjà présente : {}", config_path.display());
        println!("   Supprimez-la manuellement pour réinitialiser.");
        return Ok(());
    }

    let keypair = DeviceKeyPair::generate();
    let node_id = Uuid::new_v4();

    println!("Noeud initialisé");
    println!("  ID      : {}", node_id);
    println!("  Clé pub : {}", hex::encode(keypair.public.as_bytes()));

    let (sealed_dek_hex, recovery_salt_hex, recovery_blob_hex) = if first_node {
        // 1. Générer la DEK
        let dek = Dek::generate();

        // 2. Sceller la DEK pour ce nœud
        let sealed = keypair.public.seal_dek(&dek)?;

        // 3. Demander le code de récupération interactivement
        print!("Code de récupération (taper puis Entrée) : ");
        std::io::stdout().flush()?;
        let mut code = String::new();
        std::io::stdin().read_line(&mut code)?;
        let code = code.trim().to_string();

        // 4. Dériver une clé de récupération via Argon2id
        let salt = generate_salt();
        let rk_bytes = derive_recovery_key(&code, &salt)?;
        let rk_as_dek = Dek::from_bytes(rk_bytes);

        // 5. Chiffrer la DEK sous la clé de récupération
        let recovery_blob = rk_as_dek.encrypt(dek.as_bytes())?;

        println!("  DEK générée et scellée pour ce noeud");
        println!("  Blob de récupération créé — conservez ce code en lieu sûr !");

        (
            Some(hex::encode(&sealed)),
            Some(hex::encode(salt)),
            Some(hex::encode(&recovery_blob)),
        )
    } else {
        println!("  Aucune DEK — utilisez 'ss-node enroll' pour recevoir la DEK du cluster");
        (None, None, None)
    };

    let config = NodeConfig {
        node_id,
        secret_key_hex: hex::encode(keypair.secret_bytes()),
        sealed_dek_hex,
        recovery_salt_hex,
        recovery_blob_hex,
        port: 9001,
        tenant_id: None,
        relay_url: "http://localhost:8080".to_string(),
        pg_url: None,
        epoch: 1,
    };

    config.save(config_path)?;
    println!("Config sauvegardée : {}", config_path.display());
    Ok(())
}
