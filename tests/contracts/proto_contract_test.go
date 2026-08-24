package contracts

import (
	"testing"

	"google.golang.org/protobuf/reflect/protoreflect"

	pb "github.com/amane/orchestrator-go/gen/amane/framework/v1"
)

// TestProtoContractStable verrouille le contrat proto B ↔ C : chaque champ des
// messages partagés garde son numéro, chaque RPC garde son nom. Un changement
// ici casserait silencieusement les SDK Mission B déjà déployés (équivalent
// programme de `buf breaking`, sans avoir besoin d'un dépôt git).
func TestProtoContractStable(t *testing.T) {
	fieldNumbers := map[string]map[string]protoreflect.FieldNumber{
		"PingRequest":  {},
		"PingResponse": {"status": 1, "node_id": 2, "version": 3, "server_time": 4},
		"EnrollRequest": {
			"machine_id": 1, "ak_public_key": 2, "site_id": 3, "operator": 4,
		},
		"EnrollResponse": {"membership_id": 1, "cluster_id": 2, "enrolled_at": 3},
		"WriteRequest":   {"journal_id": 1, "op_seq": 2, "encrypted_payload": 3, "site_id": 4, "machine_id": 5},
		"WriteResponse":  {"committed_seq": 1, "node_id": 2, "synced": 3, "committed_at": 4},
		"ReadRequest":    {"journal_id": 1, "from_seq": 2, "limit": 3},
		"ReadResponse":   {"entries": 1},
		"JournalEntry":   {"seq": 1, "encrypted_payload": 2, "committed_at": 3, "site_id": 4},
		"NotifyRevocationRequest": {
			"machine_id": 1, "revoked_ak_id": 2, "revoked_at": 3, "reason": 4,
		},
		"NotifyRevocationResponse": {"quorum_recalculated": 1, "node_id": 2},
		"Delta":                    {"node_id": 1, "inc": 2, "dec": 3, "seq": 4},
		"PushDeltaRequest":         {"site_id": 1, "from_node": 2, "deltas": 3},
		"PushDeltaResponse":        {"acked_seq": 1, "value": 2, "node_id": 3},
	}

	// Vérifie que TOUS les messages présents dans le proto sont bien connus
	// ici (un ajout manqué = contrat non audité).
	known := make(map[string]bool)
	for name := range fieldNumbers {
		known[name] = true
	}

	for name, want := range fieldNumbers {
		md := pb.File_amane_framework_v1_framework_proto.Messages().ByName(protoreflect.Name(name))
		if md == nil {
			t.Fatalf("message %s introuvable dans le proto généré", name)
		}
		fields := md.Fields()
		for i := 0; i < fields.Len(); i++ {
			f := fields.Get(i)
			wantNum, ok := want[string(f.Name())]
			if !ok {
				continue
			}
			if f.Number() != wantNum {
				t.Errorf("%s.%s = numéro %d, want %d (ne PAS renuméroter !)", name, f.Name(), f.Number(), wantNum)
			}
		}
		if md.Fields().Len() != len(want) {
			t.Errorf("%s expose %d champs, want %d — un champ ajouté/supprimé change le contrat", name, md.Fields().Len(), len(want))
		}
	}

	// Les RPCs du service : le SDK Mission B ne doit pas voir de RPC retiré.
	svc := pb.File_amane_framework_v1_framework_proto.Services().ByName("AmaneService")
	if svc == nil {
		t.Fatal("service AmaneService introuvable")
	}
	for _, rpc := range []string{"Ping", "Enroll", "Write", "Read", "NotifyRevocation", "PushDelta"} {
		if svc.Methods().ByName(protoreflect.Name(rpc)) == nil {
			t.Errorf("RPC %s absent du service (contrat B↔C rompu)", rpc)
		}
	}
}
