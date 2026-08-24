package contracts

import (
	"context"
	"fmt"
	"io"
	"log/slog"
	"net"
	"os"
	"strings"
	"testing"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/status"
	"google.golang.org/grpc/test/bufconn"

	"github.com/amane/orchestrator-go/consensus"
	pb "github.com/amane/orchestrator-go/gen/amane/framework/v1"
	"github.com/amane/orchestrator-go/grpcserver"
	"github.com/amane/orchestrator-go/replication"
)

// etcdEndpoints retourne les endpoints de la DCS réelle.
func etcdEndpoints(t *testing.T) []string {
	t.Helper()
	ep := os.Getenv("AMANE_TEST_ETCD")
	if ep == "" {
		t.Skip("AMANE_TEST_ETCD non défini — cluster etcd réel requis")
	}
	return strings.Split(ep, ",")
}

func silentLogger() *slog.Logger {
	return slog.New(slog.NewTextHandler(io.Discard, nil))
}

func testClient(t *testing.T, s *grpc.Server) pb.AmaneServiceClient {
	t.Helper()
	lis := bufconn.Listen(1 << 20)
	go s.Serve(lis)
	t.Cleanup(s.Stop)
	conn, err := grpc.NewClient("passthrough:///bufnet",
		grpc.WithContextDialer(func(ctx context.Context, _ string) (net.Conn, error) {
			return lis.DialContext(ctx)
		}),
		grpc.WithTransportCredentials(insecure.NewCredentials()),
	)
	if err != nil {
		t.Fatalf("client: %v", err)
	}
	t.Cleanup(func() { conn.Close() })
	return pb.NewAmaneServiceClient(conn)
}

// TestInterface1EnrollRevoke vs etcd réel — contrat A ↔ C :
// enrôlement d'une machine (clé publique X25519) puis révocation → le quorum
// est recalculé, la machine n'est plus enrôlée.
func TestInterface1EnrollRevoke(t *testing.T) {
	cli, err := consensus.NewClient(etcdEndpoints(t))
	if err != nil {
		t.Fatalf("client etcd: %v", err)
	}
	defer cli.Close()
	registry := consensus.NewRegistry(cli, silentLogger())

	machineID := fmt.Sprintf("contract-m1-%d", time.Now().UnixMilli())
	t.Cleanup(func() { registry.Remove(context.Background(), machineID) })

	s := grpc.NewServer()
	pb.RegisterAmaneServiceServer(s, grpcserver.New(silentLogger()).WithMembership(registry))
	client := testClient(t, s)

	// Enrôlement.
	resp, err := client.Enroll(context.Background(), &pb.EnrollRequest{
		MachineId:   machineID,
		AkPublicKey: []byte("x25519-pub-clef-amane"),
		SiteId:      "site-a",
	})
	if err != nil {
		t.Fatalf("enroll: %v", err)
	}
	if resp.GetMembershipId() != machineID ||
		!strings.HasPrefix(resp.GetClusterId(), "amane-") {
		t.Errorf("enroll = %q / %q, incohérent", resp.GetMembershipId(), resp.GetClusterId())
	}
	ok, err := registry.Has(context.Background(), machineID)
	if err != nil || !ok {
		t.Fatalf("machine absente du registre après enroll (err=%v)", err)
	}

	// Double enrôlement → AlreadyExists (contrat).
	if _, err := client.Enroll(context.Background(), &pb.EnrollRequest{
		MachineId: machineID, AkPublicKey: []byte("x"),
	}); status.Code(err) != codes.AlreadyExists {
		t.Errorf("double enroll: code = %v, want AlreadyExists", status.Code(err))
	}

	// Révocation → plus enrôlée.
	rev, err := client.NotifyRevocation(context.Background(), &pb.NotifyRevocationRequest{
		MachineId: machineID, RevokedAkId: "ak-revoquee",
	})
	if err != nil {
		t.Fatalf("notify revocation: %v", err)
	}
	if !rev.GetQuorumRecalculated() {
		t.Error("quorum_recalculated = false, want true")
	}
	if ok, _ := registry.Has(context.Background(), machineID); ok {
		t.Error("machine encore enrôlée après révocation")
	}

	// Révocation d'une machine inconnue → NotFound (contrat).
	if _, err := client.NotifyRevocation(context.Background(), &pb.NotifyRevocationRequest{
		MachineId: "inconnue-xyz", RevokedAkId: "ak",
	}); status.Code(err) != codes.NotFound {
		t.Errorf("revoke inconnu: code = %v, want NotFound", status.Code(err))
	}
}

