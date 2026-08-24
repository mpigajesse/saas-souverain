package consensus

// ComputeQuorum calcule la majorité requise pour un cluster de `total` nœuds
// (formule Raft) : avec 3 membres → 2, avec 2 → 2, avec 1 → 1.
func ComputeQuorum(total int) int {
	if total <= 0 {
		return 0
	}
	return total/2 + 1
}
