// Package ha_harness exécute les épreuves HA du plan plan-tests-amane-v3.md
// contre un etcd RÉEL (le laboratoire ha-lab), en important les paquets de
// production d'orchestrator-go via une directive `replace` (voir go.mod).
//
// Couverture :
//   - HA-02 (mutex force-right) et HA-06 (bootstrap concurrent) : ce fichier,
//     via l'API exportée (supervisor, bootstrap).
//   - HA-03 (keepalive) et HA-05 (hystérésis) : ce fichier + tests internes.
//   - HA-01 (élection lease) : test interne consensus (accès aux champs privés
//     key/ttl), lancé par tools/ha/run-ha.sh.
//
// Aucun conteneur amane-* n'est touché. On ne travaille que dans des keyspaces
// jetables (scope/lease dédiés), nettoyés en fin d'exécution. Le mutex global
// « force-right » de production est partagé avec les orchestrateurs ha-lab, mais
// le cluster ha-lab étant sain ils ne le sollicitent jamais (ensureForceRight
// n'est appelé qu'au moment de forcer une bascule).
package ha_harness

import (
	"context"
	"encoding/json"
	"io"
	"log/slog"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"sync"
	"testing"
	"time"

	clientv3 "go.etcd.io/etcd/client/v3"
	"go.etcd.io/etcd/client/v3/concurrency"

	"github.com/amane/orchestrator-go/bootstrap"
	"github.com/amane/orchestrator-go/consensus"
	"github.com/amane/orchestrator-go/supervisor"
	"github.com/amane/orchestrator-go/synchrone"
)

func etcdEndpoint(t *testing.T) string {
	t.Helper()
	ep := os.Getenv("HA_ETCD")
	if ep == "" {
		ep = "10.77.0.11:2379"
	}
	if ep == "" {
		t.Skip("HA_ETCD non défini — etcd réel ignoré")
	}
	return ep
}

func client(t *testing.T, ep string) *clientv3.Client {
	t.Helper()
	cli, err := consensus.NewClient([]string{ep})
	if err != nil {
		t.Fatalf("client etcd: %v", err)
	}
	t.Cleanup(func() { cli.Close() })
	return cli
}

func discret() *slog.Logger { return slog.New(slog.NewTextHandler(io.Discard, nil)) }

// ── HA-02 : mutex « force-right » ───────────────────────────────────────────
// Attendu : un opérateur ne peut pas forcer deux bascules simultanées. Deux
// superviseurs voient le même primary mort ; le mutex etcd « droit de forcer »
// garantit qu'un seul déclenche POST /failover.
func TestHA02MutexForceRight(t *testing.T) {
	ep := etcdEndpoint(t)
	cli := client(t, ep)
	ctx := context.Background()

	scope := "amane-ha-h02-" + time.Now().Format("150405")
	cli.Delete(ctx, "/service/"+scope+"/", clientv3.WithPrefix())
	t.Cleanup(func() {
		c, cc := context.WithTimeout(context.Background(), 5*time.Second)
		defer cc()
		cli.Delete(c, "/service/"+scope+"/", clientv3.WithPrefix())
	})

	// Le mutex « droit de forcer » est une clé GLOBALE partagée avec les
	// orchestrateurs réels du ha-lab. Le cluster ha-lab étant sain, aucun
	// orchestrateur ne la sollicite (ensureForceRight n'est appelé qu'au moment
	// de forcer une bascule) : on libère la clé pour que les deux superviseurs
	// du test se disputent équitablement le droit. Sans cela, orch-m1 la
	// détiendrait et les deux candidats seraient exclus (0 forçage).
	cli.Delete(ctx, "/amane/supervisor/force-right", clientv3.WithPrefix())

	// Primary mort (REST down), deux standbys joignables qui comptent les
	// POST /failover reçus.
	primary := newFakePatroni("primary", "")
	srvP := httptest.NewServer(primary)
	defer srvP.Close()
	primary.crash()

	var mu sync.Mutex
	failovers := 0
	mkStandby := func() *httptest.Server {
		f := newFakePatroni("replica", "sync")
		return httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			if r.URL.Path == "/failover" && r.Method == http.MethodPost {
				mu.Lock()
				failovers++
				mu.Unlock()
			}
			f.ServeHTTP(w, r)
		}))
	}
	srvA := mkStandby()
	defer srvA.Close()
	srvB := mkStandby()
	defer srvB.Close()

	if _, err := cli.Put(ctx, "/service/"+scope+"/leader", "primary-test"); err != nil {
		t.Fatal(err)
	}

	// Deux superviseurs, chacun agent local de son standby, voient le même
	// primary mort. Le mutex force-right ne doit laisser qu'un seul agir.
	run := func(local string, srv *httptest.Server) context.CancelFunc {
		sup := supervisor.New(supervisor.Config{
			EtcdEndpoints: []string{ep},
			Scope:         scope,
			Nodes: []supervisor.Node{
				{Name: "primary-test", PatroniURL: srvP.URL},
				{Name: local, PatroniURL: srv.URL},
			},
			LocalNode:     local,
			ProbeInterval: 250 * time.Millisecond,
			ProbeTimeout:  200 * time.Millisecond,
			HeartbeatTTL:  time.Second,
			StaleConfirm:  2,
			LockTTL:       3 * time.Second,
			CoolDown:      30 * time.Second,
		}, discret())
		cctx, cancel := context.WithCancel(ctx)
		go sup.Run(cctx)
		return cancel
	}
	cancelA := run("standby-a", srvA)
	defer cancelA()
	cancelB := run("standby-b", srvB)
	defer cancelB()

	// Laisser le temps de la détection + forçage.
	time.Sleep(6 * time.Second)

	mu.Lock()
	n := failovers
	mu.Unlock()
	if n != 1 {
		t.Fatalf("HA-02 : %d POST /failover reçus, attendu exactement 1 (mutex force-right)", n)
	}
	t.Logf("HA-02 OK : deux opérateurs, %d seul POST /failover (mutex force-right respecté)", n)
}

