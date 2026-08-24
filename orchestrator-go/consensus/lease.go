package consensus

import (
	"context"
	"log/slog"
	"sync"
	"time"

	clientv3 "go.etcd.io/etcd/client/v3"
	"go.etcd.io/etcd/client/v3/concurrency"
)

// Leadership incarne l'élection du leader de l'orchestrator via etcd
// (lease à TTL/30s renouvelé en continu → lease-based fencing : un ancien
// leader dont le lease expire ne peut plus écrire, il n'est plus leader).
type Leadership struct {
	cli      *clientv3.Client
	logger   *slog.Logger
	nodeID   string
	key      string
	ttlSec   int
	election *concurrency.Election
	session  *concurrency.Session

	mu       sync.RWMutex
	isLeader bool
	leaderID string
}

func NewLeadership(cli *clientv3.Client, logger *slog.Logger, nodeID string) *Leadership {
	return &Leadership{
		cli:    cli,
		logger: logger,
		nodeID: nodeID,
		key:    leaderKey,
		ttlSec: ttlSeconds,
	}
}

// Run acquiert le leadership et le conserve (renégocié en boucle après une
// perte de lease -> fencing). Retourne dès que ctx est annulé. Typiquement
// lancé en goroutine dans main.
func (l *Leadership) Run(ctx context.Context) error {
	for {
		if ctx.Err() != nil {
			return nil
		}
		start := time.Now()

		sess, err := concurrency.NewSession(l.cli, concurrency.WithTTL(l.ttlSec))
		if err != nil {
			l.logger.Error("création session lease impossible", "err", err)
			if !waitCtx(ctx, 2*time.Second) {
				return ctx.Err()
			}
			continue
		}

		elec := concurrency.NewElection(sess, l.key)
		if err := elec.Campaign(ctx, l.nodeID); err != nil {
			sess.Close()
			l.logger.Error("campagne leadership échouée", "err", err)
			if !waitCtx(ctx, 2*time.Second) {
				return ctx.Err()
			}
			continue
		}

		l.mu.Lock()
		l.session, l.election = sess, elec
		l.isLeader, l.leaderID = true, l.nodeID
		l.mu.Unlock()

		l.logger.Info("leadership acquis",
			"node_id", l.nodeID,
			"leader", l.leaderID,
			"acquisition_ms", time.Since(start).Milliseconds(),
		)

		// Le keepalive est géré par la Session. Une perte de lease ou
		// une révocation nous fait perdre la campagne ici.
		select {
		case <-sess.Done():
			l.mu.Lock()
			l.isLeader = false
			l.mu.Unlock()
			l.logger.Warn("leadership perdu (lease expiré/fencing)",
				"node_id", l.nodeID,
				"leader", l.leaderID,
				"lost_at", time.Now().UTC(),
			)
		case <-ctx.Done():
			// Arrêt gracieux : on ferme la session (révoque la lease et la
			// clé d'élection) pour libérer le leadership proprement — sinon la
			// goroutine reste coincée sur sess.Done() et le keepalive maintient
			// le leader en vie indéfiniment (pas de fencing sur arrêt).
			l.mu.Lock()
			l.isLeader = false
			l.mu.Unlock()
			l.logger.Info("leadership libéré (arrêt)",
				"node_id", l.nodeID,
			)
			sess.Close()
			return nil
		}
		sess.Close()
	}
}

// IsLeader indique si ce nœud est actuellement leader.
func (l *Leadership) IsLeader() bool {
	l.mu.RLock()
	defer l.mu.RUnlock()
	return l.isLeader
}

// Leader retourne l'identifiant du leader courant (empty si pas encore élu).
func (l *Leadership) Leader(ctx context.Context) string {
	l.mu.RLock()
	election := l.election
	l.mu.RUnlock()
	if election == nil {
		return ""
	}
	resp, err := election.Leader(ctx)
	if err != nil {
		return ""
	}
	return string(resp.Kvs[0].Value)
}

func waitCtx(ctx context.Context, d time.Duration) bool {
	t := time.NewTimer(d)
	defer t.Stop()
	select {
	case <-ctx.Done():
		return false
	case <-t.C:
		return true
	}
}