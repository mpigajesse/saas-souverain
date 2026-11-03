// app-multi : SaaS multi-services (Web + Worker Redis queue)
// Mode web: HTTP :8080, enfile jobs dans Redis
// Mode worker: consomme jobs Redis, écrit résultat en DB
package main

import (
	"context"
	"encoding/json"
	"flag"
	"fmt"
	"log"
	"net/http"
	"os"
	"os/signal"
	"syscall"
	"time"

	"github.com/jackc/pgx/v5/pgxpool"
	"github.com/redis/go-redis/v9"
)

const (
	queueKey     = "jobs:queue"
	resultsTable = "jobs_results"
)

var (
	mode   = flag.String("mode", "web", "web|worker")
	port   = flag.String("port", "8080", "port HTTP (mode web)")
	redisURL = flag.String("redis", "", "Redis URL (ex: redis://redis:6379)")
)

type Job struct {
	ID      string          `json:"id"`
	Task    string          `json:"task"`
	Payload json.RawMessage `json:"payload"`
}

type Result struct {
	JobID    string          `json:"job_id"`
	Task     string          `json:"task"`
	Payload  json.RawMessage `json:"payload"`
	Result   json.RawMessage `json:"result"`
	Status   string          `json:"status"`
	Error    string          `json:"error,omitempty"`
	Duration int64           `json:"duration_ms"`
}

func main() {
	flag.Parse()

	// Configuration
	dbURL := os.Getenv("DATABASE_URL")
	if dbURL == "" {
		log.Fatal("DATABASE_URL absente")
	}
	if *redisURL == "" {
		*redisURL = os.Getenv("REDIS_URL")
	}
	if *redisURL == "" {
		*redisURL = "redis://redis:6379"
	}

	// Connexion DB
	ctx := context.Background()
	pool, err := pgxpool.New(ctx, dbURL)
	if err != nil {
		log.Fatalf("DB connect: %v", err)
	}
	defer pool.Close()

	// Attendre DB + vérifier migration
	for i := 0; i < 30; i++ {
		var v int
		err = pool.QueryRow(ctx, `SELECT COALESCE(MAX(version), 0) FROM schema_version`).Scan(&v)
		if err == nil && v >= 2 {
			break
		}
		time.Sleep(2 * time.Second)
	}
	if err != nil {
		log.Fatalf("Migration check failed: %v", err)
	}
	log.Print("Base prête, migration version >= 2")

	// Connexion Redis
	opt, err := redis.ParseURL(*redisURL)
	if err != nil {
		log.Fatalf("Redis URL parse: %v", err)
	}
	rdb := redis.NewClient(opt)
	defer rdb.Close()

	// Test Redis
	if err := rdb.Ping(ctx).Err(); err != nil {
		log.Fatalf("Redis ping: %v", err)
	}
	log.Printf("Redis connecté: %s", *redisURL)

	switch *mode {
	case "web":
		runWeb(ctx, pool, rdb)
	case "worker":
		runWorker(ctx, pool, rdb)
	default:
		log.Fatalf("Mode inconnu: %s (web|worker)", *mode)
	}
}

