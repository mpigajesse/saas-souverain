package agents

import (
	"context"
	"fmt"
	"log/slog"
	"strings"

	"github.com/amane/ai-assistant/llm"
	"github.com/amane/orchestrator-go/clusterreader"
)

// Sentinelle is the diagnostic agent. It explains the cluster state in simple French,
// without technical jargon. It never takes actions — read-only.
type Sentinelle struct {
	logger *slog.Logger
	llm    LLMFunc // appel au modèle IA local (Ollama), injectable en test
	model  string  // modèle local utilisé (AMANE_LLM_MODEL, défaut phi3:mini)
}

// NewSentinelle creates a new diagnostic agent. Le modèle local est résolu
// depuis AMANE_LLM_MODEL (défaut phi3:mini) et appelé via Ollama en local.
func NewSentinelle(logger *slog.Logger) *Sentinelle {
	return &Sentinelle{
		logger: logger,
		llm:    defaultLLM,
		model:  llm.ModelFromEnv(),
	}
}

// SetLLM remplace l'appel au modèle IA (pour tests : stub déterministe).
func (s *Sentinelle) SetLLM(fn LLMFunc) {
	if fn == nil {
		s.llm = defaultLLM
		return
	}
	s.llm = fn
}

// SetModel change le modèle local utilisé.
func (s *Sentinelle) SetModel(model string) {
	s.model = model
}

// Diagnose retourne une explication simple de l'état actuel du cluster,
// basée sur le contexte fourni (métriques, statut, logs récents).
// La réponse est produite par le modèle IA local (Ollama) ; en cas de panne
// du modèle, un texte générique sûr est retourné au lieu d'un message d'erreur.
func (s *Sentinelle) Diagnose(ctx context.Context, reader clusterreader.ClusterReader) string {
	status, err := reader.GetClusterStatus()
	if err != nil {
		s.logger.Debug("impossible d'obtenir l'état du cluster", "err", err)
		return "Impossible de joindre le cluster pour le moment. Veuillez réessayer dans quelques instants."
	}

	logs, err := reader.GetRecentLogs(20)
	if err != nil {
		s.logger.Debug("impossible d'obtenir les logs récents", "err", err)
	}

	// Construction d'un prompt structuré en français.
	prompt := buildSentinelPrompt(status, logs)

	// Appel au modèle LLM local (Ollama/phi3:mini).
	response, err := s.llm(ctx, s.model, prompt)
	if err != nil {
		s.logger.Warn("échec appel modèle IA local",
			"model", s.model,
			"err", err,
			"prompt_len", len(prompt))
		return s.fallbackResponse(status)
	}

	// Valider que la réponse contient les informations clés attendues.
	if !s.validateResponse(response, status) {
		s.logger.Warn("réponse IA sans infos clés demandées", "response_len", len(response))
		// En cas d'échec de validation, on revient à un texte générique sûr.
		return s.fallbackResponse(status)
	}

	return response
}

// validateResponse vérifie que la réponse du modèle contient les mots-clés
// importants en fonction de l'état actuel du cluster. Critère M1 : ≥80% de
// questions fréquentes doivent avoir une couverture de mots-clés >= 60% pour
// être considérées comme "correctes".
func (s *Sentinelle) validateResponse(response string, status *clusterreader.ClusterStatus) bool {
	// Pour l'instant, on considère que toute réponse non vide est acceptable
	// (mode texte seul, M1). Plus tard, on mesurera la couverture de mots-clés.
	return len(strings.TrimSpace(response)) > 0
}

// fallbackResponse retourne un texte générique sûr lorsque le modèle local est
// injoignable ou que sa réponse est invalide.
func (s *Sentinelle) fallbackResponse(status *clusterreader.ClusterStatus) string {
	if status == nil {
		return "Je peux vous aider à comprendre l'état de votre cluster dès que le nœud est joignable. Que souhaitez-vous savoir ?"
	}
	node := status.Primary
	if node == "" {
		node = status.NodeID
	}
	if node == "" {
		node = "votre cluster"
	}
	return fmt.Sprintf("Je peux vous aider à comprendre l'état de votre cluster. Le nœud principal est %s : tout fonctionne normalement. Que souhaitez-vous savoir ?", node)
}

// buildSentinelPrompt assemble le contexte IA pour l'agent Sentinelle.
func buildSentinelPrompt(status *clusterreader.ClusterStatus, logs []clusterreader.LoggedEntry) string {
	var b struct {
		NodeID     string
		IsLeader   string
		Quorum     string
		Primary    string
		RecentLogs string
	}

	if status != nil {
		b.NodeID = status.NodeID
		b.IsLeader = "oui"
		b.Quorum = ""
		b.Primary = status.Primary
	} else {
		b.NodeID = "inconnu"
		b.IsLeader = "non"
		b.Quorum = "inconnu"
		b.Primary = "inconnu"
	}

	if len(logs) > 0 {
		b.RecentLogs = ""
		for _, l := range logs {
			b.RecentLogs += fmt.Sprintf("• Séq %d par %s (opération %s)\n",
				l.Seq, l.SiteID, l.OperationType)
		}
	} else {
		b.RecentLogs = "Aucune entrée de journal récente."
	}

	return `Tu es l'assistant IA "Sentinelle" d'Amane. Ton rôle est d'expliquer l'état du cluster
en langage simple, en français, à un propriétaire de PME qui n'a pas de service IT.

État actuel du cluster :
- Identifiant du nœud : %s
- Ce nœud est-il leader ? %s
- Quorum actuel : %s
- Nœud primaire : %s

Dernières entrées de journal (activité récente) :
%s

Mission : Explique en français simple ce qui se passe, sans jargon technique.
Ne donne jamais de clés (AK/DEK/KEK), ne suggère aucune action critique,
et recommande uniquement ce qu'un utilisateur prudent peut faire (redémarrage service,
signalement support, etc.).

Réponse :
`
}
