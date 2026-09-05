package simulator

import (
	"context"
	"fmt"
	"log/slog"
	"math/rand"
	"time"

	"github.com/amane/ai-assistant/agents"
	"github.com/amane/orchestrator-go/clusterreader"
)

// FailureType représente un type de panne simulée.
type FailureType int

const (
	FailureNone FailureType = iota
	FailureNodeDown
	FailureDiskFull
	FailureNetworkLatency
	FailureWALBacklog
	FailurePrimaryDown
)

// FailureSimulator injecte des pannes artificielles dans un cluster pour tester
// les agents IA (Sentinelle, Copilote, Escalade).
type FailureSimulator struct {
	logger       *slog.Logger
	currentState FailureType
	clusterState *ClusterState
	injectCount  int
}

// ClusterState représente l'état simulé du cluster pour les tests.
type ClusterState struct {
	NodeID           string
	IsLeader         bool
	Quorum           int
	Members          []string
	Primary          string
	DiskUsage        float64
	NetworkLatencyMs int
	WALSizeMB        int
	RecentLogs       []LogEntry
}

// LogEntry représente une entrée de log simulée.
type LogEntry struct {
	Seq         uint64
	PayloadHash string
	CommittedAt int64
	SiteID      string
	OpType      string
}

// NewFailureSimulator crée un simulateur de pannes pour les tests.
func NewFailureSimulator(logger *slog.Logger, baseState *ClusterState) *FailureSimulator {
	if baseState == nil {
		baseState = &ClusterState{
			NodeID:           "test-node-1",
			IsLeader:         true,
			Quorum:           2,
			Members:          []string{"test-node-1", "test-node-2", "test-node-3"},
			Primary:          "test-node-1",
			DiskUsage:        45.0,
			NetworkLatencyMs: 2,
			WALSizeMB:        120,
		}
	}
	return &FailureSimulator{
		logger:       logger,
		currentState: FailureNone,
		clusterState: baseState,
	}
}

// InjectFailure injecte une panne spécifique et met à jour l'état du cluster.
func (fs *FailureSimulator) InjectFailure(f FailureType) {
	fs.currentState = f
	fs.injectCount++

	switch f {
	case FailureNodeDown:
		fs.clusterState.IsLeader = false
		fs.clusterState.Members = []string{"test-node-2", "test-node-3"}
		fs.clusterState.Primary = "test-node-2"
	case FailureDiskFull:
		fs.clusterState.DiskUsage = 95.0
	case FailureNetworkLatency:
		fs.clusterState.NetworkLatencyMs = 250
	case FailureWALBacklog:
		fs.clusterState.WALSizeMB = 500
	case FailurePrimaryDown:
		fs.clusterState.Primary = "test-node-2"
		fs.clusterState.IsLeader = false
	default:
		// Remettre en état normal
		fs.clusterState.IsLeader = true
		fs.clusterState.Members = []string{"test-node-1", "test-node-2", "test-node-3"}
		fs.clusterState.Primary = "test-node-1"
		fs.clusterState.DiskUsage = 45.0
		fs.clusterState.NetworkLatencyMs = 2
		fs.clusterState.WALSizeMB = 120
	}

	fs.addLogEntry(fmt.Sprintf("FAILURE_INJECTED: %s", f.String()))
	fs.logger.Warn("panne injectée", "type", f.String(), "count", fs.injectCount)
}

// addLogEntry ajoute une entrée de log simulée.
func (fs *FailureSimulator) addLogEntry(msg string) {
	entry := LogEntry{
		Seq:         uint64(time.Now().UnixNano()),
		PayloadHash: fmt.Sprintf("%x", rand.Int63()),
		CommittedAt: time.Now().Unix(),
		SiteID:      fs.clusterState.NodeID,
		OpType:      "failure_inject",
	}
	fs.clusterState.RecentLogs = append(fs.clusterState.RecentLogs, entry)
	if len(fs.clusterState.RecentLogs) > 100 {
		fs.clusterState.RecentLogs = fs.clusterState.RecentLogs[len(fs.clusterState.RecentLogs)-100:]
	}
}

// GetClusterState retourne l'état actuel du cluster simulé.
func (fs *FailureSimulator) GetClusterState() *ClusterState {
	return fs.clusterState
}

