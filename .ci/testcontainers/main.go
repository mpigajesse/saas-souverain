// Harness CI — etcd via testcontainers puis exécution de la commande passée.
//
// Usage (depuis ce répertoire, module github.com/amane/ci/testcontainers) :
//
//	go run . -- go test ./... -count=1 -race
//	go run . -workdir ../../tests -- go test ./... -count=1 -race
//
// La commande est exécutée avec AMANE_TEST_ETCD pointant sur le container etcd
// (format "127.0.0.1:<port>"), à l'identique de la stack dev (quay.io/coreos/
// etcd:v3.5.5) ; son code de sortie est propagé (0 = tout vert).
package main

import (
	"context"
	"flag"
	"fmt"
	"log"
	"os"
	"os/exec"
	"time"

	"github.com/testcontainers/testcontainers-go"
	"github.com/testcontainers/testcontainers-go/wait"
)

const etcdImage = "quay.io/coreos/etcd:v3.5.5"

func main() {
	fs := flag.NewFlagSet("testenv", flag.ExitOnError)
	workdir := fs.String("workdir", "", "répertoire de travail de la commande (défaut : répertoire courant)")
	fs.Parse(os.Args[1:])

	rest := fs.Args()
	if len(rest) > 0 && rest[0] == "--" {
		rest = rest[1:]
	}
	if len(rest) == 0 {
		log.Fatal("usage : go run . [-workdir <dir>] -- <commande> [args…]")
	}

	ctx := context.Background()
	etcd, err := testcontainers.GenericContainer(ctx, testcontainers.GenericContainerRequest{
		ContainerRequest: testcontainers.ContainerRequest{
			Image:        etcdImage,
			ExposedPorts: []string{"2379/tcp", "2380/tcp"},
			Cmd: []string{
				"etcd", "--name", "etcd-ci",
				"--data-dir", "/etcd-data",
				"--advertise-client-urls", "http://0.0.0.0:2379",
				"--listen-client-urls", "http://0.0.0.0:2379",
				"--listen-peer-urls", "http://0.0.0.0:2380",
				"--initial-advertise-peer-urls", "http://0.0.0.0:2380",
				"--initial-cluster", "etcd-ci=http://0.0.0.0:2380",
				"--initial-cluster-state", "new",
			},
			WaitingFor: wait.ForLog("ready to serve client requests").
				WithStartupTimeout(90 * time.Second),
		},
		Started: true,
	})
	if err != nil {
		log.Fatalf("démarrage etcd (testcontainers) : %v", err)
	}
	defer func() {
		if err := etcd.Terminate(ctx); err != nil {
			log.Printf("terminate etcd : %v", err)
		}
	}()

	endpoint, err := etcd.PortEndpoint(ctx, "2379/tcp", "")
	if err != nil {
		log.Fatalf("mapped port etcd : %v", err)
	}
	if err := os.Setenv("AMANE_TEST_ETCD", endpoint); err != nil {
		log.Fatalf("set AMANE_TEST_ETCD : %v", err)
	}
	log.Printf("etcd prêt à %s — lancement : %v", endpoint, rest)

	cmd := exec.Command(rest[0], rest[1:]...)
	if *workdir != "" {
		cmd.Dir = *workdir
	}
	cmd.Stdin = os.Stdin
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	cmd.Env = os.Environ()
	if err := cmd.Run(); err != nil {
		if exit, ok := err.(*exec.ExitError); ok {
			os.Exit(exit.ExitCode())
		}
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}