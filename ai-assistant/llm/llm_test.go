package llm

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
)

// TestGenerateOK vérifie le chemin nominal : le serveur répond un JSON Ollama
// et Generate retourne le champ "response".
func TestGenerateOK(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/api/generate" {
			t.Errorf("chemin inattendu : %s", r.URL.Path)
		}
		if ct := r.Header.Get("Content-Type"); ct != "application/json" {
			t.Errorf("Content-Type inattendu : %s", ct)
		}
		var req struct {
			Model  string `json:"model"`
			Prompt string `json:"prompt"`
		}
		if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
			t.Errorf("requête illisible : %v", err)
		}
		if req.Model != "phi3:mini" || !strings.Contains(req.Prompt, "test") {
			t.Errorf("payload inattendu : %+v", req)
		}
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(map[string]any{"response": "Le cluster est en bonne santé."})
	}))
	defer srv.Close()

	c := &Client{baseURL: srv.URL, http: srv.Client()}
	got, err := c.Generate(context.Background(), "phi3:mini", "prompt de test")
	if err != nil {
		t.Fatalf("Generate() error = %v", err)
	}
	if want := "Le cluster est en bonne santé."; got != want {
		t.Errorf("Generate() = %q, attendu %q", got, want)
	}
}

// TestGenerateHTTPError vérifie qu'un statut non-200 produit une erreur
// explicite (et non une réponse tronquée ignorée).
func TestGenerateHTTPError(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		http.Error(w, "modele inconnu", http.StatusNotFound)
	}))
	defer srv.Close()

	c := &Client{baseURL: srv.URL, http: srv.Client()}
	_, err := c.Generate(context.Background(), "modele-inconnu", "prompt")
	if err == nil {
		t.Fatal("Generate() attendait une erreur sur statut 404")
	}
	if !strings.Contains(err.Error(), "404") {
		t.Errorf("erreur sans statut HTTP : %v", err)
	}
}

// TestGenerateVide vérifie qu'une réponse Ollama vide est refusée.
func TestGenerateVide(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		json.NewEncoder(w).Encode(map[string]any{"response": "  \n"})
	}))
	defer srv.Close()

	c := &Client{baseURL: srv.URL, http: srv.Client()}
	if _, err := c.Generate(context.Background(), "phi3:mini", "prompt"); err == nil {
		t.Fatal("Generate() attendait une erreur sur réponse vide")
	}
}

// TestHostFromEnv vérifie la priorité OLLAMA_HOST sur la valeur par défaut.
func TestHostFromEnv(t *testing.T) {
	t.Setenv(OLLAMA_HOST, "http://192.168.1.50:11434")
	if got := hostFromEnv(); got != "http://192.168.1.50:11434" {
		t.Errorf("hostFromEnv() = %s", got)
	}
	t.Setenv(OLLAMA_HOST, "")
	if got := hostFromEnv(); got != DefaultHost {
		t.Errorf("hostFromEnv() défaut = %s, attendu %s", got, DefaultHost)
	}
}

// TestModelFromEnv vérifie la priorité AMANE_LLM_MODEL sur phi3:mini.
func TestModelFromEnv(t *testing.T) {
	t.Setenv(AMANE_LLM_MODEL, "qwen2.5:3b")
	if got := ModelFromEnv(); got != "qwen2.5:3b" {
		t.Errorf("ModelFromEnv() = %s", got)
	}
	t.Setenv(AMANE_LLM_MODEL, "")
	if got := ModelFromEnv(); got != DefaultModel {
		t.Errorf("ModelFromEnv() défaut = %s, attendu %s", got, DefaultModel)
	}
}
