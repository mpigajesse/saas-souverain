package synchrone

import (
	"context"
	"encoding/json"
	"errors"
	"io"
	"log/slog"
	"net/http"
	"net/http/httptest"
	"os"
	"testing"
	"time"

	clientv3 "go.etcd.io/etcd/client/v3"

	"github.com/amane/orchestrator-go/consensus"
	"github.com/amane/orchestrator-go/dcs"
)

func TestStrictAttendu(t *testing.T) {
	for n, attendu := range map[int]bool{1: false, 2: false, 3: true, 5: true} {
		if StrictAttendu(n) != attendu {
			t.Errorf("%d serveurs : strict=%v attendu", n, attendu)
		}
	}
}

// fauxPatroni simule GET/PATCH /config d'un membre.
type fauxPatroni struct {
	strict  bool
	panne   map[string]bool
	patches int
}

func (f *fauxPatroni) LireStrict(_ context.Context, url string) (bool, error) {
	if f.panne[url] {
		return false, errors.New("injoignable")
	}
	return f.strict, nil
}

func (f *fauxPatroni) EcrireStrict(_ context.Context, url string, strict bool) error {
	if f.panne[url] {
		return errors.New("injoignable")
	}
	f.strict = strict
	f.patches++
	return nil
}

var membres = []dcs.Membre{{Nom: "a", URLAPI: "http://a:8008"}, {Nom: "b", URLAPI: "http://b:8008"}}

func TestAppliquerPasseEnStrictATroisServeurs(t *testing.T) {
	api := &fauxPatroni{panne: map[string]bool{"http://a:8008": true}}
	strict, change, err := Appliquer(context.Background(), api, membres, 3)
	if err != nil || !strict || !change || !api.strict {
		t.Fatalf("strict attendu via le membre joignable : strict=%v change=%v err=%v", strict, change, err)
	}
	// Déjà au bon réglage : aucune écriture.
	if _, change, _ := Appliquer(context.Background(), api, membres, 3); change || api.patches != 1 {
		t.Fatalf("aucun PATCH attendu quand le réglage est déjà bon (patches=%d)", api.patches)
	}
}

func TestAppliquerNonStrictADeuxServeurs(t *testing.T) {
	api := &fauxPatroni{strict: true, panne: map[string]bool{}}
	if strict, change, _ := Appliquer(context.Background(), api, membres, 2); strict || !change || api.strict {
		t.Fatal("mode non strict attendu à deux serveurs")
	}
}

func TestAppliquerSansMembreJoignable(t *testing.T) {
	api := &fauxPatroni{panne: map[string]bool{"http://a:8008": true, "http://b:8008": true}}
	if _, _, err := Appliquer(context.Background(), api, membres, 3); err == nil {
		t.Fatal("erreur attendue sans membre joignable")
	}
	if _, _, err := Appliquer(context.Background(), api, nil, 3); err == nil {
		t.Fatal("erreur attendue sans membre")
	}
}

func TestClientHTTP(t *testing.T) {
	var recu map[string]bool
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		switch r.Method {
		case http.MethodGet:
			w.Write([]byte(`{"ttl": 20, "synchronous_mode": true, "synchronous_mode_strict": false}`))
		case http.MethodPatch:
			json.NewDecoder(r.Body).Decode(&recu)
			w.Write([]byte(`{}`))
		}
	}))
	defer srv.Close()
	c := ClientHTTP{HTTP: srv.Client()}
	strict, err := c.LireStrict(context.Background(), srv.URL)
	if err != nil || strict {
		t.Fatalf("lecture : %v, %v", strict, err)
	}
	if err := c.EcrireStrict(context.Background(), srv.URL, true); err != nil || !recu["synchronous_mode_strict"] {
		t.Fatalf("écriture : %v, reçu %v", err, recu)
	}
}

func TestClientHTTPErreurs(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		http.Error(w, "non", http.StatusServiceUnavailable)
	}))
	defer srv.Close()
	c := ClientHTTP{HTTP: srv.Client()}
	if _, err := c.LireStrict(context.Background(), srv.URL); err == nil {
		t.Fatal("erreur de lecture attendue")
	}
	if err := c.EcrireStrict(context.Background(), srv.URL, true); err == nil {
		t.Fatal("erreur d'écriture attendue")
	}
}

// TestGestionnaireAgainstEtcd : maximum mémorisé (hystérésis) et réglage appliqué.
// Usage : AMANE_TEST_ETCD=localhost:2379 go test ./synchrone/ -v
func TestGestionnaireAgainstEtcd(t *testing.T) {
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
	const scope = "amane-test-synchrone"
	cli.Delete(ctx, CleServeurs)
	defer cli.Delete(ctx, CleServeurs)

	for _, cas := range []struct{ observes, attendu int }{{2, 2}, {3, 3}, {2, 3}, {4, 4}} {
		n, err := NombreServeurs(ctx, cli, cas.observes)
		if err != nil || n != cas.attendu {
			t.Fatalf("observés %d : %d attendu, obtenu %d (%v)", cas.observes, cas.attendu, n, err)
		}
	}

	cli.Delete(ctx, CleServeurs)
	prefixe := dcs.PrefixeMembres(scope)
	defer cli.Delete(ctx, prefixe, clientv3.WithPrefix())
	api := &fauxPatroni{panne: map[string]bool{}}
	for i, nom := range []string{"a", "b", "c"} {
		cli.Put(ctx, prefixe+nom, `{"conn_url": "postgres://10.0.0.`+string(rune('1'+i))+`:5432/p", "api_url": "http://10.0.0.1:8008/patroni"}`)
	}
	g := Gestionnaire{Cli: cli, Scope: scope, API: api, Periode: time.Second,
		Logger: slog.New(slog.NewTextHandler(io.Discard, nil))}
	g.tick(ctx)
	if !api.strict {
		t.Fatal("trois membres : mode strict attendu")
	}
}
