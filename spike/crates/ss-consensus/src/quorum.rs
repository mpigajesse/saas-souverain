//! Garde de quorum pour le failover automatique.
//!
//! Règle posée par `CLAUDE.md` : « 2 nœuds → bascule manuelle uniquement ;
//! ≥ 3 nœuds → failover automatique par quorum ». Ce module implémente la
//! décision, en **fonction pure** : aucune E/S, aucun accès réseau, aucun
//! `std::env` — le nœud collecte les observations, ce module tranche.
//!
//! Ce n'est pas du consensus Raft : il n'y a pas de DCS dans la stack
//! `pme-deploy/` (réplication streaming PostgreSQL nue). La garde repose donc
//! sur deux mécanismes complémentaires :
//!
//! 1. **Majorité joignable** — un nœud isolé par une partition réseau se voit
//!    en minorité et refuse de se promouvoir. C'est ce qui empêche le
//!    split-brain de partition.
//! 2. **Élection déterministe** — parmi les standbys qui ont le quorum, un seul
//!    se promeut : le plus à jour (LSN le plus avancé) et, à égalité stricte,
//!    celui dont l'adresse est la plus grande dans l'ordre lexicographique.
//!    C'est ce qui empêche la double promotion quand les deux standbys sont
//!    sains et se voient mutuellement. Le sens du départage est arbitraire ;
//!    seule compte sa cohérence : les deux nœuds comparent le même couple
//!    `(lsn, adresse)` et désignent donc le même gagnant.
//!
//! Les deux nœuds évaluent les mêmes observations et aboutissent à la même
//! conclusion, sans échanger de message dédié.

/// Ce qu'un nœud passif observe d'un pair au moment de décider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PairObserve {
    /// Adresse du pair telle que configurée — sert de départage déterministe.
    pub addr: String,
    /// Le pair a répondu à la sonde dans le délai imparti.
    pub joignable: bool,
    /// Le pair est en recovery (standby). `false` = primaire vivant.
    /// N'a de sens que si `joignable`.
    pub en_recovery: bool,
    /// Position WAL reçue par le pair, en octets. `None` si illisible.
    pub lsn: Option<u64>,
}

/// Motif de refus, journalisé tel quel par le nœud.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MotifRefus {
    /// Moins de 3 nœuds : la règle impose la bascule manuelle.
    ClusterTropPetit { taille: usize },
    /// Un pair joignable n'est pas en recovery — un primaire est vivant.
    PrimairePresent { addr: String },
    /// Ce nœud ne voit pas la majorité du cluster : il est probablement isolé.
    PasDeQuorum {
        joignables: usize,
        taille: usize,
        requis: usize,
    },
    /// Un autre standby est meilleur candidat — il se promouvra, pas nous.
    PairPlusAJour { addr: String },
}

impl std::fmt::Display for MotifRefus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MotifRefus::ClusterTropPetit { taille } => write!(
                f,
                "cluster à {taille} nœud(s) — la bascule automatique exige ≥ 3 nœuds, \
                 promouvoir manuellement avec `ss-node failover`"
            ),
            MotifRefus::PrimairePresent { addr } => write!(
                f,
                "un primaire est vivant sur {addr} — pas de promotion"
            ),
            MotifRefus::PasDeQuorum {
                joignables,
                taille,
                requis,
            } => write!(
                f,
                "pas de quorum : {joignables}/{taille} nœud(s) joignable(s), {requis} requis — \
                 ce nœud est probablement isolé du cluster"
            ),
            MotifRefus::PairPlusAJour { addr } => write!(
                f,
                "le standby {addr} est plus à jour — il assurera la promotion"
            ),
        }
    }
}

/// Verdict de la garde de quorum.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecisionFailover {
    Promouvoir,
    Refus(MotifRefus),
}

