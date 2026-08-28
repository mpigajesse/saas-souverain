package grpcserver

import (
	"context"
	"net"
	"testing"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"

	pb "github.com/amane/orchestrator-go/gen/amane/framework/v1"
	"github.com/amane/orchestrator-go/replication"
)

// TestPropagatorE2EOverTCP : deux serveurs gRPC réels (TCP loopback) reliés
// par un propagateur — les deltas du site émetteur arrivent au pair, le
// pending est confirmé et le pair converge.
func TestPropagatorE2EOverTCP(t *testing.T) {
	relayA := replication.NewRelay("site-a") // site qui REÇOIT (serveur)
	relayB := replication.NewRelay("site-b") // site qui POUSSE (propagateur)

	// Notification receiver A (site-a) sur TCP réel.
	lisA, err := net.Listen("tcp", "127.0.0.1:0")
	if err != nil {
		t.Fatalf("listen A: %v", err)
	}
	srvA := grpc.NewServer()
	pb.RegisterAmaneServiceServer(srvA, New(silentLogger()).WithRelay(relayA))
	go srvA.Serve(lisA)
	t.Cleanup(srvA.Stop)

	conn, err := grpc.NewClient(lisA.Addr().String(), grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("dial A: %v", err)
	}
	t.Cleanup(func() { conn.Close() })

	transport := NewPushDeltaTransport(pb.NewAmaneServiceClient(conn), "site-b")
	prop := replication.NewPropagator(relayB, transport, 50*time.Millisecond, silentLogger())

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	go prop.Run(ctx)

	// site-b émet +5 puis +2 puis -3 (totaux cumulés) et les pousse.
	relayB.Add(5, 0)
	relayB.Add(2, 0)
	relayB.Add(0, 3)

	// Le pair (site-a) converge vers 4 et le pending de site-b est vidé.
	deadline := time.Now().Add(5 * time.Second)
	for {
		if relayA.LocalCount() == 4 && len(relayB.Outgoing()) == 0 {
			break
		}
		if time.Now().After(deadline) {
			t.Fatalf("convergence non atteinte : site-a=%d pending-b=%d",
				relayA.LocalCount(), len(relayB.Outgoing()))
		}
		time.Sleep(20 * time.Millisecond)
	}
}
