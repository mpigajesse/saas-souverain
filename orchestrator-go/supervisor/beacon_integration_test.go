package supervisor

import (
	"context"
	"io"
	"log/slog"
	"os"
	"testing"
	"time"

	"github.com/amane/orchestrator-go/consensus"
)

// TestHeartbeatSessionRecreatedAfterLeaseLoss vérifie qu'un superviseur
// retrouve son heartbeat après une isolation réseau plus longue que le TTL de
// sa lease. Constaté en laboratoire (ha-lab, refonte V3 étape 2a) : la session
// en cache restait morte (« requested lease not found ») et le nœud ne battait
// plus jamais le cœur — la garde anti-partition ne le protégeait plus.
//
// Usage : AMANE_TEST_ETCD=localhost:2379 go test ./supervisor/ -run 'HeartbeatSession' -v
func TestHeartbeatSessionRecreatedAfterLeaseLoss(t *testing.T) {
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
	const name = "noeud-test-lease"
	const ttl = 2 * time.Second
	cli.Delete(ctx, hbKey(name))

	w := newHBWriter(cli, slog.New(slog.NewTextHandler(io.Discard, nil)), ttl)
	defer w.close()
	primaire := patroniStatus{Role: "primary"}

	w.publish(name, primaire, true, time.Now())
	if !w.leaderAlive(name, ttl, time.Now()) {
		t.Fatal("heartbeat initial absent")
	}

	// L'isolation fait expirer la lease côté etcd : on la révoque directement.
	ancienne := w.leases[name]
	if _, err := cli.Revoke(ctx, ancienne.Lease()); err != nil {
		t.Fatalf("révocation de la lease: %v", err)
	}
	select {
	case <-ancienne.Done():
	case <-time.After(5 * time.Second):
		t.Fatal("la session n'a pas constaté la perte de sa lease")
	}

	// Au retour, les ticks suivants doivent rétablir le heartbeat.
	limite := time.Now().Add(5 * time.Second)
	for !w.leaderAlive(name, ttl, time.Now()) {
		if time.Now().After(limite) {
			t.Fatal("heartbeat non rétabli 5 s après la perte de la lease")
		}
		w.publish(name, primaire, true, time.Now())
		time.Sleep(200 * time.Millisecond)
	}
	if w.leases[name] == ancienne {
		t.Fatal("la session morte n'a pas été remplacée")
	}
}
