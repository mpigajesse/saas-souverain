package main

import (
	"context"
	"errors"
	"flag"
	"fmt"
	"log/slog"
	"net"
	"os"
	"os/signal"
	"strings"
	"syscall"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/reflection"

	"github.com/amane/orchestrator-go/consensus"
	pb "github.com/amane/orchestrator-go/gen/amane/framework/v1"
	"github.com/amane/orchestrator-go/grpcserver"
	"github.com/amane/orchestrator-go/replication"
	"github.com/amane/orchestrator-go/supervisor"
	"github.com/amane/orchestrator-go/telemetry"
)

func main() {
	listen := flag.String("listen", ":50051", "adresse d'écoute gRPC")
	tlsCert := flag.String("tls-cert", "", "certificat serveur (mTLS si tls-ca fourni)")
	tlsKey := flag.String("tls-key", "", "clé privée du certificat serveur")
	tlsCA := flag.String("tls-ca", "", "CA client (active mTLS)")
	etcdEndpoints := flag.String("etcd-endpoints", "localhost:2379", "endpoints etcd v3 (séparés par des virgules)")
	supervisorEnabled := flag.Bool("supervisor", false, "active le superviseur de failover (option C : détection crash + POST /failover Patroni)")
	patroniScope := flag.String("patroni-scope", "amane", "scope Patroni suivi (DCS /service/<scope>/)")
	patroniNodes := flag.String("patroni-nodes", "", "nœuds Patroni 'nom@http://host:port,...' (ex: postgres-primary@http://localhost:8008)")
	replicateTo := flag.String("replicate-to", "", "pairs de réplication multi-site 'site-id@host:port,...' (PushDelta périodique)")
	replicateInterval := flag.Duration("replicate-interval", time.Second, "intervalle de propagation des deltas vers les pairs")
	flag.Parse()

	logger := slog.New(slog.NewJSONHandler(os.Stdout, &slog.HandlerOptions{Level: slog.LevelInfo}))
	slog.SetDefault(logger)

	var opts []grpc.ServerOption
	if *tlsCert != "" && *tlsKey != "" {
		creds, err := grpcserver.LoadServerCredentials(*tlsCert, *tlsKey, *tlsCA)
		if err != nil {
			slog.Error("chargement TLS impossible", "err", err)
			os.Exit(1)
		}
		opts = append(opts, grpc.Creds(creds))
		slog.Info("TLS 1.3 activé", "mtls", *tlsCA != "")
	} else {
		slog.Warn("TLS désactivé — usage DEV uniquement, jamais en prod")
	}

	opts = append(opts,
		grpc.ChainUnaryInterceptor(
			telemetry.UnaryServerInterceptor(logger),
			grpcserver.LoggingUnaryInterceptor(logger),
			grpcserver.RecoveryUnaryInterceptor(logger),
		),
	)

	ctx, stop := signal.NotifyContext(context.Background(), os.Interrupt, syscall.SIGTERM)
	defer stop()

	// --- Consensus (etcd v3) : membership + quorum + leadership/fencing ---
	etcdCli, err := consensus.NewClient(strings.Split(*etcdEndpoints, ","))
	if err != nil {
		slog.Error("connexion etcd impossible", "endpoints", *etcdEndpoints, "err", err)
		os.Exit(1)
	}
	defer etcdCli.Close()
	registry := consensus.NewRegistry(etcdCli, logger)

	nodeID := os.Getenv("AMANE_NODE_ID")
	if nodeID == "" {
		nodeID, _ = os.Hostname()
	}
	leadership := consensus.NewLeadership(etcdCli, logger, nodeID)
	go func() {
		if err := leadership.Run(ctx); err != nil && !errors.Is(err, context.Canceled) {
			slog.Error("leadership arrêté", "err", err)
		}
	}()

	// --- Superviseur de failover (option C) : détection crash + /failover ---
	if *supervisorEnabled {
		nodes, err := parsePatroniNodes(*patroniNodes)
		if err != nil {
			slog.Error("superviseur : nœuds Patroni invalides", "err", err)
			os.Exit(1)
		}
		sv := supervisor.New(supervisor.Config{
			EtcdEndpoints: strings.Split(*etcdEndpoints, ","),
			Scope:         *patroniScope,
			Nodes:         nodes,
			// L'agent heartbeat du superviseur ne touche QUE son nœud local :
			// en 3 nœuds, il ne doit jamais supprimer le heartbeat du primary
			// quand SON lien REST est coupé (la garde anti-partition en dépend).
			LocalNode: nodeID,
		}, logger)
		go func() {
			if err := sv.Run(ctx); err != nil && !errors.Is(err, context.Canceled) {
				slog.Error("superviseur arrêté", "err", err)
			}
		}()
	}

	// Relay CRDT multi-site (jalon 4) : maintient l'état local de stock et
	// fusionne les deltas reçus des autres sites via PushDelta.
	relay := replication.NewRelay(replication.NodeID(nodeID))

	// Propagation périodique vers les pairs (réplication multi-site AP).
	if *replicateTo != "" {
		for _, pair := range strings.Split(*replicateTo, ",") {
			parts := strings.SplitN(strings.TrimSpace(pair), "@", 2)
			if len(parts) != 2 {
				slog.Error("pair de réplication invalide (attendu site-id@host:port)", "pair", pair)
				os.Exit(1)
			}
			siteID, target := parts[0], parts[1]
			conn, err := grpc.NewClient(target,
				grpc.WithTransportCredentials(insecure.NewCredentials()),
				grpc.WithChainUnaryInterceptor(telemetry.UnaryClientInterceptor()),
			)
			if err != nil {
				slog.Error("dial pair réplication impossible", "target", target, "err", err)
				os.Exit(1)
			}
			defer conn.Close()
			client := pb.NewAmaneServiceClient(conn)
			transport := grpcserver.NewPushDeltaTransport(client, string(relay.NodeID()))
			prop := replication.NewPropagator(relay, transport, *replicateInterval, logger)
			go func(site string, target string) {
				if err := prop.Run(ctx); err != nil && !errors.Is(err, context.Canceled) {
					slog.Error("propagateur arrêté", "site", site, "target", target, "err", err)
				}
			}(siteID, target)
			slog.Info("propagation multi-site active vers pair",
				"site", siteID, "target", target, "interval", *replicateInterval)
		}
	}

	s := grpc.NewServer(opts...)
	pb.RegisterAmaneServiceServer(s, grpcserver.New(logger).
		WithMembership(registry).
		WithLeadership(leadership).
		WithRelay(relay))
	reflection.Register(s) // introspection grpcurl/clients en dev

	lis, err := net.Listen("tcp", *listen)
	if err != nil {
		slog.Error("listen impossible", "addr", *listen, "err", err)
		os.Exit(1)
	}

	go func() {
		<-ctx.Done()
		slog.Info("arrêt demandé, shutdown gracieux")
		done := make(chan struct{})
		go func() {
			s.GracefulStop()
			close(done)
		}()
		select {
		case <-done:
		case <-time.After(5 * time.Second):
			slog.Warn("shutdown gracieux dépassé, arrêt forcé")
			s.Stop()
		}
	}()

	slog.Info("serveur gRPC démarré", "addr", *listen)
	if err := s.Serve(lis); err != nil {
		slog.Error("serveur arrêté", "err", err)
		os.Exit(1)
	}
}

// parsePatroniNodes décode 'nom@http://host:port,nom@http://host:port'.
func parsePatroniNodes(s string) ([]supervisor.Node, error) {
	var out []supervisor.Node
	for _, part := range strings.Split(s, ",") {
		part = strings.TrimSpace(part)
		if part == "" {
			continue
		}
		name, url, ok := strings.Cut(part, "@")
		if !ok || name == "" || url == "" {
			return nil, fmt.Errorf("format attendu 'nom@http://host:port' : %q", part)
		}
		out = append(out, supervisor.Node{Name: name, PatroniURL: url})
	}
	if len(out) == 0 {
		return nil, errors.New("aucun nœud Patroni fourni")
	}
	return out, nil
}
