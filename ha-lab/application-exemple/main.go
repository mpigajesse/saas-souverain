// Application d'exemple « éditeur externe » (refonte V3, étape 3) : un SaaS
// ordinaire qui ne sait rien d'AMANE. Il lit DATABASE_URL, compte les visites
// dans PostgreSQL et sert une page web. AMANE l'héberge sans modifier son image
// (construite FROM scratch : ni shell ni outils).
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
	url := os.Getenv("DATABASE_URL")
	if url == "" {
		log.Fatal("DATABASE_URL absente")
	}
	var pool *pgxpool.Pool
	for {
		p, err := pgxpool.New(ctx, url)
		if err == nil {
			if _, err = p.Exec(ctx, `CREATE TABLE IF NOT EXISTS visites (id serial PRIMARY KEY, le timestamptz DEFAULT now())`); err == nil {
				pool = p
				break
			}
			p.Close()
		}
		log.Printf("base indisponible : %v", err)
		time.Sleep(2 * time.Second)
	}
	log.Print("base prête")

	http.HandleFunc("/sante", func(w http.ResponseWriter, r *http.Request) {
		if err := pool.Ping(r.Context()); err != nil {
			http.Error(w, err.Error(), http.StatusServiceUnavailable)
			return
		}
		w.Write([]byte("ok"))
	})
	http.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) {
		var n int
		var base string
		err := pool.QueryRow(r.Context(),
			`WITH v AS (INSERT INTO visites DEFAULT VALUES RETURNING 1)
			 SELECT (SELECT count(*) FROM visites) + 1, current_database()`).Scan(&n, &base)
		if err != nil {
			http.Error(w, "base indisponible : "+err.Error(), http.StatusServiceUnavailable)
			return
		}
		fmt.Fprintf(w, "<h1>Facturation (application d'exemple)</h1><p>visites : %d</p><p>base : %s</p>", n, html.EscapeString(base))
	})
	port := os.Getenv("PORT")
	if port == "" {
		port = "8080"
	}
	log.Fatal(http.ListenAndServe(":"+port, nil))
}