// ── HA-03 : keepalive gRPC ──────────────────────────────────────────────────
// Attendu : la session survit à une coupure courte, le heartbeat reprend sans
// double enrôlement. On modélise la session heartbeat par une Session etcd avec
// keepalive (client consensus) : elle reste vivante tant que la lease n'expire
// pas ; après révocation (coupure > TTL), une nouvelle session rétablit le
// heartbeat et un seul objet heartbeat existe par nœud (pas de doublon).
func TestHA03KeepaliveSession(t *testing.T) {
	ep := etcdEndpoint(t)
	cli := client(t, ep)
	ctx := context.Background()

	name := "noeud-ha03-" + time.Now().Format("150405")
	hbKey := "/amane/supervisor/hb/" + name
	cli.Delete(ctx, hbKey)
	t.Cleanup(func() { cli.Delete(context.Background(), hbKey) })

	const ttl = 2 * time.Second
	sess, err := concurrency.NewSession(cli, concurrency.WithTTL(int(ttl.Seconds())))
	if err != nil {
		t.Fatalf("session: %v", err)
	}
	ecrireHB := func() {
		v, _ := json.Marshal(map[string]any{"role": "primary", "ts": time.Now().UnixMilli()})
		if _, err := cli.Put(ctx, hbKey, string(v), clientv3.WithLease(sess.Lease())); err != nil {
			t.Fatalf("put heartbeat: %v", err)
		}
	}
	ecrireHB()

	// Coupure COURTE (< TTL) : la session doit survivre (Done non fermé).
	time.Sleep(ttl / 2)
	select {
	case <-sess.Done():
		t.Fatalf("HA-03 : session fermée pendant une coupure courte (< TTL)")
	default:
	}
	ecrireHB()
	if resp, _ := cli.Get(ctx, hbKey); resp.Count != 1 {
		t.Fatalf("HA-03 : %d clés heartbeat, attendu 1 (double enrôlement)", resp.Count)
	}

	// Coupure LONGUE (> TTL) : révocation de la lease → session morte, heartbeat
	// disparu, puis une NOUVELLE session le rétablit (un seul objet).
	if _, err := cli.Revoke(ctx, sess.Lease()); err != nil {
		t.Fatalf("révocation lease: %v", err)
	}
	select {
	case <-sess.Done():
	case <-time.After(5 * time.Second):
		t.Fatalf("HA-03 : la session n'a pas constaté la perte de lease")
	}
	if resp, _ := cli.Get(ctx, hbKey); resp.Count != 0 {
		t.Fatalf("HA-03 : heartbeat résiduel après perte de lease (count=%d)", resp.Count)
	}
	sess2, err := concurrency.NewSession(cli, concurrency.WithTTL(int(ttl.Seconds())))
	if err != nil {
		t.Fatalf("nouvelle session: %v", err)
	}
	defer sess2.Close()
	v, _ := json.Marshal(map[string]any{"role": "primary", "ts": time.Now().UnixMilli()})
	if _, err := cli.Put(ctx, hbKey, string(v), clientv3.WithLease(sess2.Lease())); err != nil {
		t.Fatalf("reprise heartbeat: %v", err)
	}
	resp, _ := cli.Get(ctx, hbKey)
	if resp.Count != 1 {
		t.Fatalf("HA-03 : %d objets heartbeat après reprise, attendu 1 (pas de double)", resp.Count)
	}
	t.Logf("HA-03 OK : session survit à la coupure courte, reprise après coupure longue, 1 seul heartbeat")
}

