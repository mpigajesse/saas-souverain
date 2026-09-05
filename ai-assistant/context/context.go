package context

import (
	"context"

	"github.com/amane/orchestrator-go/clusterreader"
)

// AIContext rassemble toutes les informations visibles par le modèle IA,
// construites de manière read-only à partir des RPC Mission C.
// Aucun accès direct à etcd/PostgreSQL — uniquement via l'interface ClusterReader.
type AIContext struct {
	// ClusterStatus est l'état actuel du cluster (leader, membres, quorum).
	ClusterStatus *clusterreader.ClusterStatus
	// RecentLogs sont les entrées récentes du journal avec hachages de payload
	// (jamais de clair AK/DEK/KEK).
	RecentLogs []clusterreader.LoggedEntry
	// Metrics sont les métriques Patroni disponibles (vide pour l'instant).
	Metrics map[string]float64
	// NodeID est l'identifiant de ce nœud.
	NodeID string
	// IsHealthy indique si ce nœud est en état de santé minimal.
	IsHealthy bool
}

// BuildAIContext construit un AIContext à partir d'un ClusterReader.
// C'est la seule façon dont le module IA accède à l'état du cluster.
func BuildAIContext(ctx context.Context, reader clusterreader.ClusterReader) (*AIContext, error) {
	// Pour l'instant, on retourne un contexte vide.
	// Plus tard, on remplira les champs à partir du ClusterReader.
	return &AIContext{
		NodeID:    reader.GetNodeID(),
		IsHealthy: reader.IsHealthy(),
	}, nil
}
