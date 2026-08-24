package replication

import (
	"testing"
)

// Benchmarks micro (job CI `benchmark`) : chemins chauds purs, sans etcd.
// Seuils de non-régression : docs/bench/baseline.json (ns/op).

// BenchmarkRelayAdd — Add(inc, dec) : total + pending + seq monotone.
func BenchmarkRelayAdd(b *testing.B) {
	r := NewRelay("site-a")
	for i := 0; i < b.N; i++ {
		r.Add(1, 0)
	}
}

// BenchmarkRelayAccept — fusion max-par-nœud d'un delta reçu d'un pair.
func BenchmarkRelayAccept(b *testing.B) {
	r := NewRelay("site-a")
	deltas := []Delta{{NodeID: "site-b", Inc: 5, Dec: 1, Seq: 1}}
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		r.Accept("site-b", deltas)
	}
}

// BenchmarkJournalWrite — append d'une entrée chiffrée (noyau du chemin Write).
func BenchmarkJournalWrite(b *testing.B) {
	j := NewJournal()
	payload := make([]byte, 512)
	for i := 0; i < b.N; i++ {
		j.Append("stock", uint64(i), payload, "site-a")
	}
}
