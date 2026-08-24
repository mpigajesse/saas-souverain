package replication

import (
	"context"
	"log/slog"
	"time"
)

// Transport délivre les deltas d'un Relay vers un site pair et retourne la
// seq locale confirmée par le pair (gc du pending). Interface pour garder le
// propagateur testable sans gRPC réel.
type Transport interface {
	// Push envoie des deltas au pair ; retourne ackSeq (max seq locale
	// appliquée par le pair) et la valeur locale du pair après application.
	Push(ctx context.Context, fromNode NodeID, deltas []Delta) (ackSeq uint64, value int64, err error)
}

// Propagator pousse périodiquement les deltas locaux d'un Relay vers un site
// pair (pull/push asynchrone, mode AP). L'intervalle et l'échec ne perdent
// jamais de delta : ils restent dans pending jusqu'à confirmation du pair.
type Propagator struct {
	relay    *Relay
	peer     Transport
	interval time.Duration
	logger   *slog.Logger
}

// NewPropagator crée un propagateur vers un pair.
func NewPropagator(relay *Relay, peer Transport, interval time.Duration, logger *slog.Logger) *Propagator {
	if interval <= 0 {
		interval = time.Second
	}
	if logger == nil {
		logger = slog.Default()
	}
	return &Propagator{relay: relay, peer: peer, interval: interval, logger: logger}
}

// Run boucle : à chaque tick, tente de pousser les deltas en attente. S'arrête
// quand ctx est annulé. Une erreur réseau est journalisée et retentée au tick
// suivant (le pending n'est jamais jeté).
func (p *Propagator) Run(ctx context.Context) error {
	t := time.NewTicker(p.interval)
	defer t.Stop()
	for {
		select {
		case <-ctx.Done():
			return ctx.Err()
		case <-t.C:
			if err := p.SyncNow(ctx); err != nil && ctx.Err() == nil {
				p.logger.Warn("propagation vers pair échouée (retentée au prochain tick)",
					"err", err,
				)
			}
		}
	}
}

// SyncNow effectue une passe unique de propagation (sans deltas → no-op).
func (p *Propagator) SyncNow(ctx context.Context) error {
	out := p.relay.Outgoing()
	if len(out) == 0 {
		return nil
	}
	ackSeq, _, err := p.peer.Push(ctx, p.relay.NodeID(), out)
	if err != nil {
		return err
	}
	p.relay.Confirm(ackSeq)
	return nil
}
