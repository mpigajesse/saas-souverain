package main

import (
	"os"
	"path/filepath"
	"testing"
)

func TestLancerSansCommande(t *testing.T) {
	if code := cmdLancer([]string{"-secrets", filepath.Join(t.TempDir(), "x.env")}); code != 2 {
		t.Fatalf("code %d, 2 attendu sans commande d'origine", code)
	}
}

func TestLancerSecretsAbsents(t *testing.T) {
	code := cmdLancer([]string{"-secrets", filepath.Join(t.TempDir(), "absent.env"), "-attente", "1ms", "--", "true"})
	if code != 1 {
		t.Fatalf("code %d, 1 attendu quand les secrets n'arrivent jamais", code)
	}
}

func TestLancerModelesInvalides(t *testing.T) {
	secrets := filepath.Join(t.TempDir(), "application.env")
	os.WriteFile(secrets, []byte("export AMANE_APP_DB_BASE='application'\n"), 0o644)
	t.Setenv("AMANE_MODELES", `{"DATABASE_URL": "{inconnu}"}`)
	if code := cmdLancer([]string{"-secrets", secrets, "--", "true"}); code != 1 {
		t.Fatalf("code %d, 1 attendu pour un espace réservé inconnu", code)
	}
}

func TestCopierBinaire(t *testing.T) {
	dossier := filepath.Join(t.TempDir(), "bin")
	if err := copierBinaire(dossier); err != nil {
		t.Fatalf("copie: %v", err)
	}
	st, err := os.Stat(filepath.Join(dossier, "orchestrator"))
	if err != nil || st.Size() == 0 {
		t.Fatalf("binaire absent ou vide : %v", err)
	}
}
