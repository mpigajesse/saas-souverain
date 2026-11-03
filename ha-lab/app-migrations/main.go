// Application app-migrations : SaaS de test avec hook pre_start + migration SQL
// Go FROM scratch, lit DATABASE_URL, sert /sante et / avec compteur visites
package main

import (
	"context"
	"fmt"
	"html"
	"log"
	"net/http"
	"os"
	"time"

	"github.com/jackc/pgx/v5/pgxpool"
)

func main() {
	ctx := context.Background()

	// Récupérer DATABASE_URL (injectée par le lanceur AMANE via le manifeste)
	url := os.Getenv("DATABASE_URL")
	if url == "" {
		log.Fatal("DATABASE_URL absente - le lanceur AMANE doit l'injecter")
	}

	// Connexion à la base avec retry (base peut pas être prête au tout début)
	var pool *pgxpool.Pool
	for i := 0; i < 30; i++ {
		p, err := pgxpool.New(ctx, url)
		if err == nil {
			// Vérifier que la migration a été appliquée (table schema_version)
			var version int
			err = p.QueryRow(ctx, `SELECT COALESCE(MAX(version), 0) FROM schema_version`).Scan(&version)
			if err == nil && version >= 1 {
				pool = p
				break
			}
			if err != nil {
				log.Printf("Vérification migration: %v", err)
			} else {
				log.Printf("Migration non appliquée (version=%d), attente...", version)
			}
			p.Close()
		} else {
			log.Printf("Base indisponible: %v", err)
		}
		time.Sleep(2 * time.Second)
	}

	if pool == nil {
		log.Fatal("Impossible de se connecter à la base après 60s")
	}
	defer pool.Close()

	log.Print("Base prête, migration vérifiée (schema_version >= 1)")

	// Healthcheck endpoint - sondé par le nœud AMANE (ss-node)
	http.HandleFunc("/sante", func(w http.ResponseWriter, r *http.Request) {
		if err := pool.Ping(r.Context()); err != nil {
			http.Error(w, err.Error(), http.StatusServiceUnavailable)
			return
		}
		w.WriteHeader(http.StatusOK)
		w.Write([]byte("ok"))
	})

	// Page principale - affiche compteur visites + version migration
	http.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) {
		var n int
		var base string
		var migrationVersion int

		err := pool.QueryRow(r.Context(), `
			WITH v AS (INSERT INTO visites DEFAULT VALUES RETURNING 1)
			SELECT
				(SELECT COUNT(*) FROM visites) + 1,
				current_database(),
				COALESCE((SELECT MAX(version) FROM schema_version), 0)
		`).Scan(&n, &base, &migrationVersion)

		if err != nil {
			http.Error(w, "base indisponible: "+err.Error(), http.StatusServiceUnavailable)
			return
		}

		fmt.Fprintf(w, `<h1>Facturation (app-migrations)</h1>
<p>visites: %d</p>
<p>base: %s</p>
<p>migration version: %d</p>
<p><a href="/sante">Healthcheck</a></p>`, n, html.EscapeString(base), migrationVersion)
	})

	// Port configurable (défaut 8080)
	port := os.Getenv("PORT")
	if port == "" {
		port = "8080"
	}

	log.Printf("Démarrage sur port %s", port)
	log.Fatal(http.ListenAndServe(":"+port, nil))
}