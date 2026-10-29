use std::path::Path;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use hex;
use sqlx::PgPool;
use ss_consensus::{
    check_fencing, epoque_cluster, evaluer_quorum, EpochToken, EtatQuorum, FencingResult,
};
use ss_crypto::{Dek, DeviceKeyPair};
use ss_journal::Journal;
use tokio::sync::watch;
use uuid::Uuid;

use crate::config::NodeConfig;
use crate::patroni::{self, RoleLocal};

/// Période d'annonce par défaut au SaaS éditeur, en secondes.
/// La console éditeur considère un appareil « en ligne » si `last_seen` a moins
/// de 2 minutes (`ONLINE_THRESHOLD_MS`, `frontend/src/types/index.ts`) : toute
/// période supérieure à 120 s ferait clignoter l'indicateur.
const HEARTBEAT_DEFAUT_SECS: u64 = 60;
/// Bornes acceptées pour `SAAS_HEARTBEAT_SECS`.
const HEARTBEAT_MIN_SECS: u64 = 10;
const HEARTBEAT_MAX_SECS: u64 = 300;

/// Période de lecture de l'état Patroni.
const PERIODE_SUPERVISION: Duration = Duration::from_secs(5);

/// Mode d'exécution du nœud.
///
/// Refonte V3 (étape 2c) : le mode ne décide plus du rôle PostgreSQL — Patroni
/// le fait. `NODE_MODE=active` désigne seulement le premier serveur du cluster,
/// celui qui génère la DEK au premier démarrage (`ss-node init --first`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunMode {
    Active,
    Passive,
}

