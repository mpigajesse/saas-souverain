package agents

import (
	"context"

	"github.com/amane/ai-assistant/llm"
)

// LLMFunc est le type d'un appel au modèle IA local. Il est injectable pour
// les tests (déterministe) et la production utilise llm.CallLocal (Ollama).
type LLMFunc func(ctx context.Context, model, prompt string) (string, error)

// defaultLLM est l'appel par défaut : modèle OLLAMA local.
func defaultLLM(ctx context.Context, model, prompt string) (string, error) {
	return llm.CallLocal(ctx, model, prompt)
}
