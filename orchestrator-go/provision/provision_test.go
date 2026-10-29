package provision

import (
	"context"
	"errors"
	"net/url"
	"os"
	"testing"

	"github.com/jackc/pgx/v5/pgconn"
)

func TestDSNSuperutilisateur(t *testing.T) {
	dsn := DSNSuperutilisateur("10.0.0.1:5432", "a-b_C")
	u, err := url.Parse(dsn)
	if err != nil {
		t.Fatalf("DSN illisible: %v", err)
	}
	mdp, _ := u.User.Password()
	if u.Host != "10.0.0.1:5432" || u.User.Username() != "postgres" || mdp != "a-b_C" || u.Path != "/postgres" {
		t.Fatalf("DSN inattendu : %s", dsn)
	}
}

func TestGarantirRefuseUnMotDePasseNonSur(t *testing.T) {
	for _, mdp := range []string{"court", "avec'apostrophe-et-assez-long", "espace dans le mot de passe"} {
		if err := Garantir(context.Background(), DSNSuperutilisateur("10.0.0.1:5432", "x"), mdp); err == nil {
			t.Errorf("mot de passe %q accepté", mdp)
		}
	}
}

func TestGarantirRoleRefuseUnNomNonSur(t *testing.T) {
	dsn := DSNSuperutilisateur("10.0.0.1:5432", "x")
	for _, nom := range []string{"Majuscule", "avec-tiret", "un; DROP", ""} {
		if err := GarantirRole(context.Background(), dsn, nom, "base", "MotDePasseSuffisammentLong"); err == nil {
			t.Errorf("rôle %q accepté", nom)
		}
		if err := GarantirRole(context.Background(), dsn, "role", nom, "MotDePasseSuffisammentLong"); err == nil {
			t.Errorf("base %q acceptée", nom)
		}
	}
}

func TestGarantirAdresseInvalide(t *testing.T) {
	if err := Garantir(context.Background(), "postgres://postgres@sans-port/postgres", "MotDePasseSuffisammentLong"); err == nil {
		t.Fatal("adresse sans port : erreur attendue")
	}
}

func TestCodeErreur(t *testing.T) {
	if !codeErreur(&pgconn.PgError{Code: "42P04"}, "42P04") {
		t.Fatal("code PostgreSQL non reconnu")
	}
	if codeErreur(errors.New("autre"), "42P04") {
		t.Fatal("erreur non PostgreSQL reconnue à tort")
	}
}

// TestGarantirAgainstPostgres : création puis seconde exécution sans erreur.
// Usage : AMANE_TEST_PG=hote:port AMANE_TEST_PG_PASSWORD=... go test ./provision/ -v
func TestGarantirAgainstPostgres(t *testing.T) {
	hote, mdp := os.Getenv("AMANE_TEST_PG"), os.Getenv("AMANE_TEST_PG_PASSWORD")
	if hote == "" {
		t.Skip("AMANE_TEST_PG non défini — intégration PostgreSQL réelle ignorée")
	}
	dsn := DSNSuperutilisateur(hote, mdp)
	for i := 0; i < 2; i++ {
		if err := Garantir(context.Background(), dsn, "MotDePasseDeTestAmane_1"); err != nil {
			t.Fatalf("exécution %d: %v", i+1, err)
		}
	}
}
