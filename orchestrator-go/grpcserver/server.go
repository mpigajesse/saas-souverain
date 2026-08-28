package grpcserver

import (
	"context"
	"errors"
	"log/slog"
	"os"
	"time"

	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
	"google.golang.org/protobuf/types/known/timestamppb"

	"github.com/amane/orchestrator-go/consensus"
	pb "github.com/amane/orchestrator-go/gen/amane/framework/v1"
	"github.com/amane/orchestrator-go/replication"
)

// Membership est la dépendance consensus utilisée par Enroll/NotifyRevocation
// (interface pour garder le grpcserver testable sans etcd réel).
type Membership interface {
	Add(ctx context.Context, m consensus.Member) (int, error)
	Remove(ctx context.Context, machineID string) (int, error)
	Quorum(ctx context.Context) (int, error)
}

// Leadership gating le chemin d'écriture (fencing applicatif) : seul le nœud
// qui détient la lease etcd accepte les Write. Un ancien leader dont la lease a
// expiré refuse, même localement.
type Leadership interface {
	IsLeader() bool
}

// Server implémente le contrat pb.AmaneServiceServer (proto/framework.proto).
type Server struct {
	pb.UnimplementedAmaneServiceServer
	logger     *slog.Logger
	nodeID     string
	version    string
	startedAt  time.Time
	membership Membership
	journal    *replication.Journal
	leadership Leadership
	relay      *replication.Relay
}

func New(logger *slog.Logger) *Server {
	return &Server{
		logger:    logger,
		nodeID:    nodeIDFromEnv(),
		version:   "0.1.0",
		startedAt: time.Now(),
		journal:   replication.NewJournal(),
	}
}

// WithMembership ajoute le registre consensus (etcd) au serveur.
func (s *Server) WithMembership(m Membership) *Server {
	s.membership = m
	return s
}

// WithJournal remplace le journal par défaut (tests / injection).
func (s *Server) WithJournal(j *replication.Journal) *Server {
	s.journal = j
	return s
}

// WithLeadership active le fencing applicatif du chemin d'écriture : Write est
// refusé tant que ce nœud n'est pas leader (lease etcd). Nil désactive (tests).
func (s *Server) WithLeadership(l Leadership) *Server {
	s.leadership = l
	return s
}

// WithRelay enregistre le relay CRDT multi-site (jamais nil en prod : le stock
// doit être répliqué entre les sites).
func (s *Server) WithRelay(r *replication.Relay) *Server {
	s.relay = r
	return s
}

func nodeIDFromEnv() string {
	if id := os.Getenv("AMANE_NODE_ID"); id != "" {
		return id
	}
	return "orchestrator-unknown"
}

// Ping — liveness du service (jalon 1).
func (s *Server) Ping(ctx context.Context, req *pb.PingRequest) (*pb.PingResponse, error) {
	s.logger.Debug("ping reçu")
	return &pb.PingResponse{
		Status:     "ok",
		NodeId:     s.nodeID,
		Version:    s.version,
		ServerTime: timestamppb.New(time.Now()),
	}, nil
}

// Enroll — enrôlement d'une machine (Interface 1 A ↔ C, membership).
// Enregistre la machine dans etcd et recalcule le quorum.
func (s *Server) Enroll(ctx context.Context, req *pb.EnrollRequest) (*pb.EnrollResponse, error) {
	if s.membership == nil {
		return nil, status.Error(codes.Unavailable, "consensus non initialisé")
	}
	if req.GetMachineId() == "" {
		return nil, status.Error(codes.InvalidArgument, "machine_id requis")
	}
	if len(req.GetAkPublicKey()) == 0 {
		return nil, status.Error(codes.InvalidArgument, "ak_public_key requis")
	}

	member := consensus.Member{
		MachineID:   req.GetMachineId(),
		AKPublicKey: req.GetAkPublicKey(),
		SiteID:      req.GetSiteId(),
		Operator:    req.GetOperator(),
	}
	quorum, err := s.membership.Add(ctx, member)
	if err != nil {
		switch {
		case errors.Is(err, consensus.ErrMemberAlreadyExists):
			return nil, status.Error(codes.AlreadyExists, "machine déjà enrôlée")
		default:
			s.logger.Error("enrôlement impossible", "err", err)
			return nil, status.Error(codes.Unavailable, "consensus indisponible")
		}
	}
	s.logger.Info("enrôlement validé",
		"machine_id", member.MachineID,
		"quorum", quorum,
	)
	return &pb.EnrollResponse{
		MembershipId: member.MachineID,
		ClusterId:    "amane-" + member.SiteID,
		EnrolledAt:   timestamppb.New(time.Now().UTC()),
	}, nil
}

