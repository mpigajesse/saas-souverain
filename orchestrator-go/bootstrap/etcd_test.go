package bootstrap

import (
	"context"
	"errors"
	"io"
	"log/slog"
	"os"
	"path/filepath"
	"strings"
	"testing"
	"time"
)

var logDiscret = slog.New(slog.NewTextHandler(io.Discard, nil))

// fausseAPI simule le cluster etcd joint par chaque IP.
type fausseAPI struct {
	joignables map[string][]MembreEtcd
	ajouts     []string
}

func (f *fausseAPI) Lister(_ context.Context, ip string) ([]MembreEtcd, error) {
	m, ok := f.joignables[ip]
	if !ok {
		return nil, errors.New("injoignable")
	}
	return m, nil
}

func (f *fausseAPI) Ajouter(_ context.Context, ip, peer string) ([]MembreEtcd, error) {
	m, ok := f.joignables[ip]
	if !ok {
		return nil, errors.New("injoignable")
	}
	f.ajouts = append(f.ajouts, peer)
	m = append(m, MembreEtcd{PeerURLs: []string{peer}})
	f.joignables[ip] = m
	return m, nil
}

func options(t *testing.T, pairs ...string) Options {
	d := t.TempDir()
	return Options{
		Nom: "poste-b", IP: "192.168.1.20", Pairs: pairs, Jeton: "amane-tenant",
		DossierDonnees: filepath.Join(d, "etcd-data"), FichierConfig: filepath.Join(d, "etcd.env"),
		Delai: 50 * time.Millisecond, Pause: 10 * time.Millisecond,
	}
}

func lireConfig(t *testing.T, o Options) string {
	t.Helper()
	b, err := os.ReadFile(o.FichierConfig)
	if err != nil {
		t.Fatalf("configuration absente: %v", err)
	}
	return string(b)
}

func TestPremiereMachineCreeLeCluster(t *testing.T) {
	o := options(t)
	plan, err := PreparerEtcd(context.Background(), o, &fausseAPI{}, logDiscret)
	if err != nil || plan.Etat != "new" || plan.Cluster != "poste-b=http://192.168.1.20:2380" {
		t.Fatalf("plan inattendu : %+v, %v", plan, err)
	}
	cfg := lireConfig(t, o)
	for _, attendu := range []string{
		"export ETCD_INITIAL_CLUSTER_STATE='new'",
		"export ETCD_ADVERTISE_CLIENT_URLS='http://192.168.1.20:2379'",
		"export ETCD_INITIAL_CLUSTER_TOKEN='amane-tenant'",
		"export ETCD_GRPC_KEEPALIVE_MIN_TIME='1s'",
	} {
		if !strings.Contains(cfg, attendu) {
			t.Errorf("ligne absente : %q\n%s", attendu, cfg)
		}
	}
}

func TestSeulementSoiMemeDansLesPairs(t *testing.T) {
	plan, err := PreparerEtcd(context.Background(), options(t, "192.168.1.20", " "), &fausseAPI{}, logDiscret)
	if err != nil || plan.Etat != "new" {
		t.Fatalf("sa propre IP ne compte pas comme pair : %+v, %v", plan, err)
	}
}

func TestMachineSuivanteRejointLeCluster(t *testing.T) {
	api := &fausseAPI{joignables: map[string][]MembreEtcd{
		"192.168.1.10": {{Nom: "poste-a", PeerURLs: []string{"http://192.168.1.10:2380"}}},
	}}
	o := options(t, "192.168.1.99", "192.168.1.10")
	plan, err := PreparerEtcd(context.Background(), o, api, logDiscret)
	if err != nil {
		t.Fatalf("ajout: %v", err)
	}
	if plan.Etat != "existing" || plan.Cluster != "poste-a=http://192.168.1.10:2380,poste-b=http://192.168.1.20:2380" {
		t.Fatalf("plan inattendu : %+v", plan)
	}
	if len(api.ajouts) != 1 {
		t.Fatalf("un seul member add attendu, reçu %v", api.ajouts)
	}
	if !strings.Contains(lireConfig(t, o), "export ETCD_INITIAL_CLUSTER_STATE='existing'") {
		t.Fatal("état existing attendu dans la configuration")
	}
}

