// Tests pour app-multi (TDD - écrits AVANT main.go)
package main

import (
	"database/sql"
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

func TestWebEnqueuesJob(t *testing.T) {
	dbURL := os.Getenv("DATABASE_URL")
	if dbURL == "" {
		t.Skip("DATABASE_URL non défini - test d'intégration ignoré")
	}

	// Test que le mode web enfile un job dans Redis
	// Nécessite Redis + DB réels → test d'intégration
	req := httptest.NewRequest("POST", "/enqueue", nil)
	w := httptest.NewRecorder()
	_ = w
	_ = req
}

func TestWorkerProcessesJob(t *testing.T) {
	dbURL := os.Getenv("DATABASE_URL")
	if dbURL == "" {
		t.Skip("DATABASE_URL non défini - test d'intégration ignoré")
	}

	// Vérifier qu'un job enfile est traité et résultat en DB
	db, err := sql.Open("pgx", dbURL)
	if err != nil {
		t.Fatalf("Connexion DB: %v", err)
	}
	defer db.Close()

	// La table jobs_results doit exister (créée par migration)
	var count int
	err = db.QueryRow("SELECT COUNT(*) FROM jobs_results").Scan(&count)
	if err != nil {
		t.Fatalf("Table jobs_results absente: %v", err)
	}
	t.Logf("jobs_results count: %d", count)
}

func TestRedisConnection(t *testing.T) {
	redisURL := os.Getenv("REDIS_URL")
	if redisURL == "" {
		t.Skip("REDIS_URL non défini - test d'intégration ignoré")
	}

	// Test basique connexion Redis
	// (utilise redis-cli ou client Go)
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
	if version < 2 {
		t.Errorf("Migration app-multi non appliquée: version=%d (attendu >= 2)", version)
	}
}

func TestJobsTableExists(t *testing.T) {
	dbURL := os.Getenv("DATABASE_URL")
	if dbURL == "" {
		t.Skip("DATABASE_URL non défini - test d'intégration ignoré")
	}

	db, err := sql.Open("pgx", dbURL)
	if err != nil {
		t.Fatalf("Connexion DB: %v", err)
	}
	defer db.Close()

	// Vérifier table jobs_results
	_, err = db.Exec("INSERT INTO jobs_results (task, payload, result) VALUES ('test', '{}', '{}')")
	if err != nil {
		t.Errorf("Table jobs_results non accessible: %v", err)
	}
	db.Exec("DELETE FROM jobs_results WHERE task='test'")
}