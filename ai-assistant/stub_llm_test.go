package ai_assistant

import (
	"context"
	"strings"

	"github.com/amane/ai-assistant/agents"
)

// stubLLM simule un modèle IA local déterministe pour les tests.
// Il reproduit le comportement de l'ancien simulateur de mots-clés :
//   - prompt Sentinelle → texte simple en français ;
//   - prompt Copilote → JSON {action, args} basé sur l'intention extraite,
//     les intentions adverses étant volontairement mappées vers
//     {"action":"rm -rf /"} pour éprouver le rejet par la whitelist.
//
// C'est une réalisation de agents.LLMFunc : aucun réseau, aucun Ollama requis.
func stubLLM(_ context.Context, _ /* model */ string, prompt string) (string, error) {
	if strings.Contains(prompt, "Sentinelle") {
		return "Le cluster est surveillé en continu et semble en bon état général.", nil
	}

	intent := extractIntent(prompt)
	lower := strings.ToLower(intent)

	// Intentions adverses → action connue NON whitelistée (test du rejet).
	if strings.Contains(lower, "rm -rf") || strings.Contains(lower, "supprimer") ||
		strings.Contains(lower, "delete") || strings.Contains(lower, "format") ||
		strings.Contains(lower, "rotate_key") || strings.Contains(lower, "reboot") ||
		strings.Contains(lower, "redémarre la machine") || strings.Contains(lower, "tourne la clé") ||
		strings.Contains(lower, "supprime la base") || strings.Contains(lower, "désactive le fencing") ||
		strings.Contains(lower, "contourne le quorum") {
		return `{"action": "rm -rf /", "args": {}}`, nil
	}

	// Maintenance applicative.
	if strings.Contains(lower, "redémarrer") || strings.Contains(lower, "restart") ||
		strings.Contains(lower, "service") || strings.Contains(lower, "planté") ||
		strings.Contains(lower, "ne répond plus") || strings.Contains(lower, "erreur") {
		return `{"action": "restart_service", "args": {"service": "orchestrator"}}`, nil
	}

	// Nettoyage des journaux WAL.
	if strings.Contains(lower, "nettoyer") || strings.Contains(lower, "clean") ||
		strings.Contains(lower, "wal") || strings.Contains(lower, "logs") ||
		strings.Contains(lower, "disque plein") || strings.Contains(lower, "alerte disque") ||
		strings.Contains(lower, "journal s'accumule") {
		return `{"action": "clean_wal_logs", "args": {"policy": "default"}}`, nil
	}

	return `{"action": "none", "args": {}}`, nil
}

// extractIntent retrouve l'intention utilisateur dans le prompt du Copilote
// (champ « Intention utilisateur : "…" »), pour un stub déterministe.
func extractIntent(prompt string) string {
	const marker = `Intention utilisateur : "`
	i := strings.Index(prompt, marker)
	if i < 0 {
		return ""
	}
	rest := prompt[i+len(marker):]
	j := strings.IndexByte(rest, '"')
	if j < 0 {
		return rest
	}
	return rest[:j]
}

// compile-time check : le stub est conforme au contrat agents.LLMFunc.
var _ agents.LLMFunc = stubLLM
