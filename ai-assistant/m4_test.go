package ai_assistant

import (
	"context"
	"log/slog"
	"testing"
	"time"

	"github.com/amane/ai-assistant/agents"
	"github.com/amane/ai-assistant/journal"
	"github.com/amane/ai-assistant/simulator"
)

// TestSentinelleM4 valide l'étape M4 : confirmation utilisateur réelle +
// journalisation de l'acceptation ET du refus.
func TestSentinelleM4(t *testing.T) {
	ctx := context.Background()
	logger := slog.New(slog.NewTextHandler(&testWriter{t: t}, &slog.HandlerOptions{Level: slog.LevelInfo}))

	sim := simulator.NewFailureSimulator(logger, nil)
	mockReader := simulator.NewMockClusterReader(sim)

	// Journal writer pour capturer les entrées
	journalEntries := make([]journal.AIJournalEntry, 0)
	testWriter := &captureJournalWriter{entries: &journalEntries}

	// Test 1 : Copilote avec action nécessitant confirmation + acceptation
	copiloteAccept := agents.NewCopilote(logger, false) // dryRun = false
	copiloteAccept.SetLLM(stubLLM)                      // modèle stub déterministe
	copiloteAccept.SetJournal(testWriter, "test-node-1")
	copiloteAccept.SetConfirmFunc(func(actionName, prompt string) bool {
		return true // Simule acceptation utilisateur
	})

	// Injecter une panne qui devrait déclencher restart_service
	sim.InjectFailure(simulator.FailureNodeDown)

	proposal := copiloteAccept.ProposeAction(ctx, mockReader, "Redémarre le service qui a planté")

	if !proposal.Valid {
		t.Errorf("M4 : action devrait être validée et acceptée, mais rejetée : %s", proposal.Reason)
	}

	// Vérifier la journalisation d'acceptation
	acceptEntries := filterEntries(journalEntries, "copilote", "restart_service", true)
	if len(acceptEntries) == 0 {
		t.Error("M4 : aucune entrée de journal pour l'acceptation")
	} else {
		entry := acceptEntries[0]
		if !entry.UserConfirmed {
			t.Error("M4 : l'acceptation utilisateur n'a pas été journalisée (UserConfirmed=false)")
		}
		if entry.ActionExecuted {
			t.Error("M4 : l'action ne devrait pas être marquée exécutée avant l'exécution réelle")
		}
	}

	// Réinitialiser pour le test de refus
	journalEntries = make([]journal.AIJournalEntry, 0)

	// Test 2 : Copilote avec action nécessitant confirmation + REFUS
	copiloteReject := agents.NewCopilote(logger, false)
	copiloteReject.SetLLM(stubLLM) // modèle stub déterministe
	copiloteReject.SetJournal(testWriter, "test-node-1")
	copiloteReject.SetConfirmFunc(func(actionName, prompt string) bool {
		return false // Simule refus utilisateur
	})

	proposal2 := copiloteReject.ProposeAction(ctx, mockReader, "Redémarre le service qui a planté")

	if proposal2.Valid {
		t.Error("M4 : action devrait être invalide après refus utilisateur")
	}

	// Vérifier la journalisation de refus
	rejectEntries := filterEntries(journalEntries, "copilote", "restart_service", false)
	if len(rejectEntries) == 0 {
		t.Error("M4 : aucune entrée de journal pour le refus")
	} else {
		entry := rejectEntries[0]
		if entry.UserConfirmed {
			t.Error("M4 : le refus utilisateur n'a pas été journalisé correctement (UserConfirmed=true)")
		}
		if entry.ActionExecuted {
			t.Error("M4 : action refusée ne devrait pas être marquée exécutée")
		}
	}

	// Test 3 : Action sans confirmation requise (clean_wal_logs) - pas de prompt
	sim.InjectFailure(simulator.FailureDiskFull)
	journalEntries = make([]journal.AIJournalEntry, 0)

	copiloteNoConfirm := agents.NewCopilote(logger, false)
	copiloteNoConfirm.SetLLM(stubLLM) // modèle stub déterministe
	copiloteNoConfirm.SetJournal(testWriter, "test-node-1")

	proposal3 := copiloteNoConfirm.ProposeAction(ctx, mockReader, "Nettoie les logs WAL")

	if !proposal3.Valid {
		t.Errorf("M4 : clean_wal_logs devrait passer sans confirmation : %s", proposal3.Reason)
	}

	t.Log("M4 atteint : confirmation utilisateur testée (accepte + refuse), journalisation vérifiée")
}

// captureJournalWriter capture les entrées de journal pour les tests.
type captureJournalWriter struct {
	entries *[]journal.AIJournalEntry
}

func (w *captureJournalWriter) Write(entry journal.AIJournalEntry) error {
	*w.entries = append(*w.entries, entry)
	return nil
}

func (w *captureJournalWriter) WriteConfirmation(nodeID, actionName string, confirmed bool, reason string) error {
	entry := journal.AIJournalEntry{
		NodeID:             nodeID,
		Timestamp:          time.Now().UTC(),
		AgentType:          "copilote",
		UserIntent:         "confirmation required for: " + actionName,
		ProposedAction:     actionName,
		ActionValid:        true,
		ActionExecuted:     false,
		UserConfirmed:      confirmed,
		ConfirmationReason: reason,
	}
	*w.entries = append(*w.entries, entry)
	return nil
}

func (w *captureJournalWriter) WriteExecution(nodeID, actionName string, success bool, err error) error {
	entry := journal.AIJournalEntry{
		NodeID:         nodeID,
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
	*w.entries = append(*w.entries, entry)
	return nil
}

func filterEntries(entries []journal.AIJournalEntry, agentType, actionName string, confirmed bool) []journal.AIJournalEntry {
	result := make([]journal.AIJournalEntry, 0)
	for _, e := range entries {
		if e.AgentType == agentType && e.ProposedAction == actionName && e.UserConfirmed == confirmed {
			result = append(result, e)
		}
	}
	return result
}
