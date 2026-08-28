package replication

import "testing"

// TestValue vérifie la valeur nette d'un PN-Counter.
func TestValue(t *testing.T) {
	c := NewCounter()
	c = c.Apply(Delta{NodeID: "site-a", Inc: 10})
	c = c.Apply(Delta{NodeID: "site-a", Inc: 5})
	c = c.Apply(Delta{NodeID: "site-b", Dec: 3})
	if got, want := c.Value(), int64(12); got != want {
		t.Errorf("Value = %d, want %d", got, want)
	}
}

// TestDeltaEmpty : un delta nul ne doit rien changer.
func TestDeltaEmpty(t *testing.T) {
	c := NewCounter().Apply(Delta{NodeID: "site-a"})
	zero := Delta{}
	if !zero.Empty() {
		t.Error("delta nul non détecté")
	}
	if got := c.Value(); got != 0 {
		t.Errorf("Value après delta vide = %d, want 0", got)
	}
}

// TestMergeCommutativeAssociativeIdempotent valide les 3 lois du merge.
func TestMergeCommutativeAssociativeIdempotent(t *testing.T) {
	a := NewCounter().Apply(Delta{NodeID: "a", Inc: 4}).Apply(Delta{NodeID: "b", Dec: 1})
	b := NewCounter().Apply(Delta{NodeID: "b", Dec: 1}).Apply(Delta{NodeID: "c", Inc: 2})
	c := NewCounter().Apply(Delta{NodeID: "c", Inc: 2})

	ab := Merge(a, b)
	ba := Merge(b, a)
	if ab.Value() != ba.Value() || ab.Value() != 5 {
		t.Errorf("commutativité : %d vs %d, want 5", ab.Value(), ba.Value())
	}

	assoc1 := Merge(Merge(a, b), c)
	assoc2 := Merge(a, Merge(b, c))
	if assoc1.Value() != assoc2.Value() {
		t.Errorf("associativité : %d vs %d", assoc1.Value(), assoc2.Value())
	}

	aa := Merge(a, a)
	if aa.Value() != a.Value() {
		t.Errorf("idempotence : %d vs %d", aa.Value(), a.Value())
	}
}

// TestConvergence3Sites est l'indicateur de succès du jalon : des écritures
// concurrentes désordonnées sur 3 nœuds doivent converger vers le même état,
// quel que soit l'ordre d'arrivée des deltas (0 conflit non résolu).
func TestConvergence3Sites(t *testing.T) {
	// Deltas concurrents, chacun produit sur son site.
	deltas := []Delta{
		{NodeID: "site-a", Inc: 30},
		{NodeID: "site-b", Dec: 12},
		{NodeID: "site-c", Inc: 7},
		{NodeID: "site-a", Inc: 5},
		{NodeID: "site-c", Dec: 3},
		{NodeID: "site-b", Inc: 2},
	}
	want := int64(30 - 12 + 7 + 5 - 3 + 2) // 29

	// Chaque site accumule les deltas dans un ordre d'arrivée différent.
	orders := [][]int{
		{0, 1, 2, 3, 4, 5},
		{5, 4, 3, 2, 1, 0},
		{2, 0, 5, 1, 3, 4},
	}
	for i, order := range orders {
		c := NewCounter()
		for _, idx := range order {
			c = c.Apply(deltas[idx])
		}
		if got := c.Value(); got != want {
			t.Fatalf("site %d n'a pas convergé : Value = %d, want %d", i, got, want)
		}
	}

	// Fusion état-complet entre sites (scénario partage de bout en bout).
	s1 := NewCounter().Apply(deltas[0]).Apply(deltas[3])
	s2 := NewCounter().Apply(deltas[1]).Apply(deltas[5])
	s3 := NewCounter().Apply(deltas[2]).Apply(deltas[4])
	if got := Merge(Merge(s1, s2), s3).Value(); got != want {
		t.Errorf("merge complet = %d, want %d", got, want)
	}
}

// TestMergeNeMuttePasLesOperandes : le merge ne modifie jamais ses entrées.
func TestMergeNeMuttePasLesOperandes(t *testing.T) {
	a := NewCounter().Apply(Delta{NodeID: "a", Inc: 1})
	b := NewCounter().Apply(Delta{NodeID: "b", Inc: 2})
	beforeA, beforeB := a.Value(), b.Value()

	_ = Merge(a, b)

	if a.Value() != beforeA || b.Value() != beforeB {
		t.Error("Merge a muté ses opérandes")
	}
}

// TestApplyNeMuttePasLaSource : Apply retourne un nouveau compteur.
func TestApplyNeMuttePasLaSource(t *testing.T) {
	a := NewCounter().Apply(Delta{NodeID: "a", Inc: 3})
	b := a.Apply(Delta{NodeID: "a", Inc: 2})
	if a.Value() != 3 {
		t.Errorf("source mutée : %d, want 3", a.Value())
	}
	if b.Value() != 5 {
		t.Errorf("résultat = %d, want 5", b.Value())
	}
}