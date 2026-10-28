use std::path::Path;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use hex;
use sqlx::PgPool;
use ss_consensus::{
    check_fencing, decider_failover, supervision, DecisionFailover, EpochToken, FencingResult,
    PairObserve,
};
use ss_crypto::DeviceKeyPair;
use ss_journal::Journal;
use tokio::sync::watch;
use uuid::Uuid;

use crate::config::NodeConfig;

/// Période d'annonce par défaut au SaaS éditeur, en secondes.
/// La console éditeur considère un appareil « en ligne » si `last_seen` a moins
/// de 2 minutes (`ONLINE_THRESHOLD_MS`, `frontend/src/types/index.ts`) : toute
/// période supérieure à 120 s ferait clignoter l'indicateur.
const HEARTBEAT_DEFAUT_SECS: u64 = 60;
/// Bornes acceptées pour `SAAS_HEARTBEAT_SECS`.
const HEARTBEAT_MIN_SECS: u64 = 10;
const HEARTBEAT_MAX_SECS: u64 = 300;

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

    // Rôle courant du nœud, partagé avec la tâche de battement de cœur.
    // Initialisé depuis NODE_MODE, puis corrigé partout où le rôle réel change
    // (correction PostgreSQL au démarrage, promotion, failover automatique).
    let role_initial = role_depuis_node_mode(std::env::var("NODE_MODE").ok().as_deref());
    let (canal_role, recepteur_role) = watch::channel(role_initial.to_string());

    // Enregistrement SaaS en premier — fonctionne même si le nœud n'est pas encore enrôlé
    // (le standby n'a pas encore de DEK mais doit quand même apparaître dans le portail).
    if let Some(name) = annoncer_au_saas(config.node_id, role_initial, true).await.tenant_name {
        if config.tenant_name.as_deref() != Some(name.as_str()) {
            config.tenant_name = Some(name);
            let _ = config.save(config_path);
        }
    }

    // Battement de cœur SaaS — indépendant de PostgreSQL et du chemin d'exécution
    // emprunté ensuite (supervision PG, reprise de connexion PG, ou boucle relais).
    // Sans lui, `Device.last_seen` (auto_now côté Django) n'est jamais rafraîchi et
    // un nœud parfaitement sain apparaît « hors ligne » au bout de 2 minutes.
    lancer_heartbeat_saas(config.node_id, recepteur_role);

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

    // Connexion PostgreSQL anticipée — nécessaire pour le module stock
    let web_port: u16 = std::env::var("WEB_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3000);

    if let Some(pg_url) = &config.pg_url.clone() {
        // Reprise sur erreur : on n'abandonne plus `run()` quand PostgreSQL est
        // injoignable. Sortir ici faisait redémarrer le conteneur en boucle
        // (`restart: unless-stopped`) et tuait le battement de cœur à chaque cycle.
        let pool = connecter_postgres_avec_reprise(pg_url).await;
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
        // `clone()` immédiat : le garde de lecture du `watch` doit être relâché
        // avant tout `send`, sinon la mise à jour du rôle se bloquerait elle-même.
        let declared_role = canal_role.borrow().clone();
        if !actual_role.is_empty() && actual_role != declared_role.as_str() {
            println!(
                "  Rôle PG réel ({}) ≠ NODE_MODE ({}) — re-déclaration au portail",
                actual_role, declared_role
            );
            // Annonce immédiate + mise à jour du rôle lu par le heartbeat.
            majr_role(&canal_role, config.node_id, actual_role).await;
        }

        // Migrations uniquement sur le primaire — le standby est en lecture
        // seule, il reçoit les tables via la réplication PostgreSQL.
        if mode == RunMode::Active {
            if let Err(e) = crate::auth::run_migrations(&pool).await {
                println!("  Auth     : migration échouée — {}", e);
            }
            if let Err(e) = crate::auth::ensure_default_admin(&pool).await {
                println!("  Auth     : création admin échouée — {}", e);
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
        tokio::spawn(crate::web::serve(web_port, web_node_id, web_pool, web_tenant));

        // Boucle de supervision
        run_supervision_loop(
            mode,
            pool,
            &mut config,
            config_path,
            pg_url.clone(),
            canal_role,
        )
        .await?;
        return Ok(());
    }

    // Époque courante du nœud, telle que persistée dans `config.toml` par la
    // dernière promotion (`ss-node failover` ou failover automatique).
    let my_epoch = EpochToken(config.epoch);
    // Époque la plus haute vue chez les pairs — sert de référence de fencing.
    let mut epoque_cluster = config.epoch;

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
                        // Le fencing se juge contre la plus haute époque connue :
                        // un nœud en retard doit s'isoler, pas écrire.
                        epoque_cluster = epoque_cluster.max(p.epoch);
                    }
                }
            }
            Err(e) => println!("  Pairs    : échec découverte — {}", e),
        }
    } else {
        println!("  Relais   : tenant_id non configuré — annonce ignorée");
    }
    let cluster_epoch = EpochToken(epoque_cluster);

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

