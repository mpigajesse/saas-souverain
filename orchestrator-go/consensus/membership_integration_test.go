package consensus

import (
	"context"
	"errors"
	"fmt"
	"io"
	"log/slog"
	"os"
	"testing"
	"time"
)

// TestMembershipAgainstEtcd vérifie le registry ENRÔLEMENT/REVOCATION contre un
// etcd réel. Il est skip si AMANE_TEST_ETCD n'est pas défini.
// Usage : AMANE_TEST_ETCD=localhost:2379 go test ./consensus/ -run Membership -v
func TestMembershipAgainstEtcd(t *testing.T) {
	endpoints := os.Getenv("AMANE_TEST_ETCD")
	if endpoints == "" {
		t.Skip("AMANE_TEST_ETCD non défini — intégration etcd réelle ignorée")
	}

	cli, err := NewClient([]string{endpoints})
	if err != nil {
		t.Fatalf("client etcd: %v", err)
	}
	defer cli.Close()

	logger := slog.New(slog.NewTextHandler(io.Discard, nil))
	reg := NewRegistry(cli, logger)

	// nettoie un état résiduel éventuel
	machineID := "test-machine-42"
	reg.Remove(context.Background(), machineID)

	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	// 1) enrolment
	q1, err := reg.Add(ctx, Member{
		MachineID:   machineID,
		AKPublicKey: []byte("x25519-pub-abc"),
		SiteID:      "site-a",
		EnrolledAt:  time.Now().UTC(),
	})
	if err != nil {
		t.Fatalf("add: %v", err)
	}
	if q1 < 1 {
		t.Errorf("quorum apres add = %d", q1)
	}

	// 2) double enrolment -> ErrMemberAlreadyExists
	if _, err := reg.Add(ctx, Member{MachineID: machineID, AKPublicKey: []byte("x")}); !errors.Is(err, ErrMemberAlreadyExists) {
		t.Errorf("double add: err = %v, want ErrMemberAlreadyExists", err)
	}

	// 3) presence
	ok, err := reg.Has(ctx, machineID)
	if err != nil || !ok {
		t.Errorf("has = %v, %v", ok, err)
	}

	// 4) Quorum() lecture seule, sans modifier le membership (membre présent)
	qro, err := reg.Quorum(ctx)
	if err != nil {
		t.Fatalf("quorum lecture: %v", err)
	}
	if qro < 1 {
		t.Errorf("quorum lecture = %d, want >= 1", qro)
	}

	// 5) revocation -> quorum recalculé, membre absent
	q2, err := reg.Remove(ctx, machineID)
	if err != nil {
		t.Fatalf("remove: %v", err)
	}
	if q2 >= qro {
		t.Errorf("quorum apres remove (%d) doit être < avant (%d)", q2, qro)
	}
	if ok, _ := reg.Has(ctx, machineID); ok {
		t.Error("membre toujours present apres revocation")
	}

	// 6) revocation d'un inconnu -> ErrMemberNotFound
	if _, err := reg.Remove(ctx, machineID); !errors.Is(err, ErrMemberNotFound) {
		t.Errorf("remove inconnu: err = %v, want ErrMemberNotFound", err)
	}

	fmt.Println("OK quorum:", q1, "->", q2, "(lecture:", qro, ")")
}
