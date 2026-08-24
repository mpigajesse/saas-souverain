// Command wgmesh maintient la configuration WireGuard intra-site d'un nœud :
// il publie l'information publique du nœud dans etcd, découvre ses pairs et
// régénère /etc/wireguard/wg0.conf (permissions 0600) à chaque changement.
//
// La clé privée reste locale : seule la clé publique circule via etcd. L'IP
// virtuelle 10.10.<site>.X est stable quel que soit le rôle du nœud (aucune
// reconfiguration réseau au failover). L'application du tunnel reste
// wg-quick up/down (opérateur) — wgmesh ne génère que la conf.
//
// Usage :
//
//	wgmesh -etcd localhost:2379 -name node-1 -site A -index 1 \
//	       -pubkey <clé publique> -privkey-file tls/.../wg_private.key \
//	       -endpoint 192.168.1.20:51820 -conf /tmp/wg0.conf
package main

import (
	"context"
	"flag"
	"fmt"
	"log"
	"os"
	"os/signal"
	"strings"
	"syscall"
	"time"

	clientv3 "go.etcd.io/etcd/client/v3"

	"github.com/amane/orchestrator-go/mesh"
)

func main() {
	etcdEPs := flag.String("etcd", "localhost:2379", "endpoints etcd v3 (séparés par des virgules)")
	name := flag.String("name", "", "nom stable du nœud (ex. node-1)")
	site := flag.String("site", "", "lettre du site : A, B, ...")
	index := flag.Int("index", 0, "dernier octet de l'IP virtuelle (1..254)")
	pubKey := flag.String("pubkey", "", "clé publique WireGuard du nœud (jamais la privée)")
	privKeyFile := flag.String("privkey-file", "", "fichier contenant la clé privée (0600)")
	endpoint := flag.String("endpoint", "", "endpoint UDP du nœud, ex. 192.168.1.20:51820")
	confPath := flag.String("conf", "/etc/wireguard/wg0.conf", "chemin du fichier de conf WireGuard")
	interval := flag.Duration("interval", 10*time.Second, "intervalle de resynchronisation")
	flag.Parse()

	if *name == "" || *site == "" || *index < 1 || *index > 254 || *pubKey == "" || *privKeyFile == "" {
		flag.Usage()
		os.Exit(2)
	}

	priv, err := os.ReadFile(*privKeyFile)
	if err != nil {
		log.Fatalf("lecture de la clé privée (%s): %v (fichier 0600, jamais loggé)", *privKeyFile, err)
	}
	privKey := strings.TrimSpace(string(priv))

	cli, err := clientv3.New(clientv3.Config{
		Endpoints:   strings.Split(*etcdEPs, ","),
		DialTimeout: 5 * time.Second,
	})
	if err != nil {
		log.Fatalf("client etcd: %v", err)
	}
	defer cli.Close()

	ctx, stop := signal.NotifyContext(context.Background(), os.Interrupt, syscall.SIGTERM)
	defer stop()

	store := mesh.EtcdKV{Cli: cli}
	info := mesh.PeerInfo{Name: *name, Site: *site, Index: *index, PublicKey: *pubKey, Endpoint: *endpoint}
	if err := mesh.Register(ctx, store, info); err != nil {
		log.Fatalf("publication du nœud: %v", err)
	}
	fmt.Printf("mesh: nœud %s publié dans etcd (%s/amane/mesh/nodes/%s)\n", *name, *etcdEPs, *name)

	local := mesh.Node{
		Name:       *name,
		Site:       *site,
		Index:      *index,
		PrivateKey: privKey,
	}
	if err := mesh.RunSync(ctx, store, *name, local, *confPath, *interval); err != nil {
		log.Fatalf("sync mesh: %v", err)
	}
	fmt.Printf("mesh: arrêté (conf finale : %s)\n", *confPath)
}
