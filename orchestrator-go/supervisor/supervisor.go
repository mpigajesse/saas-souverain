package supervisor

import (
	"context"
	"log/slog"
	"net/http"
	"os"
	"time"

	clientv3 "go.etcd.io/etcd/client/v3"
	"go.etcd.io/etcd/client/v3/concurrency"

	"github.com/amane/orchestrator-go/consensus"
)

// forceRightKey est le lock etcd « droit de forcer » : une seule instance de
// superviseur (sur les 3 nœuds) détient ce droit et agit — jamais de double
// forçage ni de SPOF (élection en lease courte).
const forceRightKey = "/amane/supervisor/force-right"

// supervisorKey est le prefixe etcd de détection (heartbeats + droit de forcer).
const supervisorKey = "/amane/supervisor/"

// Supervisor détecte la mort d'un primary PostgreSQL et déclenche
// POST /failover Patroni (option C — décision d'architecture AGENTS.md). Il
// NE décide PAS la promotion : Patroni reste l'autorité de fencing et de
// promotion (réplique synchrone et zéro perte préservées).
//
// Chaque superviseur est aussi l'agent local de son nœud : il publie dans etcd
// un heartbeat à lease courte (indépendant du plancher ttl >= 20 s de Patroni)
// qui rapporte si SON nœud est primary. Le détecteur d'un autre nœud force le
// failover seulement si le primary suspect est jointement REST-down et
// heartbeat-stale ; un primary vivant côté etcd (partition) n'est jamais forcé.
type Supervisor struct {
	cfg    Config
	logger *slog.Logger
	http   *http.Client
	nodeID string

	cli *clientv3.Client
	hb  *hbWriter
	det *detector

	forceSess *concurrency.Session // lease du droit de forcer (fermée à l'arrêt)
	forceMtx  *concurrency.Mutex
	forceHeld bool
	coolUntil time.Time
}

// New crée un superviseur (config values zero → défauts ; service non lancé).
// Le client HTTP n'a pas de timeout global : chaque appel porte le sien
// (ProbeTimeout pour le sondage, 10 s pour le /failover qui laisse à Patroni
// le temps d'abandonner l'ancien leader).
func New(cfg Config, logger *slog.Logger) *Supervisor {
	cfg.Defaults()
	return &Supervisor{
		cfg:    cfg,
		logger: logger,
		http:   &http.Client{},
		nodeID: nodeIDFromEnv(),
		det:    newDetector(&cfg),
	}
}

func nodeIDFromEnv() string {
	if id := os.Getenv("AMANE_NODE_ID"); id != "" {
		return id
	}
	name, _ := os.Hostname()
	return name
}

// Run lance la boucle de détection jusqu'à l'annulation de ctx.
func (s *Supervisor) Run(ctx context.Context) error {
	if err := s.cfg.Validate(); err != nil {
		return err
	}
	cli, err := consensus.NewClient(s.cfg.EtcdEndpoints)
	if err != nil {
		return err
	}
	defer cli.Close()
	s.cli = cli
	s.hb = newHBWriter(cli, s.logger, s.cfg.HeartbeatTTL)
	defer s.hb.close()

	s.logger.Info("superviseur démarré",
		"node_id", s.nodeID,
		"scope", s.cfg.Scope,
		"heartbeat_ttl_ms", s.cfg.HeartbeatTTL.Milliseconds(),
		"stale_confirm", s.cfg.StaleConfirm,
		"probe_interval_ms", s.cfg.ProbeInterval.Milliseconds(),
		"nodes", len(s.cfg.Nodes),
	)

	ticker := time.NewTicker(s.cfg.ProbeInterval)
	defer ticker.Stop()
	for {
		select {
		case <-ctx.Done():
			s.releaseForceRight()
			s.logger.Info("superviseur arrêté")
			return nil
		case <-ticker.C:
			s.tick(ctx)
		}
	}
}