func runWeb(ctx context.Context, pool *pgxpool.Pool, rdb *redis.Client) {
	mux := http.NewServeMux()

	// Healthcheck
	mux.HandleFunc("/sante", func(w http.ResponseWriter, r *http.Request) {
		if err := pool.Ping(r.Context()); err != nil {
			http.Error(w, err.Error(), http.StatusServiceUnavailable)
			return
		}
		if err := rdb.Ping(r.Context()).Err(); err != nil {
			http.Error(w, "redis: "+err.Error(), http.StatusServiceUnavailable)
			return
		}
		w.WriteHeader(http.StatusOK)
		w.Write([]byte("ok"))
	})

	// Enfiler un job
	mux.HandleFunc("/enqueue", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			http.Error(w, "POST only", http.StatusMethodNotAllowed)
			return
		}

		var job Job
		if err := json.NewDecoder(r.Body).Decode(&job); err != nil {
			job = Job{Task: "default", Payload: json.RawMessage(`{}`)}
		}
		if job.ID == "" {
			job.ID = fmt.Sprintf("job-%d", time.Now().UnixNano())
		}
		if job.Task == "" {
			job.Task = "compute"
		}

		data, _ := json.Marshal(job)
		if err := rdb.LPush(ctx, queueKey, data).Err(); err != nil {
			http.Error(w, "redis push: "+err.Error(), http.StatusInternalServerError)
			return
		}

		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(map[string]string{"status": "enqueued", "job_id": job.ID})
	})

	// Stats queue
	mux.HandleFunc("/stats", func(w http.ResponseWriter, r *http.Request) {
		len, _ := rdb.LLen(ctx, queueKey).Result()
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(map[string]int64{"queue_length": len})
	})

	// Page d'accueil
	mux.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) {
		fmt.Fprintf(w, `<h1>app-multi (mode web)</h1>
<p>Endpoints:</p>
<ul>
<li><a href="/sante">/sante</a> - healthcheck</li>
<li>POST /enqueue - enfiler job {"task":"...","payload":{}}</li>
<li><a href="/stats">/stats</a> - longueur queue</li>
</ul>
<p>Worker doit tourner en mode worker pour traiter.</p>`)
	})

	srv := &http.Server{Addr: ":" + *port, Handler: mux}

	// Graceful shutdown
	go func() {
		sig := make(chan os.Signal, 1)
		signal.Notify(sig, syscall.SIGINT, syscall.SIGTERM)
		<-sig
		log.Print("Arrêt serveur web...")
		srv.Shutdown(context.Background())
	}()

	log.Printf("Web server sur port %s", *port)
	if err := srv.ListenAndServe(); err != http.ErrServerClosed {
		log.Fatalf("Server: %v", err)
	}
}

func runWorker(ctx context.Context, pool *pgxpool.Pool, rdb *redis.Client) {
	log.Print("Worker démarré - attente jobs...")

	// Assurer table résultats
	_, err := pool.Exec(ctx, fmt.Sprintf(`
		CREATE TABLE IF NOT EXISTS %s (
			id BIGSERIAL PRIMARY KEY,
			job_id TEXT NOT NULL,
			task TEXT NOT NULL,
			payload JSONB,
			result JSONB,
			status TEXT NOT NULL,
			error TEXT,
			duration_ms BIGINT,
			created_at TIMESTAMPTZ DEFAULT NOW()
		)
	`, resultsTable))
	if err != nil {
		log.Fatalf("Create results table: %v", err)
	}

	// Boucle de traitement
	for {
		select {
		case <-ctx.Done():
			log.Print("Worker arrêt demandé")
			return
		default:
			// BRPOP bloquant (timeout 5s pour vérifier context)
			res, err := rdb.BRPop(ctx, 5*time.Second, queueKey).Result()
			if err == redis.Nil {
				continue // timeout, recheck context
			}
			if err != nil {
				log.Printf("Redis BRPOP error: %v", err)
				time.Sleep(time.Second)
				continue
			}

			if len(res) < 2 {
				continue
			}

			var job Job
			if err := json.Unmarshal([]byte(res[1]), &job); err != nil {
				log.Printf("Job decode error: %v", err)
				continue
			}

			// Traiter le job
			start := time.Now()
			result, err := processJob(ctx, job)
			duration := time.Since(start).Milliseconds()

			// Sauver résultat en DB
			status := "success"
			errMsg := ""
			if err != nil {
				status = "failed"
				errMsg = err.Error()
			}

			_, err = pool.Exec(ctx, fmt.Sprintf(`
				INSERT INTO %s (job_id, task, payload, result, status, error, duration_ms)
				VALUES ($1, $2, $3, $4, $5, $6, $7)
			`, resultsTable), job.ID, job.Task, job.Payload, result, status, errMsg, duration)
			if err != nil {
				log.Printf("Save result error: %v", err)
			}

			log.Printf("Job %s (%s) -> %s (%dms)", job.ID, job.Task, status, duration)
		}
	}
}

func processJob(ctx context.Context, job Job) (json.RawMessage, error) {
	// Simulation traitement selon task
	switch job.Task {
	case "compute":
		// Simule calcul
		time.Sleep(100 * time.Millisecond)
		return json.RawMessage(`{"output": "computed", "value": 42}`), nil

	case "echo":
		return job.Payload, nil

	case "fail":
		return nil, fmt.Errorf("échec simulé pour test")

	default:
		return json.RawMessage(`{"output": "unknown task"}`), nil
	}
}