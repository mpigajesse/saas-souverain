//! Lecture de l'état du cluster auprès de l'API REST de Patroni (refonte V3,
//! étape 2c). Le nœud n'interroge plus PostgreSQL pour connaître son rôle et
//! ne sonde plus ses pairs : Patroni fait autorité, etcd décide.
//!
//! `PATRONI_URL` désigne l'API Patroni de la machine (pile PME :
//! `http://amane-postgresql:8008`).

use std::time::Duration;

use serde::Deserialize;
use ss_consensus::{MembreObserve, Observation, RoleMembre};

/// Rôle de la machine locale vu par Patroni.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoleLocal {
    Primaire,
    Standby,
    Inconnu,
}

impl RoleLocal {
    /// Rôle annoncé au portail (`node_role`).
    pub fn code_portail(self) -> &'static str {
        match self {
            Self::Primaire => "primary",
            Self::Standby => "standby",
            Self::Inconnu => "",
        }
    }
}

/// État complet lu à un instant donné.
#[derive(Debug, Clone)]
pub struct EtatPatroni {
    pub role_local: RoleLocal,
    pub timeline_locale: Option<u64>,
    pub observation: Observation,
}

#[derive(Deserialize)]
struct MembreJson {
    name: String,
    role: String,
    #[serde(default)]
    state: String,
    timeline: Option<u64>,
}

#[derive(Deserialize)]
struct ClusterJson {
    #[serde(default)]
    members: Vec<MembreJson>,
}

#[derive(Deserialize)]
struct LocalJson {
    #[serde(default)]
    role: String,
    timeline: Option<u64>,
}

/// Analyse la réponse de `GET /cluster`. Fonction pure.
pub fn analyser_cluster(corps: &str) -> Option<Observation> {
    let c: ClusterJson = serde_json::from_str(corps).ok()?;
    Some(Observation::Membres(
        c.members
            .into_iter()
            .map(|m| MembreObserve {
                nom: m.name,
                role: RoleMembre::depuis_patroni(&m.role),
                etat: m.state,
                timeline: m.timeline,
            })
            .collect(),
    ))
}

/// Analyse la réponse de `GET /patroni` (membre local). Fonction pure.
pub fn analyser_local(corps: &str) -> (RoleLocal, Option<u64>) {
    match serde_json::from_str::<LocalJson>(corps) {
        Ok(l) => {
            let role = match l.role.as_str() {
                "primary" | "master" => RoleLocal::Primaire,
                "replica" | "standby_leader" => RoleLocal::Standby,
                _ => RoleLocal::Inconnu,
            };
            (role, l.timeline)
        }
        Err(_) => (RoleLocal::Inconnu, None),
    }
}

/// Lit l'état auprès de Patroni. Ne renvoie jamais d'erreur : un Patroni muet
/// donne `Observation::Injoignable` et un rôle `Inconnu`.
pub async fn lire_etat(base: &str) -> EtatPatroni {
    let client = match reqwest::Client::builder().timeout(Duration::from_secs(3)).build() {
        Ok(c) => c,
        Err(_) => return injoignable(),
    };
    let base = base.trim_end_matches('/');
    // GET /patroni répond 200 sur le primaire et 503 sur un standby : le corps
    // est lu dans les deux cas.
    let (role_local, timeline_locale) = match client.get(format!("{base}/patroni")).send().await {
        Ok(r) => analyser_local(&r.text().await.unwrap_or_default()),
        Err(_) => return injoignable(),
    };
    let observation = match client.get(format!("{base}/cluster")).send().await {
        Ok(r) if r.status().is_success() => analyser_cluster(&r.text().await.unwrap_or_default())
            .unwrap_or(Observation::Injoignable),
        _ => Observation::Injoignable,
    };
    EtatPatroni { role_local, timeline_locale, observation }
}

fn injoignable() -> EtatPatroni {
    EtatPatroni {
        role_local: RoleLocal::Inconnu,
        timeline_locale: None,
        observation: Observation::Injoignable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ss_consensus::{evaluer_quorum, EtatQuorum};

    const CLUSTER: &str = r#"{"members": [
        {"name": "poste-1", "role": "leader", "state": "running", "host": "10.0.0.1", "port": 5432, "timeline": 3},
        {"name": "poste-2", "role": "sync_standby", "state": "streaming", "host": "10.0.0.2", "port": 5432, "timeline": 3, "lag": 0}
    ], "scope": "amane"}"#;

    #[test]
    fn cluster_patroni_analyse() {
        let obs = analyser_cluster(CLUSTER).expect("JSON valide");
        assert_eq!(evaluer_quorum(&obs), EtatQuorum::Sain);
        let Observation::Membres(m) = obs else { panic!("membres attendus") };
        assert_eq!(m[1].nom, "poste-2");
        assert_eq!(m[1].role, RoleMembre::SyncStandby);
        assert_eq!(m[1].timeline, Some(3));
    }

    #[test]
    fn cluster_illisible() {
        assert!(analyser_cluster("pas du json").is_none());
        // Aucun membre : quorum perdu.
        let vide = analyser_cluster(r#"{"scope": "amane"}"#).unwrap();
        assert_eq!(evaluer_quorum(&vide).code(), "perdu");
    }

    #[test]
    fn membre_local_analyse() {
        assert_eq!(
            analyser_local(r#"{"state": "running", "role": "primary", "timeline": 4}"#),
            (RoleLocal::Primaire, Some(4))
        );
        assert_eq!(
            analyser_local(r#"{"state": "running", "role": "replica", "timeline": 4}"#).0,
            RoleLocal::Standby
        );
        assert_eq!(analyser_local("{}").0, RoleLocal::Inconnu);
        assert_eq!(analyser_local("oups").0, RoleLocal::Inconnu);
    }

    #[test]
    fn codes_portail() {
        assert_eq!(RoleLocal::Primaire.code_portail(), "primary");
        assert_eq!(RoleLocal::Standby.code_portail(), "standby");
        assert_eq!(RoleLocal::Inconnu.code_portail(), "");
    }
}
