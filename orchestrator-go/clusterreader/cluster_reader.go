package clusterreader

// ClusterReader est une interface read-only permettant au module IA d'accéder
// à l'état du cluster sans jamais toucher directement à etcd ou PostgreSQL.
// Toute implémentation doit être thread-safe.
// Les noms de méthodes et signatures sont délibérément stables : ne pas les
// renuméroter ni les modifier sans mettre à jour le module IA en conséquence.
type ClusterReader interface {
	// GetClusterStatus retourne l'état actuel du cluster (primary, membres, quorum).
	GetClusterStatus() (*ClusterStatus, error)

	// GetNodeID retourne l'identifiant de ce nœud dans le cluster.
	GetNodeID() string

	// GetNodeRole retourne le rôle de ce nœud (primary, replica, learner).
	GetNodeRole() string

	// GetRecentLogs retourne les dernières entrées du journal (limit par défaut 50).
	// Ne retourne jamais de payload chiffré en clair — les champs de payload sont
	// masqués ou renvoyés sous forme de hash/taille uniquement.
	GetRecentLogs(limit uint32) ([]LoggedEntry, error)

	// GetMetrics retourne les métriques Patroni exposées par ce nœud.
	GetMetrics() (map[string]float64, error)

	// IsHealthy vérifie si ce nœud est en état de santé minimal (Ping OK).
	IsHealthy() bool
}

// ClusterStatus describes the overall state of the cluster.
type ClusterStatus struct {
	// NodeID is this node's identifier.
	NodeID string
	// IsLeader indicates whether this node currently holds the etcd lease.
	IsLeader bool
	// Members is the list of known cluster members.
	Members []string
	// Quorum is the current quorum size.
	Quorum int
	// Primary is the current primary node ID, if known.
	Primary string
}

// LoggedEntry represents a recent journal entry visible to the AI assistant.
// The payload field is intentionally opaque — never a raw AK/DEK/KEK,
// only a hash or size indicator, to respect the "no clear keys in prompts" rule.
type LoggedEntry struct {
	// Seq is the sequence number of the entry.
	Seq uint64
	// PayloadHash is a short hash of the encrypted payload (first 8 bytes as hex).
	// This allows the AI to reference "an entry was written" without seeing the key material.
	PayloadHash string
	// CommittedAt is when the entry was committed.
	CommittedAt int64
	// SiteID is the site that wrote the entry.
	SiteID string
	// OperationType is a best-effort description if available.
	OperationType string
}