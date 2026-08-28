package supervisor

import (
	"context"
	"encoding/json"
	"log/slog"
	"time"

	clientv3 "go.etcd.io/etcd/client/v3"
	"go.etcd.io/etcd/client/v3/concurrency"
)

// hbPrefix est le prefixe etcd des heartbeats écrits par chaque superviseur
// déployé sur un nœud du cluster PostgreSQL : /amane/supervisor/hb/<name>.
const hbPrefix = "/amane/supervisor/hb/"

// heartbeat est la valeur du heartbeat du superviseur local : le rôle local vu
// au dernier sondage et son horodatage. Écrit avec une lease courte (lease TTL
// ~ HeartbeatTTL) : si le processus superviseur meurt, la clé disparaît vite.
type heartbeat struct {
	Role string `json:"role"` // primary / replica (vue locale)
	TS   int64  `json:"ts"`   // unix milli, heure du dernier battement
}

// hbWriter maintient les leases etcd des heartbeats des nœuds dont ce
// superviseur est l'agent local. Chaque tick publie le heartbeat de la vue
// locale (probe REST) puis le détecteur le relit (vue distribuée honnête).
type hbWriter struct {
	cli    *clientv3.Client
	logger *slog.Logger
	ttl    time.Duration
	leases map[string]*concurrency.Session
}

func newHBWriter(cli *clientv3.Client, logger *slog.Logger, ttl time.Duration) *hbWriter {
	return &hbWriter{cli: cli, logger: logger, ttl: ttl, leases: make(map[string]*concurrency.Session)}
}

func hbKey(name string) string { return hbPrefix + name }

func (w *hbWriter) session(name string) (*concurrency.Session, error) {
	if s, ok := w.leases[name]; ok {
		return s, nil
	}
	sess, err := concurrency.NewSession(w.cli, concurrency.WithTTL(int(w.ttl.Seconds())))
	if err != nil {
		return nil, err
	}
	w.leases[name] = sess
	return sess, nil
}

// publish écrit le heartbeat du nœud `name` (role local + timestamp), ou le
// SUPPRIME si le nœud ne répond plus : la clé disparaît → les autres détecteurs
// voient un heartbeat absent dès le tick suivant (crash détecté immédiatement).
// En cas de mort du processus superviseur sans nettoyage, la lease courte
// révoque la clé ; en dernier recours, le timestamp sert de backstop.
func (w *hbWriter) publish(name string, st patroniStatus, ok bool, now time.Time) {
	ctx, cancel := context.WithTimeout(context.Background(), w.ttl/2)
	defer cancel()
	if !ok {
		if _, err := w.cli.Delete(ctx, hbKey(name)); err != nil {
			w.logger.Warn("heartbeat suppression impossible", "node", name, "err", err)
		}
		return
	}
	sess, err := w.session(name)
	if err != nil {
		w.logger.Warn("heartbeat impossible (session)", "node", name, "err", err)
		return
	}
	val, err := json.Marshal(heartbeat{Role: st.Role, TS: now.UnixMilli()})
	if err != nil {
		return
	}
	if _, err := w.cli.Put(ctx, hbKey(name), string(val), clientv3.WithLease(sess.Lease())); err != nil {
		w.logger.Warn("heartbeat écriture impossible", "node", name, "err", err)
	}
}

// leaderAlive lit le heartbeat du leader suspecté : true si retrouvé avec role
// primary et timestamp frais (dans ttl) — le nœud se déclare primary via son
// agent local. false si absent ou stale (agent mort / crash) → autorise le
// forçage. C'est la garde anti partition : un primary ISOLÉ en REST mais
// VIVANT côté etcd ne déclenche jamais de forçage.
func (w *hbWriter) leaderAlive(name string, ttl time.Duration, now time.Time) bool {
	ctx, cancel := context.WithTimeout(context.Background(), ttl/2)
	defer cancel()
	resp, err := w.cli.Get(ctx, hbKey(name))
	if err != nil || resp.Count == 0 {
		return false
	}
	var hb heartbeat
	if err := json.Unmarshal(resp.Kvs[0].Value, &hb); err != nil {
		return false
	}
	if hb.Role != "primary" {
		return false
	}
	return now.Sub(time.UnixMilli(hb.TS)) <= ttl
}

// close libère toutes les sessions (lease etcd → heartbeats révoqués à l'arrêt).
func (w *hbWriter) close() {
	for _, s := range w.leases {
		s.Close()
	}
	w.leases = nil
}
