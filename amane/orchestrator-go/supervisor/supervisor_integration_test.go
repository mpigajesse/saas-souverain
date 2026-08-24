package supervisor

import (
	"context"
	"encoding/json"
	"io"
	"log/slog"
	"net/http"
	"net/http/httptest"
	"os"
	"sync"
	"testing"
	"time"

	clientv3 "go.etcd.io/etcd/client/v3"
	"go.etcd.io/etcd/client/v3/concurrency"

	"github.com/amane/orchestrator-go/consensus"
)

// fakePatroni émule un nœud Patroni : GET /patroni piloté par le test et trace
// des POST /failover reçus.
type fakePatroni struct {
	mu         sync.Mutex
	role       string
	syncState  string
	down       bool
	failedOver chan map[string]any
}

func newFakePatroni(role, syncState string) *fakePatroni {
	return &fakePatroni{
		role:       role,
		syncState:  syncState,
		failedOver: make(chan map[string]any, 4),
	}
}

func (f *fakePatroni) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	if r.URL.Path == "/patroni" {
		f.mu.Lock()
		defer f.mu.Unlock()
		if f.down {
			w.WriteHeader(http.StatusServiceUnavailable)
			return
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(map[string]string{"role": f.role, "sync_state": f.syncState})
		return
	}
	if r.URL.Path == "/failover" && r.Method == http.MethodPost {
		var m map[string]any
		json.NewDecoder(r.Body).Decode(&m)
		select {
		case f.failedOver <- m:
		default:
		}
		w.WriteHeader(http.StatusOK)
		json.NewEncoder(w).Encode(map[string]any{"changed": true})
		return
	}
	w.WriteHeader(http.StatusNotFound)
}

func (f *fakePatroni) crash() {
	f.mu.Lock()
	f.down = true
	f.mu.Unlock()
}

