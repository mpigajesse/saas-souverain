package lanceur

import (
	"os"
	"path/filepath"
	"testing"
)

func secretsEssai(t *testing.T) string {
	t.Helper()
	chemin := filepath.Join(t.TempDir(), "application.env")
	contenu := "# Généré par l'orchestrateur AMANE (amorçage) — ne pas modifier.\n" +
		"export AMANE_APP_DB_UTILISATEUR='application'\n" +
		"export AMANE_APP_DB_MOT_DE_PASSE='s3cr3t-Base64_url'\n" +
		"export AMANE_APP_DB_HOTE='amane-haproxy'\n" +
		"export AMANE_APP_DB_PORT='5000'\n" +
		"export AMANE_APP_DB_BASE='application'\n"
	if err := os.WriteFile(chemin, []byte(contenu), 0o644); err != nil {
		t.Fatal(err)
	}
	return chemin
}

func TestChargerEnv(t *testing.T) {
	vars, err := ChargerEnv(secretsEssai(t))
	if err != nil {
		t.Fatalf("lecture: %v", err)
	}
	if vars["AMANE_APP_DB_MOT_DE_PASSE"] != "s3cr3t-Base64_url" || vars["AMANE_APP_DB_PORT"] != "5000" {
		t.Fatalf("variables inattendues : %v", vars)
	}
	if _, err := ChargerEnv(filepath.Join(t.TempDir(), "absent.env")); err == nil {
		t.Fatal("fichier absent : erreur attendue")
	}
	illisible := filepath.Join(t.TempDir(), "x.env")
	os.WriteFile(illisible, []byte("sans egal\n"), 0o644)
	if _, err := ChargerEnv(illisible); err == nil {
		t.Fatal("ligne sans « = » : erreur attendue")
	}
}

func TestRendreDatabaseURL(t *testing.T) {
	secrets, _ := ChargerEnv(secretsEssai(t))
	modeles, err := DecoderModeles(`{"DATABASE_URL": "postgres://{utilisateur}:{mot_de_passe}@{hote}:{port}/{base}", "DB_HOST": "{hote}"}`)
	if err != nil {
		t.Fatalf("modèles: %v", err)
	}
	rendus, err := Rendre(modeles, secrets)
	if err != nil {
		t.Fatalf("rendu: %v", err)
	}
	if rendus["DATABASE_URL"] != "postgres://application:s3cr3t-Base64_url@amane-haproxy:5000/application" {
		t.Fatalf("DATABASE_URL inattendue : %s", rendus["DATABASE_URL"])
	}
	if rendus["DB_HOST"] != "amane-haproxy" {
		t.Fatalf("DB_HOST inattendue : %s", rendus["DB_HOST"])
	}
}

func TestRendreRefuseUnEspaceReserveInconnu(t *testing.T) {
	secrets, _ := ChargerEnv(secretsEssai(t))
	if _, err := Rendre(map[string]string{"X": "{jeton_editeur}"}, secrets); err == nil {
		t.Fatal("espace réservé inconnu : erreur attendue")
	}
	if _, err := Rendre(map[string]string{"X": "{base}"}, map[string]string{}); err == nil {
		t.Fatal("secret absent : erreur attendue")
	}
}

func TestDecoderModeles(t *testing.T) {
	if m, err := DecoderModeles("  "); err != nil || len(m) != 0 {
		t.Fatalf("modèles vides : %v, %v", m, err)
	}
	if _, err := DecoderModeles("{pas du json"); err == nil {
		t.Fatal("JSON invalide : erreur attendue")
	}
}

func TestEnvironnementRemplaceEtTrie(t *testing.T) {
	env := Environnement([]string{"PATH=/usr/bin", "DATABASE_URL=ancienne", "SANS_EGAL"}, map[string]string{"DATABASE_URL": "nouvelle"})
	if len(env) != 2 || env[0] != "DATABASE_URL=nouvelle" || env[1] != "PATH=/usr/bin" {
		t.Fatalf("environnement inattendu : %v", env)
	}
}