// TestWritePathGated — chemin d'écriture B ↔ C intégral contre etcd réel :
// Write/Read sur le nœud leader + PushDelta multi-site ; Write refusé sur le
// non-leader (fencing applicatif).
func TestWritePathGated(t *testing.T) {
	cli, err := consensus.NewClient(etcdEndpoints(t))
	if err != nil {
		t.Fatalf("client etcd: %v", err)
	}
	defer cli.Close()
	registry := consensus.NewRegistry(cli, silentLogger())

	relayA := replication.NewRelay("site-a")
	journal := replication.NewJournal()

	leaderSrv := grpcserver.New(silentLogger()).
		WithMembership(registry).
		WithLeadership(&staticLeadership{leader: true}).
		WithRelay(relayA).
		WithJournal(journal)
	client := testClient(t, mkSrv(leaderSrv))

	// Write : entrée chiffrée committée dans le journal.
	wr, err := client.Write(context.Background(), &pb.WriteRequest{
		JournalId: "stock", OpSeq: 1, SiteId: "site-a",
		MachineId: "m1", EncryptedPayload: []byte("chiffre-mission-a"),
	})
	if err != nil {
		t.Fatalf("write (leader): %v", err)
	}
	if !wr.GetSynced() {
		t.Error("synced = false, want true")
	}
	if got := journal.SeqOf("stock"); got != 1 {
		t.Errorf("journal seq = %d, want 1", got)
	}

	// Read : relecture de l'entrée (numéro de séquence stable).
	rr, err := client.Read(context.Background(), &pb.ReadRequest{JournalId: "stock"})
	if err != nil || len(rr.GetEntries()) != 1 {
		t.Fatalf("read: %v, entries=%d", err, len(rr.GetEntries()))
	}
	if string(rr.GetEntries()[0].GetEncryptedPayload()) != "chiffre-mission-a" {
		t.Error("payload lue ≠ payload écrite")
	}

	// PushDelta : site-b envoie son delta à site-a → convergence.
	siteBRelay := replication.NewRelay("site-b")
	siteBRelay.Add(0, 2) // -2
	clientA := testClient(t, mkSrv(grpcserver.New(silentLogger()).WithRelay(relayA)))
	ack, err := clientA.PushDelta(context.Background(), &pb.PushDeltaRequest{
		SiteId: "site-b", FromNode: "site-b",
		Deltas: []*pb.Delta{{
			NodeId: "site-b", Inc: 0, Dec: 2, Seq: 1,
		}},
	})
	if err != nil {
		t.Fatalf("push delta: %v", err)
	}
	if ack.GetValue() != -2 { // site-a a fusionné le -2 de site-b
		t.Errorf("value = %d, want -2", ack.GetValue())
	}
	if ack.GetAckedSeq() != 1 {
		t.Errorf("acked_seq = %d, want 1", ack.GetAckedSeq())
	}
}

// Non-leader : Write refusé (codes.FailedPrecondition), jamais de compromis
// de séquence dans le journal.
func TestWriteRefusedOnNonLeader(t *testing.T) {
	journal := replication.NewJournal()
	s := grpcserver.New(silentLogger()).
		WithLeadership(&staticLeadership{leader: false}).
		WithJournal(journal)
	client := testClient(t, mkSrv(s))

	_, err := client.Write(context.Background(), &pb.WriteRequest{
		JournalId: "stock", OpSeq: 5, SiteId: "site-a",
		MachineId: "m1", EncryptedPayload: []byte("chiffre"),
	})
	if status.Code(err) != codes.FailedPrecondition {
		t.Errorf("write non-leader: code = %v, want FailedPrecondition", status.Code(err))
	}
	if got := journal.SeqOf("stock"); got != 0 {
		t.Errorf("journal seq = %d, want 0 (aucune écriture sur non-leader)", got)
	}
}

type staticLeadership struct{ leader bool }

func (s *staticLeadership) IsLeader() bool { return s.leader }

func mkSrv(s *grpcserver.Server) *grpc.Server {
	g := grpc.NewServer()
	pb.RegisterAmaneServiceServer(g, s)
	return g
}
