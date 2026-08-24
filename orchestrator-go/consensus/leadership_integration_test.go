package consensus

import (
	"context"
	"io"
	"log/slog"
	"os"
	"testing"
	"time"
)

// TestLeadershipFencingAgainstEtcd valide l'élection + fencing lease-based sur
// un etcd réel : un seul nœud est leader (jamais deux), et la perte de lease
// du leader transfère la capacité d'écriture au successeur.
// Usage : AMANE_TEST_ETCD=localhost:2379 go test ./consensus/ -run Leadership -v
func TestLeadershipFencingAgainstEtcd(t *testing.T) {
	endpoints := os.Getenv("AMANE_TEST_ETCD")
	if endpoints == "" {
		t.Skip("AMANE_TEST_ETCD non défini — etcd réel ignoré")
	}

	cli, err := NewClient([]string{endpoints})
	if err != nil {
		t.Fatalf("client etcd: %v", err)
	}
	defer cli.Close()

	logger := slog.New(slog.NewTextHandler(io.Discard, nil))

	key := "/amane-leader-itest/" + time.Now().Format("150405.000")
	mk := func(id string) *Leadership {
		l := NewLeadership(cli, logger, id)
		l.key = key
		l.ttlSec = 2 // lease courte pour un test rapide du fencing
		return l
	}

	ctxA, cancelA := context.WithCancel(context.Background())
	ctxB, cancelB := context.WithCancel(context.Background())
	defer cancelA()
	defer cancelB()

	a, b := mk("node-a"), mk("node-b")
	go a.Run(ctxA)

	waitIsLeader := func(l *Leadership, want bool, d time.Duration) bool {
		deadline := time.Now().Add(d)
		for time.Now().Before(deadline) {
			if l.IsLeader() == want {
				return true
			}
			time.Sleep(50 * time.Millisecond)
		}
		return l.IsLeader() == want
	}

	if !waitIsLeader(a, true, 5*time.Second) {
		t.Fatal("node-a ne devient pas leader")
	}

	// node-b ne candidate qu'après l'élection de node-a (sinon la course etcd
	// est FIFO et non déterministe) : on teste le transfert par fencing, pas
	// la course initiale.
	go b.Run(ctxB)

	// Jamais deux leaders (anti split-brain au niveau app).
	if !waitIsLeader(b, false, time.Second) {
		t.Fatal("node-b leader pendant que node-a l'est déjà : split-brain")
	}

	// Fencing : la perte de lease de node-a transfère le leadership à node-b.
	cancelA()
	if !waitIsLeader(b, true, 6*time.Second) {
		t.Fatal("node-b ne reprend pas le leadership après fencing de node-a")
	}

	// L'élection pointe bien vers node-b côté etcd (nécessaire pour que les
	// autres nœuds sachent qui écrire – cohérent avec Write gating).
	if lead := b.Leader(context.Background()); lead != "node-b" {
		t.Errorf("leader etcd = %q, want node-b", lead)
	}
}
