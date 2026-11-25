//go:build ignore

// render-haproxy imprime la configuration HAProxy produite par le VRAI code
// haproxy.Rendre à partir des membres Patroni réels du laboratoire ha-lab lus
// dans etcd. Utilisé par tools/ha/ha04-haproxy.sh pour piloter un HAProxy réel.
//
// Usage : HA_ETCD=10.77.0.11:2379 go run render-haproxy.go <port>
package main

import (
	"context"
	"fmt"
	"os"
	"strconv"

	"github.com/amane/orchestrator-go/consensus"
	"github.com/amane/orchestrator-go/dcs"
	"github.com/amane/orchestrator-go/haproxy"
)

func main() {
	port := 6432
	if len(os.Args) > 1 {
		if p, err := strconv.Atoi(os.Args[1]); err == nil {
			port = p
		}
	}
	ep := os.Getenv("HA_ETCD")
	if ep == "" {
		ep = "10.77.0.11:2379"
	}
	scope := os.Getenv("HA_SCOPE")
	if scope == "" {
		scope = "amane-lab"
	}
	cli, err := consensus.NewClient([]string{ep})
	if err != nil {
		fmt.Fprintln(os.Stderr, "client:", err)
		os.Exit(1)
	}
	defer cli.Close()
	membres, err := dcs.Membres(context.Background(), cli, scope)
	if err != nil {
		fmt.Fprintln(os.Stderr, "membres:", err)
		os.Exit(1)
	}
	fmt.Print(haproxy.Rendre(membres, port))
}