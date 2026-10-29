//! Garde de quorum en lecture (refonte V3, ADR-004).
//!
//! La **décision** de promouvoir un standby n'appartient plus au nœud : elle
//! est prise par Raft (etcd) et appliquée par Patroni (ADR-001). Ce module
//! garde la **vérification** : à partir de l'état publié par Patroni, une
//! fonction pure classe le cluster en « sain », « sous le seuil » ou
//! « quorum perdu ». Le nœud s'en sert pour cesser d'écrire quand il le faut
//! et pour prévenir le portail que la PME n'est plus protégée.
//!
//! Aucune E/S ici : le nœud collecte l'état, ce module tranche.

use serde::{Deserialize, Serialize};

/// Rôle d'un membre tel que le publie Patroni (`GET /cluster`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoleMembre {
    Leader,
    SyncStandby,
    Replica,
    /// Rôle inconnu ou transitoire (démarrage, rétrogradation).
    Autre,
}

impl RoleMembre {
    /// Traduit le rôle textuel de Patroni (`leader`, `sync_standby`,
    /// `replica`, `standby_leader`…).
    pub fn depuis_patroni(role: &str) -> Self {
        match role {
            "leader" | "master" | "primary" => Self::Leader,
            "sync_standby" => Self::SyncStandby,
            "replica" => Self::Replica,
            _ => Self::Autre,
        }
    }
}

/// Un membre PostgreSQL observé.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MembreObserve {
    pub nom: String,
    pub role: RoleMembre,
    /// `running`, `streaming`, `stopped`… (état Patroni du membre).
    pub etat: String,
    /// Timeline PostgreSQL du membre — l'époque du cluster (ADR-004, décision 7).
    pub timeline: Option<u64>,
}

impl MembreObserve {
    fn en_flux(&self) -> bool {
        self.etat == "streaming" || self.etat == "running"
    }
}

/// Ce que le nœud a pu lire de Patroni à un instant donné.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Observation {
    /// L'API Patroni locale ne répond pas, ou ne peut plus lire le DCS
    /// (etcd hors d'atteinte ou sans majorité).
    Injoignable,
    /// Membres du cluster publiés par Patroni.
    Membres(Vec<MembreObserve>),
}

/// Motif d'un état dégradé, journalisé et remonté au portail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Motif {
    PatroniInjoignable,
    AucunPrimaire,
    AucunStandbySynchrone,
}

impl std::fmt::Display for Motif {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PatroniInjoignable => write!(f, "état du cluster illisible (Patroni ou etcd injoignable)"),
            Self::AucunPrimaire => write!(f, "aucun primaire élu"),
            Self::AucunStandbySynchrone => {
                write!(f, "aucun standby synchrone : les écritures ne sont plus protégées")
            }
        }
    }
}

/// Classement du cluster.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EtatQuorum {
    /// Un primaire et au moins un standby synchrone en flux.
    Sain,
    /// Un primaire, mais plus aucune copie synchrone : la PME écrit sans
    /// protection (mode non strict à deux serveurs, ADR-004 décision 2).
    SousLeSeuil(Motif),
    /// Pas de primaire, ou état illisible : aucune écriture n'est possible.
    Perdu(Motif),
}

impl EtatQuorum {
    /// Code court remonté au portail (`etat_quorum`).
    pub fn code(&self) -> &'static str {
        match self {
            Self::Sain => "sain",
            Self::SousLeSeuil(_) => "sous_le_seuil",
            Self::Perdu(_) => "perdu",
        }
    }
}

/// Classe le cluster d'après l'observation. Fonction pure.
pub fn evaluer_quorum(observation: &Observation) -> EtatQuorum {
    let membres = match observation {
        Observation::Injoignable => return EtatQuorum::Perdu(Motif::PatroniInjoignable),
        Observation::Membres(m) => m,
    };
    if !membres.iter().any(|m| m.role == RoleMembre::Leader && m.en_flux()) {
        return EtatQuorum::Perdu(Motif::AucunPrimaire);
    }
    if !membres.iter().any(|m| m.role == RoleMembre::SyncStandby && m.etat == "streaming") {
        return EtatQuorum::SousLeSeuil(Motif::AucunStandbySynchrone);
    }
    EtatQuorum::Sain
}