// TestSupervisorAgainstEtcd — intégration contre un etcd RÉEL (stack docker).
// Usage : AMANE_TEST_ETCD=localhost:2379 go test ./supervisor/ -run 'SupervisorAgainstEtcd' -v
// Le superviseur se joue ici SEUL (single-host) : il bat le cœur de chaque nœud
// tant que son REST répond, puis détecte le SIGKILL simulé (REST down → plus de
// heartbeat) et force POST /failover vers le candidat en < 5 s.
func TestSupervisorAgainstEtcd(t *testing.T) {
	endpoints := os.Getenv("AMANE_TEST_ETCD")
	if endpoints == "" {
		t.Skip("AMANE_TEST_ETCD non défini — intégration etcd réelle ignorée")
	}
	const scope = "amane-test-supervisor"

	cli, err := consensus.NewClient([]string{endpoints})
	if err != nil {
		t.Fatalf("client etcd: %v", err)
	}
	defer cli.Close()

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	// Nettoyage d'un éventuel état résiduel puis keyspace propre.
	cli.Delete(ctx, supervisorKey, clientv3.WithPrefix())
	cli.Delete(ctx, "/service/"+scope+"/", clientv3.WithPrefix())

	primary := newFakePatroni("primary", "")
	standby := newFakePatroni("replica", "sync")
	srvP := httptest.NewServer(primary)
	srvS := httptest.NewServer(standby)
	defer srvP.Close()
	defer srvS.Close()

	if _, err := cli.Put(ctx, "/service/"+scope+"/leader", "primary-test"); err != nil {
		t.Fatal(err)
	}

	logger := slog.New(slog.NewTextHandler(io.Discard, nil))
	sup := New(Config{
		EtcdEndpoints: []string{endpoints},
		Scope:         scope,
		Nodes: []Node{
			{Name: "primary-test", PatroniURL: srvP.URL},
			{Name: "standby-test", PatroniURL: srvS.URL},
		},
		ProbeInterval: 250 * time.Millisecond,
		ProbeTimeout:  200 * time.Millisecond,
		HeartbeatTTL:  time.Second,
		StaleConfirm:  2,
		LockTTL:       3 * time.Second,
		CoolDown:      30 * time.Second,
	}, logger)
	go func() {
		if err := sup.Run(ctx); err != nil {
			t.Errorf("superviseur: %v", err)
		}
	}()

	// Phase 1 : sain — aucun forçage pendant ~2 s (heartbeats publiés).
	select {
	case m := <-standby.failedOver:
		t.Fatalf("failover non attendu en phase saine: %+v", m)
	case <-time.After(2 * time.Second):
	}

	// Phase 2 : SIGKILL simulé du primary (REST down → son heartbeat s'arrête).
	crashAt := time.Now()
	primary.crash()

	select {
	case m := <-standby.failedOver:
		if m["candidate"] != "standby-test" {
			t.Fatalf("payload failover incorrect: %+v", m)
		}
		// Le lock Patroni doit avoir été libéré (clé leader supprimée)
		// pour que le candidat promeuve sans attendre la lease.
		lresp, err := cli.Get(context.Background(), "/service/"+scope+"/leader")
		if err != nil {
			t.Fatalf("lecture leader key: %v", err)
		}
		if lresp.Count != 0 {
			t.Fatalf("leader key non libérée alors que le nœud est mort: %v", lresp.Kvs)
		}
		elapsed := time.Since(crashAt)
		if elapsed >= 5*time.Second {
			t.Fatalf("failover forcé hors cible < 5 s: %v", elapsed)
		}
		t.Logf("failover forcé en %v (< 5 s atteint), candidate=standby-test", elapsed)
	case <-time.After(7 * time.Second):
		t.Fatal("POST /failover jamais reçu après crash simulé")
	}

	// Nettoyage (ctx dédié, indépendant de l'annulation du superviseur).
	cancel()
	clean, cclean := context.WithTimeout(context.Background(), 5*time.Second)
	defer cclean()
	cli.Delete(clean, supervisorKey, clientv3.WithPrefix())
	cli.Delete(clean, "/service/"+scope+"/", clientv3.WithPrefix())
}

