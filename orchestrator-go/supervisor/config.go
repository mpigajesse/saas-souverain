package supervisor

import (
	"errors"
	"time"
)

// Node décrit un membre du cluster PostgreSQL (Patroni) supervisé : son nom
// (membre Patroni, ex. postgres-primary) et son endpoint REST Patroni.
type Node struct {
	Name       string `json:"name"`
	PatroniURL string `json:"patroni_url"`
}

// Config paramètre le superviseur. Les valeurs zero tombent sur des défauts.
type Config struct {
	EtcdEndpoints []string
	Scope         string        // scope Patroni (préfixe DCS /service/<scope>/)
	Nodes         []Node        // membres du cluster PostgreSQL
	LocalNode     string        // membre dont ce superviseur est l'agent local (vide = mode mono-hôte)
	ProbeInterval time.Duration // cadence de sondage REST Patroni
	ProbeTimeout  time.Duration // timeout d'un probe REST
	HeartbeatTTL  time.Duration // staleness etcd (réécriture du nœud) au-delà de qui on suspecte un crash
	StaleConfirm  int           // ticks consécutifs de suspicion avant de forcer (> 0)
	LockTTL       time.Duration // lease du lock « droit de forcer » (élection du superviseur actif)
	CoolDown      time.Duration // pause après un forçage
}

// Defaults applique les valeurs par défaut (budget failover < 5 s : détection
// ~1,5-2 s + /failover Patroni ~2 s).
func (c *Config) Defaults() {
	if c.Scope == "" {
		c.Scope = "amane"
	}
	if c.ProbeInterval <= 0 {
		c.ProbeInterval = 500 * time.Millisecond
	}
	if c.ProbeTimeout <= 0 {
		c.ProbeTimeout = 300 * time.Millisecond
	}
	if c.HeartbeatTTL <= 0 {
		c.HeartbeatTTL = 2 * time.Second
	}
	if c.StaleConfirm <= 0 {
		c.StaleConfirm = 2
	}
	if c.LockTTL <= 0 {
		c.LockTTL = 3 * time.Second
	}
	if c.CoolDown <= 0 {
		c.CoolDown = 30 * time.Second
	}
}

// Validate vérifie la cohérence de la configuration.
func (c *Config) Validate() error {
	if len(c.EtcdEndpoints) == 0 {
		return errors.New("etcd_endpoints requis")
	}
	if c.Scope == "" {
		return errors.New("scope requis")
	}
	if len(c.Nodes) == 0 {
		return errors.New("au moins un nœud Patroni requis")
	}
	return nil
}

// byName retourne le nœud de ce nom, ou nil.
func (c *Config) byName(name string) *Node {
	for i := range c.Nodes {
		if c.Nodes[i].Name == name {
			return &c.Nodes[i]
		}
	}
	return nil
}
