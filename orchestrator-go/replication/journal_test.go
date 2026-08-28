package replication

import (
	"errors"
	"testing"
)

func payload(n byte) []byte { return []byte("payload-chiffre-" + string([]byte{n})) }

func TestJournalAppendMonotone(t *testing.T) {
	j := NewJournal()
	r1, err := j.Append("stock", 1, payload(1), "site-a")
	if err != nil || r1.CommittedSeq != 0 || r1.Synced != true || r1.Replayed != false {
		t.Fatalf("append1 = %+v, %v", r1, err)
	}
	r2, err := j.Append("stock", 2, payload(2), "site-b")
	if err != nil || r2.CommittedSeq != 1 {
		t.Fatalf("append2 = %+v, %v", r2, err)
	}
	if got := j.SeqOf("stock"); got != 2 {
		t.Errorf("SeqOf = %d, want 2", got)
	}
}

func TestJournalRejeuIdempotent(t *testing.T) {
	j := NewJournal()
	r1, _ := j.Append("stock", 7, payload(1), "site-a")
	// Même (site, op_seq) rejoué → même seq committé, pas de doublon.
	r2, err := j.Append("stock", 7, payload(1), "site-a")
	if err != nil {
		t.Fatalf("rejeu: %v", err)
	}
	if !r2.Replayed || r2.CommittedSeq != r1.CommittedSeq {
		t.Errorf("rejeu = %+v, want replayed seq %d", r2, r1.CommittedSeq)
	}
	if got := j.SeqOf("stock"); got != 1 {
		t.Errorf("doublon ajouté : SeqOf = %d, want 1", got)
	}
	if n := len(j.Read("stock", 0, 0)); n != 1 {
		t.Errorf("nb entrées = %d, want 1", n)
	}
}

func TestJournalValidations(t *testing.T) {
	j := NewJournal()
	if _, err := j.Append("", 1, payload(1), "site-a"); err == nil {
		t.Error("journal_id vide accepté")
	}
	if _, err := j.Append("stock", 1, nil, "site-a"); !errors.Is(err, ErrInvalidPayload) {
		t.Errorf("payload vide: %v", err)
	}
	if _, err := j.Append("stock", 1, payload(1), ""); !errors.Is(err, ErrInvalidSite) {
		t.Errorf("site vide: %v", err)
	}
}

func TestJournalReadPagination(t *testing.T) {
	j := NewJournal()
	for i := 0; i < 5; i++ {
		if _, err := j.Append("journal-j", uint64(i+1), payload(byte(i)), "site-a"); err != nil {
			t.Fatal(err)
		}
	}
	if got := len(j.Read("journal-j", 0, 0)); got != 5 {
		t.Errorf("read all = %d, want 5", got)
	}
	if got := j.Read("journal-j", 2, 0); len(got) != 3 || got[0].Seq != 2 {
		t.Errorf("read from=2 = %d, first seq %d", len(got), got[0].Seq)
	}
	if got := j.Read("journal-j", 0, 2); len(got) != 2 {
		t.Errorf("read limit=2 = %d, want 2", len(got))
	}
	if got := j.Read("inconnu", 0, 0); got != nil {
		t.Errorf("journal inconnu = %v, want nil", got)
	}
}

// TestJournalConcurrentAppend : des écritures concurrentes ne perdent pas
// d'entrée (séquences toutes distinctes, comptage correct).
func TestJournalConcurrentAppend(t *testing.T) {
	j := NewJournal()
	const writers = 8
	const perWriter = 100
	done := make(chan error, writers)
	for w := 0; w < writers; w++ {
		go func(site byte) {
			for i := 0; i < perWriter; i++ {
				if _, err := j.Append("stock", uint64(i+1), payload(site), "site-"+string([]byte{site})); err != nil {
					done <- err
					return
				}
			}
			done <- nil
		}(byte('a' + w))
	}
	for w := 0; w < writers; w++ {
		if err := <-done; err != nil {
			t.Fatal(err)
		}
	}
	if got := j.SeqOf("stock"); got != writers*perWriter {
		t.Errorf("SeqOf = %d, want %d (aucune perte)", got, writers*perWriter)
	}
}