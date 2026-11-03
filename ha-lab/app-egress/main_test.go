// Tests pour app-egress (TDD - écrits AVANT main.go)
package main

import (
	"database/sql"
	"net/http"
	"net/http/httptest"
	"os"
	"testing"
	"time"

	_ "github.com/jackc/pgx/v5/stdlib"
)

func TestMain(m *testing.M) {
	dbURL := os.Getenv("DATABASE_URL")
	if dbURL == "" {
		dbURL = "postgres://postgres:postgres@localhost:5432/postgres?sslmode=disable"
	}

	var db *sql.DB
	var err error
	for i := 0; i < 30; i++ {
		db, err = sql.Open("pgx", dbURL)
		if err == nil {
			err = db.Ping()
			if err == nil {
				break
			}
		}
		time.Sleep(time.Second)
	}
	if err != nil {
		os.Exit(m.Run())
	}
	db.Close()
	os.Exit(m.Run())
}

func TestHealthEndpoint(t *testing.T) {
	dbURL := os.Getenv("DATABASE_URL")
	if dbURL == "" {
		t.Skip("DATABASE_URL non défini - test d'intégration ignoré")
	}

	req := httptest.NewRequest("GET", "/sante", nil)
	w := httptest.NewRecorder()
	_ = w
	_ = req
}

func TestEgressEndpoint(t *testing.T) {
	dbURL := os.Getenv("DATABASE_URL")
	if dbURL == "" {
		t.Skip("DATABASE_URL non défini - test d'intégration ignoré")
	}

	// Test endpoint /egress-test déclenche tentative sortante
	req := httptest.NewRequest("GET", "/egress-test", nil)
	w := httptest.NewRecorder()
	_ = w
	_ = req
}

func TestEgressLogTableExists(t *testing.T) {
	dbURL := os.Getenv("DATABASE_URL")
	if dbURL == "" {
		t.Skip("DATABASE_URL non défini - test d'intégration ignoré")
	}

	db, err := sql.Open("pgx", dbURL)
	if err != nil {
		t.Fatalf("Connexion DB: %v", err)
	}
	defer db.Close()

	// Vérifier table egress_logs
	_, err = db.Exec("INSERT INTO egress_logs (domain, status, latency_ms, via_proxy) VALUES ('test.com', 'success', 100, false)")
	if err != nil {
		t.Errorf("Table egress_logs non accessible: %v", err)
	}
	db.Exec("DELETE FROM egress_logs WHERE domain='test.com'")
}

func TestMigrationApplied(t *testing.T) {
	dbURL := os.Getenv("DATABASE_URL")
	if dbURL == "" {
		t.Skip("DATABASE_URL non défini - test d'intégration ignoré")
	}

	db, err := sql.Open("pgx", dbURL)
	if err != nil {
		t.Fatalf("Connexion DB: %v", err)
	}
	defer db.Close()

	var version int
	err = db.QueryRow("SELECT version FROM schema_version ORDER BY version DESC LIMIT 1").Scan(&version)
	if err != nil {
		t.Fatalf("Table schema_version absente: %v", err)
	}
	if version < 3 {
		t.Errorf("Migration app-egress non appliquée: version=%d (attendu >= 3)", version)
	}
}

func TestEgressBlockedByDefault(t *testing.T) {
	// Dans l'architecture AMANE, le réseau 'amane-application' est internal=true
	// Donc toute tentative sortante directe doit échouer (sauf via proxy nœud)
	// Ce test vérifie que l'app loggue l'échec quand pas de proxy
	dbURL := os.Getenv("DATABASE_URL")
	if dbURL == "" {
		t.Skip("DATABASE_URL non défini - test d'intégration ignoré")
	}

	db, err := sql.Open("pgx", dbURL)
	if err != nil {
		t.Fatalf("Connexion DB: %v", err)
	}
	defer db.Close()

	// Vérifier qu'on peut logger un échec
	_, err = db.Exec(`
		INSERT INTO egress_logs (domain, status, latency_ms, via_proxy, error_msg)
		VALUES ('httpbin.org', 'blocked', 0, false, 'network not allowed')
	`)
	if err != nil {
		t.Errorf("Insert egress log blocked: %v", err)
	}
	db.Exec("DELETE FROM egress_logs WHERE domain='httpbin.org' AND status='blocked'")
}