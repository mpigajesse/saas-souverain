package ai_assistant

import (
	"testing"

	"github.com/amane/ai-assistant/whitelist"
)

// TestSentinelleM2 valide l'étape M2 : liste blanche d'actions.
// Critère : 100% des actions non whitelistées rejetées, y compris tentatives
// adverses explicites (ex. prompt qui essaie de faire proposer rm -rf, un redémarrage
// non whitelisté, etc.).
func TestSentinelleM2(t *testing.T) {
	// Test 1 : action connue whitelistée devrait être validée
	_, ok := whitelist.Validate("restart_service")
	if !ok {
		t.Error("échec : 'restart_service' devrait être whitelisté")
		return
	}

	// Test 2 : actions non whitelistées devraient TOUTES être rejetées
	adversarialActions := []string{
		"rm -rf /",
		"reboot machine",
		"rotate_key",
		"get_secret",
		"poweroff",
		"format_disk",
		"sudo rm -rf /var",
		"delete_database",
		"change_cluster_config",
		"disable_fencing",
		"bypass_quorum",
		"write_unencrypted",
		"exfiltrate_data",
		"inject_malicious_code",
		"scan_ports",
		"exploit_vulnerability",
	}

	for _, badAction := range adversarialActions {
		_, ok := whitelist.Validate(badAction)
		if ok {
			t.Errorf("M2 échoué : action '%s' n'aurait pas devoir être whitelistée", badAction)
		}
	}

	// Test 3 : validation de proposition LLM avec actions adverses
	proposals := []string{
		"rm -rf /",
		"reboot machine",
		"restart_service", // devrait passer
		"clean_wal_logs",  // devrait passer
		"rotate_key",
	}

	for _, prop := range proposals {
		result := whitelist.ValidateProposal(prop, nil)
		if !result.Valid {
			t.Logf("Action '%s' correctement rejetée : %s", prop, result.Reason)
		} else if prop == "rm -rf /" || prop == "rotate_key" || prop == "reboot machine" {
			t.Errorf("M2 échoué : action '%s' aurait été acceptéealors qu'elle ne devrait pas l'être", prop)
		} else {
			t.Logf("Action '%s' correctement acceptée", prop)
		}
	}

	t.Log("M2 terminé : validation liste blanche terminée")
}
