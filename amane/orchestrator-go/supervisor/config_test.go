package supervisor

import (
	"testing"
	"time"
)

// TestSupervisorIsPrimaryRole : les deux formes de rôle primaire Patroni que le
// détecteur accepte (master = forme historique, primary = forme moderne).
func TestSupervisorIsPrimaryRole(t *testing.T) {
	s := &Supervisor{}
	for _, ok := range []struct {
		role string
		want bool
	}{
		{"primary", true},
		{"master", true},
		{"replica", false},
		{"standby", false},
		{"", false},
	} {
		if got := s.isPrimaryRole(ok.role); got != ok.want {
			t.Errorf("isPrimaryRole(%q) = %v, want %v", ok.role, got, ok.want)
		}
	}
}

// TestConfigValidate : erreurs de cohérence du paramétrage (endpoints, scope,
// nœuds). La valeur default scope ("amane") ne doit pas masquer une scope vide
// AVANT Defaults.
func TestConfigValidate(t *testing.T) {
	valid := &Config{
		EtcdEndpoints: []string{"localhost:2379"},
		Scope:         "amane",
		Nodes:         []Node{{Name: "pg-1", PatroniURL: "http://localhost:8008"}},
	}
	if err := valid.Validate(); err != nil {
		t.Fatalf("config valide rejetée: %v", err)
	}

	for name, c := range map[string]*Config{
		"endpoints vides": {},
		"scope vide":      {EtcdEndpoints: []string{"localhost:2379"}, Nodes: []Node{{Name: "pg-1"}}},
		"aucun nœud":      {EtcdEndpoints: []string{"localhost:2379"}, Scope: "amane"},
	} {
		if err := c.Validate(); err == nil {
			t.Errorf("%s : Validate a accepté une config invalide", name)
		}
	}

	// Defaults() applique les valeurs par défaut (budget failover < 5 s).
	d := &Config{}
	d.Defaults()
	if d.Scope != "amane" || d.StaleConfirm != 2 || d.ProbeInterval != 500*time.Millisecond {
		t.Errorf("Defaults incohérents: %+v", d)
	}
}
