package dcs

import (
	"context"
	"os"
	"testing"

	clientv3 "go.etcd.io/etcd/client/v3"

	"github.com/amane/orchestrator-go/consensus"
)

func TestDecoderMembre(t *testing.T) {
	m, err := DecoderMembre("pg-m1", []byte(`{"conn_url": "postgres://10.77.0.12:5432/postgres",
		"api_url": "http://10.77.0.12:8008/patroni", "role": "primary", "state": "running"}`))
	if err != nil {
		t.Fatalf("décodage: %v", err)
	}
	if m.Nom != "pg-m1" || m.HotePG != "10.77.0.12:5432" || m.URLAPI != "http://10.77.0.12:8008" {
		t.Fatalf("membre inattendu : %+v", m)
	}
}

func TestDecoderMembrePortParDefaut(t *testing.T) {
	m, err := DecoderMembre("a", []byte(`{"conn_url": "postgres://10.0.0.1/postgres", "api_url": "http://10.0.0.1:8008/patroni"}`))
	if err != nil || m.HotePG != "10.0.0.1:5432" {
		t.Fatalf("port PostgreSQL par défaut non appliqué : %+v, %v", m, err)
	}
}

func TestDecoderMembreRejets(t *testing.T) {
	cas := map[string]struct {
		nom    string
		valeur string
	}{
		"json invalide":     {"a", `{`},
		"conn_url absente":  {"a", `{"api_url": "http://h:8008/patroni"}`},
		"api_url sans hôte": {"a", `{"conn_url": "postgres://h:5432/p", "api_url": "/patroni"}`},
		"nom vide":          {"", `{"conn_url": "postgres://h:5432/p", "api_url": "http://h:8008/patroni"}`},
		"nom avec chemin":   {"a/b", `{"conn_url": "postgres://h:5432/p", "api_url": "http://h:8008/patroni"}`},
	}
	for nom, c := range cas {
		if _, err := DecoderMembre(c.nom, []byte(c.valeur)); err == nil {
			t.Errorf("%s : erreur attendue", nom)
		}
	}
}

// TestMembresAgainstEtcd : lecture réelle, entrée illisible ignorée, tri par nom.
// Usage : AMANE_TEST_ETCD=localhost:2379 go test ./dcs/ -v
func TestMembresAgainstEtcd(t *testing.T) {
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
	const scope = "amane-test-dcs"
	cli.Delete(ctx, PrefixeMembres(scope), clientv3.WithPrefix())
	defer cli.Delete(ctx, PrefixeMembres(scope), clientv3.WithPrefix())

	cli.Put(ctx, PrefixeMembres(scope)+"pg-b", `{"conn_url": "postgres://10.0.0.2:5432/postgres", "api_url": "http://10.0.0.2:8008/patroni"}`)
	cli.Put(ctx, PrefixeMembres(scope)+"pg-a", `{"conn_url": "postgres://10.0.0.1:5432/postgres", "api_url": "http://10.0.0.1:8008/patroni"}`)
	cli.Put(ctx, PrefixeMembres(scope)+"pg-casse", `pas du json`)

	membres, err := Membres(ctx, cli, scope)
	if err != nil {
		t.Fatalf("lecture: %v", err)
	}
	if len(membres) != 2 || membres[0].Nom != "pg-a" || membres[1].Nom != "pg-b" {
		t.Fatalf("membres inattendus : %+v", membres)
	}

	cleLeader := "/service/" + scope + "/leader"
	defer cli.Delete(ctx, cleLeader)
	cli.Delete(ctx, cleLeader)
	if _, err := Primaire(ctx, cli, scope); err == nil {
		t.Fatal("sans leader : erreur attendue")
	}
	cli.Put(ctx, cleLeader, "pg-b")
	p, err := Primaire(ctx, cli, scope)
	if err != nil || p.HotePG != "10.0.0.2:5432" {
		t.Fatalf("primaire inattendu : %+v, %v", p, err)
	}
	cli.Put(ctx, cleLeader, "pg-inconnu")
	if _, err := Primaire(ctx, cli, scope); err == nil {
		t.Fatal("leader absent des membres : erreur attendue")
	}
}
