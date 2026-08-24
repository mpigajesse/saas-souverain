package grpcserver

import (
	"context"
	"log/slog"
	"net"
	"testing"

	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/status"
	"google.golang.org/grpc/test/bufconn"

	"github.com/amane/orchestrator-go/consensus"
	pb "github.com/amane/orchestrator-go/gen/amane/framework/v1"
	"github.com/amane/orchestrator-go/replication"
)

func newTestServer(t *testing.T) (pb.AmaneServiceClient, *grpc.Server) {
	t.Helper()
	logger := slog.New(slog.NewTextHandler(discardWriter{}, nil))

	lis := bufconn.Listen(1024 * 1024)
	s := grpc.NewServer(
		grpc.ChainUnaryInterceptor(
			LoggingUnaryInterceptor(logger),
			RecoveryUnaryInterceptor(logger),
		),
	)
	pb.RegisterAmaneServiceServer(s, New(logger))

	go func() {
		if err := s.Serve(lis); err != nil {
			t.Errorf("serve: %v", err)
		}
	}()
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

	return pb.NewAmaneServiceClient(conn), s
}

func TestEnrollWithConsensus(t *testing.T) {
	fake := &fakeMembership{}
	logger := silentLogger()
	lis := bufconn.Listen(1024 * 1024)
	s := grpc.NewServer()
	pb.RegisterAmaneServiceServer(s, New(logger).WithMembership(fake))
	go s.Serve(lis)
	t.Cleanup(s.Stop)
	client := bufclient(t, lis)

	resp, err := client.Enroll(context.Background(), &pb.EnrollRequest{
		MachineId:   "m1",
		AkPublicKey: []byte("x25519-pub"),
		SiteId:      "site-a",
	})
	if err != nil {
		t.Fatalf("enroll: %v", err)
	}
	if resp.GetMembershipId() != "m1" {
		t.Errorf("membership_id = %q", resp.GetMembershipId())
	}
	if !resp.GetEnrolledAt().IsValid() {
		t.Error("enrolled_at invalide")
	}
}

func TestEnrollDuplicate(t *testing.T) {
	_, client := newTestServerWithMembership(t, &fakeMembership{alreadyExists: true})

	_, err := client.Enroll(context.Background(), &pb.EnrollRequest{
		MachineId:   "m1",
		AkPublicKey: []byte("pub"),
	})
	st := status.Convert(err)
	if st.Code() != codes.AlreadyExists {
		t.Errorf("code = %v, want AlreadyExists", st.Code())
	}
}

func TestEnrollInvalidArgument(t *testing.T) {
	_, client := newTestServerWithMembership(t, &fakeMembership{})

	_, err := client.Enroll(context.Background(), &pb.EnrollRequest{MachineId: ""})
	st := status.Convert(err)
	if st.Code() != codes.InvalidArgument {
		t.Errorf("code = %v, want InvalidArgument", st.Code())
	}
}

func TestNotifyRevocationWithConsensus(t *testing.T) {
	fake := &fakeMembership{}
	_, client := newTestServerWithMembership(t, fake)

	resp, err := client.NotifyRevocation(context.Background(), &pb.NotifyRevocationRequest{
		MachineId: "m1",
	})
	if err != nil {
		t.Fatalf("revoke: %v", err)
	}
	if !resp.GetQuorumRecalculated() {
		t.Error("quorum_recalculated = false")
	}
}

func TestNotifyRevocationNotFound(t *testing.T) {
	_, client := newTestServerWithMembership(t, &fakeMembership{notFound: true})

	_, err := client.NotifyRevocation(context.Background(), &pb.NotifyRevocationRequest{MachineId: "absent"})
	st := status.Convert(err)
	if st.Code() != codes.NotFound {
		t.Errorf("code = %v, want NotFound", st.Code())
	}
}

// --- fakes / helpers ---

// fakeMembership simule le registre consensus pour isoler le grpcserver.
type fakeMembership struct {
	alreadyExists bool
	notFound      bool
}

func (f *fakeMembership) Add(ctx context.Context, m consensus.Member) (int, error) {
	if f.alreadyExists {
		return 0, consensus.ErrMemberAlreadyExists
	}
	return 1, nil
}

func (f *fakeMembership) Remove(ctx context.Context, machineID string) (int, error) {
	if f.notFound {
		return 0, consensus.ErrMemberNotFound
	}
	return 1, nil
}