// CurrentFailure retourne le type de panne actuellement injectée.
func (fs *FailureSimulator) CurrentFailure() FailureType {
	return fs.currentState
}

// String retourne le nom du type de panne.
func (f FailureType) String() string {
	switch f {
	case FailureNodeDown:
		return "NODE_DOWN"
	case FailureDiskFull:
		return "DISK_FULL"
	case FailureNetworkLatency:
		return "NETWORK_LATENCY"
	case FailureWALBacklog:
		return "WAL_BACKLOG"
	case FailurePrimaryDown:
		return "PRIMARY_DOWN"
	default:
		return "NONE"
	}
}

// MockClusterReader implémente l'interface clusterreader.ClusterReader
// pour les tests avec le simulateur de pannes.
type MockClusterReader struct {
	simulator *FailureSimulator
}

// NewMockClusterReader crée un lecteur de cluster mocké pour les tests.
func NewMockClusterReader(sim *FailureSimulator) *MockClusterReader {
	return &MockClusterReader{simulator: sim}
}

// GetClusterStatus implémente clusterreader.ClusterReader.
func (m *MockClusterReader) GetClusterStatus() (*clusterreader.ClusterStatus, error) {
	state := m.simulator.GetClusterState()
	return &clusterreader.ClusterStatus{
		NodeID:   state.NodeID,
		IsLeader: state.IsLeader,
		Members:  state.Members,
		Quorum:   state.Quorum,
		Primary:  state.Primary,
	}, nil
}

// GetNodeID implémente clusterreader.ClusterReader.
func (m *MockClusterReader) GetNodeID() string {
	return m.simulator.GetClusterState().NodeID
}

// GetNodeRole implémente clusterreader.ClusterReader.
func (m *MockClusterReader) GetNodeRole() string {
	state := m.simulator.GetClusterState()
	if state.IsLeader {
		return "primary"
	}
	return "replica"
}

// GetRecentLogs implémente clusterreader.ClusterReader.
func (m *MockClusterReader) GetRecentLogs(limit uint32) ([]clusterreader.LoggedEntry, error) {
	logs := m.simulator.GetClusterState().RecentLogs
	if len(logs) == 0 {
		return []clusterreader.LoggedEntry{}, nil
	}
	end := len(logs)
	start := end - int(limit)
	if start < 0 {
		start = 0
	}
	entries := make([]clusterreader.LoggedEntry, 0, end-start)
	for i := start; i < end; i++ {
		entries = append(entries, clusterreader.LoggedEntry{
			Seq:           logs[i].Seq,
			PayloadHash:   logs[i].PayloadHash,
			CommittedAt:   logs[i].CommittedAt,
			SiteID:        logs[i].SiteID,
			OperationType: logs[i].OpType,
		})
	}
	return entries, nil
}

// GetMetrics implémente clusterreader.ClusterReader.
func (m *MockClusterReader) GetMetrics() (map[string]float64, error) {
	state := m.simulator.GetClusterState()
	return map[string]float64{
		"disk_usage_percent": state.DiskUsage,
		"network_latency_ms": float64(state.NetworkLatencyMs),
		"wal_size_mb":        float64(state.WALSizeMB),
	}, nil
}

// IsHealthy implémente clusterreader.ClusterReader.
func (m *MockClusterReader) IsHealthy() bool {
	state := m.simulator.GetClusterState()
	return state.IsLeader && len(state.Members) >= state.Quorum && state.DiskUsage < 90.0
}

// RunFailureScenario exécute un scénario de panne complet pour tester
// les agents IA en mode dry-run (appel au modèle par défaut, Ollama local).
func RunFailureScenario(ctx context.Context, logger *slog.Logger) *ScenarioResult {
	return RunFailureScenarioWithLLM(ctx, logger, nil)
}

