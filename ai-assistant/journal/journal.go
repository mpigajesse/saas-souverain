package journal

import (
	"encoding/json"
	"log/slog"
	"time"
)

// AIJournalEntry représente une entrée de journal pour les interactions IA.
// Ces entrées sont écrites dans ss-journal (append-only) pour traçabilité complète.
type AIJournalEntry struct {
	Timestamp          time.Time              `json:"timestamp"`
	NodeID             string                 `json:"node_id"`
	AgentType          string                 `json:"agent_type"`      // "sentinelle", "copilote", "escalade"
	UserIntent         string                 `json:"user_intent"`     // ce que l'utilisateur a demandé
	ProposedAction     string                 `json:"proposed_action"` // action proposée par l'IA
	ActionValid        bool                   `json:"action_valid"`    // validée par whitelist
	ActionExecuted     bool                   `json:"action_executed"` // réellement exécutée
	UserConfirmed      bool                   `json:"user_confirmed"`  // confirmation explicite (si requise)
	ConfirmationReason string                 `json:"confirmation_reason,omitempty"`
	Error              string                 `json:"error,omitempty"`
	Metadata           map[string]interface{} `json:"metadata,omitempty"`
}

// AIJournalWriter écrit les interactions IA dans le journal append-only.
// Interface pour permettre l'injection de dépendances (tests).
type AIJournalWriter interface {
	Write(entry AIJournalEntry) error
	WriteConfirmation(nodeID, actionName string, confirmed bool, reason string) error
	WriteExecution(nodeID, actionName string, success bool, err error) error
}

// SimpleJournalWriter implémente AIJournalWriter avec un logger slog.
// En production, ceci écrirait vers le mécanisme ss-journal (Mission A).
type SimpleJournalWriter struct {
	logger *slog.Logger
	nodeID string
}

// NewSimpleJournalWriter crée un writer de journal pour les interactions IA.
func NewSimpleJournalWriter(logger *slog.Logger, nodeID string) *SimpleJournalWriter {
	return &SimpleJournalWriter{
		logger: logger,
		nodeID: nodeID,
	}
}

// Write écrit une entrée complète d'interaction IA.
func (w *SimpleJournalWriter) Write(entry AIJournalEntry) error {
	entry.NodeID = w.nodeID
	entry.Timestamp = time.Now().UTC()

	// Sérialisation JSON pour stockage append-only
	data, err := json.Marshal(entry)
	if err != nil {
		return err
	}

	// Log structuré pour traçabilité
	w.logger.Info("IA journal entry",
		"timestamp", entry.Timestamp.Format(time.RFC3339),
		"agent_type", entry.AgentType,
		"user_intent", entry.UserIntent,
		"proposed_action", entry.ProposedAction,
		"action_valid", entry.ActionValid,
		"action_executed", entry.ActionExecuted,
		"user_confirmed", entry.UserConfirmed,
		"confirmation_reason", entry.ConfirmationReason,
		"error", entry.Error,
		"payload", string(data), // payload complet pour ss-journal
	)
	return nil
}

// WriteConfirmation journalise explicitement l'acceptation OU le refus de l'utilisateur.
// Critère M4 : les DEUX cas (accepte ET refuse) doivent être journalisés.
// L'entrée ne marque PAS l'action comme exécutée : la confirmation et l'exécution
// sont deux événements distincts (ActionExecuted est journalisé par WriteExecution).
func (w *SimpleJournalWriter) WriteConfirmation(nodeID, actionName string, confirmed bool, reason string) error {
	entry := AIJournalEntry{
		NodeID:             w.nodeID,
		Timestamp:          time.Now().UTC(),
		AgentType:          "copilote",
		UserIntent:         "confirmation required for: " + actionName,
		ProposedAction:     actionName,
		ActionValid:        true,
		ActionExecuted:     false, // la confirmation n'est pas l'exécution
		UserConfirmed:      confirmed,
		ConfirmationReason: reason,
	}
	return w.Write(entry)
}

// WriteExecution journalise le résultat de l'exécution d'une action.
func (w *SimpleJournalWriter) WriteExecution(nodeID, actionName string, success bool, err error) error {
	entry := AIJournalEntry{
		NodeID:         w.nodeID,
		Timestamp:      time.Now().UTC(),
		AgentType:      "copilote",
		UserIntent:     "execution of: " + actionName,
		ProposedAction: actionName,
		ActionValid:    true,
		ActionExecuted: true,
		UserConfirmed:  true,
		Error:          "",
	}
	if err != nil {
		entry.Error = err.Error()
	}
	return w.Write(entry)
}
