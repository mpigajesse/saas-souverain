package agents

import (
	"context"
	"encoding/json"
	"fmt"
	"log/slog"
	"strings"
	"sync"
	"time"

	"github.com/amane/ai-assistant/journal"
	"github.com/amane/ai-assistant/llm"
	"github.com/amane/ai-assistant/whitelist"
	"github.com/amane/orchestrator-go/clusterreader"
)

// Copilote is the action agent. It proposes actions from the whitelist
// and waits for explicit user confirmation before executing.
// It NEVER decides critical infrastructure actions (failover, promotion, key rotation).
type Copilote struct {
	logger      *slog.Logger
	dryRun      bool                                 // mode dry-run : propose mais n'exécute jamais
	confirmFunc func(actionName, prompt string) bool // callback pour demander confirmation
	llm         LLMFunc                              // appel au modèle IA local (Ollama), injectable en test
	model       string                               // modèle local utilisé (AMANE_LLM_MODEL, défaut phi3:mini)
	mu          sync.Mutex
	journal     journal.AIJournalWriter // writer pour journalisation ss-journal
	nodeID      string                  // identifiant du nœud pour le journal
}

// NewCopilote creates a new action agent. Le modèle local est résolu depuis
// AMANE_LLM_MODEL (défaut phi3:mini) et appelé via Ollama en local.
func NewCopilote(logger *slog.Logger, dryRun bool) *Copilote {
	return &Copilote{
		logger:      logger,
		dryRun:      dryRun,
		confirmFunc: defaultConfirmFunc,
		llm:         defaultLLM,
		model:       llm.ModelFromEnv(),
	}
}

// SetLLM remplace l'appel au modèle IA (pour tests : stub déterministe).
func (c *Copilote) SetLLM(fn LLMFunc) {
	c.mu.Lock()
	defer c.mu.Unlock()
	if fn == nil {
		c.llm = defaultLLM
		return
	}
	c.llm = fn
}

// SetModel change le modèle local utilisé.
func (c *Copilote) SetModel(model string) {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.model = model
}

// SetJournal configure le writer de journal pour la traçabilité (M4).
func (c *Copilote) SetJournal(writer journal.AIJournalWriter, nodeID string) {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.journal = writer
	c.nodeID = nodeID
}

// SetConfirmFunc permet de remplacer la fonction de confirmation (pour tests).
func (c *Copilote) SetConfirmFunc(f func(actionName, prompt string) bool) {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.confirmFunc = f
}

// SetDryRun active/désactive le mode dry-run.
func (c *Copilote) SetDryRun(dryRun bool) {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.dryRun = dryRun
}

// IsDryRun retourne l'état actuel du mode dry-run.
func (c *Copilote) IsDryRun() bool {
	c.mu.Lock()
	defer c.mu.Unlock()
	return c.dryRun
}

// ProposeAction analyse l'intention de l'utilisateur via le LLM local,
// valide contre la whitelist, et retourne une proposition validée.
// Le LLM doit retourner un JSON avec { "action": "nom_action", "args": {...} }.
// La sortie du LLM est NON fiable par construction : seule la whitelist
// autorise une action, jamais le modèle.
func (c *Copilote) ProposeAction(ctx context.Context, reader clusterreader.ClusterReader, userIntent string) *whitelist.ProposeAction {
	// Construire le prompt pour le LLM local.
	prompt := buildCopilotPrompt(reader, userIntent)

	// Appel au modèle local (Ollama). En cas de panne du modèle, on ne propose
	// aucune action : le Copilote ne décide jamais seul.
	llmResponse, err := c.llm(ctx, c.model, prompt)
	if err != nil {
		c.logger.Error("échec appel modèle IA local",
			"model", c.model,
			"err", err,
			"user_intent", userIntent)
		return &whitelist.ProposeAction{
			Name:   "none",
			Valid:  false,
			Reason: "modèle IA local indisponible : " + err.Error(),
		}
	}

	// Parser la réponse JSON du LLM pour extraire l'action et ses arguments.
	actionName, args := c.parseLLMResponse(llmResponse)

	// Validation stricte contre la liste blanche (critère M2)
	proposal := whitelist.ValidateProposal(actionName, args)
	if !proposal.Valid {
		c.logger.Warn("proposition LLM rejetée par liste blanche",
			"action_proposée", actionName,
			"raison", proposal.Reason)
		return &proposal
	}

	// Mode dry-run (M3) : on ne fait qu'enregistrer la proposition
	c.mu.Lock()
	dryRun := c.dryRun
	c.mu.Unlock()

	if dryRun {
		c.logger.Info("MODE DRY-RUN : proposition LLM reçue mais NON exécutée",
			"action", actionName,
			"args", args,
			"user_intent", userIntent)
		// Marquer explicitement que c'est du dry-run
		proposal.Reason = "dry-run : action proposée mais non exécutée (mode test)"
		return &proposal
	}

	// Journaliser la proposition initiale
	if c.journal != nil {
		c.journal.Write(journal.AIJournalEntry{
			Timestamp:      time.Now().UTC(),
			NodeID:         c.nodeID,
			AgentType:      "copilote",
			UserIntent:     userIntent,
			ProposedAction: actionName,
			ActionValid:    proposal.Valid,
			ActionExecuted: false,
			UserConfirmed:  false,
			Metadata: map[string]interface{}{
				"args": args,
			},
		})
	}

	// Si confirmation requise pour cette action
	if whitelist.Whitelist[actionName].RequiresConfirmation {
		confirmed := c.confirmFunc(actionName, prompt)

		// M4 : Journaliser l'acceptation ET le refus explicites
		if c.journal != nil {
			reason := "acceptée"
			if !confirmed {
				reason = "refusée par l'utilisateur"
			}
			c.journal.WriteConfirmation(c.nodeID, actionName, confirmed, reason)
		}

		if !confirmed {
			c.logger.Info("action refusée par l'utilisateur",
				"action", actionName,
				"user_intent", userIntent)
			proposal.Reason = "refusée par l'utilisateur"
			proposal.Valid = false
			return &proposal
		}
	}

	// Exécution réelle (hors dry-run, confirmation obtenue)
	err = c.executeAction(ctx, actionName, args)

	// M4 : Journaliser le résultat de l'exécution
	if c.journal != nil {
		c.journal.WriteExecution(c.nodeID, actionName, err == nil, err)
	}

	if err != nil {
		c.logger.Error("échec exécution action",
			"action", actionName,
			"err", err)
		proposal.Valid = false
		proposal.Reason = "échec exécution : " + err.Error()
		return &proposal
	}

	c.logger.Info("action exécutée avec succès",
		"action", actionName,
		"user_intent", userIntent)
	return &proposal
}