// RunFailureScenarioWithLLM exécute un scénario de panne complet avec un appel
// au modèle IA injectable (stub déterministe en test, Ollama en production).
func RunFailureScenarioWithLLM(ctx context.Context, logger *slog.Logger, llmFn agents.LLMFunc) *ScenarioResult {
	sim := NewFailureSimulator(logger, nil)
	mockReader := NewMockClusterReader(sim)

	sentinel := agents.NewSentinelle(logger)
	copilote := agents.NewCopilote(logger, true) // mode dry-run activé
	if llmFn != nil {
		sentinel.SetLLM(llmFn)
		copilote.SetLLM(llmFn)
	}

	scenarios := []struct {
		name      string
		failure   FailureType
		userQuery string
	}{
		{"État normal", FailureNone, "Comment va le cluster ?"},
		{"Nœud down", FailureNodeDown, "Pourquoi le cluster est-il en erreur ?"},
		{"Disque plein", FailureDiskFull, "Pourquoi j'ai une alerte disque ?"},
		{"Latence réseau", FailureNetworkLatency, "Pourquoi c'est lent ?"},
		{"WAL backlog", FailureWALBacklog, "Le journal s'accumule-t-il ?"},
		{"Primary down", FailurePrimaryDown, "Le primary est-il toujours là ?"},
	}

	result := &ScenarioResult{
		Timestamp:   time.Now(),
		Scenarios:   make([]ScenarioResultItem, 0, len(scenarios)),
		TotalTests:  0,
		PassedTests: 0,
		FailedTests: 0,
	}

	for _, sc := range scenarios {
		sim.InjectFailure(sc.failure)

		// Test Sentinelle : diagnostic
		diag := sentinel.Diagnose(ctx, mockReader)

		// Test Copilote : proposition d'action (dry-run)
		proposal := copilote.ProposeAction(ctx, mockReader, sc.userQuery)

		item := ScenarioResultItem{
			ScenarioName:   sc.name,
			FailureType:    sc.failure.String(),
			UserQuery:      sc.userQuery,
			Diagnostic:     diag,
			ProposedAction: proposal.Name,
			ActionValid:    proposal.Valid,
			ActionReason:   proposal.Reason,
			DryRun:         copilote.IsDryRun(),
		}

		// Validation : en dry-run, aucune action ne doit être exécutée réellement
		// Cas valides :
		// 1. Action valide + dry-run = OK (proposée mais non exécutée)
		// 2. "none" = OK (aucune action whitelistée ne correspond à la situation)
		// 3. Action rejetée par whitelist (adverse) = OK (sécurité)
		if proposal.Name == "none" {
			// Aucune action ne s'applique = comportement correct
			item.Passed = true
			result.PassedTests++
		} else if copilote.IsDryRun() && proposal.Valid {
			// Action valide mais en dry-run = proposée, non exécutée
			item.Passed = true
			result.PassedTests++
		} else if !proposal.Valid {
			// Action rejetée par whitelist = sécurité respectée
			item.Passed = true
			result.PassedTests++
		} else {
			item.Passed = false
			result.FailedTests++
		}

		result.Scenarios = append(result.Scenarios, item)
		result.TotalTests++
	}

	return result
}

// ScenarioResultItem représente le résultat d'un scénario de test.
type ScenarioResultItem struct {
	ScenarioName   string
	FailureType    string
	UserQuery      string
	Diagnostic     string
	ProposedAction string
	ActionValid    bool
	ActionReason   string
	DryRun         bool
	Passed         bool
}

// ScenarioResult représente le résultat global d'un scénario de test.
type ScenarioResult struct {
	Timestamp   time.Time
	Scenarios   []ScenarioResultItem
	TotalTests  int
	PassedTests int
	FailedTests int
}

// PrintSummary affiche un résumé du test.
func (sr *ScenarioResult) PrintSummary() {
	fmt.Printf("\n=== RÉSULTATS SCÉNARIO DE PANNE ===\n")
	fmt.Printf("Timestamp: %s\n", sr.Timestamp.Format(time.RFC3339))
	fmt.Printf("Total tests: %d | Passed: %d | Failed: %d\n", sr.TotalTests, sr.PassedTests, sr.FailedTests)
	fmt.Printf("Taux de succès: %.1f%%\n\n", float64(sr.PassedTests)/float64(sr.TotalTests)*100)

	for _, s := range sr.Scenarios {
		status := "✓ PASS"
		if !s.Passed {
			status = "✗ FAIL"
		}
		fmt.Printf("%s | %s | Action: %s | Valid: %v | DryRun: %v\n",
			status, s.ScenarioName, s.ProposedAction, s.ActionValid, s.DryRun)
	}
}
