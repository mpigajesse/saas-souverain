use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

mod application;
mod auth;
mod cmd;
mod config;
mod enrolement;
mod patroni;
mod relay_client;
mod sauvegarde;
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
    /// Démarre le nœud. La bascule PostgreSQL appartient à etcd/Patroni :
    /// pour une bascule manuelle, `patronictl switchover` (refonte V3).
    Run {
        /// "active" = premier serveur du cluster (a généré la DEK), "passive" sinon
        #[arg(long, default_value = "passive")]
        mode: String,
    },
    /// Affiche le statut du nœud
    Status,
    /// Dé-enrôler un appareil et effectuer une rotation DEK
    Delist {
        /// UUID de l'appareil à retirer du cluster
        #[arg(long)]
        device_id: String,
    },
    /// Recevoir la DEK du cluster depuis un serveur qui la détient (code à usage unique)
    Enroler {
        /// Adresse web du serveur qui a émis le code (ex. http://192.168.1.10:9001)
        #[arg(long)]
        depuis: String,
        /// Code d'enrôlement affiché par sa console (/amane/admin/enrolement)
        #[arg(long)]
        code: String,
    },
    /// Sauvegarder maintenant les bases, chiffrées par la DEK, sur le relais
    Sauvegarder,
    /// Restaurer une base depuis sa sauvegarde chiffrée sur le relais
    Restaurer {
        /// Base à restaurer : amane ou application
        #[arg(long)]
        base: String,
        /// Jour de la sauvegarde : 1 (lundi) à 7 (dimanche)
        #[arg(long)]
        jour: u32,
    },
    /// Créer ou mettre à jour un utilisateur dans la base de données
    Adduser {
        #[arg(long)]
        username: String,
        #[arg(long)]
        name: String,
        #[arg(long, default_value = "employee")]
        role: String,
        #[arg(long)]
        password: String,
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
        Commands::Delist { device_id } => {
            cmd::delist::run(&device_id, &config_path).await?;
        }
        Commands::Enroler { depuis, code } => {
            cmd::enroler::run(&depuis, &code, &config_path).await?;
        }
        Commands::Sauvegarder => {
            cmd::sauvegarder::run(&config_path).await?;
        }
        Commands::Restaurer { base, jour } => {
            cmd::sauvegarder::restaurer_base(&base, jour, &config_path).await?;
        }
        Commands::Adduser { username, name, role, password } => {
            cmd::adduser::run(&username, &name, &role, &password, &config_path).await?;
        }
    }

    Ok(())
}
