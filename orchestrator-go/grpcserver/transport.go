package grpcserver

import (
	"context"

	pb "github.com/amane/orchestrator-go/gen/amane/framework/v1"
	"github.com/amane/orchestrator-go/replication"
	"github.com/amane/orchestrator-go/telemetry"
)

// PushDeltaTransport adapte un client gRPC AmaneService en
// replication.Transport : il pousse les deltas locaux d'un site vers un pair
// via le RPC PushDelta et rend l'ack de séquence (gc du pending côté émetteur).
type PushDeltaTransport struct {
	client pb.AmaneServiceClient
	siteID string
}

// NewPushDeltaTransport crée le transport vers un site pair.
func NewPushDeltaTransport(client pb.AmaneServiceClient, siteID string) *PushDeltaTransport {
	return &PushDeltaTransport{client: client, siteID: siteID}
}

// Push implémente replication.Transport. Le contexte doit porter une trace
// (telemetry.UnaryClientInterceptor l'injecte dans le metadata sortant) ; à
// défaut, on en crée une racine pour que chaque poussée soit corrélée.
func (t *PushDeltaTransport) Push(ctx context.Context, fromNode replication.NodeID, deltas []replication.Delta) (uint64, int64, error) {
	if telemetry.FromContext(ctx) == nil {
		ctx = telemetry.ContextWithTrace(ctx, telemetry.Root())
	}
	req := &pb.PushDeltaRequest{SiteId: t.siteID, FromNode: string(fromNode)}
	for _, d := range deltas {
		req.Deltas = append(req.Deltas, &pb.Delta{
			NodeId: string(d.NodeID),
			Inc:    d.Inc,
			Dec:    d.Dec,
			Seq:    d.Seq,
		})
	}
	resp, err := t.client.PushDelta(ctx, req)
	if err != nil {
		return 0, 0, err
	}
	return resp.GetAckedSeq(), resp.GetValue(), nil
}
