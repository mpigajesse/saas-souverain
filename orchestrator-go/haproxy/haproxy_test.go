package haproxy

import (
	"bufio"
	"context"
	"io"
	"log/slog"
	"net"
	"os"
	"path/filepath"
	"runtime"
	"strings"
	"testing"

	"github.com/amane/orchestrator-go/dcs"
)

func TestRendreDeuxMembres(t *testing.T) {
	cfg := Rendre([]dcs.Membre{
		{Nom: "pg-m1", HotePG: "10.0.0.1:5432", URLAPI: "http://10.0.0.1:8008"},
		{Nom: "pg-m2", HotePG: "10.0.0.2:5432", URLAPI: "http://10.0.0.2:8009"},
	}, 5000)
	for _, attendu := range []string{
		"bind *:5000",
		"option httpchk GET /primary",
		"on-marked-down shutdown-sessions",
		"server pg-m1 10.0.0.1:5432 check port 8008",
		"server pg-m2 10.0.0.2:5432 check port 8009",
	} {
		if !strings.Contains(cfg, attendu) {
			t.Errorf("ligne absente : %q\n%s", attendu, cfg)
		}
	}
}

func TestRendreSansMembreEtNomAssaini(t *testing.T) {
	vide := Rendre(nil, 5000)
	if strings.Contains(vide, "\n    server ") {
		t.Fatalf("aucun serveur attendu :\n%s", vide)
	}
	cfg := Rendre([]dcs.Membre{{Nom: "poste de Léa", HotePG: "10.0.0.3:5432", URLAPI: "http://10.0.0.3:8008"}}, 5000)
	if !strings.Contains(cfg, "server poste_de_L_a 10.0.0.3:5432") {
		t.Fatalf("nom non assaini :\n%s", cfg)
	}
}

func TestRendreDeterministe(t *testing.T) {
	m := []dcs.Membre{{Nom: "a", HotePG: "10.0.0.1:5432", URLAPI: "http://10.0.0.1:8008"}}
	if Rendre(m, 5000) != Rendre(m, 5000) {
		t.Fatal("deux rendus identiques attendus (sinon rechargements inutiles)")
	}
}

// Un faux processus maître HAProxy sur socket unix : vérifie que la commande
// « reload » est bien envoyée, puis rend la réponse du maître.
func fauxMaitre(t *testing.T, reponse string) (string, <-chan string) {
	t.Helper()
	socket := filepath.Join(t.TempDir(), SocketMaitre)
	l, err := net.Listen("unix", socket)
	if err != nil {
		if runtime.GOOS == "windows" {
			t.Skipf("socket unix indisponible sur ce Windows : %v", err)
		}
		t.Fatalf("écoute: %v", err)
	}
	t.Cleanup(func() { l.Close() })
	recu := make(chan string, 1)
	go func() {
		c, err := l.Accept()
		if err != nil {
			return
		}
		defer c.Close()
		ligne, _ := bufio.NewReader(c).ReadString('\n')
		recu <- ligne
		io.WriteString(c, reponse)
	}()
	return socket, recu
}

func TestRechargerEnvoieReload(t *testing.T) {
	socket, recu := fauxMaitre(t, "Success=1\n")
	if err := Recharger(context.Background(), socket); err != nil {
		t.Fatalf("rechargement: %v", err)
	}
	if ligne := <-recu; ligne != "reload\n" {
		t.Fatalf("commande inattendue : %q", ligne)
	}
}

func TestRechargerConfigurationRefusee(t *testing.T) {
	socket, _ := fauxMaitre(t, "Success=0\n--\n[ALERT] parsing error\n")
	if err := Recharger(context.Background(), socket); err == nil {
		t.Fatal("un refus de HAProxy doit remonter en erreur")
	}
}

func TestRechargerSansHAProxy(t *testing.T) {
	if err := Recharger(context.Background(), filepath.Join(t.TempDir(), "absent.sock")); err == nil {
		t.Fatal("erreur attendue sans socket")
	}
}

func TestAppliquerEcritPuisRecharge(t *testing.T) {
	socket, recu := fauxMaitre(t, "Success=1\n")
	dossier := filepath.Dir(socket)
	g := NouveauGestionnaire(nil, "s", dossier, 5000, slog.New(slog.NewTextHandler(io.Discard, nil)))

	// Première écriture : HAProxy pas encore démarré, aucun rechargement.
	if err := g.appliquer(context.Background(), nil); err != nil {
		t.Fatalf("première écriture: %v", err)
	}
	if _, err := os.Stat(filepath.Join(dossier, FichierConfig)); err != nil {
		t.Fatalf("configuration absente: %v", err)
	}
	// Membre ajouté : réécriture et rechargement.
	m := []dcs.Membre{{Nom: "pg-m1", HotePG: "10.0.0.1:5432", URLAPI: "http://10.0.0.1:8008"}}
	if err := g.appliquer(context.Background(), m); err != nil {
		t.Fatalf("mise à jour: %v", err)
	}
	if ligne := <-recu; ligne != "reload\n" {
		t.Fatalf("rechargement attendu, reçu %q", ligne)
	}
	contenu, _ := os.ReadFile(filepath.Join(dossier, FichierConfig))
	if !strings.Contains(string(contenu), "server pg-m1") {
		t.Fatalf("serveur absent de la configuration :\n%s", contenu)
	}
	// Même configuration : rien à faire.
	if err := g.appliquer(context.Background(), m); err != nil {
		t.Fatalf("configuration inchangée: %v", err)
	}
}
