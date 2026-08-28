package replication

import (
	"context"
	"io"
	"log/slog"
	"testing"
	"time"
)

// fakeTransport capture les pushes et joue le pair.
type fakeTransport struct {
	received  [][]Delta
	failCount int // nombre de push à faire échouer avant de réussir
}

func (f *fakeTransport) Push(ctx context.Context, from NodeID, deltas []Delta) (uint64, int64, error) {
	if f.failCount > 0 {
		f.failCount--
		return 0, 0, errSimulated
	}
	f.received = append(f.received, deltas)
	ack := uint64(0)
	for _, d := range deltas {
		if d.Seq > ack {
			ack = d.Seq
		}
	}
	return ack, 0, nil
}

var errSimulated = io.EOF

func TestPropagatorPushesAndConfirms(t *testing.T) {
	relay := NewRelay("site-a")
	peer := &fakeTransport{}
	prop := NewPropagator(relay, peer, time.Second, nil)

	relay.Add(5, 0)
	relay.Add(2, 0) // seq 1 et 2

	if err := prop.SyncNow(context.Background()); err != nil {
		t.Fatalf("sync: %v", err)
	}
	if len(peer.received) != 1 {
		t.Fatalf("push count = %d, want 1", len(peer.received))
	}
	// Tous les deltas envoyés, pending confirmé (seq 2) → vidé.
	if got := relay.Outgoing(); len(got) != 0 {
		t.Errorf("pending après ack = %d, want 0", len(got))
	}

	// Nouveau delta → poussé de nouveau.
	relay.Add(0, 1)
	prop.SyncNow(context.Background())
	if len(peer.received) != 2 || len(peer.received[1]) != 1 {
		t.Fatalf("deuxième push = %d deltas, want 1", len(peer.received[1]))
	}
	if peer.received[1][0].Dec != 1 {
		t.Errorf("delta poussé = %+v, want dec 1", peer.received[1][0])
	}
}

func TestPropagatorRetriesOnErrorWithoutLosingDeltas(t *testing.T) {
	relay := NewRelay("site-a")
	peer := &fakeTransport{failCount: 2}
	prop := NewPropagator(relay, peer, time.Second, slog.New(slog.NewTextHandler(io.Discard, nil)))

	relay.Add(3, 0)

	// Deux échecs simulés : le delta reste en pending (jamais perdu).
	for i := 0; i < 2; i++ {
		if err := prop.SyncNow(context.Background()); err == nil {
			t.Fatalf("sync %d: nil err, want simulation", i)
		}
		if got := len(relay.Outgoing()); got != 1 {
			t.Fatalf("pending après échec %d = %d, want 1 (jamais perdu)", i, got)
		}
	}

	// Troisième passe : le peer répond, le delta est confirmé et retiré.
	if err := prop.SyncNow(context.Background()); err != nil {
		t.Fatalf("sync final: %v", err)
	}
	if got := relay.Outgoing(); len(got) != 0 {
		t.Errorf("pending = %d, want 0", len(got))
	}
}

func TestPropagatorNoopWithoutPending(t *testing.T) {
	relay := NewRelay("site-a")
	peer := &fakeTransport{}
	prop := NewPropagator(relay, peer, time.Second, nil)

	if err := prop.SyncNow(context.Background()); err != nil {
		t.Fatalf("sync vide: %v", err)
	}
	if len(peer.received) != 0 {
		t.Errorf("push sur pending vide = %d, want 0", len(peer.received))
	}
}

func TestPropagatorRunStopsOnCancel(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	prop := NewPropagator(NewRelay("site-a"), &fakeTransport{}, 10*time.Millisecond, nil)
	done := make(chan error, 1)
	go func() { done <- prop.Run(ctx) }()
	cancel()
	select {
	case err := <-done:
		if err != context.Canceled {
			t.Errorf("err = %v, want context.Canceled", err)
		}
	case <-time.After(2 * time.Second):
		t.Fatal("Run ne s'arrête pas sur cancel")
	}
}