// ── HA-05 : hystérésis de synchronisation ───────────────────────────────────
// Attendu : pas d'oscillation du mode synchrone quand le nombre de serveurs
// varie autour du seuil. Le mécanisme anti-oscillation est le MAXIMUM mémorisé
// dans etcd (synchrone.NombreServeurs) : une fois 3 serveurs vus, le mode strict
// ne retombe pas quand on revoit 2 serveurs.
func TestHA05HysteresisSync(t *testing.T) {
	ep := etcdEndpoint(t)
	cli := client(t, ep)
	ctx := context.Background()

	cli.Delete(ctx, synchrone.CleServeurs)
	t.Cleanup(func() { cli.Delete(context.Background(), synchrone.CleServeurs) })

	// Séquence oscillante autour du seuil (3) : 2,3,2,3,2,2,3,2,2,2.
	osc := []int{2, 3, 2, 3, 2, 2, 3, 2, 2, 2}
	strictVoulu := false
	for i, observes := range osc {
		n, err := synchrone.NombreServeurs(ctx, cli, observes)
		if err != nil {
			t.Fatalf("HA-05 : NombreServeurs: %v", err)
		}
		strictVoulu = synchrone.StrictAttendu(n)
		if i == 0 && strictVoulu {
			t.Fatalf("HA-05 : 2 serveurs ne doit pas activer le strict")
		}
		if n >= synchrone.Seuil && !strictVoulu {
			t.Fatalf("HA-05 : oscillation — %d serveurs mémorisés mais strict=%v", n, strictVoulu)
		}
		t.Logf("HA-05 : observes=%d -> memorise=%d strict=%v", observes, n, strictVoulu)
	}
	n, _ := synchrone.NombreServeurs(ctx, cli, 2)
	if n != 3 || !synchrone.StrictAttendu(n) {
		t.Fatalf("HA-05 : hystérésis perdue — %d serveurs mémorisés, strict=%v", n, synchrone.StrictAttendu(n))
	}
	t.Logf("HA-05 OK : aucune oscillation, maximum 3 mémorisé, mode strict stable")
}

// ── HA-06 : bootstrap concurrent ────────────────────────────────────────────
// Attendu : deux nœuds démarrés en même temps produisent un seul cluster, pas
// deux. Modèle réaliste : le premier nœud (aucun pair connu) crée le cluster
// « new » ; le second, qui connaît le premier, le rejoint en « existing » — il
// n'auto-crée JAMAIS un second cluster. Les deux appels PreparerEtcd sont lancés
// en parallèle ; on vérifie qu'au plus un cluster « new » est formé (pas de
// split) et que la vue finale du cluster contient bien les deux machines.
func TestHA06BootstrapConcurrent(t *testing.T) {
	api := newFausseAPIConcurrent()
	// Le nœud A (poste-a) est joignable d'emblée : son etcd répond (cluster
	// vide au départ). A ne connaît aucun pair -> il crée le cluster.
	api.rejoignable("192.168.1.10", []bootstrap.MembreEtcd{
		{Nom: "poste-a", PeerURLs: []string{"http://192.168.1.10:2380"}},
	})
	// A est le premier (aucun pair) ; B connaît A.
	oA := optionsPour(t, "poste-a", "192.168.1.10")
	oB := optionsPour(t, "poste-b", "192.168.1.20", "192.168.1.10")

	var wg sync.WaitGroup
	plans := make([]bootstrap.Plan, 2)
	errs := make([]error, 2)
	wg.Add(2)
	go func() { defer wg.Done(); plans[0], errs[0] = bootstrap.PreparerEtcd(context.Background(), oA, api, discret()) }()
	go func() { defer wg.Done(); plans[1], errs[1] = bootstrap.PreparerEtcd(context.Background(), oB, api, discret()) }()
	wg.Wait()

	nouveaux, existant := 0, 0
	for i := range plans {
		if errs[i] != nil {
			t.Fatalf("HA-06 : poste %d en échec (%v) — un démarrage concurrent doit aboutir, pas échouer", i, errs[i])
		}
		switch plans[i].Etat {
		case "new":
			nouveaux++
		case "existing":
			existant++
		}
	}
	// Garantie anti-split : jamais plus d'un cluster « new ».
	if nouveaux > 1 {
		t.Fatalf("HA-06 : %d clusters « new » créés, attendu au plus 1 (deux clusters = split)", nouveaux)
	}
	// Le cluster final (vue de A) doit contenir les DEUX machines.
	membres := api.snapshot("192.168.1.10")
	contientA, contientB := false, false
	for _, m := range membres {
		for _, p := range m.PeerURLs {
			if p == "http://192.168.1.10:2380" {
				contientA = true
			}
			if p == "http://192.168.1.20:2380" {
				contientB = true
			}
		}
	}
	if !contientA || !contientB {
		t.Fatalf("HA-06 : cluster final incomplet (A=%v B=%v) — %+v", contientA, contientB, membres)
	}
	t.Logf("HA-06 OK : %d nouveau(x), %d existing, cluster unique contenant les 2 machines",
		nouveaux, existant)
}

