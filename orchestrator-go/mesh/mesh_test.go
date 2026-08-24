package mesh

import (
	"os"
	"strings"
	"testing"
)

func conf3Noeuds() Config {
	return Config{
		Node: Node{Name: "node-1", Site: "A", Index: 1, PrivateKey: "cle-privee-1"},
		Peers: []Peer{
			{PublicKey: "cle-publique-2", Site: "A", Index: 2, Endpoint: "192.168.1.20:51820"},
			{PublicKey: "cle-publique-3", Site: "B", Index: 1, Endpoint: "vps.example.com:51820"},
		},
	}
}

func TestRenderMesh3Noeuds(t *testing.T) {
	out, err := conf3Noeuds().Render()
	if err != nil {
		t.Fatalf("render: %v", err)
	}
	for _, want := range []string{
		"[Interface]",
		"Address = 10.10.1.1/24",
		"ListenPort = 51820",
		"[Peer]",
		"AllowedIPs = 10.10.1.2/32",
		"AllowedIPs = 10.10.2.1/32",
		"PersistentKeepalive = 25",
		"Endpoint = 192.168.1.20:51820",
	} {
		if !strings.Contains(out, want) {
			t.Errorf("config sans %q:\n%s", want, out)
		}
	}
}

func TestRenderJamaisRoutageGlobal(t *testing.T) {
	out, err := conf3Noeuds().Render()
	if err != nil {
		t.Fatal(err)
	}
	if strings.Contains(out, "0.0.0.0/0") || strings.Contains(out, "::/0") {
		t.Errorf("AllowedIPs global détecté dans:\n%s", out)
	}
}

func TestValidateAllowedIPs(t *testing.T) {
	if err := ValidateAllowedIPs("10.10.1.2/32"); err != nil {
		t.Errorf("/32 ciblé refusé: %v", err)
	}
	for _, bad := range []string{"0.0.0.0/0", "::/0"} {
		if err := ValidateAllowedIPs(bad); !strings.Contains(err.Error(), "routage global interdit") {
			t.Errorf("%s accepté: %v", bad, err)
		}
	}
}

func TestAdressesParSite(t *testing.T) {
	cases := []struct {
		site string
		idx  int
		want string
	}{
		{"A", 1, "10.10.1.1/24"},
		{"A", 2, "10.10.1.2/24"},
		{"B", 1, "10.10.2.1/24"},
		{"Z", 254, "10.10.26.254/24"},
	}
	for _, c := range cases {
		got, err := (Node{Name: "n", Site: c.site, Index: c.idx}).Address()
		if err != nil || got != c.want {
			t.Errorf("site %s idx %d = %q, %v — want %q", c.site, c.idx, got, err, c.want)
		}
	}
	if _, err := (Node{Name: "n", Site: "ab", Index: 1}).Address(); err == nil {
		t.Error("site multi-lettre accepté")
	}
	if _, err := (Node{Name: "n", Site: "A", Index: 0}).Address(); err == nil {
		t.Error("index 0 accepté")
	}
}

func TestRenderRejetteCleVide(t *testing.T) {
	_, err := Config{Node: Node{Name: "n", Site: "A", Index: 1}}.Render()
	if err == nil || !strings.Contains(err.Error(), "PrivateKey") {
		t.Errorf("clé privée vide acceptée: %v", err)
	}
}

func TestWriteConfPermissions(t *testing.T) {
	path := t.TempDir() + "/wg0.conf"
	if err := WriteConf(path, "[Interface]\n"); err != nil {
		t.Fatal(err)
	}
	info, err := os.Stat(path)
	if err != nil {
		t.Fatal(err)
	}
	if perm := info.Mode().Perm(); perm != 0o600 {
		t.Errorf("permissions = %o, want 600 (clé privée protégée)", perm)
	}
}
