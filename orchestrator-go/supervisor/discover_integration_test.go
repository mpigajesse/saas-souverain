package supervisor

import (
	"context"
	"io"
	"log/slog"
	"os"
	"testing"

	clientv3 "go.etcd.io/etcd/client/v3"

	"github.com/amane/orchestrator-go/consensus"
	"github.com/amane/orchestrator-go/dcs"
)

// TestDiscoverNodesAgainstEtcd : en pile PME, le superviseur suit les membres
// Patroni enregistrés dans etcd, y compris une machine ajoutée après coup.
// Usage : AMANE_TEST_ETCD=localhost:2379 go test ./supervisor/ -run 'DiscoverNodes' -v
func TestDiscoverNodesAgainstEtcd(t *testing.T) {
	endpoints := os.Getenv("AMANE_TEST_ETCD")
	if endpoints == "" {
		t.Skip("AMANE_TEST_ETCD non défini — intégration etcd réelle ignorée")
	}
	cli, err := consensus.NewClient([]string{endpoints})
	if err != nil {
		t.Fatalf("client etcd: %v", err)
	}
	defer cli.Close()
	ctx := context.Background()
	const scope = "amane-test-decouverte"
	prefixe := dcs.PrefixeMembres(scope)
	cli.Delete(ctx, prefixe, clientv3.WithPrefix())
	defer cli.Delete(ctx, prefixe, clientv3.WithPrefix())

	s := New(Config{EtcdEndpoints: []string{endpoints}, Scope: scope, Discover: true},
		slog.New(slog.NewTextHandler(io.Discard, nil)))
	s.cli = cli

	cli.Put(ctx, prefixe+"poste-a", `{"conn_url": "postgres://10.0.0.1:5432/postgres", "api_url": "http://10.0.0.1:8008/patroni"}`)
	s.refreshNodes(ctx)
	if len(s.cfg.Nodes) != 1 || s.cfg.Nodes[0].PatroniURL != "http://10.0.0.1:8008" {
		t.Fatalf("premier membre non découvert : %+v", s.cfg.Nodes)
	}

	cli.Put(ctx, prefixe+"poste-b", `{"conn_url": "postgres://10.0.0.2:5432/postgres", "api_url": "http://10.0.0.2:8008/patroni"}`)
	s.refreshNodes(ctx)
	if len(s.cfg.Nodes) != 2 || s.cfg.byName("poste-b") == nil {
		t.Fatalf("machine ajoutée non découverte : %+v", s.cfg.Nodes)
	}
}
