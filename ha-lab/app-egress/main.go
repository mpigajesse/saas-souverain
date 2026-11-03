// app-egress : SaaS test sortie réseau contrôlée
// Tente requêtes HTTP sortantes → logge résultat en DB (succès/échec/latence/via_proxy)
// En confinement AMANE (réseau internal), sorties directes bloquées
// Phase 3: proxy nœud avec whitelist autorise domaines déclarés dans manifeste.egress
package main

import (
	"context"
	"encoding/json"
	"log"
	"net/http"
	"os"
	"time"

	"github.com/jackc/pgx/v5/pgxpool"
)

const egressTable = "egress_logs"

type EgressLog struct {
	Domain    string `json:"domain"`
	Status    string `json:"status"`     // success | blocked | error
	LatencyMs int64  `json:"latency_ms"`
	ViaProxy  bool   `json:"via_proxy"`
	ErrorMsg  string `json:"error_msg,omitempty"`
}

func main() {
	ctx := context.Background()

	dbURL := os.Getenv("DATABASE_URL")
	if dbURL == "" {
		log.Fatal("DATABASE_URL absente")
	}

	pool, err := pgxpool.New(ctx, dbURL)
	if err != nil {
		log.Fatalf("DB connect: %v", err)
	}
	defer pool.Close()

	// Attendre DB + migration
	for i := 0; i < 30; i++ {
		var v int
		err = pool.QueryRow(ctx, `SELECT COALESCE(MAX(version), 0) FROM schema_version`).Scan(&v)
		if err == nil && v >= 3 {
			break
		}
		time.Sleep(2 * time.Second)
	}
	if err != nil {
		log.Fatalf("Migration check: %v", err)
	}
	log.Print("Base prête, migration version >= 3")

	// Assurer table egress_logs
	_, err = pool.Exec(ctx, `
		CREATE TABLE IF NOT EXISTS `+egressTable+` (
			id BIGSERIAL PRIMARY KEY,
			domain TEXT NOT NULL,
			status TEXT NOT NULL,  -- success | blocked | error
			latency_ms BIGINT,
			via_proxy BOOLEAN DEFAULT false,
			error_msg TEXT,
			created_at TIMESTAMPTZ DEFAULT NOW()
		)
	`)
	if err != nil {
		log.Fatalf("Create egress_logs: %v", err)
	}

	// Test egress au démarrage (domaines de test)
	testDomains := []string{
		"http://httpbin.org/get",
		"https://api.github.com",
	}
	go func() {
		time.Sleep(2 * time.Second) // Laisser serveur démarrer
		for _, url := range testDomains {
			testEgress(ctx, pool, url)
		}
	}()

	// HTTP handlers
	mux := http.NewServeMux()

	mux.HandleFunc("/sante", func(w http.ResponseWriter, r *http.Request) {
		if err := pool.Ping(r.Context()); err != nil {
			http.Error(w, err.Error(), http.StatusServiceUnavailable)
			return
		}
		w.WriteHeader(http.StatusOK)
		w.Write([]byte("ok"))
	})

	mux.HandleFunc("/egress-test", func(w http.ResponseWriter, r *http.Request) {
		url := r.URL.Query().Get("url")
		if url == "" {
			url = "http://httpbin.org/get"
		}
		result := testEgress(r.Context(), pool, url)
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(result)
	})

	mux.HandleFunc("/egress-logs", func(w http.ResponseWriter, r *http.Request) {
		rows, err := pool.Query(r.Context(), `
			SELECT domain, status, latency_ms, via_proxy, error_msg, created_at
			FROM `+egressTable+`
			ORDER BY created_at DESC LIMIT 50
		`)
		if err != nil {
			http.Error(w, err.Error(), http.StatusInternalServerError)
			return
		}
		defer rows.Close()

		var logs []map[string]any
		for rows.Next() {
			var l EgressLog
			var created time.Time
			if err := rows.Scan(&l.Domain, &l.Status, &l.LatencyMs, &l.ViaProxy, &l.ErrorMsg, &created); err != nil {
				continue
			}
			logs = append(logs, map[string]any{
				"domain":      l.Domain,
				"status":      l.Status,
				"latency_ms":  l.LatencyMs,
				"via_proxy":   l.ViaProxy,
				"error_msg":   l.ErrorMsg,
				"created_at":  created,
			})
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(logs)
	})

	mux.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "text/html")
		w.Write([]byte(`<h1>app-egress</h1>
<p>Test sortie réseau contrôlée (confinement AMANE)</p>
<ul>
<li><a href="/sante">/sante</a> - healthcheck</li>
<li><a href="/egress-test">/egress-test</a> - tester egress (url param: ?url=...)</li>
<li><a href="/egress-logs">/egress-logs</a> - voir logs egress</li>
</ul>
<p>En confinement strict (réseau internal), sorties directes bloquées.</p>
<p>Phase 3: proxy nœud avec whitelist autorise domaines déclarés.</p>`))
	})

	port := os.Getenv("PORT")
	if port == "" {
		port = "8080"
	}

	log.Printf("Démarrage sur port %s", port)
	log.Fatal(http.ListenAndServe(":"+port, mux))
}

func testEgress(ctx context.Context, pool *pgxpool.Pool, url string) EgressLog {
	start := time.Now()
	entry := EgressLog{Domain: url}

	client := &http.Client{Timeout: 5 * time.Second}
	req, _ := http.NewRequestWithContext(ctx, "GET", url, nil)
	resp, err := client.Do(req)
	latency := time.Since(start).Milliseconds()
	entry.LatencyMs = latency

	if err != nil {
		entry.Status = "blocked"
		entry.ErrorMsg = err.Error()
		entry.ViaProxy = false
		saveEgressLog(ctx, pool, entry)
		return entry
	}
	defer resp.Body.Close()

	if resp.StatusCode >= 200 && resp.StatusCode < 300 {
		entry.Status = "success"
	} else {
		entry.Status = "error"
		entry.ErrorMsg = resp.Status
	}
	entry.ViaProxy = false
	saveEgressLog(ctx, pool, entry)
	return entry
}

func saveEgressLog(ctx context.Context, pool *pgxpool.Pool, entry EgressLog) {
	_, err := pool.Exec(ctx, `
		INSERT INTO `+egressTable+` (domain, status, latency_ms, via_proxy, error_msg)
		VALUES ($1, $2, $3, $4, $5)
	`, entry.Domain, entry.Status, entry.LatencyMs, entry.ViaProxy, entry.ErrorMsg)
	if err != nil {
		log.Printf("Save egress log error: %v", err)
	}
}