func TestDejaMembreNonRajoute(t *testing.T) {
	// Relance après un ajout réussi mais un etcd jamais démarré : pas de second member add.
	api := &fausseAPI{joignables: map[string][]MembreEtcd{
		"192.168.1.10": {
			{Nom: "poste-a", PeerURLs: []string{"http://192.168.1.10:2380"}},
			{PeerURLs: []string{"http://192.168.1.20:2380"}},
		},
	}}
	if _, err := PreparerEtcd(context.Background(), options(t, "192.168.1.10"), api, logDiscret); err != nil {
		t.Fatalf("relance: %v", err)
	}
	if len(api.ajouts) != 0 {
		t.Fatalf("aucun ajout attendu, reçu %v", api.ajouts)
	}
}

func TestPairsInjoignablesRefusDeCreerUnClusterSepare(t *testing.T) {
	o := options(t, "192.168.1.10")
	_, err := PreparerEtcd(context.Background(), o, &fausseAPI{joignables: map[string][]MembreEtcd{}}, logDiscret)
	if !errors.Is(err, ErrPairsInjoignables) {
		t.Fatalf("refus attendu, reçu %v", err)
	}
	if _, err := os.Stat(o.FichierConfig); err == nil {
		t.Fatal("aucune configuration ne doit être écrite")
	}
}

func TestNouveauClusterForce(t *testing.T) {
	o := options(t, "192.168.1.10")
	o.NouveauCluster = true
	plan, err := PreparerEtcd(context.Background(), o, &fausseAPI{}, logDiscret)
	if err != nil || plan.Etat != "new" {
		t.Fatalf("création forcée attendue : %+v, %v", plan, err)
	}
}

func TestAutreMachineEnCoursDAjout(t *testing.T) {
	api := &fausseAPI{joignables: map[string][]MembreEtcd{
		"192.168.1.10": {
			{Nom: "poste-a", PeerURLs: []string{"http://192.168.1.10:2380"}},
			{PeerURLs: []string{"http://192.168.1.30:2380"}}, // ajouté, jamais démarré
		},
	}}
	_, err := PreparerEtcd(context.Background(), options(t, "192.168.1.10"), api, logDiscret)
	if !errors.Is(err, ErrAjoutEnCours) {
		t.Fatalf("attente de l'autre machine attendue, reçu %v", err)
	}
}

func TestRedemarrageConserveLaConfiguration(t *testing.T) {
	o := options(t, "192.168.1.10")
	if err := os.MkdirAll(filepath.Join(o.DossierDonnees, "member"), 0o755); err != nil {
		t.Fatal(err)
	}
	if _, err := PreparerEtcd(context.Background(), o, &fausseAPI{}, logDiscret); err == nil {
		t.Fatal("données sans configuration : erreur attendue")
	}
	os.WriteFile(o.FichierConfig, []byte("export ETCD_NAME='poste-b'\n"), 0o644)
	plan, err := PreparerEtcd(context.Background(), o, &fausseAPI{}, logDiscret)
	if err != nil || !plan.DejaInitialise {
		t.Fatalf("redémarrage : %+v, %v", plan, err)
	}
	if lireConfig(t, o) != "export ETCD_NAME='poste-b'\n" {
		t.Fatal("la configuration d'origine doit être conservée")
	}
}

func TestOptionsInvalides(t *testing.T) {
	for _, modif := range []func(*Options){
		func(o *Options) { o.Nom = "mauvais nom" },
		func(o *Options) { o.IP = "pas-une-ip" },
		func(o *Options) { o.Jeton = "" },
	} {
		o := options(t)
		modif(&o)
		if _, err := PreparerEtcd(context.Background(), o, &fausseAPI{}, logDiscret); err == nil {
			t.Errorf("options invalides acceptées : %+v", o)
		}
	}
}

func TestEcrireEnvRefuseLesApostrophes(t *testing.T) {
	err := ecrireEnv(filepath.Join(t.TempDir(), "x.env"), [][2]string{{"A", "l'injection"}}, 0o644)
	if err == nil {
		t.Fatal("une valeur contenant une apostrophe doit être refusée")
	}
}
