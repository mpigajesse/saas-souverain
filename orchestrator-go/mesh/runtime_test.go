package mesh

import (
	"context"
	"os"
	"path/filepath"
	"runtime"
	"strings"
	"sync"
	"testing"
	"time"
)

// fakeKV imite etcd pour tester la publication/découverte sans etcd réel.
type fakeKV struct {
	mu   sync.Mutex
	data map[string]string
}

func newFakeKV() *fakeKV { return &fakeKV{data: map[string]string{}} }

func (f *fakeKV) Put(ctx context.Context, key, value string) error {
	f.mu.Lock()
	defer f.mu.Unlock()
	f.data[key] = value
	return nil
}

func (f *fakeKV) List(ctx context.Context, prefix string) (map[string]string, error) {
	f.mu.Lock()
	defer f.mu.Unlock()
	out := map[string]string{}
	for k, v := range f.data {
		if strings.HasPrefix(k, prefix) {
			out[k] = v
		}
	}
	return out, nil
}

const testPubKey = "QJhXL6nzOL+XJ20Jq6qDWW7F5tWJ4G5X2aWtCpt5WVk="

func TestRegisterAndDiscoverExcludesSelf(t *testing.T) {
	ctx := context.Background()
	store := newFakeKV()

	if err := Register(ctx, store, PeerInfo{Name: "node-1", Site: "A", Index: 1, PublicKey: testPubKey, Endpoint: "192.168.1.1:51820"}); err != nil {
		t.Fatal(err)
	}
	if err := Register(ctx, store, PeerInfo{Name: "node-2", Site: "A", Index: 2, PublicKey: testPubKey}); err != nil {
		t.Fatal(err)
	}

	peers, err := DiscoverPeers(ctx, store, "node-1")
	if err != nil {
		t.Fatal(err)
	}
	if len(peers) != 1 {
		t.Fatalf("pairs attendus=1, obtenus=%d (%+v)", len(peers), peers)
	}
	if peers[0].PublicKey != testPubKey || peers[0].Site != "A" || peers[0].Index != 2 {
		t.Fatalf("pair mal découvert: %+v", peers[0])
	}
	if _, err := peers[0].ipVirtuelle(); err != nil {
		t.Fatalf("IP virtuelle du pair invalide: %v", err)
	}
}

func TestValidatePeerInfo(t *testing.T) {
	good := PeerInfo{Name: "n", Site: "B", Index: 3, PublicKey: testPubKey}
	if err := ValidatePeerInfo(good); err != nil {
		t.Fatalf("PeerInfo valide rejetée: %v", err)
	}
	bad := []PeerInfo{
		{Name: "", Site: "A", Index: 1, PublicKey: testPubKey},
		{Name: "n", Site: "a1", Index: 1, PublicKey: testPubKey},
		{Name: "n", Site: "A", Index: 300, PublicKey: testPubKey},
		{Name: "n", Site: "A", Index: 1, PublicKey: "court"},
	}
	for i, b := range bad {
		if err := ValidatePeerInfo(b); err == nil {
			t.Fatalf("PeerInfo invalide #%d acceptée: %+v", i, b)
		}
	}
}

func TestGenerateConfigWrites0600AndSkipsWhenUnchanged(t *testing.T) {
	path := filepath.Join(t.TempDir(), "wg0.conf")
	local := Node{Name: "node-1", Site: "A", Index: 1, PrivateKey: "secret-node-1"}
	peers := []Peer{{PublicKey: testPubKey, Site: "A", Index: 2, Endpoint: "192.168.1.2:51820"}}

	changed, err := GenerateConfig(path, local, peers)
	if err != nil {
		t.Fatal(err)
	}
	if !changed {
		t.Fatal("première génération doit signaler un changement")
	}
	info, err := os.Stat(path)
	if err != nil {
		t.Fatal(err)
	}
	if runtime.GOOS != "windows" {
		if info.Mode().Perm() != 0o600 {
			t.Fatalf("permissions attendues 0600, obtenues %o", info.Mode().Perm())
		}
	}
	content, _ := os.ReadFile(path)
	s := string(content)
	if !strings.Contains(s, "PrivateKey = secret-node-1") {
		t.Fatal("clé privée absente de la conf")
	}
	if !strings.Contains(s, "AllowedIPs = 10.10.1.2/32") {
		t.Fatal("AllowedIPs /32 du pair absent")
	}
	if !strings.Contains(s, "PersistentKeepalive = 25") {
		t.Fatal("PersistentKeepalive 25 absent")
	}
	if strings.Contains(s, testPubKey+"\n") && !strings.Contains(s, "PublicKey = "+testPubKey) {
		t.Fatal("clé publique mal rendue")
	}

	changed, err = GenerateConfig(path, local, peers)
	if err != nil {
		t.Fatal(err)
	}
	if changed {
		t.Fatal("aucun changement → ne doit pas réécrire ni signaler")
	}
}

func TestGenerateConfigNoDangerousRouting(t *testing.T) {
	path := filepath.Join(t.TempDir(), "wg0.conf")
	local := Node{Name: "n", Site: "A", Index: 1, PrivateKey: "sk"}
	// Peer avec Endpoint mais AllowedIPs réduit : Endpoint ne route RIEN,
	// seul AllowedIPs définit le trafic encapsulé. Pas de 0.0.0.0/0 possible.
	if _, err := GenerateConfig(path, local, []Peer{{PublicKey: testPubKey, Site: "A", Index: 2}}); err != nil {
		t.Fatalf("conf légitime rejetée: %v", err)
	}
	content, _ := os.ReadFile(path)
	if strings.Contains(string(content), "0.0.0.0/0") {
		t.Fatal("routage global interdit dans la conf générée")
	}
}

func TestRunSyncRegeneratesOnPeerChange(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	store := newFakeKV()
	path := filepath.Join(t.TempDir(), "wg0.conf")
	local := Node{Name: "node-1", Site: "A", Index: 1, PrivateKey: "sk-1"}

	done := make(chan error, 1)
	go func() { done <- RunSync(ctx, store, "node-1", local, path, 20*time.Millisecond) }()

	// Nouveau pair enregistré après le démarrage → la boucle doit le découvrir.
	if err := Register(ctx, store, PeerInfo{Name: "node-2", Site: "A", Index: 2, PublicKey: testPubKey}); err != nil {
		t.Fatal(err)
	}
	deadline := time.After(2 * time.Second)
	for {
		content, err := os.ReadFile(path)
		if err == nil && strings.Contains(string(content), "10.10.1.2/32") {
			break
		}
		select {
		case <-deadline:
			t.Fatal("le nouveau pair n'a jamais été découvert dans wg0.conf")
		default:
			time.Sleep(10 * time.Millisecond)
		}
	}
	cancel()
	if err := <-done; err != nil {
		t.Fatalf("RunSync doit s'arrêter proprement à l'annulation: %v", err)
	}
}