/// Démarre le nœud.
///
/// Le nœud ne promeut plus PostgreSQL et ne sonde plus ses pairs : etcd décide,
/// Patroni applique (ADR-001). Il **lit** l'état publié par Patroni, le classe
/// avec la garde de quorum (`ss_consensus::evaluer_quorum`), fait respecter
/// l'époque (timeline PostgreSQL) en continu et remonte le tout au portail.
pub async fn run(mode: RunMode, config_path: &Path) -> Result<()> {
    let mut config = NodeConfig::load(config_path)?;
    config.apply_env_overrides();

    // Rôle et état du cluster, partagés avec la tâche de battement de cœur.
    // Vides tant que Patroni n'a pas été lu : le portail ne reçoit alors aucun
    // rôle plutôt qu'un rôle supposé.
    let (canal_role, recepteur_role) = watch::channel(String::new());
    let (canal_etat, recepteur_etat) = watch::channel(String::new());
    // État de l'application hébergée (`ok` / `ko`), si le cluster en héberge une.
    let (canal_app, recepteur_app) = watch::channel(String::new());
    let application = crate::application::Application::depuis_env().map(std::sync::Arc::new);
    if let Some(app) = &application {
        crate::application::lancer_sonde(app.clone(), canal_app);
    }

    // Enregistrement SaaS en premier — fonctionne même si le nœud n'est pas encore enrôlé.
    if let Some(name) = annoncer_au_saas(config.node_id, "", "", "", true).await.tenant_name {
        if config.tenant_name.as_deref() != Some(name.as_str()) {
            config.tenant_name = Some(name);
            let _ = config.save(config_path);
        }
    }

    // Battement de cœur SaaS — indépendant de PostgreSQL et de Patroni. Sans
    // lui, `Device.last_seen` n'est jamais rafraîchi et un nœud sain apparaît
    // « hors ligne » au bout de 2 minutes.
    lancer_heartbeat_saas(config.node_id, recepteur_role, recepteur_etat, recepteur_app);

    // Journal chiffré, si le nœud détient la DEK. Un serveur qui n'est pas le
    // premier du cluster démarre sans DEK : il attend son enrôlement, sans
    // redémarrer en boucle.
    let journal = ouvrir_journal(&config, config_path)?;
    println!("=== Noeud {} démarré (mode {:?}) ===", config.node_id, mode);
    match &journal {
        Some(j) => println!("  Journal  : {} entrée(s)", j.len()),
        None => println!("  Journal  : en attente d'enrôlement (DEK absente sur cette machine)"),
    }

    let web_port: u16 = std::env::var("WEB_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3000);

    if let Some(pg_url) = &config.pg_url.clone() {
        // Reprise sur erreur : on n'abandonne jamais `run()` quand PostgreSQL
        // est injoignable, sinon le conteneur redémarre en boucle et le
        // battement de cœur s'interrompt à chaque cycle.
        let pool = connecter_postgres_avec_reprise(pg_url).await;
        println!("  PostgreSQL : connecté");

        // Pile V3 : PG_URL passe par HAProxy, donc mène toujours au primaire.
        // Les migrations sont idempotentes : chaque nœud peut les jouer.
        if let Err(e) = crate::auth::run_migrations(&pool).await {
            println!("  Auth     : migration impossible — {} (primaire indisponible ?)", e);
        }
        if let Err(e) = crate::auth::ensure_default_admin(&pool).await {
            println!("  Auth     : création admin impossible — {}", e);
        }

        let web_tenant = config.tenant_name.clone().unwrap_or_else(|| "PME".to_string());
        // DEK partagée par la console (enrôlement) et les sauvegardes.
        let dek = ouvrir_dek(&config)?.map(std::sync::Arc::new);
        if application.is_some() {
            println!("  Application : interface relayée à la racine du port {web_port} ; console du nœud sous /amane");
        }
        tokio::spawn(crate::web::serve(web_port, config.node_id, pool.clone(), web_tenant, application, dek.clone()));

        match std::env::var("PATRONI_URL") {
            Ok(base) if !base.is_empty() => {
                let sauvegardes = match (&dek, crate::sauvegarde::cible_depuis_env(&config.relay_url, config.tenant_id)) {
                    (Some(dek), Ok(cible)) => Some((dek.clone(), cible)),
                    (None, _) => {
                        println!("  Sauvegardes : DEK absente sur cette machine — enrôlez-la pour qu'elle puisse sauvegarder");
                        None
                    }
                    (Some(_), Err(e)) => {
                        println!("  Sauvegardes : désactivées — {e}");
                        None
                    }
                };
                boucle_supervision(&base, journal, config.node_id, canal_role, canal_etat, sauvegardes).await
            }
            _ => {
                // Hors pile V3 (pas de Patroni) : rôle lu dans PostgreSQL, aucune
                // décision de bascule — c'est à etcd/Patroni d'en prendre.
                println!("  Patroni  : PATRONI_URL absent — rôle lu dans PostgreSQL, sans garde de quorum");
                boucle_role_postgresql(pool, config.node_id, canal_role).await
            }
        }
        return Ok(());
    }

    // Sans PostgreSQL : annonce au relais toutes les 30 s.
    let relay = crate::relay_client::RelayClient::new(&config.relay_url);
    println!("  PostgreSQL non configuré — boucle d'annonce au relais (30 s).");
    loop {
        if let Some(tid) = config.tenant_id {
            let annonce = crate::relay_client::AnnounceRequest {
                node_id: config.node_id,
                tenant_id: tid,
                addr: node_addr(&config),
                role: if mode == RunMode::Active { "active".into() } else { "passive".into() },
                epoch: config.epoch,
            };
            if let Err(e) = relay.announce(&annonce).await {
                println!("  Relais injoignable : {}", e);
            }
        }
        tokio::time::sleep(Duration::from_secs(30)).await;
    }
}

/// Ouvre la DEK du cluster, si cette machine la détient (scellée pour sa clé).
pub fn ouvrir_dek(config: &NodeConfig) -> Result<Option<Dek>> {
    let Some(sealed_hex) = config.sealed_dek_hex.as_ref() else {
        return Ok(None);
    };
    let secret = hex::decode(&config.secret_key_hex).context("Clé secrète hex invalide dans la config")?;
    let arr: [u8; 32] = secret
        .try_into()
        .map_err(|_| anyhow!("Clé secrète de taille inattendue dans la config"))?;
    let keypair = DeviceKeyPair::from_secret_bytes(arr);
    let sealed = hex::decode(sealed_hex).context("Sealed DEK hex invalide dans la config")?;
    let dek = keypair
        .open_sealed_dek(&sealed)
        .context("Impossible de déchiffrer la DEK — config corrompue ?")?;
    Ok(Some(dek))
}

/// Ouvre le journal chiffré si la DEK est disponible sur cette machine.
fn ouvrir_journal(config: &NodeConfig, config_path: &Path) -> Result<Option<Journal>> {
    let Some(dek) = ouvrir_dek(config)? else {
        return Ok(None);
    };
    let chemin = config_path.parent().unwrap_or(Path::new(".")).join("journal.bin");
    Ok(Some(Journal::open(chemin, dek).context("Impossible d'ouvrir le journal")?))
}

/// Délai avant la première sauvegarde après une prise de rôle primaire, puis
/// période entre deux sauvegardes (`AMANE_SAUVEGARDE_DELAI_S`,
/// `AMANE_SAUVEGARDE_PERIODE_S`).
fn duree_env(nom: &str, defaut: u64) -> Duration {
    Duration::from_secs(std::env::var(nom).ok().and_then(|v| v.parse().ok()).unwrap_or(defaut))
}

/// Décision du nœud pour un tick : rôle à annoncer, droit d'écrire.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Decision {
    /// Rôle annoncé au portail (`""` = inconnu, rien n'est annoncé).
    role: &'static str,
    /// Le nœud peut écrire dans son journal en tant que primaire.
    ecriture_autorisee: bool,
    /// Le nœud se croit primaire mais une époque plus récente existe.
    fence: bool,
}

