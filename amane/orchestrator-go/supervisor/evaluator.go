package supervisor

import "time"

// nodeHealth est la vue rafraîchie à chaque tick pour un membre du cluster.
type nodeHealth struct {
	restOK    bool
	role      string // primary / replica (peut être vide si down)
	sync      string // sync_state vu au REST (sync/async/potential)
	hbPrimary bool   // le heartbeat etcd du nœud le déclare primary (frais) ?
}

// snapshot est l'image cohérente du cluster à un instant donné.
type snapshot struct {
	now    time.Time
	leader string
	nodes  map[string]nodeHealth
}

// action est la décision du détecteur pour ce tick.
type action struct {
	trigger   bool // forcer le failover maintenant ?
	guarded   bool // suspicion écartée par une garde (partition) — on n'agit pas
	leader    string
	candidate string
	reason    string
	detectAt  time.Time // début de suspicion (pour le timing)
}

// detector accumule les ticks consécutifs de suspicion (mémoire de crash),
// sans dépendre d'aucune I/O — testable en unitaire.
type detector struct {
	cfg         *Config
	consecutive int
	detectSince time.Time
}

func newDetector(cfg *Config) *detector {
	return &detector{cfg: cfg}
}

// isReplicaSuitable : la promotion ne vise qu'un réplica REST joignable.
func isReplicaSuitable(h nodeHealth) bool {
	return h.restOK && h.role == "replica"
}

// pickCandidate choisit le candidat à promouvoir : un réplica en sync_state
// sync (zéro perte garantie), sinon tout réplica joignable. "" si aucun.
func pickCandidate(nodes map[string]nodeHealth, leader string) string {
	best := ""
	for name, h := range nodes {
		if name == leader || !isReplicaSuitable(h) {
			continue
		}
		if h.sync == "sync" {
			return name
		}
		if best == "" {
			best = name
		}
	}
	return best
}

// tick rend la décision du détecteur. Règles (garde-fous AGENTS.md) :
//   - leader REST joignable            → sain, rien à faire ;
//   - leader REST down mais heartbeat etcd FRAIS déclarant primary → partition
//     réseau (leader vivant/isolé) : ne JAMAIS forcer — seule Patroni tranche ;
//   - leader REST down ET heartbeat etcd stale/absent → crash probable (process
//     mort, agent local mort) : on suspecte puis on force (POST /failover) dès
//     StaleConfirm ticks consécutifs et un candidat disponible.
func (d *detector) tick(s snapshot) action {
	leader := s.leader

	act := action{leader: leader, detectAt: d.detectSince}
	if h, ok := s.nodes[leader]; ok && h.restOK {
		d.reset()
		return act
	}

	// Règle de partition : un leader qui bat encore son cœur (etcd, role=primary)
	// est vivant — même si ses ports REST ne répondent plus depuis notre poste.
	if h, ok := s.nodes[leader]; ok && h.hbPrimary {
		d.reset()
		act.guarded = true
		act.reason = "partition suspecte : heartbeat primary frais mais REST down — pas de forçage"
		return act
	}

	if d.consecutive == 0 {
		d.detectSince = s.now
	}
	d.consecutive++
	act.detectAt = d.detectSince

	candidate := pickCandidate(s.nodes, leader)
	if candidate == "" {
		act.reason = "leader suspecté mort mais aucun candidat joignable"
		return act
	}
	act.candidate = candidate
	if d.consecutive >= d.cfg.StaleConfirm {
		act.trigger = true
		act.reason = "leader mort (REST+heartbeat stale) — failover forcé"
	}
	return act
}

// reset reamorce le compteur après un retour à la normale ou après forçage.
func (d *detector) reset() {
	d.consecutive = 0
	var zero time.Time
	d.detectSince = zero
}