/// Époque du cluster : la plus haute timeline publiée par ses membres.
/// Un ancien primaire revenu porte une timeline inférieure : il est fencé.
pub fn epoque_cluster(observation: &Observation) -> Option<u64> {
    match observation {
        Observation::Injoignable => None,
        Observation::Membres(m) => m.iter().filter_map(|x| x.timeline).max(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn membre(nom: &str, role: &str, etat: &str, tl: u64) -> MembreObserve {
        MembreObserve {
            nom: nom.into(),
            role: RoleMembre::depuis_patroni(role),
            etat: etat.into(),
            timeline: Some(tl),
        }
    }

    #[test]
    fn primaire_et_standby_synchrone_donnent_un_cluster_sain() {
        let obs = Observation::Membres(vec![
            membre("poste-1", "leader", "running", 3),
            membre("poste-2", "sync_standby", "streaming", 3),
        ]);
        assert_eq!(evaluer_quorum(&obs), EtatQuorum::Sain);
        assert_eq!(evaluer_quorum(&obs).code(), "sain");
    }

    #[test]
    fn sans_standby_synchrone_le_cluster_est_sous_le_seuil() {
        let seul = Observation::Membres(vec![membre("poste-1", "leader", "running", 3)]);
        assert_eq!(
            evaluer_quorum(&seul),
            EtatQuorum::SousLeSeuil(Motif::AucunStandbySynchrone)
        );
        // Un réplica asynchrone ne protège pas les écritures.
        let asynchrone = Observation::Membres(vec![
            membre("poste-1", "leader", "running", 3),
            membre("poste-2", "replica", "streaming", 3),
        ]);
        assert_eq!(evaluer_quorum(&asynchrone).code(), "sous_le_seuil");
        // Un standby synchrone arrêté non plus.
        let arrete = Observation::Membres(vec![
            membre("poste-1", "leader", "running", 3),
            membre("poste-2", "sync_standby", "stopped", 3),
        ]);
        assert_eq!(evaluer_quorum(&arrete).code(), "sous_le_seuil");
    }

    #[test]
    fn sans_primaire_le_quorum_est_perdu() {
        let obs = Observation::Membres(vec![
            membre("poste-1", "replica", "running", 3),
            membre("poste-2", "replica", "streaming", 3),
        ]);
        assert_eq!(evaluer_quorum(&obs), EtatQuorum::Perdu(Motif::AucunPrimaire));
        // Un leader arrêté ne compte pas.
        let arrete = Observation::Membres(vec![membre("poste-1", "leader", "stopped", 3)]);
        assert_eq!(evaluer_quorum(&arrete).code(), "perdu");
    }

    #[test]
    fn patroni_injoignable_signifie_quorum_perdu() {
        assert_eq!(
            evaluer_quorum(&Observation::Injoignable),
            EtatQuorum::Perdu(Motif::PatroniInjoignable)
        );
        assert_eq!(epoque_cluster(&Observation::Injoignable), None);
    }

    #[test]
    fn l_epoque_est_la_plus_haute_timeline() {
        let obs = Observation::Membres(vec![
            membre("ancien", "replica", "running", 2),
            membre("nouveau", "leader", "running", 4),
        ]);
        assert_eq!(epoque_cluster(&obs), Some(4));
    }

    #[test]
    fn roles_patroni_reconnus() {
        assert_eq!(RoleMembre::depuis_patroni("leader"), RoleMembre::Leader);
        assert_eq!(RoleMembre::depuis_patroni("primary"), RoleMembre::Leader);
        assert_eq!(RoleMembre::depuis_patroni("sync_standby"), RoleMembre::SyncStandby);
        assert_eq!(RoleMembre::depuis_patroni("replica"), RoleMembre::Replica);
        assert_eq!(RoleMembre::depuis_patroni("standby_leader"), RoleMembre::Autre);
    }

    #[test]
    fn motifs_lisibles() {
        assert!(Motif::AucunStandbySynchrone.to_string().contains("plus protégées"));
    }
}
