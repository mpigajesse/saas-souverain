// Package llm fournit un client minimal pour un modèle IA LOCAL via l'API
// HTTP d'Ollama (par défaut http://127.0.0.1:11434, surchargeable par
// OLLAMA_HOST). Le modèle est appelé en local uniquement — jamais via un
// service cloud — conformément à la souveraineté d'Amane.
//
// Aucune donnée sensible n'est ajoutée au prompt côté client : les agents
// (Sentinelle/Copilote) ne transmettent que de l'état du cluster (nœuds,
// séquences du journal, types d'opération), jamais de clé AK/DEK/KEK.
package llm

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net/http"
	"os"
	"strings"
	"time"
)

// Env vars de configuration du modèle local.
const (
	// OLLAMA_HOST point de terminaison HTTP local d'Ollama (ex. conserve les
	// valeurs par défaut si absent).
	OLLAMA_HOST = "OLLAMA_HOST"
	// AMANE_LLM_MODEL nom du modèle local utilisé par les agents IA.
	AMANE_LLM_MODEL = "AMANE_LLM_MODEL"

	// DefaultHost est l'adresse locale par défaut d'Ollama.
	DefaultHost = "http://127.0.0.1:11434"
	// DefaultModel est le modèle local par défaut des agents (léger, hors-ligne).
	DefaultModel = "phi3:mini"

	// maxResponseBytes borne la taille de la réponse Ollama lue (1 MiB).
	maxResponseBytes = 1 << 20
	// defaultTimeout borne la durée d'un appel (Ollama génère sur CPU).
	defaultTimeout = 90 * time.Second
)

// Client est un client HTTP vers une instance Ollama locale.
type Client struct {
	baseURL string
	http    *http.Client
}

// NewClient crée un client Ollama en lisant les variables d'environnement.
func NewClient() *Client {
	return &Client{
		baseURL: hostFromEnv(),
		http:    &http.Client{Timeout: defaultTimeout},
	}
}

// Generate envoie un prompt au modèle et retourne la réponse textuelle.
// stream est désactivé (le client lit une réponse JSON complète).
func (c *Client) Generate(ctx context.Context, model, prompt string) (string, error) {
	body, err := json.Marshal(map[string]any{
		"model":  model,
		"prompt": prompt,
		"stream": false,
		"options": map[string]any{
			"num_predict": 256, // borne la génération pour un petit modèle local
		},
	})
	if err != nil {
		return "", fmt.Errorf("encodage requête Ollama : %w", err)
	}

	req, err := http.NewRequestWithContext(ctx, http.MethodPost, c.baseURL+"/api/generate", bytes.NewReader(body))
	if err != nil {
		return "", fmt.Errorf("construction requête Ollama : %w", err)
	}
	req.Header.Set("Content-Type", "application/json")

	resp, err := c.http.Do(req)
	if err != nil {
		return "", fmt.Errorf("appel du modèle local impossible (%s) : %w", c.baseURL, err)
	}
	defer resp.Body.Close()

	data, err := io.ReadAll(io.LimitReader(resp.Body, maxResponseBytes))
	if err != nil {
		return "", fmt.Errorf("lecture réponse Ollama : %w", err)
	}
	if resp.StatusCode != http.StatusOK {
		return "", fmt.Errorf("Ollama a répondu %d : %s", resp.StatusCode, msgLimit(data))
	}

	var out struct {
		Response string `json:"response"`
	}
	if err := json.Unmarshal(data, &out); err != nil {
		return "", fmt.Errorf("réponse Ollama illisible : %w", err)
	}
	if strings.TrimSpace(out.Response) == "" {
		return "", errors.New("réponse du modèle local vide")
	}
	return out.Response, nil
}

// CallLocal est le point d'appel de plus haut niveau : client par défaut,
// envoi synchrone du prompt et retour de la réponse texte.
func CallLocal(ctx context.Context, model, prompt string) (string, error) {
	return NewClient().Generate(ctx, model, prompt)
}

// hostFromEnv retourne l'adresse Ollama configurée (OLLAMA_HOST), sinon la
// valeur par défaut locale.
func hostFromEnv() string {
	if h := strings.TrimSpace(os.Getenv(OLLAMA_HOST)); h != "" {
		return h
	}
	return DefaultHost
}

// ModelFromEnv retourne le modèle local configuré (AMANE_LLM_MODEL), sinon
// le modèle par défaut.
func ModelFromEnv() string {
	if m := strings.TrimSpace(os.Getenv(AMANE_LLM_MODEL)); m != "" {
		return m
	}
	return DefaultModel
}

// msgLimit tronque un message d'erreur volumineux avant journalisation.
func msgLimit(data []byte) string {
	s := string(data)
	if len(s) > 200 {
		return s[:200] + "…"
	}
	return s
}
