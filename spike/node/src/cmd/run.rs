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
    if let Some(name) = register_with_saas(&config).await {
        if config.tenant_name.as_deref() != Some(name.as_str()) {
            config.tenant_name = Some(name);
            let _ = config.save(config_path);
        }
    }

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

    // Copie des 32 octets de la DEK avant que le journal ne la consomme — sert au
    // dépôt du blob de récupération zero-knowledge sur le relais (voir plus bas).
    let dek_bytes: [u8; 32] = *dek.as_bytes();

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

    // Dépôt zero-knowledge du blob de récupération sur le relais (best-effort).
    // Le relais reçoit la DEK emballée sous un code de récupération (Argon2id) :
    // des bytes chiffrés opaques qu'il ne peut PAS déchiffrer. Cela donne au
    // tenant un blob réel sur le relais — l'éditeur sait qu'il EXISTE, sans
    // jamais pouvoir le lire.
    // Seul le primaire (Active) génère/dépose le blob de récupération : le code
    // est per-tenant (une seule DEK), deux nœuds le régénéreraient en s'écrasant.
    // L'admin consulte la clé sur le nœud primaire (le standby redirige vers lui).
    if mode == RunMode::Active {
        ensure_recovery_blob(&mut config, config_path, &dek_bytes).await;
    }

    // Connexion PostgreSQL anticipée — nécessaire pour le module stock
    let web_port: u16 = std::env::var("WEB_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3000);

    if let Some(pg_url) = &config.pg_url.clone() {
        match PgPool::connect(pg_url).await {
            Ok(pool) => {
                println!("  PostgreSQL : connecté ({})", pg_url);

                // Vérifier le rôle réel PostgreSQL et corriger l'enregistrement SaaS si besoin.
                // Après un redémarrage du serveur SaaS, un nœud promu peut se re-déclarer avec
                // le mauvais NODE_MODE → on laisse PostgreSQL faire autorité.
                let pg_role = sqlx::query_scalar::<_, bool>("SELECT pg_is_in_recovery()")
                    .fetch_one(&pool)
                    .await
                    .ok();
                let actual_role = match pg_role {
                    Some(true)  => "standby",
                    Some(false) => "primary",
                    None        => "",
                };
                let declared_role = match std::env::var("NODE_MODE").as_deref() {
                    Ok("active") => "primary",
                    _            => "standby",
                };

                // Époque de ce nœud = timeline PostgreSQL (monotone, +1 à chaque promotion).
                let my_epoch = supervision::current_timeline(&pool).await.unwrap_or(1);
                println!("  Époque   : timeline PostgreSQL = {}", my_epoch);

                if !actual_role.is_empty() && actual_role != declared_role {
                    println!(
                        "  Rôle PG réel ({}) ≠ NODE_MODE ({}) — re-déclaration au portail",
                        actual_role, declared_role
                    );
                    let streaming = if actual_role == "primary" {
                        Some(supervision::connected_standby_count(&pool).await.unwrap_or(0))
                    } else {
                        None
                    };
                    register_with_saas_role(&config, actual_role, streaming, Some(my_epoch)).await;
                }

                // ── FENCING : auto-clôture d'un ancien primaire déchu ──────────────
                // Si ce nœud est primaire mais que le cluster a une époque supérieure
                // (un autre nœud a été promu depuis), c'est un primaire périmé. Il
                // REFUSE de servir (pas de serveur web → aucune écriture métier) pour
                // empêcher le split-brain, et attend un re-clone manuel.
                if actual_role == "primary" {
                    if let Some(cluster_epoch) = cluster_epoch_from_saas().await {
                        if my_epoch < cluster_epoch {
                            run_fenced_idle_loop(&config, my_epoch, cluster_epoch).await;
                            return Ok(()); // inatteignable (boucle infinie), mais explicite
                        }
                        println!("  Fencing  : OK (époque {} ≥ cluster {})", my_epoch, cluster_epoch);
                    }
                }

                // Migrations uniquement sur le primaire — le standby est en lecture
                // seule, il reçoit les tables via la réplication PostgreSQL.
                if mode == RunMode::Active {
                    if let Err(e) = crate::auth::run_migrations(&pool).await {
                        println!("  Auth     : migration échouée — {}", e);
                    }
                    match crate::auth::ensure_default_admin(&pool).await {
                        Ok(Some(pwd)) => {
                            // Persister le mot de passe initial sur la machine PME
                            // pour l'afficher sur la page de connexion (souverain).
                            config.initial_admin_password = Some(pwd);
                            let _ = config.save(config_path);
                        }
                        Ok(None) => {}
                        Err(e) => println!("  Auth     : création admin échouée — {}", e),
                    }
                    if let Err(e) = crate::stock::run_migrations(&pool).await {
                        println!("  Stock    : migration échouée — {}", e);
                    }
                }

                // Serveur web métier
                let web_pool = pool.clone();
                let web_node_id = config.node_id;
                let web_tenant = config.tenant_name.clone()
                    .unwrap_or_else(|| "PME".to_string());
                let web_recovery = config.recovery_code.clone();
                let web_admin_pwd = config.initial_admin_password.clone();
                tokio::spawn(crate::web::serve(web_port, web_node_id, web_pool, web_tenant, web_recovery, web_admin_pwd));

                // Boucle de supervision
                run_supervision_loop(mode, pool, &config, config_path).await?;
            }
            Err(e) => {
                println!("  PostgreSQL : impossible de se connecter — {} (spike: boucle ignorée)", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }
        }
        return Ok(());
    }

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

    // Sans PostgreSQL — boucle d'annonce au relais toutes les 30 s
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
                            let my_epoch = supervision::current_timeline(&pool).await.unwrap_or(1);
                            // Fencing à chaud : un primaire plus récent est-il apparu pendant
                            // qu'on tournait ? Si oui, ce nœud est devenu un primaire périmé.
                            if let Some(cluster_epoch) = cluster_epoch_from_saas().await {
                                if my_epoch < cluster_epoch {
                                    println!(
                                        "  [tick {tick}] ⛔ CLÔTURE À CHAUD : époque {my_epoch} < cluster {cluster_epoch} \
                                         — arrêt immédiat pour éviter le split-brain (re-clone requis)."
                                    );
                                    return Ok(()); // le process sort → au redémarrage, clôture au démarrage
                                }
                            }
                            println!("  [tick {tick}] PG primaire OK — {} standby(s) connecté(s)", standbys);
                            // Re-registration périodique : corrige les cas où le SaaS a rétrogradé
                            // ce nœud en "standby" suite à un split-brain détecté ailleurs.
                            register_with_saas_role(config, "primary", Some(standbys), Some(my_epoch)).await;
                        }
                        wal_miss = 0;
                    }
                    (RunMode::Active, false) => {
                        println!("  [tick {tick}] ERREUR : ce nœud actif est en mode recovery — état incohérent !");
                    }
                    (RunMode::Passive, true) => {
                        // Déjà promu (failover précédent dans cette session)
                        let standbys = supervision::connected_standby_count(&pool).await.unwrap_or(0);
                        if tick % 12 == 1 {
                            let my_epoch = supervision::current_timeline(&pool).await.unwrap_or(1);
                            println!("  [tick {tick}] Ce nœud est désormais primaire (époque {my_epoch}).");
                            register_with_saas_role(config, "primary", Some(standbys), Some(my_epoch)).await;
                        }
                        wal_miss = 0;
                    }
                    (RunMode::Passive, false) => {
                        // Standby normal — vérifier que le WAL receiver est actif
                        let wal_ok = supervision::wal_receiver_active(&pool).await.unwrap_or(true);
                        if wal_ok {
                            wal_miss = 0;
                            if tick % 12 == 1 {
                                let my_epoch = supervision::current_timeline(&pool).await.unwrap_or(1);
                                println!("  [tick {tick}] PG standby : réplication active");
                                register_with_saas_role(config, "standby", None, Some(my_epoch)).await;
                            }
                        } else {
                            wal_miss += 1;
                            println!(
                                "  [tick {tick}] ALERTE : WAL receiver inactif ({}/{}) — primaire injoignable ?",
                                wal_miss, WAL_FAILOVER_THRESHOLD
                            );

                            if wal_miss >= WAL_FAILOVER_THRESHOLD {
                                // Décision actée n°2 : à < 3 nœuds → BASCULE MANUELLE uniquement.
                                // L'auto-promotion à 2 nœuds provoquait un split-brain (deux
                                // primaires) qui cassait définitivement la réplication.
                                // On interroge le SaaS pour la taille réelle du cluster, au
                                // franchissement du seuil puis périodiquement (évite le spam HTTP).
                                if wal_miss == WAL_FAILOVER_THRESHOLD || tick % 12 == 1 {
                                    let node_count = cluster_node_count().await;
                                    if matches!(node_count, Some(n) if n >= 3) {
                                        println!("  [tick {tick}] Cluster ≥3 nœuds — FAILOVER AUTOMATIQUE par quorum");
                                        match supervision::promote_standby(&pool).await {
                                            Ok(true) => {
                                                // Promotion → PostgreSQL a incrémenté le timeline : nouvelle époque.
                                                let new_epoch = supervision::current_timeline(&pool).await.unwrap_or(1);
                                                println!("  [tick {tick}] ✓ FAILOVER : nœud promu en primaire (époque {new_epoch}) !");
                                                // Fraîchement promu : aucun standby encore rattaché → 0.
                                                register_with_saas_role(config, "primary", Some(0), Some(new_epoch)).await;
                                                wal_miss = 0;
                                            }
                                            Ok(false) => {
                                                println!("  [tick {tick}] Promotion déjà en cours ou inutile.");
                                                wal_miss = 0;
                                            }
                                            Err(e) => println!("  [tick {tick}] Erreur lors de la promotion : {}", e),
                                        }
                                    } else {
                                        // < 3 nœuds OU SaaS injoignable (fail-safe) : on ne promeut
                                        // PAS. L'alerte reste active (wal_miss non remis à 0) et le
                                        // SaaS continue d'afficher la perte de redondance.
                                        let n_txt = node_count
                                            .map(|n| n.to_string())
                                            .unwrap_or_else(|| "inconnu".to_string());
                                        println!(
                                            "  [tick {tick}] Primaire injoignable — cluster {n_txt} nœud(s) (<3) : \
                                             BASCULE MANUELLE requise (anti split-brain, décision n°2)."
                                        );
                                    }
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

/// Génère un code de récupération haute entropie (160 bits) au format groupé.
/// S'appuie sur 2 UUID v4 (aléatoire cryptographique via getrandom) — pas de
/// dépendance supplémentaire.
fn generate_recovery_code() -> String {
    let raw = format!(
        "{}{}",
        uuid::Uuid::new_v4().to_string().replace('-', ""),
        uuid::Uuid::new_v4().to_string().replace('-', "")
    )
    .to_uppercase();
    raw.as_bytes()
        .chunks(4)
        .take(10) // 10 groupes de 4 = 40 hex = 160 bits
        .map(|c| std::str::from_utf8(c).unwrap_or(""))
        .collect::<Vec<_>>()
        .join("-")
}

/// Dépose, si absent, le blob de récupération du tenant sur le relais.
///
/// Zero-knowledge : le relais reçoit la DEK emballée sous un code de récupération
/// (Argon2id + XChaCha20-Poly1305). Sans ce code — que le relais ne connaît pas —
/// le blob est indéchiffrable. Idempotent : ne re-pousse pas si le blob existe.
/// Best-effort : toute erreur est loguée, jamais bloquante.
/// Emballe la DEK sous (code, sel) → blob opaque → dépose sur le relais. true si OK.
async fn push_recovery_blob(
    relay_url: &str,
    tenant_id: uuid::Uuid,
    salt: &[u8; 16],
    code: &str,
    dek_bytes: &[u8; 32],
) -> bool {
    let recovery_key = match ss_crypto::derive_recovery_key(code, salt) {
        Ok(k) => k,
        Err(_) => return false,
    };
    let wrapped = match ss_crypto::Dek::from_bytes(recovery_key).encrypt(dek_bytes) {
        Ok(w) => w,
        Err(_) => return false,
    };
    // Blob = sel(16) ‖ (nonce ‖ ciphertext). Opaque pour le relais.
    let mut blob = Vec::with_capacity(16 + wrapped.len());
    blob.extend_from_slice(salt);
    blob.extend_from_slice(&wrapped);
    crate::relay_client::RelayClient::new(relay_url)
        .put_blob(tenant_id, "recovery-dek", blob)
        .await
        .is_ok()
}

async fn ensure_recovery_blob(config: &mut NodeConfig, config_path: &Path, dek_bytes: &[u8; 32]) {
    let tenant_id = match config.tenant_id {
        Some(t) => t,
        None => return,
    };
    let relay = crate::relay_client::RelayClient::new(&config.relay_url);

    // Cas 1 — code déjà généré (persisté sur cette machine PME). On ne régénère
    // jamais : on s'assure seulement que le blob est bien présent sur le relais.
    if let (Some(code), Some(salt_hex)) =
        (config.recovery_code.clone(), config.recovery_salt_hex.clone())
    {
        if !relay.blob_exists(tenant_id, "recovery-dek").await {
            if let Ok(v) = hex::decode(&salt_hex) {
                if v.len() == 16 {
                    let mut salt = [0u8; 16];
                    salt.copy_from_slice(&v);
                    if push_recovery_blob(&config.relay_url, tenant_id, &salt, &code, dek_bytes).await {
                        println!("  Relais : blob de récupération re-déposé depuis la config locale.");
                    }
                }
            }
        }
        return;
    }

    // Cas 2 — première génération : créer code + sel, déposer, puis PERSISTER.
    let salt = ss_crypto::generate_salt();
    let code = generate_recovery_code();
    if push_recovery_blob(&config.relay_url, tenant_id, &salt, &code, dek_bytes).await {
        config.recovery_code = Some(code.clone());
        config.recovery_salt_hex = Some(hex::encode(salt));
        let _ = config.save(config_path);
        println!("  Relais : blob de récupération déposé (chiffré, opaque — zero-knowledge).");
        println!("  ⚠️  CODE DE RÉCUPÉRATION (aussi dans l'UI : Admin → Clé de récupération) : {code}");
    } else {
        println!("  Relais : dépôt du blob de récupération échoué — réessai au prochain démarrage (non bloquant).");
    }
}

/// Interroge le SaaS pour connaître le nombre de nœuds actifs du cluster.
///
/// Garde-fou anti split-brain (décision actée n°2) : tant que le cluster compte
/// moins de 3 nœuds, la bascule reste MANUELLE. Retourne `None` si le SaaS est
/// injoignable ou non configuré → fail-safe : l'appelant ne promeut pas.
async fn cluster_node_count() -> Option<usize> {
    let saas_url = match std::env::var("SAAS_URL") {
        Ok(u) if !u.is_empty() => u,
        _ => return None,
    };
    let reg_token = match std::env::var("REGISTRATION_TOKEN") {
        Ok(t) if !t.is_empty() => t,
        _ => return None,
    };

    let url = format!(
        "{}/api/devices/cluster-status/?token={}",
        saas_url.trim_end_matches('/'),
        reg_token
    );

    let resp = reqwest::Client::new()
        .get(&url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let json: serde_json::Value = resp.json().await.ok()?;
    json["node_count"].as_u64().map(|n| n as usize)
}

/// Interroge le SaaS pour l'époque courante du cluster (timeline PostgreSQL max).
///
/// Sert au fencing : si l'époque de ce nœud est inférieure à celle du cluster,
/// c'est un primaire périmé qui doit se clôturer. Retourne `None` si le SaaS est
/// injoignable → fail-safe : pas de clôture sur une simple panne réseau du SaaS.
async fn cluster_epoch_from_saas() -> Option<i64> {
    let saas_url = match std::env::var("SAAS_URL") {
        Ok(u) if !u.is_empty() => u,
        _ => return None,
    };
    let reg_token = match std::env::var("REGISTRATION_TOKEN") {
        Ok(t) if !t.is_empty() => t,
        _ => return None,
    };

    let url = format!(
        "{}/api/devices/cluster-status/?token={}",
        saas_url.trim_end_matches('/'),
        reg_token
    );

    let resp = reqwest::Client::new()
        .get(&url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let json: serde_json::Value = resp.json().await.ok()?;
    json["cluster_epoch"].as_i64()
}

/// Boucle de clôture (fencing) — un ancien primaire déchu attend un re-clone manuel.
///
/// Ne lance JAMAIS le serveur web : aucune écriture métier n'est possible, donc
/// pas de split-brain. Re-déclare périodiquement son état pour rester visible
/// dans le portail éditeur. Ne se termine pas (l'opérateur doit re-cloner).
async fn run_fenced_idle_loop(config: &NodeConfig, my_epoch: i64, cluster_epoch: i64) {
    println!("\n========================================================");
    println!("  ⛔ NŒUD CLÔTURÉ (FENCED)");
    println!("  Époque de ce nœud (timeline PG) : {my_epoch}");
    println!("  Époque courante du cluster       : {cluster_epoch}");
    println!("  Un primaire plus récent existe. Ce nœud est un ancien");
    println!("  primaire déchu : il REFUSE de servir pour éviter le split-brain.");
    println!("  → Action opérateur : re-cloner ce nœud en standby.");
    println!("========================================================\n");

    loop {
        // Visibilité portail : on se signale en standby clôturé (jamais primaire).
        register_with_saas_role(config, "standby", None, Some(my_epoch)).await;
        println!("  [fenced] En attente de re-clone manuel (époque {my_epoch} < {cluster_epoch}).");
        tokio::time::sleep(std::time::Duration::from_secs(30)).await;
    }
}

/// Enregistre le nœud avec le rôle détecté depuis NODE_MODE.
/// Retourne le nom du tenant si le SaaS le fournit.
async fn register_with_saas(config: &NodeConfig) -> Option<String> {
    let role = match std::env::var("NODE_MODE").as_deref() {
        Ok("active") => "primary",
        _ => "standby",
    };
    // Au démarrage, un primaire n'a encore aucun standby rattaché ; le compte réel
    // sera transmis aux ticks suivants. Un standby ne rapporte pas cette métrique.
    let streaming = if role == "primary" { Some(0) } else { None };
    // Époque inconnue à ce stade (PostgreSQL pas encore interrogé) → None.
    register_with_saas_role(config, role, streaming, None).await
}

/// Enregistre le nœud avec un rôle explicite — utilisé après failover automatique.
/// Retourne le nom du tenant fourni par le SaaS. Non bloquant — erreurs loguées.
async fn register_with_saas_role(
    config: &NodeConfig,
    node_role: &str,
    streaming_standbys: Option<i64>,
    epoch: Option<i64>,
) -> Option<String> {
    let saas_url = match std::env::var("SAAS_URL") {
        Ok(u) if !u.is_empty() => u,
        _ => return None,
    };
    let reg_token = match std::env::var("REGISTRATION_TOKEN") {
        Ok(t) if !t.is_empty() => t,
        _ => return None,
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
        "node_role": node_role,
        // Vérité de réplication mesurée localement (pg_stat_replication).
        // Renseigné uniquement par un primaire ; `null` côté standby → le SaaS n'y touche pas.
        "streaming_standby_count": streaming_standbys,
        // Époque = timeline PostgreSQL. Sert au fencing : le SaaS conserve le max
        // et refuse qu'un ancien primaire (époque périmée) rétrograde le primaire légitime.
        "epoch": epoch
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
                let json: serde_json::Value = resp.json().await.unwrap_or_default();
                let tenant_name = json["tenant_name"].as_str().map(str::to_string);
                if let Some(ref name) = tenant_name {
                    println!("  SaaS     : appareil enregistré — tenant : {}", name);
                } else {
                    println!("  SaaS     : appareil enregistré dans le portail");
                }
                return tenant_name;
            } else {
                let text = resp.text().await.unwrap_or_default();
                println!("  SaaS     : enregistrement — {} {}", status, text.trim());
            }
        }
        Err(e) => println!("  SaaS     : injoignable — {} (non bloquant)", e),
    }
    None
}
