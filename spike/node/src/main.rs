use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

mod auth;
mod cmd;
mod config;
mod relay_client;
mod stock;
mod web;

#[derive(Parser)]
#[command(name = "ss-node", about = "SaaS Souverain — noeud PME")]
#[command(version, author)]
struct Cli {
    /// Chemin vers le fichier de configuration du nœud
    #[arg(long, global = true)]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialise ce nœud (génère les clés X25519)
    Init {
        /// Premier nœud du cluster (génère la DEK et le code de récupération)
        #[arg(long)]
        first: bool,
    },
    /// Démarre le nœud en mode actif ou passif
    Run {
        /// Mode d'exécution : "active" ou "passive"
        #[arg(long, default_value = "passive")]
        mode: String,
    },
    /// Affiche le statut du nœud
    Status,
    /// Promouvoir ce nœud standby en primaire PostgreSQL et incrémenter l'époque
    Failover,
    /// Dé-enrôler un appareil et effectuer une rotation DEK
    Delist {
        /// UUID de l'appareil à retirer du cluster
        #[arg(long)]
        device_id: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    let config_path = cli
        .config
        .unwrap_or_else(|| config::NodeConfig::default_path());

    match cli.command {
        Commands::Init { first } => {
            cmd::init::run(first, &config_path).await?;
        }
        Commands::Run { mode } => {
            let run_mode = match mode.as_str() {
                "active" => cmd::run::RunMode::Active,
                "passive" => cmd::run::RunMode::Passive,
                other => {
                    eprintln!("Mode inconnu : {}. Utiliser 'active' ou 'passive'", other);
                    std::process::exit(1);
                }
            };
            cmd::run::run(run_mode, &config_path).await?;
        }
        Commands::Status => {
            cmd::status::run(&config_path).await?;
        }
        Commands::Failover => {
            cmd::failover::run(&config_path).await?;
        }
        Commands::Delist { device_id } => {
            cmd::delist::run(&device_id, &config_path).await?;
        }
    }

    Ok(())
}