// tick : une itération — sondage REST, publication du heartbeat local, évaluation.
func (s *Supervisor) tick(ctx context.Context) {
	// Garde : etcd doit être joignable pour toute décision.
	leaderKey := "/service/" + s.cfg.Scope + "/leader"
	lresp, err := s.cli.Get(ctx, leaderKey)
	if err != nil {
		s.det.reset()
		s.logger.Warn("etcd injoignable — pas de décision ce tick")
		return
	}
	leader := ""
	if lresp.Count > 0 {
		leader = string(lresp.Kvs[0].Value)
	}

	now := time.Now()
	snap := snapshot{now: now, leader: leader, nodes: make(map[string]nodeHealth, len(s.cfg.Nodes))}

	for i := range s.cfg.Nodes {
		n := &s.cfg.Nodes[i]
		st, ok := s.probePatroni(ctx, n)
		h := nodeHealth{restOK: ok, hbPrimary: false}
		if ok {
			h.role, h.sync = st.Role, st.SyncState
		}
		// Publication du heartbeat local : le superviseur n'est l'agent heartbeat
		// QUE de son membre LOCAL (deployé sur le nœud), sinon il supprimerait le
		// heartbeat du primary quand SON lien REST tombe — la garde anti-partition
		// serait inopérante en déploiement 3 nœuds. LocalNode vide = mode mono-hôte
		// (agent de tous les nœuds sondables, tests).
		if s.cfg.LocalNode == "" || n.Name == s.cfg.LocalNode {
			s.hb.publish(n.Name, st, ok, now)
		}
		if n.Name == leader && leader != "" {
			h.hbPrimary = s.hb.leaderAlive(n.Name, s.cfg.HeartbeatTTL, now)
		}
		snap.nodes[n.Name] = h
	}
	if leader == "" {
		// Identité du leader inconnue : on la déduit du REST (premier démarrage).
		for name, h := range snap.nodes {
			if h.role == "primary" || h.role == "master" {
				leader = name
				snap.leader = name
				break
			}
		}
	}

	act := s.det.tick(snap)
	switch {
	case act.guarded:
		s.logger.Warn("garde anti partition", "leader", act.leader, "reason", act.reason)
		return
	case !act.trigger:
		if act.candidate != "" {
			s.logger.Debug("suspicion en cours",
				"leader", act.leader,
				"consecutive", s.det.consecutive,
				"candidate", act.candidate,
				"reason", act.reason,
			)
		}
		return
	case time.Now().Before(s.coolUntil):
		s.logger.Warn("failover forcé sauté — cooldown actif", "until", s.coolUntil)
		return
	}

	candidate := act.candidate
	rendezvous := s.cfg.byName(candidate)
	if rendezvous == nil {
		s.logger.Error("candidat inconnu", "candidate", candidate)
		s.det.reset()
		return
	}
	if !s.ensureForceRight(ctx) {
		s.logger.Info("droit de forcer non détenu — une autre instance agit", "leader", act.leader)
		return
	}

	start := time.Now()
	// Démotion forcée : Patroni ne promeut Pendant la détection de la lease.
	// On libère donc le lock etcd du leader mort (fencing par autorité externe,
	// SAFE : l'agent heartbeat du nœud est confirmé stale — le nœud n'est plus
	// en vie pour se battre — puis on déclenche /failover sans spécifier leader
	// (faire = qui promouvrait via un switchover exigerait l'ancien joignable).
	if err := s.releasePatroniLock(ctx, act.leader); err != nil {
		s.logger.Error("lock Patroni non libéré", "leader", act.leader, "err", err)
		s.det.reset()
		return
	}
	if err := s.forceFailover(ctx, rendezvous, act.leader, candidate); err != nil {
		s.logger.Error("failover forcé échoué", "leader", act.leader, "candidate", candidate, "err", err)
		s.det.reset()
		return
	}
	s.coolUntil = time.Now().Add(s.cfg.CoolDown)
	s.logger.Warn("failover forcé déclenché — Patroni promeut",
		"leader", act.leader,
		"candidate", candidate,
		"detection_ms", start.Sub(act.detectAt).Milliseconds(),
		"total_ms", time.Since(start).Milliseconds(),
		"reason", act.reason,
	)
}

// releasePatroniLock supprime la clé leader de Patroni (le lock) dans etcd.
// Patroni promeut dès que le cluster est déverrouillé, sans attendre la lease.
func (s *Supervisor) releasePatroniLock(ctx context.Context, leader string) error {
	key := "/service/" + s.cfg.Scope + "/leader"
	ctx, cancel := context.WithTimeout(ctx, s.cfg.ProbeTimeout)
	defer cancel()
	resp, err := s.cli.Delete(ctx, key)
	if err != nil {
		return err
	}
	if resp.Deleted == 0 {
		s.logger.Warn("lock Patroni déjà libéré par ailleurs", "leader", leader)
	}
	return nil
}

// ensureForceRight acquiert le lock « droit de forcer » (une seule instance) si
// ce superviseur ne le détient pas encore. Retourne true si le droit est détenu.
func (s *Supervisor) ensureForceRight(ctx context.Context) bool {
	if s.forceHeld {
		return true
	}
	sess, err := concurrency.NewSession(s.cli, concurrency.WithTTL(int(s.cfg.LockTTL.Seconds())))
	if err != nil {
		s.logger.Error("session droit de forcer impossible", "err", err)
		return false
	}
	m := concurrency.NewMutex(sess, forceRightKey)
	lockCtx, cancel := context.WithTimeout(ctx, s.cfg.ProbeTimeout)
	defer cancel()
	if err := m.Lock(lockCtx); err != nil {
		sess.Close()
		s.logger.Debug("droit de forcer non acquis", "err", err)
		return false
	}
	s.forceSess = sess
	s.forceMtx = m
	s.forceHeld = true
	s.logger.Info("droit de forcer acquis (élection superviseur)")
	return true
}

// releaseForceRight libère le lock « droit de forcer » (fermeture de la lease
// etcd → le mutex est relâché automatiquement, l'élection repasse à un autre).
func (s *Supervisor) releaseForceRight() {
	if s.forceSess != nil {
		s.forceSess.Close()
		s.forceSess = nil
		s.forceMtx = nil
	}
	s.forceHeld = false
}
