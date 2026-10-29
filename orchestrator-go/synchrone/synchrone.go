// Package synchrone règle la réplication synchrone de Patroni selon la taille
// du cluster (refonte V3, ADR-004 décision 2) :
//
//   - à partir de 3 serveurs PostgreSQL, mode STRICT : une écriture n'est
//     acquittée que si elle existe sur deux machines ; sans standby synchrone,
//     les écritures sont bloquées ;
//   - en dessous, mode NON STRICT : le cluster continue d'écrire quand son
//     standby tombe, et la PME est prévenue (alerte « sous le seuil »).
//
// La taille retenue est le MAXIMUM de serveurs observés, mémorisé dans etcd :
// la panne d'un serveur ne doit pas retirer la protection au moment où elle
// compte le plus. Seul l'orchestrateur élu (lease etcd) applique le réglage.
package synchrone

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"log/slog"
	"net/http"
	"strconv"
	"time"

	clientv3 "go.etcd.io/etcd/client/v3"

	"github.com/amane/orchestrator-go/dcs"
)

// CleServeurs mémorise le nombre maximal de serveurs PostgreSQL observés.
// Retirer définitivement un serveur du cluster se fait en abaissant cette
// valeur (`etcdctl put /amane/config/serveurs_installes <n>`).
const CleServeurs = "/amane/config/serveurs_installes"

// Seuil à partir duquel le mode strict s'applique.
const Seuil = 3

// StrictAttendu indique le mode voulu pour ce nombre de serveurs.
func StrictAttendu(serveurs int) bool { return serveurs >= Seuil }

// NombreServeurs renvoie le maximum de serveurs observés, en l'élevant si
// `observes` le dépasse (écriture conditionnelle : deux orchestrateurs ne
// s'écrasent pas).
func NombreServeurs(ctx context.Context, cli *clientv3.Client, observes int) (int, error) {
	resp, err := cli.Get(ctx, CleServeurs)
	if err != nil {
		return 0, err
	}
	memorise, rev := 0, int64(0)
	if resp.Count > 0 {
		memorise, _ = strconv.Atoi(string(resp.Kvs[0].Value))
		rev = resp.Kvs[0].ModRevision
	}
	if observes <= memorise {
		return memorise, nil
	}
	cond := clientv3.Compare(clientv3.ModRevision(CleServeurs), "=", rev)
	if rev == 0 {
		cond = clientv3.Compare(clientv3.CreateRevision(CleServeurs), "=", 0)
	}
	if _, err := cli.Txn(ctx).If(cond).Then(clientv3.OpPut(CleServeurs, strconv.Itoa(observes))).Commit(); err != nil {
		return 0, err
	}
	return observes, nil
}

// APIPatroni lit et règle synchronous_mode_strict (configuration dynamique
// de Patroni, partagée par tout le cluster via le DCS).
type APIPatroni interface {
	LireStrict(ctx context.Context, urlAPI string) (bool, error)
	EcrireStrict(ctx context.Context, urlAPI string, strict bool) error
}

// ClientHTTP est l'implémentation réelle (GET/PATCH /config).
type ClientHTTP struct{ HTTP *http.Client }

// LireStrict lit `synchronous_mode_strict` dans GET /config.
func (c ClientHTTP) LireStrict(ctx context.Context, urlAPI string) (bool, error) {
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, urlAPI+"/config", nil)
	if err != nil {
		return false, err
	}
	resp, err := c.HTTP.Do(req)
	if err != nil {
		return false, err
	}
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusOK {
		return false, fmt.Errorf("GET /config : statut %d", resp.StatusCode)
	}
	var cfg struct {
		Strict bool `json:"synchronous_mode_strict"`
	}
	if err := json.NewDecoder(io.LimitReader(resp.Body, 1<<20)).Decode(&cfg); err != nil {
		return false, err
	}
	return cfg.Strict, nil
}

// EcrireStrict règle `synchronous_mode_strict` par PATCH /config.
func (c ClientHTTP) EcrireStrict(ctx context.Context, urlAPI string, strict bool) error {
	corps, _ := json.Marshal(map[string]bool{"synchronous_mode_strict": strict})
	req, err := http.NewRequestWithContext(ctx, http.MethodPatch, urlAPI+"/config", bytes.NewReader(corps))
	if err != nil {
		return err
	}
	req.Header.Set("Content-Type", "application/json")
	resp, err := c.HTTP.Do(req)
	if err != nil {
		return err
	}
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusOK {
		b, _ := io.ReadAll(io.LimitReader(resp.Body, 1<<12))
		return fmt.Errorf("PATCH /config : statut %d : %s", resp.StatusCode, bytes.TrimSpace(b))
	}
	return nil
}

// Appliquer règle le mode voulu sur le cluster via le premier membre dont
// l'API répond. Renvoie le mode appliqué et s'il a fallu le changer.
func Appliquer(ctx context.Context, api APIPatroni, membres []dcs.Membre, serveurs int) (strict bool, change bool, err error) {
	attendu := StrictAttendu(serveurs)
	var derniere error = fmt.Errorf("aucun membre Patroni")
	for _, m := range membres {
		actuel, err := api.LireStrict(ctx, m.URLAPI)
		if err != nil {
			derniere = err
			continue
		}
		if actuel == attendu {
			return attendu, false, nil
		}
		if err := api.EcrireStrict(ctx, m.URLAPI, attendu); err != nil {
			derniere = err
			continue
		}
		return attendu, true, nil
	}
	return attendu, false, derniere
}

// Gestionnaire applique périodiquement le réglage, s'il est l'orchestrateur élu.
type Gestionnaire struct {
	Cli     *clientv3.Client
	Scope   string
	EstElu  func() bool
	API     APIPatroni
	Periode time.Duration
	Logger  *slog.Logger
}

// Run tourne jusqu'à l'annulation de ctx.
func (g Gestionnaire) Run(ctx context.Context) {
	ticker := time.NewTicker(g.Periode)
	defer ticker.Stop()
	for {
		select {
		case <-ctx.Done():
			return
		case <-ticker.C:
			if g.EstElu != nil && !g.EstElu() {
				continue
			}
			g.tick(ctx)
		}
	}
}

func (g Gestionnaire) tick(ctx context.Context) {
	ctx, cancel := context.WithTimeout(ctx, g.Periode)
	defer cancel()
	membres, err := dcs.Membres(ctx, g.Cli, g.Scope)
	if err != nil || len(membres) == 0 {
		return
	}
	serveurs, err := NombreServeurs(ctx, g.Cli, len(membres))
	if err != nil {
		g.Logger.Warn("synchrone : nombre de serveurs illisible", "err", err)
		return
	}
	strict, change, err := Appliquer(ctx, g.API, membres, serveurs)
	switch {
	case err != nil:
		g.Logger.Warn("synchrone : réglage non appliqué", "err", err)
	case change:
		g.Logger.Info("synchrone : mode appliqué", "strict", strict, "serveurs", serveurs)
	}
}