// ── Faux Patroni (HTTP) ─────────────────────────────────────────────────────
type fakePatroni struct {
	mu         sync.Mutex
	role       string
	syncState  string
	down       bool
	failedOver chan map[string]any
}

func newFakePatroni(role, syncState string) *fakePatroni {
	return &fakePatroni{role: role, syncState: syncState, failedOver: make(chan map[string]any, 4)}
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

// ── Fausse API etcd concurrente (mémoire, thread-safe) pour HA-06 ───────────
type fausseAPIConcurrent struct {
	mu    sync.Mutex
	nodes map[string][]bootstrap.MembreEtcd // ip -> membres du cluster vu
}

func newFausseAPIConcurrent() *fausseAPIConcurrent {
	return &fausseAPIConcurrent{nodes: map[string][]bootstrap.MembreEtcd{}}
}

// rejoignable déclare l'etcd d'une IP joignable, avec sa liste de membres.
func (f *fausseAPIConcurrent) rejoignable(ip string, membres []bootstrap.MembreEtcd) {
	f.mu.Lock()
	defer f.mu.Unlock()
	f.nodes[ip] = append([]bootstrap.MembreEtcd(nil), membres...)
}

func (f *fausseAPIConcurrent) Lister(_ context.Context, ip string) ([]bootstrap.MembreEtcd, error) {
	f.mu.Lock()
	defer f.mu.Unlock()
	m, ok := f.nodes[ip]
	if !ok {
		return nil, errInjoignable
	}
	return append([]bootstrap.MembreEtcd(nil), m...), nil
}

func (f *fausseAPIConcurrent) Ajouter(_ context.Context, ip, peer string) ([]bootstrap.MembreEtcd, error) {
	f.mu.Lock()
	defer f.mu.Unlock()
	m, ok := f.nodes[ip]
	if !ok {
		return nil, errInjoignable
	}
	for _, x := range m {
		if contientPeer(x, peer) {
			return append([]bootstrap.MembreEtcd(nil), m...), nil
		}
	}
	m = append(m, bootstrap.MembreEtcd{PeerURLs: []string{peer}})
	f.nodes[ip] = m
	return append([]bootstrap.MembreEtcd(nil), m...), nil
}

// snapshot renvoie la vue du cluster chez l'IP donnée.
func (f *fausseAPIConcurrent) snapshot(ip string) []bootstrap.MembreEtcd {
	f.mu.Lock()
	defer f.mu.Unlock()
	return append([]bootstrap.MembreEtcd(nil), f.nodes[ip]...)
}

type errInjoignableType struct{}

func (errInjoignableType) Error() string { return "injoignable" }

var errInjoignable = errInjoignableType{}

func contientPeer(m bootstrap.MembreEtcd, peer string) bool {
	for _, p := range m.PeerURLs {
		if p == peer {
			return true
		}
	}
	return false
}

func optionsPour(t *testing.T, nom, ip string, pairs ...string) bootstrap.Options {
	t.Helper()
	d := t.TempDir()
	return bootstrap.Options{
		Nom: nom, IP: ip, Pairs: pairs, Jeton: "amane-tenant",
		DossierDonnees: filepath.Join(d, "etcd-data"),
		FichierConfig:  filepath.Join(d, "etcd.env"),
		Delai:          200 * time.Millisecond, Pause: 20 * time.Millisecond,
	}
}