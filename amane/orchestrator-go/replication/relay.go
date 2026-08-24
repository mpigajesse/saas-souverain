package replication

import (
	"sync"
)

// Relay orchestre la réplication multi-site d'un Counter (delta-CRDT).
//
// Un delta porte le TOTAL cumulé d'un nœud (Inc/Dec depuis son origine), pas
// un simple incrément. La fusion par max par nœud émetteur est alors
// commutative, associative et idempotente : les doublons, les
// réordonnancements et les trous de séquence ne compromettent jamais la
// convergence — c'est la garantie AP du state-based CRDT.
//
// Le séquence (Seq) ne sert qu'au côté émetteur : ack du pair + gc du
// pending (fuite de mémoire évitée).
type Relay struct {
	mu       sync.Mutex
	node     NodeID
	counter  *Counter
	incTotal int64   // total cumulé local (incréments de notre nœud)
	decTotal int64   // total cumulé local (décréments de notre nœud)
	seq      uint64  // séquence locale croissante (générée par Add)
	pending  []Delta // deltas locaux émis, dans l'ordre (position i ↦ seq i+1)
	base     int     // index du premier delta non confirmé par le pair
}

// NewRelay retourne un Relay lié à un nœud CRDT.
func NewRelay(node NodeID) *Relay {
	return &Relay{
		node:    node,
		counter: NewCounter(),
	}
}

// NodeID identifie le nœud CRDT local.
func (r *Relay) NodeID() NodeID { return r.node }

// LocalCount retourne la valeur nette locale (stock courant).
func (r *Relay) LocalCount() int64 {
	r.mu.Lock()
	defer r.mu.Unlock()
	return r.counter.Value()
}

// Counter retourne une copie de l'état local (immutable).
func (r *Relay) Counter() *Counter {
	r.mu.Lock()
	defer r.mu.Unlock()
	return r.counter.Clone()
}

// Add applique une variation locale (inc/dec) : met à jour l'état local et
// le total cumulé du nœud, met le delta en pending avec sa séquence, et le
// retourne pour propagation.
func (r *Relay) Add(inc, dec int64) Delta {
	r.mu.Lock()
	defer r.mu.Unlock()
	r.incTotal += inc
	r.decTotal += dec
	r.seq++
	d := Delta{NodeID: r.node, Inc: r.incTotal, Dec: r.decTotal, Seq: r.seq}
	out := r.counter.Clone()
	out.Inc[r.node] = r.incTotal
	out.Dec[r.node] = r.decTotal
	r.counter = out
	r.pending = append(r.pending, d)
	return d
}

// Accept applique des deltas reçus d'un pair. La fusion max par nœud est
// idempotente et commutative : doublons, ré-ordonnancement et trous de
// séquence sont sans effet sur la convergence. Un delta auto-émis depuis un
// pair (reflexion de mesh) est ignoré. Retourne le nombre de deltas appliqués
// et la seq max de l'émetteur `fromNode` traitée (utilisée pour l'ack).
func (r *Relay) Accept(fromNode NodeID, deltas []Delta) (int, uint64) {
	r.mu.Lock()
	defer r.mu.Unlock()
	applied := 0
	ackSeq := uint64(0)
	for _, d := range deltas {
		if d.NodeID == r.node {
			// Reflexion d'un delta émis localement : rien à faire.
			continue
		}
		merged := r.counter.Clone()
		merged.Inc[d.NodeID] = max(merged.Inc[d.NodeID], d.Inc)
		merged.Dec[d.NodeID] = max(merged.Dec[d.NodeID], d.Dec)
		r.counter = merged
		applied++
		if d.NodeID == fromNode && d.Seq > ackSeq {
			ackSeq = d.Seq
		}
	}
	return applied, ackSeq
}

// Outgoing retourne les deltas locaux non confirmés (à envoyer au pair).
func (r *Relay) Outgoing() []Delta {
	r.mu.Lock()
	defer r.mu.Unlock()
	if r.base >= len(r.pending) {
		return nil
	}
	out := make([]Delta, len(r.pending)-r.base)
	copy(out, r.pending[r.base:])
	return out
}

// Confirm confirme que le pair a appliqué jusqu'à la séquence ackSeq de nos
// deltas locaux : on les retire du pending (fuite de mémoire évitée).
func (r *Relay) Confirm(ackSeq uint64) {
	r.mu.Lock()
	defer r.mu.Unlock()
	if ackSeq <= uint64(r.base) {
		return
	}
	if ackSeq >= uint64(len(r.pending)) {
		r.pending = nil
		r.base = 0
		return
	}
	r.base = int(ackSeq)
}