/// Fait respecter le quorum et l'époque. Fonction pure.
///
/// Un nœud n'écrit en primaire que si Patroni le dit primaire, que le quorum
/// n'est pas perdu, et que sa timeline est la plus récente du cluster : un
/// ancien primaire revenu (timeline inférieure) est fencé, même si son
/// PostgreSQL se croit encore primaire. Le contrôle est fait à chaque tick, et
/// non plus au seul démarrage.
fn decider(role_local: RoleLocal, timeline_locale: Option<u64>, epoque_max: u64, quorum: &EtatQuorum) -> Decision {
    match (role_local, timeline_locale) {
        (RoleLocal::Primaire, Some(tl)) => {
            let a_jour = matches!(check_fencing(EpochToken(tl), EpochToken(epoque_max)), FencingResult::Allowed);
            let quorum_ok = !matches!(quorum, EtatQuorum::Perdu(_));
            Decision {
                role: if a_jour { "primary" } else { "standby" },
                ecriture_autorisee: a_jour && quorum_ok,
                fence: !a_jour,
            }
        }
        (RoleLocal::Primaire, None) => Decision { role: "", ecriture_autorisee: false, fence: false },
        (RoleLocal::Standby, _) => Decision { role: "standby", ecriture_autorisee: false, fence: false },
        (RoleLocal::Inconnu, _) => Decision { role: "", ecriture_autorisee: false, fence: false },
    }
}

