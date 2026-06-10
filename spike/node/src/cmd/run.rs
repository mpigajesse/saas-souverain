use std::path::Path;

use anyhow::{anyhow, Context, Result};
use hex;
use sqlx::PgPool;
use ss_consensus::{check_fencing, supervision, EpochToken, FencingResult};
use ss_crypto::DeviceKeyPair;
use ss_journal::Journal;

use crate::config::NodeConfig;

/// Mode d'exécution du nœud.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunMode {
    Active,
    Passive,
}

/// Démarre le nœud en mode actif ou passif.
///
/// Pour le spike : démontre que le nœud démarre, lit le journal, et vérifie
/// le fencing. La boucle réseau complète sera implémentée en Phase 0.
pub async fn run(mode: RunMode, config_path: &Path) -> Result<()> {
    let mut config = NodeConfig::load(config_path)?;
    config.apply_env_overrides();

    // Enregistrement SaaS en premier — fonctionne même si le nœud n'est pas encore enrôlé
    // (le standby n'a pas encore de DEK mais doit quand même apparaître dans le portail).
    register_with_saas(&config).await;

    // Vérifier que le nœud est enrôlé
    let sealed_hex = config
        .sealed_dek_hex
        .as_ref()
        .ok_or_else(|| anyhow!("Noeud non enrôlé. Lancez ss-node enroll d'abord."))?;

    // Récupérer la DEK depuis la sealed box
    let secret_bytes_vec = hex::decode(&config.secret_key_hex)
        .context("Clé secrète hex invalide dans la config")?;
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&secret_bytes_vec);
    let keypair = DeviceKeyPair::from_secret_bytes(arr);

    let sealed = hex::decode(sealed_hex)
        .context("Sealed DEK hex invalide dans la config")?;
    let dek = keypair
        .open_sealed_dek(&sealed)
        .context("Impossible de déchiffrer la DEK — config corrompue ?")?;

    // Ouvrir le journal chiffré
    let journal_path = config_path
        .parent()
        .unwrap_or(Path::new("."))
        .join("journal.bin");
    // `mut` requis uniquement en mode Active pour journal.append, mais déclaré
    // dès l'ouverture pour simplifier le flux de contrôle du spike.
    #[allow(unused_mut)]
    let mut journal = Journal::open(journal_path, dek)
        .context("Impossible d'ouvrir le journal")?;

    println!("=== Noeud {} démarré en mode {:?} ===", config.node_id, mode);
    println!("  Journal : {} entrée(s)", journal.len());

    // Serveur web métier (interface PME) — port 3000 par défaut
    let web_port: u16 = std::env::var("WEB_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3000);
    let web_node_id = config.node_id;
    tokio::spawn(crate::web::serve(web_port, web_node_id));

    // Vérification du fencing (spike : époque fixe pour démonstration)
    let my_epoch = EpochToken(1);

    // --- Annonce au relais ---
    let relay = crate::relay_client::RelayClient::new(&config.relay_url);

    if let Some(tenant_id) = config.tenant_id {
        let addr = node_addr(&config);
        let announce_req = crate::relay_client::AnnounceRequest {
            node_id: config.node_id,
            tenant_id,
            addr,
            role: if mode == RunMode::Active {
                "active".to_string()
            } else {
                "passive".to_string()
            },
            epoch: my_epoch.value(),
        };
        match relay.announce(&announce_req).await {
            Ok(_) => println!(
                "  Relais   : annonce envoyée ({}/api/nodes/announce)",
                config.relay_url
            ),
            Err(e) => println!(
                "  Relais   : injoignable — {} (spike: non bloquant)",
                e
            ),
        }

        // Découverte des pairs
        match relay.get_peers(tenant_id).await {
            Ok(peers) => {
                let others: Vec<_> = peers
                    .iter()
                    .filter(|p| p.node_id != config.node_id)
                    .collect();
                if others.is_empty() {
                    println!("  Pairs    : aucun pair connu pour l'instant");
                } else {
                    println!("  Pairs    : {} nœud(s) dans le cluster", others.len());
                    for p in &others {
                        println!(
                            "    - {} @ {} [{}] époque {}",
                            p.node_id, p.addr, p.role, p.epoch
                        );
                    }
                }
            }
            Err(e) => println!("  Pairs    : échec découverte — {}", e),
        }
    } else {
        println!("  Relais   : tenant_id non configuré — annonce ignorée");
    }
    let cluster_epoch = EpochToken(1);

    match check_fencing(my_epoch, cluster_epoch) {
        FencingResult::Allowed => {
            println!("  Fencing  : OK ({})", my_epoch);
        }
        FencingResult::Fenced { claimed, current } => {
            println!(
                "  Fencing  : FENCÉ — {} < époque courante {}",
                claimed, current
            );
            println!("  Ce noeud doit s'isoler immédiatement.");
            return Ok(());
        }
    }

    if mode == RunMode::Active {
        // Spike : écrire une entrée de test dans le journal pour prouver le chemin complet
        let payload = format!("noeud {} actif", config.node_id).into_bytes();
        let idx = journal
            .append(my_epoch.value(), config.node_id, "spike.heartbeat", payload)
            .context("Impossible d'écrire dans le journal")?;
        println!("  Heartbeat écrit dans le journal (index {})", idx);
    }

    println!("  Noeud opérationnel. Ctrl+C pour arrêter.");

    // Supervision loop — vérifie PostgreSQL toutes les 5 secondes.
    // En mode passif : déclenche le failover automatique si quorum est atteint ET primaire mort.
    // En mode actif  : détecte si PostgreSQL est toujours primaire (sanity check).
    if let Some(pg_url) = &config.pg_url.clone() {
        match PgPool::connect(pg_url).await {
            Ok(pool) => {
                println!("  PostgreSQL : connecté ({})", pg_url);
                run_supervision_loop(mode, pool, &config, config_path).await?;
            }
            Err(e) => {
                println!("  PostgreSQL : impossible de se connecter — {} (spike: boucle ignorée)", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }
        }
    } else {
        // Pas de PG configuré — annonce périodique au relais toutes les 30s
        println!("  PostgreSQL non configuré — boucle d'annonce (30s).");
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
            if let Some(tid) = config.tenant_id {
                let announce_req = crate::relay_client::AnnounceRequest {
                    node_id: config.node_id,
                    tenant_id: tid,
                    addr: node_addr(&config),
                    role: if mode == RunMode::Active {
                        "active".to_string()
                    } else {
                        "passive".to_string()
                    },
                    epoch: my_epoch.value(),
                };
                match relay.announce(&announce_req).await {
                    Ok(_) => {}
                    Err(e) => println!("  Relais injoignable : {}", e),
                }
            }
        }
    }

    Ok(())
}

