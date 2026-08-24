package consensus

import (
	"testing"
)

func TestComputeQuorum(t *testing.T) {
	cases := []struct{ total, want int }{
		{1, 1},
		{2, 2},
		{3, 2},
		{4, 3},
		{5, 3},
		{0, 0},
		{-1, 0},
	}
	for _, c := range cases {
		if got := ComputeQuorum(c.total); got != c.want {
			t.Errorf("ComputeQuorum(%d) = %d, want %d", c.total, got, c.want)
		}
	}
}

func TestFingerprintStableAndNeverRaw(t *testing.T) {
	pub := []byte("x25519-public-key-example")
	f1 := Fingerprint(pub)
	f2 := Fingerprint(pub)
	if f1 != f2 {
		t.Errorf("empreinte non deterministe: %s vs %s", f1, f2)
	}
	if len(f1) != 16 { // 8 octets en hex
		t.Errorf("empreinte inattendue: %q (len %d)", f1, len(f1))
	}
	if f1 == Fingerprint([]byte("other-key")) {
		t.Error("deux cles differentes ont la meme empreinte")
	}
}