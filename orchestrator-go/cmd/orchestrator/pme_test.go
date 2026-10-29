package main

import (
	"context"
	"io"
	"log/slog"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"strconv"
	"strings"
	"sync/atomic"
	"testing"
	"time"

	"github.com/amane/orchestrator-go/bootstrap"
	"github.com/amane/orchestrator-go/consensus"
	"github.com/amane/orchestrator-go/portail"
)

var logDiscret = slog.New(slog.NewTextHandler(io.Discard, nil))

func TestJetonClusterStableEtOpaque(t *testing.T) {
	a := jetonCluster("6f1c2d3e-0000-4000-8000-000000000001")
	if a != jetonCluster("6f1c2d3e-0000-4000-8000-000000000001") {
		t.Fatal("le jeton de cluster doit être identique sur toutes les machines")
	}
	if a == jetonCluster("autre-tenant") || strings.Contains(a, "6f1c2d3e") {
		t.Fatalf("jeton de cluster non opaque ou non distinctif : %q", a)
	}
}

func TestParsePatroniNodes(t *testing.T) {
	nodes, err := parsePatroniNodes("pg-a@http://10.0.0.1:8008, pg-b@http://10.0.0.2:8008")
	if err != nil || len(nodes) != 2 || nodes[1].PatroniURL != "http://10.0.0.2:8008" {
		t.Fatalf("analyse : %+v, %v", nodes, err)
	}
	for _, mauvais := range []string{"", "sans-arobase", "@http://h:8008"} {
		if _, err := parsePatroniNodes(mauvais); err == nil {
			t.Errorf("%q aurait dû être refusé", mauvais)
		}
	}
}

func argsInit(t *testing.T, extra ...string) ([]string, string) {
	d := t.TempDir()
	config := filepath.Join(d, "etcd", "etcd.env")
	args := []string{
		"-nom", "poste-a", "-ip", "192.168.1.10", "-jeton", "jeton-1",
		"-donnees-etcd", filepath.Join(d, "etcd-data"), "-config-etcd", config,
		"-dossier-secrets", filepath.Join(d, "secrets"), "-dossier-haproxy", filepath.Join(d, "haproxy"),
		"-uid-orchestrateur", strconv.Itoa(os.Getuid()),
	}
	return append(args, extra...), config
}

func TestCmdInitPremiereMachine(t *testing.T) {
	args, config := argsInit(t)
	if code := cmdInit(args); code != 0 {
		t.Fatalf("code de sortie %d", code)
	}
	b, err := os.ReadFile(config)
	if err != nil || !strings.Contains(string(b), "ETCD_INITIAL_CLUSTER_STATE='new'") {
		t.Fatalf("configuration inattendue : %s, %v", b, err)
	}
}

func TestCmdInitPairsDuPortailInjoignables(t *testing.T) {
	// Le portail annonce une autre machine, qui ne répond pas : refus.
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Write([]byte(`{"active_nodes": [{"node_addr": "127.0.0.1:9001"}]}`))
	}))
	defer srv.Close()
	args, config := argsInit(t, "-portail", srv.URL, "-delai", "1ms")
	if code := cmdInit(args); code == 0 {
		t.Fatal("l'amorçage doit échouer plutôt que créer un cluster séparé")
	}
	if _, err := os.Stat(config); err == nil {
		t.Fatal("aucune configuration etcd ne doit être écrite")
	}
}

func TestSignalerTemoin(t *testing.T) {
	var appels atomic.Int32
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		appels.Add(1)
		w.WriteHeader(http.StatusOK)
	}))
	defer srv.Close()
	ctx, cancel := context.WithTimeout(context.Background(), 120*time.Millisecond)
	defer cancel()
	signalerTemoin(ctx, portail.Nouveau(srv.URL, "jeton-1"), portail.Annonce{Type: "temoin"}, 20*time.Millisecond, logDiscret)
	if appels.Load() < 2 {
		t.Fatalf("signalements répétés attendus, reçu %d", appels.Load())
	}
}

// TestProduireSecretsAgainstEtcd : le fichier de secrets est écrit à partir
// des secrets du cluster. Usage : AMANE_TEST_ETCD=localhost:2379 go test ./cmd/orchestrator/ -v
func TestProduireSecretsAgainstEtcd(t *testing.T) {
	endpoints := os.Getenv("AMANE_TEST_ETCD")
	if endpoints == "" {
		t.Skip("AMANE_TEST_ETCD non défini — intégration etcd réelle ignorée")
	}
	cli, err := consensus.NewClient([]string{endpoints})
	if err != nil {
		t.Fatalf("client etcd: %v", err)
	}
	defer cli.Close()
	cli.Delete(context.Background(), bootstrap.CleSecrets)
	defer cli.Delete(context.Background(), bootstrap.CleSecrets)

	chemin := filepath.Join(t.TempDir(), "postgresql.env")
	produireSecrets(context.Background(), cli, chemin, logDiscret)
	b, err := os.ReadFile(chemin)
	if err != nil || !strings.Contains(string(b), "export PGPASSWORD_SUPERUSER=") {
		t.Fatalf("fichier de secrets inattendu : %s, %v", b, err)
	}
}
