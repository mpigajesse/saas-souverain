package bootstrap

import (
	"context"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/amane/orchestrator-go/consensus"
)

func TestGenererSecretsDistinctsEtComplets(t *testing.T) {
	a, err := genererSecrets()
	if err != nil || !a.complets() {
		t.Fatalf("génération: %+v, %v", a, err)
	}
	b, _ := genererSecrets()
	if a.Superutilisateur == b.Superutilisateur || a.Superutilisateur == a.Replication {
		t.Fatal("mots de passe non aléatoires")
	}
}

func TestEcrireEnvSecrets(t *testing.T) {
	chemin := filepath.Join(t.TempDir(), "secrets", "postgresql.env")
	if err := EcrireEnvSecrets(chemin, Secrets{"su", "repl", "adm", "web"}); err != nil {
		t.Fatalf("écriture: %v", err)
	}
	b, _ := os.ReadFile(chemin)
	for _, attendu := range []string{
		"export PGPASSWORD_SUPERUSER='su'",
		"export PGPASSWORD_STANDBY='repl'",
		"export AMANE_PG_PASSWORD='adm'",
		"export PGADMIN_DEFAULT_PASSWORD='web'",
	} {
		if !strings.Contains(string(b), attendu) {
			t.Errorf("ligne absente : %q", attendu)
		}
	}
}

func TestPreparerDossiers(t *testing.T) {
	d := t.TempDir()
	secrets, hap := filepath.Join(d, "secrets"), filepath.Join(d, "haproxy")
	if err := PreparerDossiers([]string{secrets}, hap, os.Getuid()); err != nil {
		t.Fatalf("préparation: %v", err)
	}
	for _, x := range []string{secrets, hap} {
		if st, err := os.Stat(x); err != nil || !st.IsDir() {
			t.Fatalf("dossier absent : %s", x)
		}
	}
}

// TestObtenirSecretsAgainstEtcd : la première machine crée les secrets, les
// suivantes lisent exactement les mêmes.
// Usage : AMANE_TEST_ETCD=localhost:2379 go test ./bootstrap/ -run Secrets -v
func TestObtenirSecretsAgainstEtcd(t *testing.T) {
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
	cli.Delete(ctx, CleSecrets)
	defer cli.Delete(ctx, CleSecrets)

	premiers, cree, err := ObtenirSecrets(ctx, cli)
	if err != nil || !cree {
		t.Fatalf("création: %v (créé=%v)", err, cree)
	}
	seconds, cree, err := ObtenirSecrets(ctx, cli)
	if err != nil || cree {
		t.Fatalf("lecture: %v (créé=%v)", err, cree)
	}
	if premiers != seconds {
		t.Fatal("la seconde machine doit obtenir les mêmes secrets")
	}

	cli.Put(ctx, CleSecrets, `{"superutilisateur": "x"}`)
	if _, _, err := ObtenirSecrets(ctx, cli); err == nil {
		t.Fatal("des secrets incomplets doivent être refusés")
	}
}
