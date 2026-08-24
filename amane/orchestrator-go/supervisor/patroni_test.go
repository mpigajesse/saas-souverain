package supervisor

import (
	"context"
	"encoding/json"
	"io"
	"log/slog"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
	"time"
)

func quietLogger() *slog.Logger { return slog.New(slog.NewTextHandler(io.Discard, nil)) }

// TestProbePatroni : parsing de GET /patroni (primary sync / replica sync / down).
func TestProbePatroni(t *testing.T) {
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path == "/patroni" {
			w.Header().Set("Content-Type", "application/json")
			json.NewEncoder(w).Encode(map[string]string{"role": "replica", "sync_state": "sync"})
			return
		}
		w.WriteHeader(http.StatusNotFound)
	}))
	defer ts.Close()

	cfg := Config{Scope: "amane", Nodes: []Node{{Name: "n1", PatroniURL: ts.URL}}}
	cfg.Defaults()
	s := New(cfg, quietLogger())
	st, ok := s.probePatroni(context.Background(), &cfg.Nodes[0])
	if !ok {
		t.Fatal("réplica joignable devrait être rapporté ok")
	}
	if st.Role != "replica" || st.SyncState != "sync" {
		t.Fatalf("parsing Patroni incorrect: %+v", st)
	}

	// nœud qui refuse la connexion → down
	down := Node{Name: "down", PatroniURL: "http://127.0.0.1:1"}
	if _, ok := s.probePatroni(context.Background(), &down); ok {
		t.Fatal("nœud injoignable doit être rapporté down")
	}
}

// TestForceFailover : POST /failover correct (candidate, PAS de leader — sinon
// Patroni le traite comme switchover) et gestion des refus (503 → erreur).
func TestForceFailover(t *testing.T) {
	var got map[string]any
	accepted := make(chan struct{}, 1)
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost || r.URL.Path != "/failover" {
			w.WriteHeader(http.StatusMethodNotAllowed)
			return
		}
		if v := r.URL.Query().Get("force"); v != "true" {
			w.WriteHeader(http.StatusBadRequest)
			return
		}
		json.NewDecoder(r.Body).Decode(&got)
		accepted <- struct{}{}
		w.WriteHeader(http.StatusOK)
		json.NewEncoder(w).Encode(map[string]any{"changed": true})
	}))
	defer ts.Close()

	cfg := Config{Scope: "amane", Nodes: []Node{{Name: "n1", PatroniURL: ts.URL}}}
	cfg.Defaults()
	s := New(cfg, quietLogger())
	ctx, cancel := context.WithTimeout(context.Background(), 2*time.Second)
	defer cancel()

	if err := s.forceFailover(ctx, &cfg.Nodes[0], "primary", "standby"); err != nil {
		t.Fatalf("failover refusé à tort: %v", err)
	}
	select {
	case <-accepted:
	case <-time.After(2 * time.Second):
		t.Fatal("POST /failover jamais reçu")
	}
	if got["candidate"] != "standby" {
		t.Fatalf("payload failover incorrect: %+v", got)
	}
	if _, hasLeader := got["leader"]; hasLeader {
		t.Fatalf("leader ne doit pas être transmis (sinon switchover): %+v", got)
	}

	// refus : serveur répond 503 → erreur propagée
	refuse := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusServiceUnavailable)
		io.WriteString(w, "no way")
	}))
	defer refuse.Close()
	cfg2 := Config{Scope: "amane", Nodes: []Node{{Name: "n1", PatroniURL: refuse.URL}}}
	cfg2.Defaults()
	s2 := New(cfg2, quietLogger())
	if err := s2.forceFailover(context.Background(), &cfg2.Nodes[0], "a", "b"); err == nil ||
		!strings.Contains(err.Error(), "503") {
		t.Fatalf("un refus 503 doit remonter en erreur: %v", err)
	}
}
