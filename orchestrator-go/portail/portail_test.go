package portail

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"path/filepath"
	"regexp"
	"testing"
)

func TestPairsExtraitLesIPDesAutresMachines(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/api/devices/cluster-status/" || r.URL.Query().Get("token") != "jeton-1" {
			http.Error(w, "inattendu", http.StatusBadRequest)
			return
		}
		w.Write([]byte(`{"tenant_name": "PME", "node_count": 4, "active_nodes": [
			{"node_addr": "192.168.1.10:9001"}, {"node_addr": "192.168.1.20:9001"},
			{"node_addr": "192.168.1.30:2379"}, {"node_addr": ""}, {"node_addr": "192.168.1.10:9001"}]}`))
	}))
	defer srv.Close()

	ips, err := Nouveau(srv.URL+"/", "jeton-1").Pairs(context.Background(), "192.168.1.20")
	if err != nil {
		t.Fatalf("pairs: %v", err)
	}
	if len(ips) != 2 || ips[0] != "192.168.1.10" || ips[1] != "192.168.1.30" {
		t.Fatalf("pairs inattendus : %v", ips)
	}
}

func TestPairsErreurDuPortail(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		http.Error(w, `{"error": "Token invalide"}`, http.StatusForbidden)
	}))
	defer srv.Close()
	if _, err := Nouveau(srv.URL, "x").Pairs(context.Background(), ""); err == nil {
		t.Fatal("un refus du portail doit remonter en erreur")
	}
}

func TestAnnoncerEnvoieLeTypeDeMachine(t *testing.T) {
	var recu map[string]string
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		json.NewDecoder(r.Body).Decode(&recu)
		w.WriteHeader(http.StatusCreated)
	}))
	defer srv.Close()
	err := Nouveau(srv.URL, "jeton-1").Annoncer(context.Background(), Annonce{
		InstallationID: "id-1", Adresse: "192.168.1.30:2379", Hostname: "temoin", Type: "temoin",
	})
	if err != nil {
		t.Fatalf("annonce: %v", err)
	}
	if recu["tenant_token"] != "jeton-1" || recu["node_kind"] != "temoin" || recu["node_addr"] != "192.168.1.30:2379" {
		t.Fatalf("corps inattendu : %v", recu)
	}
}

func TestAnnoncerRefus(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		http.Error(w, "licence", http.StatusForbidden)
	}))
	defer srv.Close()
	if err := Nouveau(srv.URL, "x").Annoncer(context.Background(), Annonce{}); err == nil {
		t.Fatal("un refus doit remonter en erreur")
	}
}

func TestIdentifiantInstallationStable(t *testing.T) {
	chemin := filepath.Join(t.TempDir(), "temoin", "installation-id")
	a, err := IdentifiantInstallation(chemin)
	if err != nil {
		t.Fatalf("création: %v", err)
	}
	if !regexp.MustCompile(`^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$`).MatchString(a) {
		t.Fatalf("UUID v4 attendu : %q", a)
	}
	b, _ := IdentifiantInstallation(chemin)
	if a != b {
		t.Fatal("l'identifiant doit rester le même d'un lancement à l'autre")
	}
}
