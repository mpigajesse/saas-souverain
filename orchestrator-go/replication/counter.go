// Package replication implémente la réplication CRDT delta multi-site.
//
// Cas d'usage Amane : quantités de stock (PN-Counter). Chaque nœud émet un
// delta (changement local) propagé aux autres sites ; la fusion reste
// commutative, associative et idempotente (state-based, CvRDT).
package replication

import "fmt"

// NodeID identifie un nœud/site émetteur dans le CRDT.
type NodeID string

// Delta est le changement propagé d'un nœud vers les autres (delta-CRDT).
// Il ne transporte que le différentiel local, jamais l'état complet.
type Delta struct {
	NodeID NodeID
	Inc    int64  // incrément (entrées stock)
	Dec    int64  // décrément (sorties stock)
	Seq    uint64 // séquence locale de l'émetteur (détection de trous / gc)
}

func (d Delta) String() string {
	return fmt.Sprintf("delta{node=%s inc=%d dec=%d}", d.NodeID, d.Inc, d.Dec)
}

// Empty indique si le delta est vide (rien à propager).
func (d Delta) Empty() bool { return d.Inc == 0 && d.Dec == 0 }

// Counter est un PN-Counter (Positive-Negative) répliqué.
// Inc/Dec conservent les contributions par nœud : le max par nœud à la fusion
// rend le merge idempotent et commutatif, garantissant la convergence.
type Counter struct {
	Inc map[NodeID]int64
	Dec map[NodeID]int64
}

// NewCounter retourne un compteur vide prêt à l'emploi.
func NewCounter() *Counter {
	return &Counter{Inc: make(map[NodeID]int64), Dec: make(map[NodeID]int64)}
}

// Apply applique un delta localement (émission ou réception) et retourne un
// nouveau compteur. Le compteur source n'est pas modifié (immutabilité).
func (c *Counter) Apply(d Delta) *Counter {
	out := NewCounter()
	for n, v := range c.Inc {
		out.Inc[n] = v
	}
	for n, v := range c.Dec {
		out.Dec[n] = v
	}
	if d.Inc != 0 {
		out.Inc[d.NodeID] += d.Inc
	}
	if d.Dec != 0 {
		out.Dec[d.NodeID] += d.Dec
	}
	return out
}

// Value retourne la valeur nette du compteur (stock courant).
func (c *Counter) Value() int64 {
	var total int64
	for _, v := range c.Inc {
		total += v
	}
	for _, v := range c.Dec {
		total -= v
	}
	return total
}

// Merge fusionne b dans a (max par nœud) et retourne le résultat. Les
// opérandes ne sont pas modifiés. Merge est commutatif, associatif, idempotent.
func Merge(a, b *Counter) *Counter {
	out := NewCounter()
	for n, v := range a.Inc {
		out.Inc[n] = v
	}
	for n, v := range b.Inc {
		out.Inc[n] = max(out.Inc[n], v)
	}
	for n, v := range a.Dec {
		out.Dec[n] = v
	}
	for n, v := range b.Dec {
		out.Dec[n] = max(out.Dec[n], v)
	}
	return out
}

// Clone retourne une copie indépendante du compteur.
func (c *Counter) Clone() *Counter {
	out := NewCounter()
	for n, v := range c.Inc {
		out.Inc[n] = v
	}
	for n, v := range c.Dec {
		out.Dec[n] = v
	}
	return out
}