/// Boucle de supervision : lit Patroni, classe le cluster, fait respecter
/// l'époque, remonte rôle et état au portail. Ne décide aucune bascule.
async fn boucle_supervision(
    base: &str,
    mut journal: Option<Journal>,
    node_id: Uuid,
    canal_role: watch::Sender<String>,
    canal_etat: watch::Sender<String>,
    sauvegardes: Option<(std::sync::Arc<Dek>, crate::sauvegarde::Cible)>,
) {
    println!("  Patroni  : supervision en lecture ({base}) — la bascule appartient à etcd/Patroni");
    // Sauvegardes chiffrées (étape 4) : faites par le primaire autorisé à
    // écrire ; les résultats reviennent par ce canal pour être consignés dans
    // le journal sans bloquer la supervision.
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<(String, std::result::Result<(String, usize), String>)>();
    let mut prochaine_sauvegarde: Option<std::time::Instant> = None;
    let mut epoque_max: u64 = 0;
    let mut ecrivait = false;
    let mut dernier_etat = String::new();
    let mut fence_signale = false;

    loop {
        let etat = patroni::lire_etat(base).await;
        let quorum = evaluer_quorum(&etat.observation);
        if let Some(e) = epoque_cluster(&etat.observation) {
            epoque_max = epoque_max.max(e);
        }
        let d = decider(etat.role_local, etat.timeline_locale, epoque_max, &quorum);

        if !d.role.is_empty() {
            majr_role(&canal_role, node_id, d.role, quorum.code()).await;
        }

        let resume = match &quorum {
            EtatQuorum::Sain => "sain".to_string(),
            EtatQuorum::SousLeSeuil(m) => format!("sous le seuil — {m}"),
            EtatQuorum::Perdu(m) => format!("quorum perdu — {m}"),
        };
        if resume != dernier_etat {
            println!("  Cluster  : {resume} (époque {epoque_max})");
            dernier_etat = resume;
            let _ = canal_etat.send_replace(quorum.code().to_string());
            // Annonce immédiate : la PME doit apprendre sans attendre le
            // prochain battement de cœur (jusqu'à 60 s) qu'elle n'est plus
            // protégée — ou qu'elle l'est de nouveau.
            let role = canal_role.borrow().clone();
            annoncer_au_saas(node_id, &role, quorum.code(), "", false).await;
        }

        if d.fence && !fence_signale {
            println!(
                "  FENCING  : timeline locale {:?} < époque du cluster {} — ce nœud n'écrit plus",
                etat.timeline_locale, epoque_max
            );
        }
        fence_signale = d.fence;

        if d.ecriture_autorisee && !ecrivait {
            if let Some(j) = journal.as_mut() {
                let payload = format!("noeud {node_id} primaire").into_bytes();
                match j.append(epoque_max, node_id, "noeud.primaire", payload) {
                    Ok(idx) => println!("  Journal  : prise de rôle primaire consignée (index {idx}, époque {epoque_max})"),
                    Err(e) => println!("  Journal  : écriture impossible — {e}"),
                }
            }
        }
        ecrivait = d.ecriture_autorisee;

        if let Some((dek, cible)) = &sauvegardes {
            let maintenant = std::time::Instant::now();
            match (d.ecriture_autorisee, prochaine_sauvegarde) {
                (false, _) => prochaine_sauvegarde = None,
                (true, None) => {
                    prochaine_sauvegarde = Some(maintenant + duree_env("AMANE_SAUVEGARDE_DELAI_S", 300));
                }
                (true, Some(echeance)) if maintenant >= echeance => {
                    prochaine_sauvegarde = Some(maintenant + duree_env("AMANE_SAUVEGARDE_PERIODE_S", 86_400));
                    let (dek, cible, tx) = (dek.clone(), cible.clone(), tx.clone());
                    tokio::spawn(async move {
                        for base in crate::sauvegarde::bases_a_sauvegarder() {
                            let res = crate::sauvegarde::sauvegarder(&dek, &cible, base).await.map_err(|e| e.to_string());
                            let _ = tx.send((base.to_string(), res));
                        }
                    });
                }
                _ => {}
            }
        }
        while let Ok((base, res)) = rx.try_recv() {
            match res {
                Ok((cle, taille)) => {
                    println!("  Sauvegarde : {base} chiffrée et déposée sur le relais ({cle}, {taille} octets)");
                    if let Some(j) = journal.as_mut() {
                        let _ = j.append(epoque_max, node_id, &format!("sauvegarde.{base}"), format!("{cle} {taille}").into_bytes());
                    }
                }
                Err(e) => println!("  Sauvegarde : {base} ÉCHEC — {e}"),
            }
        }

        tokio::time::sleep(PERIODE_SUPERVISION).await;
    }
}

