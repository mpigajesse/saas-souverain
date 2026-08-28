package supervisor

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"time"
)

// patroniStatus est le sous-ensemble de GET /patroni utilisé par le superviseur.
type patroniStatus struct {
	Role      string `json:"role"`       // primary / replica
	SyncState string `json:"sync_state"` // sync / async / potential
}

// probePatroni interroge GET /patroni d'un nœud avec un timeout court.
// Retourne (false, nil) si le nœud ne répond pas (probablement down).
func (s *Supervisor) probePatroni(ctx context.Context, n *Node) (patroniStatus, bool) {
	ctx, cancel := context.WithTimeout(ctx, s.cfg.ProbeTimeout)
	defer cancel()
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, n.PatroniURL+"/patroni", nil)
	if err != nil {
		s.logger.Error("requête Patroni impossible", "node", n.Name, "err", err)
		return patroniStatus{}, false
	}
	resp, err := s.http.Do(req)
	if err != nil {
		return patroniStatus{}, false
	}
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusOK {
		return patroniStatus{}, false
	}
	var st patroniStatus
	if err := json.NewDecoder(io.LimitReader(resp.Body, 1<<20)).Decode(&st); err != nil {
		return patroniStatus{}, false
	}
	return st, true
}

func (s *Supervisor) isPrimaryRole(role string) bool {
	return role == "primary" || role == "master"
}

// forceFailover déclenche POST /failover (SANS "leader" : avec leader, Patroni
// bascule en switchover et exige l'ancien primary joignable pour le démette —
// impossible sur un crash ; sans leader, c'est un vrai failover vers candidate).
//
// Le lock etcd du leader doit avoir été libéré avant (releasePatroniLock) :
// c'est cette libération qui permet au candidat de s'approprier le lock
// immédiatement au lieu d'attendre l'expiration de la lease (ttl >= 20 s).
func (s *Supervisor) forceFailover(ctx context.Context, rendezvous *Node, leader, candidate string) error {
	payload, err := json.Marshal(map[string]any{
		"candidate": candidate,
		"force":     true,
		"timeout":   3, // secondes max d'attente de l'ancien leader avant de forcer
	})
	if err != nil {
		return fmt.Errorf("encodage failover: %w", err)
	}
	url := fmt.Sprintf("%s/failover?force=true", rendezvous.PatroniURL)
	reqCtx, cancel := context.WithTimeout(ctx, 25*time.Second)
	defer cancel()
	req, err := http.NewRequestWithContext(reqCtx, http.MethodPost, url, bytes.NewReader(payload))
	if err != nil {
		return fmt.Errorf("requête failover: %w", err)
	}
	req.Header.Set("Content-Type", "application/json")
	resp, err := s.http.Do(req)
	if err != nil {
		return fmt.Errorf("POST /failover: %w", err)
	}
	defer resp.Body.Close()
	b, _ := io.ReadAll(io.LimitReader(resp.Body, 1<<16))
	if resp.StatusCode != http.StatusOK {
		return fmt.Errorf("POST /failover refusé (%d): %s", resp.StatusCode, string(b))
	}
	// Patroni répond 200 quand la promotion est terminée (poll synchrone interne).
	return nil
}

var _ = time.Second
