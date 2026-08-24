package replication

import (
	"errors"
	"strconv"
	"sync"
	"time"
)

var (
	// ErrInvalidPayload : payload chiffré vide (jamais de clair côté C).
	ErrInvalidPayload = errors.New("payload chiffré vide")
	// ErrInvalidSite : site_id manquant.
	ErrInvalidSite = errors.New("site_id manquant")
)

// JournalEntry est une entrée du journal multi-site (Interface 3, chemin
// d'écriture B ↔ C). Le payload est toujours chiffré par Mission A — le
// journal ne stocke et ne log jamais de clair.
type JournalEntry struct {
	Seq              uint64
	EncryptedPayload []byte
	CommittedAt      time.Time
	SiteID           string
}

// WriteResult décrit le résultat d'une écriture.
type WriteResult struct {
	CommittedSeq uint64
	Synced       bool // réplication synchrone confirmée (locale en dev)
	Replayed     bool // entrée déjà committée (rejeu idempotent)
}

// Journal est un journal append-only par journal_id, convergent par
// idempotence : un même (site_id, op_seq) ne peut être committé qu'une fois.
// Thread-safe.
type Journal struct {
	mu      sync.Mutex
	entries map[string][]JournalEntry // journal_id -> entrées ordonnées par Seq
	byRef   map[string]uint64         // "journal_id/site_id/op_seq" -> Seq committé
	nextSeq map[string]uint64         // journal_id -> prochain Seq
}

// NewJournal retourne un journal vide.
func NewJournal() *Journal {
	return &Journal{
		entries: make(map[string][]JournalEntry),
		byRef:   make(map[string]uint64),
		nextSeq: make(map[string]uint64),
	}
}

// Append committe une entrée chiffrée dans le journal. Un rejeu (même
// journal_id + site_id + op_seq) retourne le seq déjà committé sans dupliquer.
func (j *Journal) Append(journalID string, opSeq uint64, encryptedPayload []byte, siteID string) (WriteResult, error) {
	if journalID == "" {
		return WriteResult{}, errors.New("journal_id manquant")
	}
	if len(encryptedPayload) == 0 {
		return WriteResult{}, ErrInvalidPayload
	}
	if siteID == "" {
		return WriteResult{}, ErrInvalidSite
	}

	j.mu.Lock()
	defer j.mu.Unlock()

	ref := journalID + "/" + siteID + "/" + strconv.FormatUint(opSeq, 10)
	if seq, ok := j.byRef[ref]; ok {
		return WriteResult{CommittedSeq: seq, Synced: true, Replayed: true}, nil
	}

	seq := j.nextSeq[journalID]
	entry := JournalEntry{
		Seq:              seq,
		EncryptedPayload: encryptedPayload,
		CommittedAt:      time.Now().UTC(),
		SiteID:           siteID,
	}
	j.entries[journalID] = append(j.entries[journalID], entry)
	j.byRef[ref] = seq
	j.nextSeq[journalID] = seq + 1
	return WriteResult{CommittedSeq: seq, Synced: true}, nil
}

// Read retourne les entrées de Seq >= fromSeq, au plus limit entrées
// (limit 0 = pas de limite). Journal inconnu → liste vide, sans erreur.
func (j *Journal) Read(journalID string, fromSeq uint64, limit uint32) []JournalEntry {
	j.mu.Lock()
	defer j.mu.Unlock()

	entries := j.entries[journalID]
	var out []JournalEntry
	for _, e := range entries {
		if e.Seq < fromSeq {
			continue
		}
		out = append(out, e)
		if limit > 0 && uint32(len(out)) >= limit {
			break
		}
	}
	return out
}

// SeqOf retourne le prochain seq à committer pour un journal (stats/tests).
func (j *Journal) SeqOf(journalID string) uint64 {
	j.mu.Lock()
	defer j.mu.Unlock()
	return j.nextSeq[journalID]
}