// TestSupervisorPartitionGuardAgainstEtcd — intégration (etcd réel) de la garde
// anti-partition : le primary est INJOIGNABLE en REST (simulation de la coupure)
// MAIS son agent local reste vivant et publie son heartbeat frais (role primary).
// Le superviseur ne doit JAMAIS forcer POST /failover, même longtemps après —
// Patroni reste l'autorité de fencing pour trancher la vraie partition.
// Usage : AMANE_TEST_ETCD=localhost:2379 go test ./supervisor/ -run 'PartitionGuard' -v
func TestSupervisorPartitionGuardAgainstEtcd(t *testing.T) {
	endpoints := os.Getenv("AMANE_TEST_ETCD")
	if endpoints == "" {
		t.Skip("AMANE_TEST_ETCD non défini — intégration etcd réelle ignorée")
	}
	const scope = "amane-test-supervisor-guard"

	cli, err := consensus.NewClient([]string{endpoints})
	if err != nil {
		t.Fatalf("client etcd: %v", err)
	}
	defer cli.Close()

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	cli.Delete(ctx, supervisorKey, clientv3.WithPrefix())
	cli.Delete(ctx, "/service/"+scope+"/", clientv3.WithPrefix())

	primary := newFakePatroni("primary", "")
	standby := newFakePatroni("replica", "sync")
	srvP := httptest.NewServer(primary)
	srvS := httptest.NewServer(standby)
	defer srvP.Close()
	defer srvS.Close()

	if _, err := cli.Put(ctx, "/service/"+scope+"/leader", "primary-test"); err != nil {
		t.Fatal(err)
	}

	// L'agent superviseur du nœud PRIMARY (inatteignable par le reste du réseau)
	// continue de battre le cœur : sa lease etcd reste vivante et la clé de
	// heartbeat reste fraîche — exactement ce qui se passe pendant une partition
	// où seul le lien REST est coupé.
	agentSess, err := concurrency.NewSession(cli, concurrency.WithTTL(3))
	if err != nil {
		t.Fatalf("session agent primary: %v", err)
	}
	defer agentSess.Close()
	hbVal, err := json.Marshal(heartbeat{Role: "primary", TS: time.Now().UnixMilli()})
	if err != nil {
		t.Fatal(err)
	}
	if _, err := cli.Put(ctx, hbKey("primary-test"), string(hbVal), clientv3.WithLease(agentSess.Lease())); err != nil {
		t.Fatal(err)
	}
	// Rafraîchit le timestamp du heartbeat à chaque battement (2×/s).
	go func() {
		tk := time.NewTicker(500 * time.Millisecond)
		defer tk.Stop()
		for {
			select {
			case <-ctx.Done():
				return
			case now := <-tk.C:
				v, _ := json.Marshal(heartbeat{Role: "primary", TS: now.UnixMilli()})
				cli.Put(ctx, hbKey("primary-test"), string(v), clientv3.WithLease(agentSess.Lease()))
			}
		}
	}()

	logger := slog.New(slog.NewTextHandler(io.Discard, nil))
	sup := New(Config{
		EtcdEndpoints: []string{endpoints},
		Scope:         scope,
		Nodes: []Node{
			{Name: "primary-test", PatroniURL: srvP.URL},
			{Name: "standby-test", PatroniURL: srvS.URL},
		},
		// Ce superviseur incarne l'agent du nœud STANDBY (déploiement réel :
		// un superviseur par nœud). Il sonde le primary en REST mais ne publie
		// le heartbeat QUE pour son propre nœud — le heartbeat du primary
		// vient exclusivement de SON agent (goroutine plus haut).
		LocalNode:     "standby-test",
		ProbeInterval: 250 * time.Millisecond,
		ProbeTimeout:  200 * time.Millisecond,
		HeartbeatTTL:  time.Second,
		StaleConfirm:  2,
		LockTTL:       3 * time.Second,
		CoolDown:      30 * time.Second,
	}, logger)
	go func() {
		if err := sup.Run(ctx); err != nil {
			t.Errorf("superviseur: %v", err)
		}
	}()

	// Phase 1 : sain — aucun forçage.
	select {
	case m := <-standby.failedOver:
		t.Fatalf("failover non attendu en phase saine: %+v", m)
	case <-time.After(2 * time.Second):
	}

	// Phase 2 : partition réseau — le REST du primary devient injoignable pour
	// CE superviseur, mais son agent continue de publier un heartbeat frais.
	primary.crash()

	// La garde doit tenir : on vérifie qu'aucun POST /failover n'arrive pendant
	// une longue fenêtre (bien au-delà de StaleConfirm ticks et de la lease).
	select {
	case m := <-standby.failedOver:
		t.Fatalf("partition (heartbeat frais) ne doit JAMAIS déclencher un forçage: %+v", m)
	case <-time.After(6 * time.Second):
	}

	// Le lock Patroni doit être INTACT : seul Patroni peut promouvoir ici.
	resp, err := cli.Get(context.Background(), "/service/"+scope+"/leader")
	if err != nil {
		t.Fatalf("lecture leader key: %v", err)
	}
	if resp.Count != 1 {
		t.Fatalf("leader key non préservée alors que le primary n'est pas mort (: %d)", resp.Count)
	}
	t.Log("garde anti-partition OK : aucun forçage malgré REST down (heartbeat frais), lock Patroni intact")

	cancel()
	clean, cclean := context.WithTimeout(context.Background(), 5*time.Second)
	defer cclean()
	cli.Delete(clean, supervisorKey, clientv3.WithPrefix())
	cli.Delete(clean, "/service/"+scope+"/", clientv3.WithPrefix())
}
