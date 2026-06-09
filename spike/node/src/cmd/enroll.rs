use std::io::Write;
use std::path::Path;

use anyhow::{Context, Result};
use hex;
use qrcode::render::unicode;
use qrcode::QrCode;
use ss_crypto::DeviceKeyPair;

use crate::config::NodeConfig;

/// Affiche la clé publique du nœud sous forme de QR code ASCII et attend
/// la DEK scellée (hex) collée depuis stdin.
///
/// Pour le spike : la transmission de la DEK scellée est simplifiée — on la
/// lit depuis stdin en hex. TCP sera implémenté dans la phase complète.
pub async fn run(config_path: &Path) -> Result<()> {
    let mut config = NodeConfig::load(config_path)?;

    if config.sealed_dek_hex.is_some() {
        println!("Ce noeud est déjà enrôlé (DEK présente).");
        return Ok(());
    }

    // Recomputer la clé publique depuis la clé secrète stockée
    let secret_bytes_vec = hex::decode(&config.secret_key_hex)
        .context("Clé secrète hex invalide dans la config")?;
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&secret_bytes_vec);
    let keypair = DeviceKeyPair::from_secret_bytes(arr);

    let pubkey_hex = hex::encode(keypair.public.as_bytes());

    println!("=== Enrôlement du noeud {} ===", config.node_id);
    println!("Clé publique : {}", pubkey_hex);
    println!();

    // Générer et afficher le QR code en ASCII dans le terminal
    let code = QrCode::new(pubkey_hex.as_bytes())
        .context("Impossible de générer le QR code")?;
    let image = code
        .render::<unicode::Dense1x2>()
        .dark_color(unicode::Dense1x2::Dark)
        .light_color(unicode::Dense1x2::Light)
        .build();
    println!("{}", image);

    println!("Faites scanner ce QR par un noeud déjà autorisé.");
    println!("Collez ensuite la DEK scellée (hex) reçue :");
    print!("> ");
    std::io::stdout().flush()?;

    let mut sealed_hex = String::new();
    std::io::stdin().read_line(&mut sealed_hex)?;
    let sealed_hex = sealed_hex.trim();

    // Décoder et vérifier qu'on peut ouvrir la DEK avec notre clé privée
    let sealed = hex::decode(sealed_hex)
        .context("La DEK scellée fournie n'est pas un hex valide")?;
    let _dek = keypair
        .open_sealed_dek(&sealed)
        .context("Impossible d'ouvrir la DEK scellée — vérifiez qu'elle correspond à ce noeud")?;

    println!("DEK reçue et vérifiée. Noeud enrôlé avec succès.");

    config.sealed_dek_hex = Some(sealed_hex.to_string());
    config.save(config_path)?;
    Ok(())
}