// parseLLMResponse extrait l'action et les arguments de la réponse JSON du LLM.
// Réponse mal formée → action "none" (aucune action n'est jamais inventée).
func (c *Copilote) parseLLMResponse(response string) (string, map[string]any) {
	var parsed struct {
		Action string         `json:"action"`
		Args   map[string]any `json:"args"`
	}
	if err := json.Unmarshal([]byte(strings.TrimSpace(response)), &parsed); err != nil {
		c.logger.Warn("réponse du modèle local non interprétable en JSON",
			"err", err,
			"response_len", len(response))
		return "none", map[string]any{}
	}
	if strings.TrimSpace(parsed.Action) == "" {
		return "none", map[string]any{}
	}
	return strings.TrimSpace(parsed.Action), parsed.Args
}

// executeAction exécute l'action validée via la whitelist.
// Le ctx passé aux exécuteurs porte le logger du Copilote : les actions
// whitelistées le lisent via ctx.Value("logger") (whitelist.restartService, cleanWALLogs).
func (c *Copilote) executeAction(ctx context.Context, actionName string, args map[string]any) error {
	action, ok := whitelist.Whitelist[actionName]
	if !ok {
		return fmt.Errorf("action '%s' non trouvée dans whitelist", actionName)
	}
	logger := c.logger
	if logger == nil {
		logger = slog.Default()
	}
	ctx = context.WithValue(ctx, "logger", logger)
	return action.Execute(ctx, args)
}

// defaultConfirmFunc est la fonction de confirmation par défaut (stdin).
func defaultConfirmFunc(actionName, prompt string) bool {
	fmt.Printf("CONFIRMATION REQUISE pour '%s' : %s\n", actionName, prompt)
	fmt.Print("Confirmez-vous ? [o/n] : ")
	var response string
	fmt.Scanln(&response)
	return len(response) > 0 && (response[0] == 'o' || response[0] == 'O')
}

// buildCopilotPrompt construit le prompt pour l'agent Copilote.
func buildCopilotPrompt(reader clusterreader.ClusterReader, userIntent string) string {
	status, _ := reader.GetClusterStatus()
	nodeID := "inconnu"
	if status != nil {
		nodeID = status.NodeID
	}
	return fmt.Sprintf(`Tu es l'agent "Copilote" d'Amane. Ton rôle est de proposer des actions SÛRES
basées sur l'intention de l'utilisateur.

Nœud actuel : %s
Intention utilisateur : "%s"

Actions autorisées (liste blanche stricte) :
1. restart_service : redémarre un service applicatif planté (avec confirmation utilisateur)
2. clean_wal_logs : nettoie les logs WAL selon politique prédéfinie (sans confirmation)

RÈGLES STRICTES :
- Ne propose JAMAIS d'action qui n'est pas dans cette liste
- Ne propose JAMAIS de redémarrage machine, rotation de clé, suppression de données
- Retourne UNIQUEMENT un JSON : {"action": "nom_action", "args": {...}}
- Si aucune action ne correspond, retourne {"action": "none", "args": {}}

JSON :`, nodeID, userIntent)
}