// Write — chemin d'écriture (Interface 3 B ↔ C).
// Committe une entrée chiffrée dans le journal multi-site ; le seq retourné est
// monotone et l'écriture est idempotente (rejeu sans doublon). Le payload n'est
// jamais loggé (chiffré par Mission A, clair interdit côté C).
func (s *Server) Write(ctx context.Context, req *pb.WriteRequest) (*pb.WriteResponse, error) {
	if req.GetJournalId() == "" {
		return nil, status.Error(codes.InvalidArgument, "journal_id requis")
	}
	if req.GetSiteId() == "" {
		return nil, status.Error(codes.InvalidArgument, "site_id requis")
	}
	if req.GetMachineId() == "" {
		return nil, status.Error(codes.InvalidArgument, "machine_id requis")
	}
	if len(req.GetEncryptedPayload()) == 0 {
		return nil, status.Error(codes.InvalidArgument, "encrypted_payload requis")
	}

	// Fencing applicatif (lease-based) : seul le leader du cluster accepte les
	// écritures. Un nœud dont la lease a expiré (ancien primary, split post-
	// failover) voit ses Write rejetés ici avant tout compromis du journal.
	if s.leadership != nil && !s.leadership.IsLeader() {
		s.logger.Warn("write refusé : nœud non leader (fencing)",
			"node_id", s.nodeID,
		)
		return nil, status.Error(codes.FailedPrecondition, "write refusé : nœud non leader (fencing lease)")
	}

	res, err := s.journal.Append(req.GetJournalId(), req.GetOpSeq(), req.GetEncryptedPayload(), req.GetSiteId())
	if err != nil {
		s.logger.Error("écriture refusée", "journal_id", req.GetJournalId(), "err", err)
		return nil, status.Error(codes.InvalidArgument, err.Error())
	}
	s.logger.Info("écriture committée",
		"journal_id", req.GetJournalId(),
		"committed_seq", res.CommittedSeq,
		"op_seq", req.GetOpSeq(),
		"site_id", req.GetSiteId(),
		"encrypted_payload_len", len(req.GetEncryptedPayload()),
		"replayed", res.Replayed,
	)
	return &pb.WriteResponse{
		CommittedSeq: res.CommittedSeq,
		NodeId:       s.nodeID,
		Synced:       res.Synced,
		CommittedAt:  timestamppb.New(time.Now().UTC()),
	}, nil
}

// Read — lecture du journal (Interface 3 B ↔ C).
// Retourne les entrées de seq >= from_seq (limit bornée à 1000).
func (s *Server) Read(ctx context.Context, req *pb.ReadRequest) (*pb.ReadResponse, error) {
	if req.GetJournalId() == "" {
		return nil, status.Error(codes.InvalidArgument, "journal_id requis")
	}
	limit := req.GetLimit()
	if limit == 0 {
		limit = 100
	}
	if limit > 1000 {
		return nil, status.Error(codes.InvalidArgument, "limit ne peut excéder 1000")
	}

	entries := s.journal.Read(req.GetJournalId(), req.GetFromSeq(), limit)
	out := make([]*pb.JournalEntry, 0, len(entries))
	for _, e := range entries {
		out = append(out, &pb.JournalEntry{
			Seq:              e.Seq,
			EncryptedPayload: e.EncryptedPayload,
			CommittedAt:      timestamppb.New(e.CommittedAt),
			SiteId:           e.SiteID,
		})
	}
	s.logger.Info("lecture servie",
		"journal_id", req.GetJournalId(),
		"from_seq", req.GetFromSeq(),
		"limit", req.GetLimit(),
		"count", len(out),
	)
	return &pb.ReadResponse{Entries: out}, nil
}

// NotifyRevocation — dé-enrôlement d'une machine (Interface 1 A ↔ C).
// Révocation immédiate (< 1s) + recalcul du quorum dans le consensus.
func (s *Server) NotifyRevocation(ctx context.Context, req *pb.NotifyRevocationRequest) (*pb.NotifyRevocationResponse, error) {
	if s.membership == nil {
		return nil, status.Error(codes.Unavailable, "consensus non initialisé")
	}
	if req.GetMachineId() == "" {
		return nil, status.Error(codes.InvalidArgument, "machine_id requis")
	}
	quorum, err := s.membership.Remove(ctx, req.GetMachineId())
	if err != nil {
		switch {
		case errors.Is(err, consensus.ErrMemberNotFound):
			return nil, status.Error(codes.NotFound, "machine introuvable")
		default:
			s.logger.Error("révocation impossible", "err", err)
			return nil, status.Error(codes.Unavailable, "consensus indisponible")
		}
	}
	s.logger.Warn("dé-enrôlement exécuté — quorum recalculé",
		"machine_id", req.GetMachineId(),
		"revoked_ak_id", req.GetRevokedAkId(),
		"reason", req.GetReason(),
		"quorum", quorum,
	)
	return &pb.NotifyRevocationResponse{
		QuorumRecalculated: true,
		NodeId:             s.nodeID,
	}, nil
}

// PushDelta — propagation d'un delta CRDT (stock) d'un site vers un autre
// (jalon 4, réplication multi-site en mode AP). La fusion max-par-nœud est
// idempotente : l'ordre d'arrivée et les doublons ne compromettent pas la
// convergence. Ack = seq max appliquée côté émetteur (gc du pending).
func (s *Server) PushDelta(ctx context.Context, req *pb.PushDeltaRequest) (*pb.PushDeltaResponse, error) {
	if s.relay == nil {
		return nil, status.Error(codes.Unavailable, "réplication CRDT non initialisée")
	}
	if req.GetSiteId() == "" || req.GetFromNode() == "" {
		return nil, status.Error(codes.InvalidArgument, "site_id et from_node requis")
	}

	deltas := make([]replication.Delta, 0, len(req.GetDeltas()))
	for _, d := range req.GetDeltas() {
		deltas = append(deltas, replication.Delta{
			NodeID: replication.NodeID(d.GetNodeId()),
			Inc:    d.GetInc(),
			Dec:    d.GetDec(),
			Seq:    d.GetSeq(),
		})
	}
	applied, ackSeq := s.relay.Accept(replication.NodeID(req.GetFromNode()), deltas)
	value := s.relay.LocalCount()

	s.logger.Info("delta reçu et fusionné",
		"from_site", req.GetSiteId(),
		"from_node", req.GetFromNode(),
		"applied", applied,
		"acked_seq", ackSeq,
		"stock_local", value,
	)
	return &pb.PushDeltaResponse{
		AckedSeq: ackSeq,
		Value:    value,
		NodeId:   s.nodeID,
	}, nil
}
