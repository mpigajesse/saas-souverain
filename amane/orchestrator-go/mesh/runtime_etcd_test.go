package mesh

import (
	"context"
	"fmt"
	"os"
	"testing"
	"time"

	clientv3 "go.etcd.io/etcd/client/v3"
)

// TestEtcdKVPutAndList — intégration du KV etcd réel (publication + découverte
// du mesh). Skip sans AMANE_TEST_ETCD (même convention que consensus/).
// couvre EtcdKV.Put + EtcdKV.List (branches heureuses).
func TestEtcdKVPutAndList(t *testing.T) {
	ep := os.Getenv("AMANE_TEST_ETCD")
	if ep == "" {
		t.Skip("AMANE_TEST_ETCD non défini — etcd réel requis")
	}

	cli, err := clientv3.New(clientv3.Config{
		Endpoints:   []string{ep},
		DialTimeout: 5 * time.Second,
	})
	if err != nil {
		t.Fatalf("client etcd: %v", err)
	}
	defer cli.Close()

	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	store := EtcdKV{Cli: cli}
	prefix := fmt.Sprintf("/amane/mesh/test/%d/", time.Now().UnixNano())
	if err := store.Put(ctx, prefix+"node-a", `{"name":"node-a","site":"A"}`); err != nil {
		t.Fatalf("put: %v", err)
	}
	if err := store.Put(ctx, prefix+"node-b", `{"name":"node-b","site":"A"}`); err != nil {
		t.Fatalf("put: %v", err)
	}

	got, err := store.List(ctx, prefix)
	if err != nil {
		t.Fatalf("list: %v", err)
	}
	if len(got) != 2 {
		t.Errorf("list = %d entrées, want 2", len(got))
	}
	if got[prefix+"node-a"] != `{"name":"node-a","site":"A"}` {
		t.Errorf("round-trip node-a corrompu: %s", got[prefix+"node-a"])
	}
}