/// Délai maximal accordé à la sonde d'un pair (connexion + requêtes).
/// Court volontairement : la sonde ne tourne que dans la fenêtre de décision
/// du failover, où chaque seconde compte.
const SONDE_TIMEOUT: Duration = Duration::from_secs(3);

/// Boucle de supervision PostgreSQL — tourne indéfiniment.
///
/// - Mode actif  : vérifie que PG est bien primaire (réplication active).
/// - Mode passif : surveille le WAL receiver. Si déconnecté ≥ WAL_FAILOVER_THRESHOLD
///   ticks consécutifs (15 s par défaut), soumet la promotion à la **garde de
///   quorum** (`ss_consensus::decider_failover`) avant d'agir.
///
/// La garde applique la règle de `CLAUDE.md` : sous 3 nœuds, aucune promotion
/// automatique. Au-delà, un seul standby se promeut — celui qui a le quorum et
/// le WAL le plus avancé. Voir `spike/crates/ss-consensus/src/quorum.rs`.
///
/// Les ré-enregistrements périodiques (`tick % 12`) ont été retirés : c'est
/// désormais la tâche de battement de cœur qui entretient `last_seen`. Seules
/// subsistent ici les annonces **immédiates** lors d'un changement de rôle
/// (promotion, failover), qui donnent la réactivité côté console éditeur.
async fn run_supervision_loop(
    mode: RunMode,
    pool: PgPool,
    config: &mut NodeConfig,
    config_path: &Path,
    pg_url: String,
    canal_role: watch::Sender<String>,
) -> Result<()> {
    // Nombre de ticks WAL inactif consécutifs avant d'examiner la promotion.
    const WAL_FAILOVER_THRESHOLD: u32 = 3; // 3 × 5 s = 15 s

    let node_id = config.node_id;

    // Composition du cluster, lue une fois au démarrage. `CLUSTER_PEERS` doit
    // lister *tous* les autres nœuds, éteints compris : c'est cette liste qui
    // donne la taille du cluster, donc le seuil de majorité.
    let pairs = pairs_configures(std::env::var("CLUSTER_PEERS").ok().as_deref());
    let mon_addr_pg = mon_adresse_pg(std::env::var("NODE_ADDR").ok().as_deref());

    if mode == RunMode::Passive {
        match (&mon_addr_pg, pairs.len()) {
            (Some(addr), n) if n + 1 >= 3 => println!(
                "  Quorum   : {} (moi) + {} pair(s) déclaré(s) — bascule automatique armée",
                addr, n
            ),
            (Some(_), n) => println!(
                "  Quorum   : {} nœud(s) déclaré(s) — sous le seuil de 3, bascule MANUELLE \
                 uniquement (`ss-node failover`)",
                n + 1
            ),
            (None, _) => println!(
                "  Quorum   : NODE_ADDR absent — bascule automatique désactivée, \
                 promouvoir avec `ss-node failover`"
            ),
        }
    }

    let mut tick: u32 = 0;
    let mut wal_miss: u32 = 0;
    // Dernier motif de refus journalisé, pour ne pas répéter la même ligne
    // toutes les 5 s tant que la situation n'évolue pas.
    let mut dernier_refus: Option<String> = None;

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
                        // Le rôle réel est « primary » : on le publie pour le heartbeat.
                        // `majr_role` n'annonce au SaaS que s'il y a réellement changement.
                        majr_role(&canal_role, node_id, "primary").await;
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
                        majr_role(&canal_role, node_id, "primary").await;
                        wal_miss = 0;
                    }
                    (RunMode::Passive, false) => {
                        // Standby normal — vérifier que le WAL receiver est actif
                        let wal_ok = supervision::wal_receiver_active(&pool).await.unwrap_or(true);
                        if wal_ok {
                            wal_miss = 0;
                            dernier_refus = None;
                            if tick % 12 == 1 {
                                println!("  [tick {tick}] PG standby : réplication active");
                            }
                            majr_role(&canal_role, node_id, "standby").await;
                        } else {
                            wal_miss += 1;
                            println!(
                                "  [tick {tick}] ALERTE : WAL receiver inactif ({}/{}) — primaire injoignable ?",
                                wal_miss, WAL_FAILOVER_THRESHOLD
                            );

                            if wal_miss >= WAL_FAILOVER_THRESHOLD {
                                // Sans adresse propre, l'élection ne peut pas être
                                // départagée : on refuse plutôt que de risquer une
                                // double promotion.
                                let Some(mon_addr) = mon_addr_pg.as_deref() else {
                                    journaliser_refus(
                                        tick,
                                        "NODE_ADDR absent — bascule automatique désactivée",
                                        &mut dernier_refus,
                                    );
                                    continue;
                                };

                                // Garde de quorum : c'est ici que se joue
                                // l'anti-split-brain. Les pairs sont sondés à chaque
                                // évaluation — un nœud qui revient change la décision.
                                let observations = sonder_pairs(&pg_url, &pairs).await;
                                let mon_lsn =
                                    supervision::wal_receive_lsn(&pool).await.unwrap_or(None);

                                match decider_failover(mon_addr, mon_lsn, &observations) {
                                    DecisionFailover::Refus(motif) => {
                                        journaliser_refus(
                                            tick,
                                            &format!("promotion refusée — {motif}"),
                                            &mut dernier_refus,
                                        );
                                        // `wal_miss` n'est PAS remis à zéro : la
                                        // situation reste anormale et sera réévaluée
                                        // au tick suivant.
                                    }
                                    DecisionFailover::Promouvoir => {
                                        println!(
                                            "  [tick {tick}] Quorum atteint ({}/{} nœuds) — FAILOVER AUTOMATIQUE",
                                            1 + observations.iter().filter(|p| p.joignable).count(),
                                            observations.len() + 1
                                        );
                                        match supervision::promote_standby(&pool).await {
                                            Ok(true) => {
                                                // Fencing : incrémenter et persister
                                                // l'époque, exactement comme le fait
                                                // `ss-node failover`. Sans cela, un
                                                // ancien primaire qui revient porte la
                                                // même époque et n'est pas fencé.
                                                let ancienne = config.epoch;
                                                config.epoch = ancienne + 1;
                                                match config.save(config_path) {
                                                    Ok(()) => println!(
                                                        "  [tick {tick}] Époque {} -> {} (fencing posé)",
                                                        ancienne, config.epoch
                                                    ),
                                                    Err(e) => println!(
                                                        "  [tick {tick}] ATTENTION : époque non persistée — {} \
                                                         (l'ancien primaire ne sera pas fencé)",
                                                        e
                                                    ),
                                                }
                                                println!("  [tick {tick}] ✓ FAILOVER : nœud promu en primaire !");
                                                // Annonce immédiate — c'est elle qui donne la réactivité.
                                                majr_role(&canal_role, node_id, "primary").await;
                                                wal_miss = 0;
                                                dernier_refus = None;
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
                }
            }
            Err(e) => {
                println!("  [tick {tick}] PG local injoignable : {}", e);
            }
        }
    }
}

/// Journalise un motif de refus **une seule fois** tant qu'il ne change pas.
///
/// La boucle réévalue toutes les 5 s ; sans ce filtre, un cluster durablement
/// dégradé noierait les logs sous la même ligne.
fn journaliser_refus(tick: u32, motif: &str, dernier: &mut Option<String>) {
    if dernier.as_deref() != Some(motif) {
        println!("  [tick {tick}] {motif}");
        *dernier = Some(motif.to_string());
    }
}

/// Sonde tous les pairs déclarés, en parallèle.
///
/// Les sondes tournent en parallèle (`JoinSet`) : la durée totale est bornée
/// par le `SONDE_TIMEOUT` d'un seul pair, et non par leur somme — la fenêtre de
/// décision reste courte même avec plusieurs nœuds éteints.
async fn sonder_pairs(pg_url: &str, pairs: &[String]) -> Vec<PairObserve> {
    let mut taches = tokio::task::JoinSet::new();
    for pair in pairs {
        let url = pg_url.to_string();
        let addr = pair.clone();
        taches.spawn(async move { sonder_pair(&url, &addr).await });
    }

    let mut par_addr: std::collections::HashMap<String, PairObserve> =
        std::collections::HashMap::new();
    while let Some(resultat) = taches.join_next().await {
        if let Ok(observe) = resultat {
            par_addr.insert(observe.addr.clone(), observe);
        }
    }

    // Reconstruire dans l'ordre déclaré. Un pair dont la sonde a disparu
    // (tâche annulée ou paniquée) doit rester présent en « muet » : l'omettre
    // réduirait la taille du cluster, donc le seuil de majorité — exactement
    // le relâchement que la garde est censée empêcher.
    pairs
        .iter()
        .map(|pair| {
            par_addr.remove(pair).unwrap_or_else(|| PairObserve {
                addr: pair.clone(),
                joignable: false,
                en_recovery: false,
                lsn: None,
            })
        })
        .collect()
}

/// Sonde un pair : joignable ? en recovery ? à quelle position WAL ?
///
/// Un pair muet (éteint, partitionné, PostgreSQL arrêté) revient
/// `joignable: false` — il compte alors dans la taille du cluster mais pas
/// dans le quorum, ce qui est exactement le comportement voulu.
async fn sonder_pair(pg_url: &str, pair: &str) -> PairObserve {
    let muet = PairObserve {
        addr: pair.to_string(),
        joignable: false,
        en_recovery: false,
        lsn: None,
    };

    let Some(url) = url_pg_pour_pair(pg_url, pair) else {
        return muet;
    };

    let pool = match tokio::time::timeout(SONDE_TIMEOUT, PgPool::connect(&url)).await {
        Ok(Ok(p)) => p,
        _ => return muet,
    };

    let observe = match supervision::is_primary(&pool).await {
        Ok(primaire) => PairObserve {
            addr: pair.to_string(),
            joignable: true,
            en_recovery: !primaire,
            lsn: supervision::wal_receive_lsn(&pool).await.unwrap_or(None),
        },
        Err(_) => muet,
    };

    pool.close().await;
    observe
}

/// Analyse `CLUSTER_PEERS` : liste d'adresses séparées par des virgules,
/// `hôte` ou `hôte:port` (port PostgreSQL, 5432 par défaut).
///
/// Ne gère pas les adresses IPv6 nues (le `:` y est ambigu) — le banc PME est
/// en IPv4. Fonction pure, testable sans toucher à l'environnement.
fn pairs_configures(valeur: Option<&str>) -> Vec<String> {
    valeur
        .unwrap_or("")
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| {
            if s.contains(':') {
                s.to_string()
            } else {
                format!("{s}:5432")
            }
        })
        .collect()
}

/// Adresse PostgreSQL de ce nœud, dans le même espace de nommage que
/// `CLUSTER_PEERS` — c'est elle qui départage l'élection à LSN égal.
///
/// Dérivée de `NODE_ADDR` (`ip:9001`) en substituant le port PostgreSQL, pour
/// n'introduire aucune variable d'environnement supplémentaire. Fonction pure.
fn mon_adresse_pg(node_addr: Option<&str>) -> Option<String> {
    let brut = node_addr?.trim();
    if brut.is_empty() {
        return None;
    }
    let hote = brut.rsplit_once(':').map(|(h, _)| h).unwrap_or(brut);
    if hote.is_empty() {
        return None;
    }
    Some(format!("{hote}:5432"))
}

/// Réécrit l'URL PostgreSQL locale pour viser un pair.
///
/// Les identifiants du nœud sont réutilisés tels quels : `primary-init.sh`
/// ouvre déjà `pg_hba.conf` au sous-réseau PME, donc aucun secret
/// supplémentaire n'est nécessaire. Fonction pure.
fn url_pg_pour_pair(pg_url: &str, pair: &str) -> Option<String> {
    let (schema, reste) = pg_url.split_once("://")?;
    if schema.is_empty() || pair.is_empty() {
        return None;
    }
    // L'autorité s'arrête au premier '/' (base) ou '?' (paramètres).
    let (autorite, suffixe) = match reste.find(['/', '?']) {
        Some(i) => (&reste[..i], &reste[i..]),
        None => (reste, ""),
    };
    // Ne conserver que les identifiants, en jetant l'hôte courant.
    match autorite.rsplit_once('@') {
        Some((creds, _)) => Some(format!("{schema}://{creds}@{pair}{suffixe}")),
        None => Some(format!("{schema}://{pair}{suffixe}")),
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

/// Traduit la variable d'environnement `NODE_MODE` en rôle SaaS.
/// Fonction pure — testable sans toucher à l'environnement du processus.
fn role_depuis_node_mode(node_mode: Option<&str>) -> &'static str {
    match node_mode {
        Some("active") => "primary",
        _ => "standby",
    }
}

/// Calcule la période du battement de cœur à partir de `SAAS_HEARTBEAT_SECS`.
///
/// Politique retenue (fonction pure pour rester testable sans `std::env`) :
/// - variable absente, vide ou invalide → `HEARTBEAT_DEFAUT_SECS` (60 s) ;
/// - `0` → `None`, c.-à-d. battement de cœur **désactivé** (utile en test) ;
/// - toute autre valeur → bornée à [`HEARTBEAT_MIN_SECS`, `HEARTBEAT_MAX_SECS`].
///
/// La borne haute de 300 s est volontairement au-delà des 120 s du seuil
/// « en ligne » de la console : c'est un garde-fou, pas une recommandation.
/// Au-delà de 120 s l'indicateur de la console se met à clignoter.
fn periode_heartbeat(valeur: Option<&str>) -> Option<Duration> {
    let secondes = match valeur.map(str::trim) {
        Some("0") => return None,
        Some(v) => v.parse::<u64>().unwrap_or(HEARTBEAT_DEFAUT_SECS),
        None => HEARTBEAT_DEFAUT_SECS,
    };
    Some(Duration::from_secs(
        secondes.clamp(HEARTBEAT_MIN_SECS, HEARTBEAT_MAX_SECS),
    ))
}

/// Publie le rôle courant dans le canal `watch` et indique s'il a changé.
/// Fonction pure vis-à-vis du réseau — c'est elle qui est testée.
fn appliquer_role(canal: &watch::Sender<String>, role: &str) -> bool {
    // Le garde de lecture est relâché avant l'écriture (le booléen est copié).
    let change = canal.borrow().as_str() != role;
    if change {
        // `send_replace` et non `send` : `send` échoue et **n'écrit pas** quand
        // plus aucun récepteur n'existe (battement de cœur désactivé par
        // SAAS_HEARTBEAT_SECS=0). Le rôle resterait alors figé et chaque tick
        // de supervision re-détecterait un « changement », donc une annonce HTTP.
        let _ = canal.send_replace(role.to_string());
    }
    change
}

/// Met à jour le rôle courant et, **uniquement s'il a réellement changé**,
/// envoie une annonce immédiate au SaaS. Le régime nominal est entretenu par
/// le battement de cœur ; ici on ne paie un aller-retour HTTP que sur bascule.
async fn majr_role(canal: &watch::Sender<String>, node_id: Uuid, role: &str) {
    if appliquer_role(canal, role) {
        annoncer_au_saas(node_id, role, true).await;
    }
}

/// Se connecte à PostgreSQL en réessayant indéfiniment.
///
/// Auparavant un échec de connexion faisait sortir de `run()` : le conteneur
/// redémarrait en boucle (`restart: unless-stopped`) et le nœud n'annonçait
/// jamais assez longtemps pour rester « en ligne » côté console. On boucle
/// désormais sur place, le battement de cœur continuant de tourner en tâche de fond.
async fn connecter_postgres_avec_reprise(pg_url: &str) -> PgPool {
    const DELAI: Duration = Duration::from_secs(10);
    /// Une ligne de log toutes les 6 tentatives (≈ 1 min) pour ne pas noyer la sortie.
    const LOG_TOUTES_LES: u32 = 6;

    let mut essais: u32 = 0;
    loop {
        match PgPool::connect(pg_url).await {
            Ok(pool) => {
                if essais > 0 {
                    println!("  PostgreSQL : reconnecté après {} tentative(s)", essais + 1);
                }
                return pool;
            }
            Err(e) => {
                if essais % LOG_TOUTES_LES == 0 {
                    println!(
                        "  PostgreSQL : injoignable — {} (nouvel essai dans {} s)",
                        e,
                        DELAI.as_secs()
                    );
                }
                essais = essais.saturating_add(1);
                tokio::time::sleep(DELAI).await;
            }
        }
    }
}

/// Démarre la tâche de battement de cœur vers le SaaS éditeur.
///
/// Elle n'emprunte rien : seul `node_id` (un `Uuid`, `Copy`) est déplacé dans la
/// tâche, plus un récepteur `watch` sur le rôle courant. On ne clone
/// délibérément **pas** `NodeConfig` : il porte `secret_key_hex` et
/// `sealed_dek_hex`, et dupliquer du matériel de clé sans nécessité contrevient
/// à la règle « socle crypto » du dépôt. Tout le reste (SAAS_URL,
/// REGISTRATION_TOKEN, HOSTNAME, NODE_ADDR, WEB_PORT) est relu dans
/// l'environnement à chaque appel, ce qui rend la tâche autonome.
fn lancer_heartbeat_saas(node_id: Uuid, recepteur_role: watch::Receiver<String>) {
    let periode = match periode_heartbeat(std::env::var("SAAS_HEARTBEAT_SECS").ok().as_deref()) {
        Some(p) => p,
        None => {
            println!("  SaaS     : battement de cœur désactivé (SAAS_HEARTBEAT_SECS=0)");
            return;
        }
    };
    println!(
        "  SaaS     : battement de cœur toutes les {} s",
        periode.as_secs()
    );

    tokio::spawn(async move {
        // Politique de log : silence en régime nominal. On ne trace que le tout
        // premier succès, chaque échec, et la reprise après échec.
        let mut premier_succes_logue = false;
        let mut en_echec = false;
        loop {
            tokio::time::sleep(periode).await;
            // Rôle courant, pas celui du démarrage (failover, promotion…).
            let role = recepteur_role.borrow().clone();
            let a_loguer = !premier_succes_logue || en_echec;
            // Une panne du SaaS ne doit jamais arrêter la boucle : `annoncer_au_saas`
            // absorbe et logue ses propres erreurs.
            if annoncer_au_saas(node_id, &role, a_loguer).await.reussi {
                premier_succes_logue = true;
                en_echec = false;
            } else {
                en_echec = true;
            }
        }
    });
}

/// Résultat d'une annonce au SaaS éditeur.
struct Annonce {
    /// L'appel HTTP a abouti avec un code 2xx.
    reussi: bool,
    /// Nom du tenant renvoyé par le SaaS, s'il en fournit un.
    tenant_name: Option<String>,
}

impl Annonce {
    fn echec() -> Self {
        Self { reussi: false, tenant_name: None }
    }
}

/// Annonce le nœud au SaaS éditeur avec un rôle explicite.
///
/// Protocole inchangé : `POST {SAAS_URL}/api/devices/register/`, mêmes champs
/// JSON, même timeout de 10 s. Non bloquant — toutes les erreurs sont loguées
/// et absorbées. `logger_succes` ne concerne que la ligne de succès : les
/// échecs sont toujours tracés, sinon une panne du SaaS passerait inaperçue.
async fn annoncer_au_saas(node_id: Uuid, node_role: &str, logger_succes: bool) -> Annonce {
    let saas_url = match std::env::var("SAAS_URL") {
        Ok(u) if !u.is_empty() => u,
        // Pas de SaaS configuré : silencieux, y compris pour le battement de cœur.
        _ => return Annonce::echec(),
    };
    let reg_token = match std::env::var("REGISTRATION_TOKEN") {
        Ok(t) if !t.is_empty() => t,
        _ => return Annonce::echec(),
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

    let body = serde_json::json!({
        "tenant_token": reg_token,
        "installation_id": node_id.to_string(),
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
        .timeout(Duration::from_secs(10))
        .send()
        .await
    {
        Ok(resp) => {
            let status = resp.status();
            if status.is_success() {
                let json: serde_json::Value = resp.json().await.unwrap_or_default();
                let tenant_name = json["tenant_name"].as_str().map(str::to_string);
                if logger_succes {
                    match tenant_name {
                        Some(ref name) => {
                            println!("  SaaS     : appareil enregistré — tenant : {}", name)
                        }
                        None => println!("  SaaS     : appareil enregistré dans le portail"),
                    }
                }
                return Annonce { reussi: true, tenant_name };
            } else {
                let text = resp.text().await.unwrap_or_default();
                println!("  SaaS     : enregistrement — {} {}", status, text.trim());
            }
        }
        Err(e) => println!("  SaaS     : injoignable — {} (non bloquant)", e),
    }
    Annonce::echec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn periode_heartbeat_defaut_sans_variable() {
        assert_eq!(periode_heartbeat(None), Some(Duration::from_secs(60)));
    }

    #[test]
    fn periode_heartbeat_zero_desactive() {
        assert_eq!(periode_heartbeat(Some("0")), None);
        // Espaces autour : la valeur est tout de même reconnue.
        assert_eq!(periode_heartbeat(Some("  0  ")), None);
    }

    #[test]
    fn periode_heartbeat_valeur_invalide_retombe_sur_defaut() {
        for valeur in ["", "abc", "-5", "12.5", "  "] {
            assert_eq!(
                periode_heartbeat(Some(valeur)),
                Some(Duration::from_secs(60)),
                "valeur invalide : {valeur:?}"
            );
        }
    }

    #[test]
    fn periode_heartbeat_bornee() {
        // Sous la borne basse → 10 s.
        assert_eq!(periode_heartbeat(Some("1")), Some(Duration::from_secs(10)));
        // Au-dessus de la borne haute → 300 s.
        assert_eq!(periode_heartbeat(Some("9999")), Some(Duration::from_secs(300)));
        // Dans l'intervalle → valeur conservée.
        assert_eq!(periode_heartbeat(Some("30")), Some(Duration::from_secs(30)));
        assert_eq!(periode_heartbeat(Some("120")), Some(Duration::from_secs(120)));
    }

    #[test]
    fn periode_heartbeat_reste_sous_le_seuil_en_ligne_par_defaut() {
        // Garde-fou : la valeur par défaut doit rester < 120 s (ONLINE_THRESHOLD_MS).
        let defaut = periode_heartbeat(None).expect("défaut actif");
        assert!(defaut < Duration::from_secs(120));
    }

    #[test]
    fn role_depuis_node_mode_traduit_les_valeurs() {
        assert_eq!(role_depuis_node_mode(Some("active")), "primary");
        assert_eq!(role_depuis_node_mode(Some("passive")), "standby");
        assert_eq!(role_depuis_node_mode(Some("")), "standby");
        assert_eq!(role_depuis_node_mode(Some("ACTIVE")), "standby");
        assert_eq!(role_depuis_node_mode(None), "standby");
    }

    #[test]
    fn suivi_role_courant_ne_signale_que_les_changements() {
        let (canal, recepteur) = watch::channel("standby".to_string());

        // Ré-affirmer le même rôle : aucun changement, donc aucune annonce immédiate.
        assert!(!appliquer_role(&canal, "standby"));
        assert_eq!(recepteur.borrow().as_str(), "standby");

        // Promotion : changement détecté et rôle publié.
        assert!(appliquer_role(&canal, "primary"));
        assert_eq!(recepteur.borrow().as_str(), "primary");

        // Idempotence après promotion.
        assert!(!appliquer_role(&canal, "primary"));

        // Rétrogradation.
        assert!(appliquer_role(&canal, "standby"));
        assert_eq!(recepteur.borrow().as_str(), "standby");
    }

    #[test]
    fn suivi_role_courant_tient_sans_recepteur() {
        // Battement de cœur désactivé (SAAS_HEARTBEAT_SECS=0) : plus aucun
        // récepteur. Le rôle doit tout de même être mémorisé, sinon chaque tick
        // de supervision redéclencherait une annonce HTTP.
        let (canal, recepteur) = watch::channel("standby".to_string());
        drop(recepteur);

        assert!(appliquer_role(&canal, "primary"));
        assert_eq!(canal.borrow().as_str(), "primary");
        assert!(!appliquer_role(&canal, "primary"));
    }

    #[test]
    fn le_heartbeat_lit_le_role_courant_pas_celui_du_demarrage() {
        let role_initial = role_depuis_node_mode(Some("passive"));
        let (canal, recepteur) = watch::channel(role_initial.to_string());
        assert_eq!(recepteur.borrow().as_str(), "standby");

        // Failover : le rôle change après le démarrage.
        appliquer_role(&canal, "primary");
        // Le récepteur détenu par la tâche de battement de cœur voit la nouvelle valeur.
        assert_eq!(recepteur.borrow().as_str(), "primary");
    }

    // --- Composition du cluster (garde de quorum) -------------------------

    #[test]
    fn pairs_configures_ajoute_le_port_postgres_par_defaut() {
        assert_eq!(
            pairs_configures(Some("10.174.40.45,10.174.40.50")),
            vec!["10.174.40.45:5432", "10.174.40.50:5432"]
        );
    }

    #[test]
    fn pairs_configures_respecte_un_port_explicite() {
        assert_eq!(
            pairs_configures(Some("10.174.40.45:6432")),
            vec!["10.174.40.45:6432"]
        );
    }

    #[test]
    fn pairs_configures_tolere_espaces_et_entrees_vides() {
        assert_eq!(
            pairs_configures(Some("  10.0.0.1 , , 10.0.0.2:5432 ,")),
            vec!["10.0.0.1:5432", "10.0.0.2:5432"]
        );
    }

    #[test]
    fn pairs_configures_absent_donne_une_liste_vide() {
        // Cluster de taille 1 ⇒ la garde refusera toute promotion automatique.
        assert!(pairs_configures(None).is_empty());
        assert!(pairs_configures(Some("")).is_empty());
    }

    // --- Adresse propre, utilisée pour départager l'élection --------------

    #[test]
    fn mon_adresse_pg_substitue_le_port_postgres() {
        assert_eq!(
            mon_adresse_pg(Some("10.174.40.77:9001")).as_deref(),
            Some("10.174.40.77:5432")
        );
    }

    #[test]
    fn mon_adresse_pg_accepte_une_adresse_sans_port() {
        assert_eq!(
            mon_adresse_pg(Some("10.174.40.77")).as_deref(),
            Some("10.174.40.77:5432")
        );
    }

    #[test]
    fn mon_adresse_pg_absente_desarme_la_bascule_automatique() {
        // Sans adresse, deux standbys ne peuvent pas être départagés :
        // la boucle refuse la promotion plutôt que de risquer un split-brain.
        assert_eq!(mon_adresse_pg(None), None);
        assert_eq!(mon_adresse_pg(Some("   ")), None);
        assert_eq!(mon_adresse_pg(Some(":9001")), None);
    }

    // --- Réécriture de l'URL PostgreSQL vers un pair ----------------------

    #[test]
    fn url_pg_pour_pair_conserve_identifiants_et_base() {
        assert_eq!(
            url_pg_pour_pair(
                "postgres://metier:secret@postgres:5432/metier",
                "10.0.0.3:5432"
            )
            .as_deref(),
            Some("postgres://metier:secret@10.0.0.3:5432/metier")
        );
    }

    #[test]
    fn url_pg_pour_pair_conserve_les_parametres() {
        assert_eq!(
            url_pg_pour_pair(
                "postgres://metier:secret@postgres:5432/metier?sslmode=disable",
                "10.0.0.3:5432"
            )
            .as_deref(),
            Some("postgres://metier:secret@10.0.0.3:5432/metier?sslmode=disable")
        );
    }

    #[test]
    fn url_pg_pour_pair_sans_identifiants() {
        assert_eq!(
            url_pg_pour_pair("postgres://postgres:5432/metier", "10.0.0.3:5432").as_deref(),
            Some("postgres://10.0.0.3:5432/metier")
        );
    }

    #[test]
    fn url_pg_pour_pair_ne_garde_jamais_l_hote_courant() {
        // Garde-fou : sonder « soi-même » au lieu du pair fausserait le quorum
        // en comptant le même nœud plusieurs fois.
        let url = url_pg_pour_pair(
            "postgres://metier:secret@postgres:5432/metier",
            "10.0.0.3:5432",
        )
        .unwrap();
        assert!(!url.contains("postgres:5432/"));
        assert!(url.contains("10.0.0.3:5432"));
    }

    #[test]
    fn url_pg_pour_pair_rejette_une_url_malformee() {
        assert_eq!(url_pg_pour_pair("pas-une-url", "10.0.0.3:5432"), None);
        assert_eq!(
            url_pg_pour_pair("postgres://metier@postgres:5432/metier", ""),
            None
        );
    }
}