/// Décide si ce nœud passif peut se promouvoir.
///
/// `mon_addr` et `mon_lsn` décrivent le nœud appelant ; `pairs` contient une
/// observation par **autre** nœud du cluster (joignable ou non — un pair muet
/// doit figurer dans la liste avec `joignable: false`, sinon la taille du
/// cluster est sous-estimée et le quorum devient trop facile à atteindre).
///
/// L'ordre des contrôles est significatif : un primaire vivant l'emporte sur
/// toute autre considération, et la taille du cluster est vérifiée avant le
/// quorum pour produire le message le plus actionnable.
pub fn decider_failover(
    mon_addr: &str,
    mon_lsn: Option<u64>,
    pairs: &[PairObserve],
) -> DecisionFailover {
    let taille = pairs.len() + 1;

    // 1. Règle CLAUDE.md : pas d'automatisme sous 3 nœuds.
    if taille < 3 {
        return DecisionFailover::Refus(MotifRefus::ClusterTropPetit { taille });
    }

    // 2. Un primaire joignable ⇒ la panne était transitoire (WAL receiver
    //    coupé mais primaire vivant). Se promouvoir ici créerait le
    //    split-brain qu'on cherche à éviter.
    if let Some(p) = pairs.iter().find(|p| p.joignable && !p.en_recovery) {
        return DecisionFailover::Refus(MotifRefus::PrimairePresent {
            addr: p.addr.clone(),
        });
    }

    // 3. Majorité stricte, ce nœud inclus.
    let joignables = 1 + pairs.iter().filter(|p| p.joignable).count();
    let requis = taille / 2 + 1;
    if joignables < requis {
        return DecisionFailover::Refus(MotifRefus::PasDeQuorum {
            joignables,
            taille,
            requis,
        });
    }

    // 4. Élection déterministe parmi les standbys joignables.
    //    Meilleur = LSN le plus avancé ; à égalité, adresse la plus grande.
    //    Un LSN illisible est traité comme 0 : un nœud dont on ne sait rien ne
    //    doit pas gagner l'élection.
    let mon_rang = (mon_lsn.unwrap_or(0), mon_addr);
    for p in pairs.iter().filter(|p| p.joignable && p.en_recovery) {
        let rang_pair = (p.lsn.unwrap_or(0), p.addr.as_str());
        // `>` strict sur le couple : à égalité parfaite de LSN, l'adresse
        // départage. L'ordre total garantit qu'un seul nœud se juge meilleur —
        // et donc qu'un seul ne trouve aucun pair supérieur à lui.
        if rang_pair > mon_rang {
            return DecisionFailover::Refus(MotifRefus::PairPlusAJour {
                addr: p.addr.clone(),
            });
        }
    }

    DecisionFailover::Promouvoir
}

