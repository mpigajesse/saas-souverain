use std::net::SocketAddr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::EpochToken;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeRole {
    Active,
    Passive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub id: Uuid,
    pub addr: SocketAddr,
    pub role: NodeRole,
    pub epoch: EpochToken,
    pub last_seen: DateTime<Utc>,
}

/// État courant du cluster, tel que vu par un nœud.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterState {
    pub nodes: Vec<NodeInfo>,
    pub current_epoch: EpochToken,
}

impl ClusterState {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            current_epoch: EpochToken::initial(),
        }
    }

    /// Ajouter ou mettre à jour un nœud (identifié par son UUID).
    pub fn upsert_node(&mut self, node: NodeInfo) {
        if let Some(existing) = self.nodes.iter_mut().find(|n| n.id == node.id) {
            *existing = node;
        } else {
            self.nodes.push(node);
        }
    }

    /// Nombre de nœuds actifs connus.
    pub fn active_nodes(&self) -> usize {
        self.nodes.iter().filter(|n| n.role == NodeRole::Active).count()
    }

    /// Le failover automatique est possible si ≥ 3 nœuds sont connus.
    pub fn can_auto_failover(&self) -> bool {
        self.nodes.len() >= 3
    }

    /// Taille du quorum (majorité simple).
    pub fn quorum_size(&self) -> usize {
        self.nodes.len() / 2 + 1
    }

    /// Alerte : le cluster est passé sous 3 nœuds → bascule manuelle uniquement.
    pub fn below_auto_failover_threshold(&self) -> bool {
        self.nodes.len() < 3
    }
}

impl Default for ClusterState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_node(role: NodeRole, epoch: u64) -> NodeInfo {
        NodeInfo {
            id: Uuid::new_v4(),
            addr: "127.0.0.1:9000".parse().unwrap(),
            role,
            epoch: EpochToken(epoch),
            last_seen: chrono::Utc::now(),
        }
    }

    #[test]
    fn auto_failover_requires_3() {
        let mut s = ClusterState::new();
        assert!(!s.can_auto_failover());
        s.upsert_node(make_node(NodeRole::Active, 1));
        s.upsert_node(make_node(NodeRole::Passive, 1));
        assert!(!s.can_auto_failover());
        s.upsert_node(make_node(NodeRole::Passive, 1));
        assert!(s.can_auto_failover());
    }

    #[test]
    fn quorum_majority() {
        let mut s = ClusterState::new();
        for _ in 0..5 {
            s.upsert_node(make_node(NodeRole::Passive, 1));
        }
        assert_eq!(s.quorum_size(), 3);
    }

    #[test]
    fn upsert_updates_existing_node() {
        let mut s = ClusterState::new();
        let id = Uuid::new_v4();
        let node = NodeInfo {
            id,
            addr: "127.0.0.1:9000".parse().unwrap(),
            role: NodeRole::Passive,
            epoch: EpochToken(1),
            last_seen: Utc::now(),
        };
        s.upsert_node(node);
        assert_eq!(s.nodes.len(), 1);

        // Mettre à jour le même nœud
        let updated = NodeInfo {
            id,
            addr: "127.0.0.1:9001".parse().unwrap(),
            role: NodeRole::Active,
            epoch: EpochToken(2),
            last_seen: Utc::now(),
        };
        s.upsert_node(updated);
        // Toujours un seul nœud
        assert_eq!(s.nodes.len(), 1);
        assert_eq!(s.nodes[0].role, NodeRole::Active);
        assert_eq!(s.nodes[0].epoch.value(), 2);
    }

    #[test]
    fn below_threshold_when_fewer_than_3() {
        let mut s = ClusterState::new();
        assert!(s.below_auto_failover_threshold());
        s.upsert_node(make_node(NodeRole::Active, 1));
        s.upsert_node(make_node(NodeRole::Passive, 1));
        assert!(s.below_auto_failover_threshold());
        s.upsert_node(make_node(NodeRole::Passive, 1));
        assert!(!s.below_auto_failover_threshold());
    }

    #[test]
    fn active_nodes_count() {
        let mut s = ClusterState::new();
        s.upsert_node(make_node(NodeRole::Active, 1));
        s.upsert_node(make_node(NodeRole::Passive, 1));
        s.upsert_node(make_node(NodeRole::Active, 1));
        assert_eq!(s.active_nodes(), 2);
    }
}