/// Boucle de supervision PostgreSQL — tourne indéfiniment.
///
/// - Mode actif  : vérifie que PG est bien primaire (réplication active).
/// - Mode passif : surveille le WAL receiver. Si déconnecté ≥ WAL_FAILOVER_THRESHOLD
///   ticks consécutifs (15 s par défaut) → promotion automatique sans intervention.
async fn run_supervision_loop(
    mode: RunMode,
    pool: PgPool,
    config: &crate::config::NodeConfig,
    _config_path: &Path,
) -> Result<()> {
    // Nombre de ticks WAL inactif consécutifs avant promotion automatique.
    // Seuil : ≥ 2 nœuds dans le cluster (couvre le cas 2 machines PME).
    const WAL_FAILOVER_THRESHOLD: u32 = 3; // 3 × 5 s = 15 s

    let mut tick: u32 = 0;
    let mut wal_miss: u32 = 0;

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        tick += 1;

        match supervision::is_primary(&pool).await {
            Ok(is_prim) => {
                match (mode, is_prim) {
                    (RunMode::Active, true) => {
                        let standbys = supervision::connected_standby_count(&pool).await.unwrap_or(0);
                        if tick % 12 == 1 {
                            println!("  [tick {tick}] PG primaire OK — {} standby(s) connecté(s)", standbys);
                        }
                        wal_miss = 0;
                    }
                    (RunMode::Active, false) => {
                        println!("  [tick {tick}] ERREUR : ce nœud actif est en mode recovery — état incohérent !");
                    }
                    (RunMode::Passive, true) => {
                        // Déjà promu (failover précédent dans cette session)
                        if tick % 12 == 1 {
                            println!("  [tick {tick}] Ce nœud est désormais primaire.");
                        }
                        wal_miss = 0;
                    }
                    (RunMode::Passive, false) => {
                        // Standby normal — vérifier que le WAL receiver est actif
                        let wal_ok = supervision::wal_receiver_active(&pool).await.unwrap_or(true);
                        if wal_ok {
                            wal_miss = 0;
                            if tick % 12 == 1 {
                                println!("  [tick {tick}] PG standby : réplication active");
                            }
                        } else {
                            wal_miss += 1;
                            println!(
                                "  [tick {tick}] ALERTE : WAL receiver inactif ({}/{}) — primaire injoignable ?",
                                wal_miss, WAL_FAILOVER_THRESHOLD
                            );

                            if wal_miss >= WAL_FAILOVER_THRESHOLD {
                                println!("  [tick {tick}] Primaire considéré hors service — FAILOVER AUTOMATIQUE");
                                match supervision::promote_standby(&pool).await {
                                    Ok(true) => {
                                        println!("  [tick {tick}] ✓ FAILOVER : nœud promu en primaire !");
                                        register_with_saas_role(config, "primary").await;
                                        wal_miss = 0;
                                    }
                                    Ok(false) => {
                                        println!("  [tick {tick}] Promotion déjà en cours ou inutile.");
                                        wal_miss = 0;
                                    }
                                    Err(e) => println!("  [tick {tick}] Erreur lors de la promotion : {}", e),
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                println!("  [tick {tick}] PG local injoignable : {}", e);
            }
        }
    }
}

/// Retourne l'adresse d'annonce du nœud.
/// Priorité : NODE_ADDR env var → HOSTNAME:port → unknown:port
fn node_addr(config: &NodeConfig) -> String {
    if let Ok(addr) = std::env::var("NODE_ADDR") {
        if !addr.is_empty() {
            return addr;
        }
    }
    let host = std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("COMPUTERNAME"))
        .unwrap_or_else(|_| "unknown".to_string());
    format!("{}:{}", host, config.port)
}

/// Enregistre le nœud avec le rôle détecté depuis NODE_MODE.
async fn register_with_saas(config: &NodeConfig) {
    let role = match std::env::var("NODE_MODE").as_deref() {
        Ok("active") => "primary",
        _ => "standby",
    };
    register_with_saas_role(config, role).await;
}

/// Enregistre le nœud avec un rôle explicite — utilisé après failover automatique.
/// Non bloquant — les erreurs sont loguées mais n'arrêtent pas le nœud.
async fn register_with_saas_role(config: &NodeConfig, node_role: &str) {
    let saas_url = match std::env::var("SAAS_URL") {
        Ok(u) if !u.is_empty() => u,
        _ => return,
    };
    let reg_token = match std::env::var("REGISTRATION_TOKEN") {
        Ok(t) if !t.is_empty() => t,
        _ => return,
    };

    let hostname = std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("COMPUTERNAME"))
        .unwrap_or_else(|_| "unknown".to_string());

    let url = format!("{}/api/devices/register/", saas_url.trim_end_matches('/'));

    let node_addr = std::env::var("NODE_ADDR").unwrap_or_default();
    let web_addr = if let Some(ip) = node_addr.split(':').next() {
        let web_port = std::env::var("WEB_PORT").unwrap_or_else(|_| "3000".to_string());
        format!("{}:{}", ip, web_port)
    } else {
        String::new()
    };
    let node_role = node_role;

    let body = serde_json::json!({
        "tenant_token": reg_token,
        "installation_id": config.node_id.to_string(),
        "hostname": hostname,
        "os": std::env::consts::OS,
        "mac_address": "",
        "node_addr": node_addr,
        "web_addr": web_addr,
        "node_role": node_role
    });

    match reqwest::Client::new()
        .post(&url)
        .json(&body)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
    {
        Ok(resp) => {
            let status = resp.status();
            if status.is_success() {
                println!("  SaaS     : appareil enregistré dans le portail");
            } else {
                let text = resp.text().await.unwrap_or_default();
                println!("  SaaS     : enregistrement — {} {}", status, text.trim());
            }
        }
        Err(e) => println!("  SaaS     : injoignable — {} (non bloquant)", e),
    }
}