/// Hors pile V3 : rôle lu dans PostgreSQL, sans garde de quorum ni bascule.
async fn boucle_role_postgresql(pool: PgPool, node_id: Uuid, canal_role: watch::Sender<String>) {
    loop {
        let role = match sqlx::query_scalar::<_, bool>("SELECT pg_is_in_recovery()").fetch_one(&pool).await {
            Ok(true) => "standby",
            Ok(false) => "primary",
            Err(_) => "",
        };
        if !role.is_empty() {
            majr_role(&canal_role, node_id, role, "").await;
        }
        tokio::time::sleep(Duration::from_secs(30)).await;
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
async fn majr_role(canal: &watch::Sender<String>, node_id: Uuid, role: &str, etat_quorum: &str) {
    if appliquer_role(canal, role) {
        annoncer_au_saas(node_id, role, etat_quorum, "", true).await;
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
/// tâche, plus trois récepteurs `watch` (rôle, état du cluster, état de l'application). On ne clone
/// délibérément **pas** `NodeConfig` : il porte `secret_key_hex` et
/// `sealed_dek_hex`, et dupliquer du matériel de clé sans nécessité contrevient
/// à la règle « socle crypto » du dépôt. Tout le reste (SAAS_URL,
/// REGISTRATION_TOKEN, HOSTNAME, NODE_ADDR, WEB_PORT) est relu dans
/// l'environnement à chaque appel, ce qui rend la tâche autonome.
fn lancer_heartbeat_saas(
    node_id: Uuid,
    recepteur_role: watch::Receiver<String>,
    recepteur_etat: watch::Receiver<String>,
    recepteur_app: watch::Receiver<String>,
) {
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
            // Rôle et état courants, pas ceux du démarrage (bascule…).
            let role = recepteur_role.borrow().clone();
            let etat = recepteur_etat.borrow().clone();
            let app = recepteur_app.borrow().clone();
            let a_loguer = !premier_succes_logue || en_echec;
            // Une panne du SaaS ne doit jamais arrêter la boucle : `annoncer_au_saas`
            // absorbe et logue ses propres erreurs.
            if annoncer_au_saas(node_id, &role, &etat, &app, a_loguer).await.reussi {
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

/// Annonce le nœud au SaaS éditeur.
///
/// `POST {SAAS_URL}/api/devices/register/`, timeout de 10 s. Non bloquant —
/// toutes les erreurs sont loguées et absorbées. Rôle et état du cluster vides
/// ne sont pas envoyés (inconnus). Seules des métadonnées partent (ADR-003).
async fn annoncer_au_saas(
    node_id: Uuid,
    node_role: &str,
    etat_quorum: &str,
    etat_application: &str,
    logger_succes: bool,
) -> Annonce {
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

    let mut body = serde_json::json!({
        "tenant_token": reg_token,
        "installation_id": node_id.to_string(),
        "hostname": hostname,
        "os": std::env::consts::OS,
        "mac_address": "",
        "node_addr": node_addr,
        "web_addr": web_addr,
        "node_kind": "serveur",
    });
    if !node_role.is_empty() {
        body["node_role"] = node_role.into();
    }
    if !etat_quorum.is_empty() {
        body["etat_quorum"] = etat_quorum.into();
    }
    if !etat_application.is_empty() {
        body["etat_application"] = etat_application.into();
    }
    // Chiffrement du disque constaté par l'installeur (ADR-008).
    if let Ok(c) = std::env::var("CHIFFREMENT_DISQUE") {
        if matches!(c.as_str(), "oui" | "non" | "inconnu") {
            body["chiffrement_disque"] = c.into();
        }
    }

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
    use ss_consensus::Motif;

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
        let (canal, recepteur) = watch::channel(String::new());
        assert_eq!(recepteur.borrow().as_str(), "");
        // Bascule : le rôle change après le démarrage.
        appliquer_role(&canal, "primary");
        // Le récepteur détenu par la tâche de battement de cœur voit la nouvelle valeur.
        assert_eq!(recepteur.borrow().as_str(), "primary");
    }

    // --- Garde de quorum et fencing continu ------------------------------

    #[test]
    fn primaire_a_jour_ecrit() {
        let d = decider(RoleLocal::Primaire, Some(3), 3, &EtatQuorum::Sain);
        assert_eq!(d, Decision { role: "primary", ecriture_autorisee: true, fence: false });
        // Sous le seuil (plus de standby synchrone) : il reste primaire et
        // écrit, mais le portail sera prévenu.
        let seul = decider(RoleLocal::Primaire, Some(3), 3, &EtatQuorum::SousLeSeuil(Motif::AucunStandbySynchrone));
        assert!(seul.ecriture_autorisee);
    }

    #[test]
    fn ancien_primaire_revenu_est_fence() {
        // Il se croit primaire, mais le cluster est passé à la timeline 4.
        let d = decider(RoleLocal::Primaire, Some(3), 4, &EtatQuorum::Sain);
        assert_eq!(d, Decision { role: "standby", ecriture_autorisee: false, fence: true });
    }

    #[test]
    fn quorum_perdu_interdit_l_ecriture() {
        let d = decider(RoleLocal::Primaire, Some(3), 3, &EtatQuorum::Perdu(Motif::PatroniInjoignable));
        assert!(!d.ecriture_autorisee);
        assert!(!d.fence);
    }

    #[test]
    fn standby_et_inconnu_n_ecrivent_jamais() {
        assert_eq!(decider(RoleLocal::Standby, Some(3), 3, &EtatQuorum::Sain).role, "standby");
        assert!(!decider(RoleLocal::Standby, Some(3), 3, &EtatQuorum::Sain).ecriture_autorisee);
        let inconnu = decider(RoleLocal::Inconnu, None, 3, &EtatQuorum::Sain);
        assert_eq!(inconnu.role, "");
        assert!(!inconnu.ecriture_autorisee);
        // Primaire sans timeline lisible : prudence, rien n'est annoncé ni écrit.
        assert!(!decider(RoleLocal::Primaire, None, 3, &EtatQuorum::Sain).ecriture_autorisee);
    }
}
