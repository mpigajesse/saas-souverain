package supervisor

import (
	"testing"
	"time"
)

func testCfg() *Config {
	c := &Config{
		Scope:         "amane",
		ProbeInterval: time.Second,
		ProbeTimeout:  100 * time.Millisecond,
		HeartbeatTTL:  4 * time.Second,
		StaleConfirm:  2,
		CoolDown:      30 * time.Second,
	}
	c.Defaults()
	return c
}

func snap(leader string, nodes map[string]nodeHealth) snapshot {
	return snapshot{now: time.Now(), leader: leader, nodes: nodes}
}

// TestDetectorHealthy : leader joignable → aucun déclenchement.
func TestDetectorHealthy(t *testing.T) {
	d := newDetector(testCfg())
	a := d.tick(snap("primary", map[string]nodeHealth{
		"primary": {restOK: true, role: "primary", hbPrimary: true},
		"standby": {restOK: true, role: "replica", sync: "sync", hbPrimary: true},
	}))
	if a.trigger || a.guarded {
		t.Fatalf("leader sain ne doit rien déclencher: %+v", a)
	}
}

// TestDetectorCrash : leader SIGKILLé (REST down + heartbeat stale) →
// après StaleConfirm ticks consécutifs, déclenchement sur le candidat sync.
func TestDetectorCrash(t *testing.T) {
	d := newDetector(testCfg())
	nodes := map[string]nodeHealth{
		"primary": {restOK: false, hbPrimary: false},
		"standby": {restOK: true, role: "replica", sync: "sync", hbPrimary: true},
	}
	for i := 0; i < 2; i++ {
		a := d.tick(snap("primary", nodes))
		if i < 1 && a.trigger {
			t.Fatalf("déclenchement trop tôt (%d/2)", i+1)
		}
	}
	a := d.tick(snap("primary", nodes))
	if !a.trigger {
		t.Fatalf("crash non déclenché après StaleConfirm: %+v", a)
	}
	if a.candidate != "standby" {
		t.Fatalf("candidat attendu=standby, obtenu=%q", a.candidate)
	}
	if a.leader != "primary" {
		t.Fatalf("leader attendu=primary, obtenu=%q", a.leader)
	}
}

// TestDetectorPartitionNeverForces : leader REST down MAIS heartbeat frais (= primary vivant)
// (partition réseau, leader vivant) → garde : jamais de forçage, même longtemps.
func TestDetectorPartitionNeverForces(t *testing.T) {
	d := newDetector(testCfg())
	nodes := map[string]nodeHealth{
		"primary": {restOK: false, hbPrimary: true}, // vivant côté etcd, injoignable en REST
		"standby": {restOK: true, role: "replica", sync: "sync", hbPrimary: true},
	}
	for i := 0; i < 10; i++ {
		a := d.tick(snap("primary", nodes))
		if a.trigger {
			t.Fatal("partition réseau ne doit JAMAIS déclencher un forçage")
		}
		if !a.guarded {
			t.Fatalf("tick %d: la garde anti-partition ne s'est pas levée", i)
		}
	}
}

// TestDetectorNoCandidate : leader mort mais aucun réplica joignable → pas de
// déclenchement (on n'invente pas de candidat).
func TestDetectorNoCandidate(t *testing.T) {
	d := newDetector(testCfg())
	nodes := map[string]nodeHealth{
		"primary": {restOK: false, hbPrimary: false},
		"standby": {restOK: false, hbPrimary: false}, // standby absent aussi
	}
	for i := 0; i < 5; i++ {
		a := d.tick(snap("primary", nodes))
		if a.trigger {
			t.Fatalf("déclenchement sans candidat: %+v", a)
		}
	}
}

// TestDetectorLeaderUnknown : identité du leader inconnue → après StaleConfirm,
// déclenche quand un réplica est joignable (leader = celui qu'on n'atteint pas).
func TestDetectorLeaderUnknown(t *testing.T) {
	d := newDetector(testCfg())
	nodes := map[string]nodeHealth{
		"standby": {restOK: true, role: "replica", sync: "sync", hbPrimary: true},
	}
	d.tick(snap("", nodes))
	d.tick(snap("", nodes))
	a := d.tick(snap("", nodes))
	if !a.trigger || a.candidate != "standby" {
		t.Fatalf("leader inconnu + candidat disponible devrait déclencher: %+v", a)
	}
}

// TestDetectorReset : le retour à la normale (leader joignable) réarme le compteur.
func TestDetectorReset(t *testing.T) {
	d := newDetector(testCfg())
	down := map[string]nodeHealth{
		"primary": {restOK: false, hbPrimary: false},
		"standby": {restOK: true, role: "replica", sync: "sync", hbPrimary: true},
	}
	a := d.tick(snap("primary", down))
	if a.trigger {
		t.Fatal("premier tick ne déclenche pas")
	}
	up := map[string]nodeHealth{
		"primary": {restOK: true, role: "primary", hbPrimary: true},
		"standby": {restOK: true, role: "replica", sync: "sync", hbPrimary: true},
	}
	a = d.tick(snap("primary", up))
	// légère latence de re-arm : le prochain tick ne doit pas déclencher
	a = d.tick(snap("primary", down))
	if a.trigger {
		t.Fatalf("le re-arm n'a pas réinitialisé le compteur: %+v", a)
	}
}
