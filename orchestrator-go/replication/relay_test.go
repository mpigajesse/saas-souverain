package replication

import "testing"

func TestRelayLocalAddAndValue(t *testing.T) {
	r := NewRelay("site-a")
	if r.LocalCount() != 0 {
		t.Fatalf("count initial = %d, want 0", r.LocalCount())
	}
	d := r.Add(5, 0)
	if d.NodeID != "site-a" || d.Seq != 1 || d.Inc != 5 {
		t.Fatalf("delta = %+v, want node site-a seq 1 inc 5", d)
	}
	r.Add(0, 2)
	if got := r.LocalCount(); got != 3 {
		t.Errorf("count = %d, want 3", got)
	}
}

func TestRelayConvergenceThreeNodesReorderedAndDuplicated(t *testing.T) {
	a := NewRelay("site-a")
	b := NewRelay("site-b")
	c := NewRelay("site-c")

	a.Add(5, 0)
	b.Add(2, 0)
	c.Add(0, 3)

	// Propagation en désordre + doublons : chaque pair est alimenté de façon
	// asynchrone et se re-fait nourrir plusieurs fois (livraison au moins une
	// fois, ordre arbitraire).
	ab := a.Outgoing()
	bc := b.Outgoing()
	ca := c.Outgoing()

	b.Accept("site-a", ab)
	c.Accept("site-b", bc)
	a.Accept("site-c", ca) // a reçoit c
	c.Accept("site-a", ab) // c reçoit a (intermédiaire via b)
	b.Accept("site-c", ca) // b reçoit c
	c.Accept("site-a", ab) // doublon (délivrance au moins une fois)
	a.Accept("site-b", bc) // a reçoit b
	b.Accept("site-c", ca) // doublon
	a.Accept("site-c", ca) // doublon
	c.Accept("site-b", bc) // c reçoit b
	b.Accept("site-a", ab) // doublon
	c.Accept("site-a", ab) // doublon
	a.Accept("site-b", bc) // doublon
	c.Accept("site-c", ca) // reflexion (delta de c vers c) : ignoré

	// Convergence : 5 + 2 - 3 = 4 partout, quel que soit l'ordre des achats.
	want := int64(4)
	if m := map[string]int64{
		"site-a": a.LocalCount(),
		"site-b": b.LocalCount(),
		"site-c": c.LocalCount(),
	}; m["site-a"] != want || m["site-b"] != want || m["site-c"] != want {
		t.Errorf("convergence = %+v, want %d partout", m, want)
	}
}

func TestRelayAckGarbageCollectsPending(t *testing.T) {
	a := NewRelay("site-a")
	b := NewRelay("site-b")

	for i := 0; i < 3; i++ {
		a.Add(1, 0)
	}
	if got := len(a.Outgoing()); got != 3 {
		t.Fatalf("outgoing = %d, want 3", got)
	}

	applied, ackSeq := b.Accept("site-a", a.Outgoing())
	if applied != 3 || ackSeq != 3 {
		t.Fatalf("accept = %d/%d, want 3/3", applied, ackSeq)
	}
	a.Confirm(ackSeq)
	if got := len(a.Outgoing()); got != 0 {
		t.Errorf("outgoing après ack = %d, want 0 (gc)", got)
	}

	// Nouvelle émission continue de fonctionner après le trim.
	d := a.Add(7, 0)
	if d.Seq != 4 {
		t.Errorf("seq = %d, want 4 (suite monotone)", d.Seq)
	}
	if got := len(a.Outgoing()); got != 1 {
		t.Errorf("outgoing = %d, want 1", got)
	}

	// Ack partiel : seul le premier delta est confirmé.
	a.Confirm(4)
	if got := len(a.Outgoing()); got != 0 {
		t.Errorf("outgoing = %d, want 0 après ack partiel", got)
	}
}

func TestRelayRejectsOwnDeltasFromPeer(t *testing.T) {
	a := NewRelay("site-a")
	a.Add(5, 0)
	own := a.Outgoing()

	// Le propriétaire se voit renvoyer son propre delta (boucle de mesh) :
	// il doit l'ignorer sans double-compter.
	applied, _ := a.Accept("site-a", own)
	if applied != 0 {
		t.Errorf("applied = %d, want 0 (reflexion ignorée)", applied)
	}
	if got := a.LocalCount(); got != 5 {
		t.Errorf("count = %d, want 5 (pas de double)", got)
	}
}

func TestRelaySeqGapAcceptedIdempotently(t *testing.T) {
	a := NewRelay("site-a")
	b := NewRelay("site-b")

	// Les deltas portent le total cumulé (seq1→1, seq2→3, seq3→6) : la
	// réception dans le désordre ou avec des trous ne compromet pas la
	// convergence — la fusion max est idempotente.
	a.Add(1, 0) // seq 1, total inc 1
	a.Add(2, 0) // seq 2, total inc 3
	a.Add(3, 0) // seq 3, total inc 6
	fromA := a.Outgoing()

	gapped := []Delta{fromA[2]} // seulement seq 3 (total 6)
	applied, ack := b.Accept("site-a", gapped)
	if applied != 1 || ack != 3 {
		t.Fatalf("accept = %d/%d, want 1/3", applied, ack)
	}
	if got := b.LocalCount(); got != 6 {
		t.Errorf("count = %d, want 6", got)
	}

	// Les seq 2 puis 1 arrivent ensuite : idempotent, résultat inchangé.
	b.Accept("site-a", fromA[1:2])
	b.Accept("site-a", fromA[0:1])
	if got := b.LocalCount(); got != 6 {
		t.Errorf("count après retour du manquant = %d, want 6", got)
	}
}

func TestMergeAssociative(t *testing.T) {
	a := NewRelay("site-a")
	b := NewRelay("site-b")
	a.Add(5, 0)
	b.Add(0, 2)
	aPos := a.Counter()
	bPos := b.Counter()

	// a ∪ b puis b ∪ a doivent donner le même résultat.
	ab := Merge(aPos, bPos)
	ba := Merge(bPos, aPos)
	if ab.Value() != ba.Value() {
		t.Errorf("merge non commutatif : ab=%d ba=%d", ab.Value(), ba.Value())
	}
	// Idempotent : Merge(ab, ab) == ab.
	if Merge(ab, ab).Value() != ab.Value() {
		t.Error("merge non idempotent")
	}
}