/// Convertit un LSN PostgreSQL textuel (`0/3000060`) en octets.
///
/// Le format est `XXXXXXXX/YYYYYYYY`, deux entiers hexadécimaux : le premier
/// est le numéro de fichier WAL, le second le décalage. Retourne `None` sur
/// toute entrée malformée — l'appelant la traitera comme « position inconnue ».
pub fn parse_lsn(brut: &str) -> Option<u64> {
    let (haut, bas) = brut.trim().split_once('/')?;
    let haut = u64::from_str_radix(haut.trim(), 16).ok()?;
    let bas = u64::from_str_radix(bas.trim(), 16).ok()?;
    Some((haut << 32) | bas)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pair(addr: &str, joignable: bool, en_recovery: bool, lsn: Option<u64>) -> PairObserve {
        PairObserve {
            addr: addr.to_string(),
            joignable,
            en_recovery,
            lsn,
        }
    }

    // --- Règle des 3 nœuds ------------------------------------------------

    #[test]
    fn deux_noeuds_refusent_la_bascule_automatique() {
        // Un seul pair ⇒ cluster de 2 ⇒ bascule manuelle uniquement.
        let pairs = vec![pair("10.0.0.2:5432", false, false, None)];
        assert_eq!(
            decider_failover("10.0.0.1:5432", Some(100), &pairs),
            DecisionFailover::Refus(MotifRefus::ClusterTropPetit { taille: 2 })
        );
    }

    #[test]
    fn noeud_isole_sans_pair_refuse() {
        assert_eq!(
            decider_failover("10.0.0.1:5432", Some(100), &[]),
            DecisionFailover::Refus(MotifRefus::ClusterTropPetit { taille: 1 })
        );
    }

    // --- Primaire vivant --------------------------------------------------

    #[test]
    fn primaire_joignable_bloque_la_promotion() {
        // Le WAL receiver local est tombé, mais le primaire répond : c'est
        // notre lien qui est cassé, pas le primaire.
        let pairs = vec![
            pair("10.0.0.1:5432", true, false, None),
            pair("10.0.0.3:5432", true, true, Some(50)),
        ];
        assert_eq!(
            decider_failover("10.0.0.2:5432", Some(100), &pairs),
            DecisionFailover::Refus(MotifRefus::PrimairePresent {
                addr: "10.0.0.1:5432".to_string()
            })
        );
    }

    // --- Quorum -----------------------------------------------------------

    #[test]
    fn partition_reseau_met_le_noeud_en_minorite() {
        // 3 nœuds, aucun pair joignable : 1/3 < 2 requis.
        let pairs = vec![
            pair("10.0.0.1:5432", false, false, None),
            pair("10.0.0.3:5432", false, false, None),
        ];
        assert_eq!(
            decider_failover("10.0.0.2:5432", Some(100), &pairs),
            DecisionFailover::Refus(MotifRefus::PasDeQuorum {
                joignables: 1,
                taille: 3,
                requis: 2
            })
        );
    }

    #[test]
    fn quorum_atteint_avec_un_pair_sur_trois() {
        // Primaire mort (injoignable), l'autre standby répond et est en retard.
        let pairs = vec![
            pair("10.0.0.1:5432", false, false, None),
            pair("10.0.0.3:5432", true, true, Some(50)),
        ];
        assert_eq!(
            decider_failover("10.0.0.2:5432", Some(100), &pairs),
            DecisionFailover::Promouvoir
        );
    }

    #[test]
    fn cluster_de_cinq_exige_trois_joignables() {
        let mut pairs = vec![
            pair("10.0.0.1:5432", false, false, None),
            pair("10.0.0.3:5432", true, true, Some(10)),
        ];
        pairs.push(pair("10.0.0.4:5432", false, false, None));
        pairs.push(pair("10.0.0.5:5432", false, false, None));
        // 2 joignables sur 5 ⇒ requis 3 ⇒ refus.
        assert_eq!(
            decider_failover("10.0.0.2:5432", Some(100), &pairs),
            DecisionFailover::Refus(MotifRefus::PasDeQuorum {
                joignables: 2,
                taille: 5,
                requis: 3
            })
        );
    }

    // --- Élection déterministe : le cœur de l'anti-split-brain ------------

    #[test]
    fn un_seul_des_deux_standbys_se_promeut() {
        // Le scénario du test A de TEST-FAILOVER.md : primaire coupé, deux
        // standbys sains qui se voient. Chacun évalue de son côté ; un seul
        // doit conclure « Promouvoir ».
        let primaire_mort = pair("10.174.40.45:5432", false, false, None);

        // Vu par Fedora (LSN 200, le plus avancé)
        let vu_par_fedora = vec![
            primaire_mort.clone(),
            pair("10.174.40.50:5432", true, true, Some(150)),
        ];
        // Vu par DEATHSTROKE (LSN 150, en retard)
        let vu_par_deathstroke = vec![
            primaire_mort,
            pair("10.174.40.77:5432", true, true, Some(200)),
        ];

        assert_eq!(
            decider_failover("10.174.40.77:5432", Some(200), &vu_par_fedora),
            DecisionFailover::Promouvoir
        );
        assert_eq!(
            decider_failover("10.174.40.50:5432", Some(150), &vu_par_deathstroke),
            DecisionFailover::Refus(MotifRefus::PairPlusAJour {
                addr: "10.174.40.77:5432".to_string()
            })
        );
    }

    #[test]
    fn egalite_de_lsn_departagee_par_adresse() {
        // Deux standbys exactement au même point : l'adresse tranche. Ce qui
        // compte n'est pas le sens du départage mais son unicité — exactement
        // un des deux doit conclure « Promouvoir ».
        let primaire_mort = pair("10.0.0.1:5432", false, false, None);

        let vu_par_deux = vec![
            primaire_mort.clone(),
            pair("10.0.0.3:5432", true, true, Some(100)),
        ];
        let vu_par_trois = vec![primaire_mort, pair("10.0.0.2:5432", true, true, Some(100))];

        // L'adresse la plus grande l'emporte : .3 se promeut, .2 s'efface.
        assert_eq!(
            decider_failover("10.0.0.3:5432", Some(100), &vu_par_trois),
            DecisionFailover::Promouvoir
        );
        assert_eq!(
            decider_failover("10.0.0.2:5432", Some(100), &vu_par_deux),
            DecisionFailover::Refus(MotifRefus::PairPlusAJour {
                addr: "10.0.0.3:5432".to_string()
            })
        );
    }

    #[test]
    fn lsn_illisible_perd_contre_lsn_connu() {
        let pairs = vec![
            pair("10.0.0.1:5432", false, false, None),
            pair("10.0.0.3:5432", true, true, Some(50)),
        ];
        // Notre LSN est inconnu (0) face à 50 ⇒ on cède.
        assert_eq!(
            decider_failover("10.0.0.2:5432", None, &pairs),
            DecisionFailover::Refus(MotifRefus::PairPlusAJour {
                addr: "10.0.0.3:5432".to_string()
            })
        );
    }

    #[test]
    fn un_standby_injoignable_ne_participe_pas_a_l_election() {
        // Cluster de 5 pour que le quorum soit atteint (3 joignables sur 5) et
        // que seul l'aspect « élection » soit sous test.
        //
        // Le pair .3 serait le meilleur candidat (LSN 900) mais il est muet :
        // il ne peut pas se promouvoir, donc il ne doit pas nous en empêcher.
        let pairs = vec![
            pair("10.0.0.1:5432", false, false, None),  // primaire mort
            pair("10.0.0.3:5432", false, true, Some(900)), // standby muet
            pair("10.0.0.4:5432", true, true, Some(10)),
            pair("10.0.0.5:5432", true, true, Some(20)),
        ];
        assert_eq!(
            decider_failover("10.0.0.2:5432", Some(100), &pairs),
            DecisionFailover::Promouvoir
        );
    }

    #[test]
    fn un_standby_muet_compte_quand_meme_dans_la_taille_du_cluster() {
        // Contre-épreuve du test précédent : si l'on retirait les pairs muets
        // de la liste au lieu de les marquer `joignable: false`, la taille
        // tomberait à 3, le seuil à 2, et la promotion passerait à tort.
        let pairs = vec![
            pair("10.0.0.1:5432", false, false, None),
            pair("10.0.0.3:5432", false, true, Some(900)),
            pair("10.0.0.4:5432", false, true, Some(10)),
            pair("10.0.0.5:5432", true, true, Some(20)),
        ];
        assert_eq!(
            decider_failover("10.0.0.2:5432", Some(100), &pairs),
            DecisionFailover::Refus(MotifRefus::PasDeQuorum {
                joignables: 2,
                taille: 5,
                requis: 3
            })
        );
    }

    // --- Analyse des LSN --------------------------------------------------

    #[test]
    fn parse_lsn_format_postgres() {
        assert_eq!(parse_lsn("0/3000060"), Some(0x3000060));
        assert_eq!(parse_lsn("1/0"), Some(1 << 32));
        assert_eq!(parse_lsn("  2/A0  "), Some((2u64 << 32) | 0xA0));
    }

    #[test]
    fn parse_lsn_rejette_les_entrees_malformees() {
        assert_eq!(parse_lsn(""), None);
        assert_eq!(parse_lsn("3000060"), None);
        assert_eq!(parse_lsn("zz/3000060"), None);
        assert_eq!(parse_lsn("0/zz"), None);
    }

    #[test]
    fn lsn_croissants_restent_ordonnes() {
        // Garde-fou sur le décalage 32 bits : un changement de fichier WAL doit
        // rester supérieur à n'importe quel décalage du fichier précédent.
        let a = parse_lsn("0/FFFFFFFF").unwrap();
        let b = parse_lsn("1/0").unwrap();
        assert!(b > a);
    }
}