func (f *fakeMembership) Quorum(ctx context.Context) (int, error) { return 1, nil }

func newTestServerWithMembership(t *testing.T, m Membership) (*grpc.Server, pb.AmaneServiceClient) {
	t.Helper()
	lis := bufconn.Listen(1024 * 1024)
	s := grpc.NewServer()
	pb.RegisterAmaneServiceServer(s, New(silentLogger()).WithMembership(m))
	go s.Serve(lis)
	t.Cleanup(s.Stop)
	return s, bufclient(t, lis)
}

func bufclient(t *testing.T, lis *bufconn.Listener) pb.AmaneServiceClient {
	t.Helper()
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

func silentLogger() *slog.Logger {
	return slog.New(slog.NewTextHandler(discardWriter{}, nil))
}

type discardWriter struct{}

func (discardWriter) Write(p []byte) (int, error) { return len(p), nil }

func TestPing(t *testing.T) {
	client, _ := newTestServer(t)

	resp, err := client.Ping(context.Background(), &pb.PingRequest{})
	if err != nil {
		t.Fatalf("ping: %v", err)
	}
	if resp.GetStatus() != "ok" {
		t.Errorf("status = %q, want ok", resp.GetStatus())
	}
	if resp.GetNodeId() == "" {
		t.Error("node_id vide")
	}
	if resp.GetServerTime() == nil {
		t.Error("server_time absent")
	}
}

func TestEnrollWithoutConsensus(t *testing.T) {
	client, _ := newTestServer(t)

	_, err := client.Enroll(context.Background(), &pb.EnrollRequest{MachineId: "m1", AkPublicKey: []byte("pub")})
	st := status.Convert(err)
	if st.Code() != codes.Unavailable {
		t.Errorf("code = %v, want Unavailable", st.Code())
	}
}

func TestWriteCommit(t *testing.T) {
	client, _ := newTestServer(t)

	resp, err := client.Write(context.Background(), &pb.WriteRequest{
		JournalId:        "stock",
		OpSeq:            1,
		EncryptedPayload: []byte("secret-chiffre"),
		SiteId:           "site-a",
		MachineId:        "m1",
	})
	if err != nil {
		t.Fatalf("write: %v", err)
	}
	if resp.GetCommittedSeq() != 0 || !resp.GetSynced() {
		t.Errorf("resp = %+v, want committed_seq 0 synced", resp)
	}
	if !resp.GetCommittedAt().IsValid() {
		t.Error("committed_at invalide")
	}

	// rejeu idempotent : même (site, op_seq) → même seq, pas de doublon
	resp2, err := client.Write(context.Background(), &pb.WriteRequest{
		JournalId:        "stock",
		OpSeq:            1,
		EncryptedPayload: []byte("secret-chiffre"),
		SiteId:           "site-a",
		MachineId:        "m1",
	})
	if err != nil || resp2.GetCommittedSeq() != 0 {
		t.Errorf("rejeu = %+v, %v — want committed_seq 0", resp2, err)
	}
}

func TestWriteInvalidArgument(t *testing.T) {
	client, _ := newTestServer(t)

	for name, req := range map[string]*pb.WriteRequest{
		"sans journal_id": {OpSeq: 1, EncryptedPayload: []byte("x"), SiteId: "a", MachineId: "m"},
		"sans payload":    {JournalId: "j", OpSeq: 1, SiteId: "a", MachineId: "m"},
		"sans site_id":    {JournalId: "j", OpSeq: 1, EncryptedPayload: []byte("x"), MachineId: "m"},
		"sans machine_id": {JournalId: "j", OpSeq: 1, EncryptedPayload: []byte("x"), SiteId: "a"},
	} {
		_, err := client.Write(context.Background(), req)
		if st := status.Convert(err); st.Code() != codes.InvalidArgument {
			t.Errorf("%s : code = %v, want InvalidArgument", name, st.Code())
		}
	}
}

func TestReadAndPagination(t *testing.T) {
	client, _ := newTestServer(t)

	for i := 0; i < 3; i++ {
		if _, err := client.Write(context.Background(), &pb.WriteRequest{
			JournalId:        "stock",
			OpSeq:            uint64(i + 1),
			EncryptedPayload: []byte("s"),
			SiteId:           "site-a",
			MachineId:        "m1",
		}); err != nil {
			t.Fatalf("write %d: %v", i, err)
		}
	}

	// lecture complète
	resp, err := client.Read(context.Background(), &pb.ReadRequest{JournalId: "stock"})
	if err != nil {
		t.Fatalf("read: %v", err)
	}
	if n := len(resp.GetEntries()); n != 3 {
		t.Errorf("nb entrées = %d, want 3", n)
	}
	if got := resp.GetEntries()[2].GetSeq(); got != 2 {
		t.Errorf("last seq = %d, want 2", got)
	}

	// pagination depuis seq 1
	resp2, err := client.Read(context.Background(), &pb.ReadRequest{JournalId: "stock", FromSeq: 1, Limit: 1})
	if err != nil || len(resp2.GetEntries()) != 1 || resp2.GetEntries()[0].GetSeq() != 1 {
		t.Errorf("pagination = %v, %v — want 1 entrée seq 1", resp2.GetEntries(), err)
	}

	// journal inconnu → liste vide
	resp3, _ := client.Read(context.Background(), &pb.ReadRequest{JournalId: "inconnu"})
	if n := len(resp3.GetEntries()); n != 0 {
		t.Errorf("journal inconnu = %d entrées, want 0", n)
	}

	// limit > 1000 → InvalidArgument
	_, err = client.Read(context.Background(), &pb.ReadRequest{JournalId: "stock", Limit: 1001})
	if st := status.Convert(err); st.Code() != codes.InvalidArgument {
		t.Errorf("limit énorme : code = %v, want InvalidArgument", st.Code())
	}
}

func TestNotifyRevocationWithoutConsensus(t *testing.T) {
	client, _ := newTestServer(t)

	_, err := client.NotifyRevocation(context.Background(), &pb.NotifyRevocationRequest{
		MachineId:   "m1",
		RevokedAkId: "ak-123",
	})
	st := status.Convert(err)
	if st.Code() != codes.Unavailable {
		t.Errorf("code = %v, want Unavailable", st.Code())
	}
}

// ---------- réplication multi-site (delta CRDT) via PushDelta ----------

func newTestServerWithRelay(t *testing.T, relay *replication.Relay) pb.AmaneServiceClient {
	t.Helper()
	lis := bufconn.Listen(1024 * 1024)
	s := grpc.NewServer()
	pb.RegisterAmaneServiceServer(s, New(silentLogger()).WithRelay(relay))
	go s.Serve(lis)
	t.Cleanup(s.Stop)
	return bufclient(t, lis)
}

func TestPushDeltaAcrossSites(t *testing.T) {
	ra := replication.NewRelay("site-a")
	rb := replication.NewRelay("site-b")
	clientA := newTestServerWithRelay(t, ra)
	clientB := newTestServerWithRelay(t, rb)

	// site-a enregistre +5 puis propage vers site-b.
	da := ra.Add(5, 0)
	ack, err := clientB.PushDelta(context.Background(), &pb.PushDeltaRequest{
		SiteId:   "site-a",
		FromNode: "site-a",
		Deltas: []*pb.Delta{{
			NodeId: string(da.NodeID), Inc: da.Inc, Dec: da.Dec, Seq: da.Seq,
		}},
	})
	if err != nil {
		t.Fatalf("push a→b: %v", err)
	}
	if ack.GetAckedSeq() != 1 || ack.GetValue() != 5 {
		t.Errorf("ack = seq %d / value %d, want 1 / 5", ack.GetAckedSeq(), ack.GetValue())
	}
	if got := rb.LocalCount(); got != 5 {
		t.Errorf("site-b = %d, want 5", got)
	}

	// site-b décrémente -2, site-a reçoit : les deux convergent à 3.
	db := rb.Add(0, 2)
	dbR, err := clientA.PushDelta(context.Background(), &pb.PushDeltaRequest{
		SiteId:   "site-b",
		FromNode: "site-b",
		Deltas: []*pb.Delta{{
			NodeId: string(db.NodeID), Inc: db.Inc, Dec: db.Dec, Seq: db.Seq,
		}},
	})
	if err != nil {
		t.Fatalf("push b→a: %v", err)
	}
	if got := dbR.GetValue(); got != 3 {
		t.Errorf("site-a après push b = %d, want 3", got)
	}
	if got := rb.LocalCount(); got != 3 {
		t.Errorf("site-b = %d, want 3", got)
	}

	// Duplicate du delta de a vers b : idempotent, pas de changement.
	clientB.PushDelta(context.Background(), &pb.PushDeltaRequest{
		SiteId: "site-a", FromNode: "site-a",
		Deltas: []*pb.Delta{{NodeId: string(da.NodeID), Inc: da.Inc, Dec: da.Dec, Seq: da.Seq}},
	})
	if got := rb.LocalCount(); got != 3 {
		t.Errorf("site-b après doublon = %d, want 3 (idempotence)", got)
	}
}

func TestPushDeltaWithoutRelay(t *testing.T) {
	client, _ := newTestServer(t)
	_, err := client.PushDelta(context.Background(), &pb.PushDeltaRequest{
		SiteId: "site-a", FromNode: "site-a",
		Deltas: []*pb.Delta{{NodeId: "site-a", Inc: 1}},
	})
	if st := status.Convert(err); st.Code() != codes.Unavailable {
		t.Errorf("code = %v, want Unavailable (relay non initialisé)", st.Code())
	}
}

// ---------- fencing applicatif : Write gated par la lease leader ----------

// fakeLeadership simule la lease etcd (IsLeader) pour isoler le gating.
type fakeLeadership struct{ leader bool }

func (f *fakeLeadership) IsLeader() bool { return f.leader }

func newTestServerWithLeadership(t *testing.T, l Leadership) (pb.AmaneServiceClient, *replication.Journal) {
	t.Helper()
	lis := bufconn.Listen(1024 * 1024)
	s := grpc.NewServer()
	j := replication.NewJournal()
	pb.RegisterAmaneServiceServer(s, New(silentLogger()).WithJournal(j).WithLeadership(l))
	go s.Serve(lis)
	t.Cleanup(s.Stop)
	return bufclient(t, lis), j
}

func TestWriteRefusedWhenNotLeader(t *testing.T) {
	client, j := newTestServerWithLeadership(t, &fakeLeadership{leader: false})

	_, err := client.Write(context.Background(), &pb.WriteRequest{
		JournalId:        "stock",
		OpSeq:            1,
		EncryptedPayload: []byte("secret-chiffre"),
		MachineId:        "m1",
		SiteId:           "site-a",
	})
	if st := status.Convert(err); st.Code() != codes.FailedPrecondition {
		t.Fatalf("code = %v, want FailedPrecondition", st.Code())
	}

	// Le journal ne doit contenir aucune entrée (compromis de séquence évité).
	if resp, err := client.Read(context.Background(), &pb.ReadRequest{JournalId: "stock"}); err != nil {
		t.Fatalf("read: %v", err)
	} else if n := len(resp.GetEntries()); n != 0 {
		t.Errorf("journal = %d entrées après refus, want 0", n)
	}
	if n := len(j.Read("stock", 0, 1000)); n != 0 {
		t.Errorf("journal interne = %d entrées, want 0", n)
	}
}

func TestWriteAllowedWhenLeader(t *testing.T) {
	client, _ := newTestServerWithLeadership(t, &fakeLeadership{leader: true})

	resp, err := client.Write(context.Background(), &pb.WriteRequest{
		JournalId:        "stock",
		OpSeq:            1,
		MachineId:        "m1",
		EncryptedPayload: []byte("secret-chiffre"),
		SiteId:           "site-a",
	})
	if err != nil {
		t.Fatalf("write: %v", err)
	}
	if resp.GetCommittedSeq() != 0 || resp.GetSynced() != true {
		t.Errorf("committed = %v synced = %v, want 0/true", resp.GetCommittedSeq(), resp.GetSynced())
	}
}

func TestWriteAllowedWithoutLeadership(t *testing.T) {
	// Pas de WithLeadership : aucun gating (rétro-compat léger pour les tests
	// et orchestrations mononœud sans élection).
	client, _ := newTestServerWithoutLeadership(t)

	_, err := client.Write(context.Background(), &pb.WriteRequest{
		JournalId:        "stock",
		MachineId:        "m1",
		OpSeq:            1,
		EncryptedPayload: []byte("secret-chiffre"),
		SiteId:           "site-a",
	})
	if err != nil {
		t.Fatalf("write sans fencing: %v", err)
	}
}

func newTestServerWithoutLeadership(t *testing.T) (pb.AmaneServiceClient, *replication.Journal) {
	t.Helper()
	lis := bufconn.Listen(1024 * 1024)
	s := grpc.NewServer()
	j := replication.NewJournal()
	pb.RegisterAmaneServiceServer(s, New(silentLogger()).WithJournal(j))
	go s.Serve(lis)
	t.Cleanup(s.Stop)
	return bufclient(t, lis), j
}
