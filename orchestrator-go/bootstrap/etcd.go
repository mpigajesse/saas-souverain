// Package bootstrap amorce une machine PME avant le démarrage de sa pile HA
// (ADR-005) : appartenance au cluster etcd, secrets PostgreSQL partagés,
// dossiers des volumes. Il est exécuté par le conteneur d'initialisation
// (`orchestrator init`), une fois, avant etcd.
//
// Le portail de l'éditeur ne fournit que des métadonnées (IP des machines
// déjà installées) ; l'ajout au cluster se fait directement sur le réseau
// local de la PME.
package bootstrap

import (
	"context"
	"errors"
	"fmt"
	"log/slog"
	"net"
	"os"
	"path/filepath"
	"regexp"
	"sort"
	"strings"
	"time"

	"go.etcd.io/etcd/api/v3/etcdserverpb"
	clientv3 "go.etcd.io/etcd/client/v3"

	"github.com/amane/orchestrator-go/consensus"
)

// ErrPairsInjoignables : des machines sont connues mais aucune ne répond. Créer
// un nouveau cluster à ce stade produirait deux clusters indépendants, donc
// deux primaires : l'amorçage refuse.
var ErrPairsInjoignables = errors.New("machines connues mais injoignables : refus de créer un cluster etcd séparé")

// ErrAjoutEnCours : une autre machine a été ajoutée au cluster sans avoir
// encore démarré son etcd. Son nom est inconnu, la configuration initiale ne
// peut pas être écrite ; il faut attendre son démarrage (ou la retirer).
var ErrAjoutEnCours = errors.New("une autre machine est en cours d'ajout au cluster etcd")

var nomValide = regexp.MustCompile(`^[A-Za-z0-9][A-Za-z0-9_.-]{0,62}$`)

// MembreEtcd est un membre du cluster etcd tel que le décrit l'API MemberList.
type MembreEtcd struct {
	Nom      string // vide tant que le membre ajouté n'a pas démarré
	PeerURLs []string
}

// APIMembres interroge le cluster etcd d'une autre machine de la PME.
type APIMembres interface {
	Lister(ctx context.Context, ip string) ([]MembreEtcd, error)
	Ajouter(ctx context.Context, ip, peerURL string) ([]MembreEtcd, error)
}

// Options de l'amorçage etcd d'une machine.
type Options struct {
	Nom            string        // nom du membre etcd (et du membre Patroni)
	IP             string        // IP LAN annoncée aux autres machines
	Pairs          []string      // IP des autres machines connues du portail
	Jeton          string        // initial-cluster-token (propre au tenant)
	DossierDonnees string        // data-dir etcd (même chemin dans le conteneur etcd)
	FichierConfig  string        // fichier de variables ETCD_* produit
	NouveauCluster bool          // créer un cluster malgré des pairs connus (action explicite)
	Delai          time.Duration // attente maximale d'un pair joignable
	Pause          time.Duration // pause entre deux tentatives
}

// Plan est la configuration initiale décidée pour etcd.
type Plan struct {
	DejaInitialise bool   // données etcd présentes : rien à décider
	Etat           string // "new" ou "existing"
	Cluster        string // ETCD_INITIAL_CLUSTER : nom=url,...
}

func (o Options) peerURL() string { return "http://" + net.JoinHostPort(o.IP, "2380") }

func (o Options) valider() error {
	if !nomValide.MatchString(o.Nom) {
		return fmt.Errorf("nom de machine invalide pour etcd : %q", o.Nom)
	}
	if net.ParseIP(o.IP) == nil {
		return fmt.Errorf("IP LAN invalide : %q", o.IP)
	}
	if o.Jeton == "" || o.DossierDonnees == "" || o.FichierConfig == "" {
		return errors.New("jeton, dossier de données et fichier de configuration requis")
	}
	return nil
}

// PreparerEtcd décide comment etcd doit démarrer sur cette machine, ajoute la
// machine au cluster existant si besoin, et écrit la configuration.
func PreparerEtcd(ctx context.Context, o Options, api APIMembres, logger *slog.Logger) (Plan, error) {
	if err := o.valider(); err != nil {
		return Plan{}, err
	}
	if _, err := os.Stat(filepath.Join(o.DossierDonnees, "member")); err == nil {
		// Redémarrage : etcd ignore toute configuration initiale une fois ses
		// données créées. La configuration de la première fois fait foi.
		if _, err := os.Stat(o.FichierConfig); err != nil {
			return Plan{}, fmt.Errorf("données etcd présentes mais configuration %s absente", o.FichierConfig)
		}
		logger.Info("etcd déjà initialisé sur cette machine — configuration conservée")
		return Plan{DejaInitialise: true}, nil
	}

	pairs := autresIP(o.Pairs, o.IP)
	if o.NouveauCluster || len(pairs) == 0 {
		plan := Plan{Etat: "new", Cluster: o.Nom + "=" + o.peerURL()}
		logger.Info("création d'un nouveau cluster etcd", "nom", o.Nom, "nouveau_cluster_force", o.NouveauCluster)
		return plan, EcrireConfig(o, plan)
	}

	limite := time.Now().Add(o.Delai)
	for {
		plan, err := rejoindre(ctx, o, pairs, api, logger)
		if err == nil {
			logger.Info("machine ajoutée au cluster etcd", "cluster", plan.Cluster)
			return plan, EcrireConfig(o, plan)
		}
		if !errors.Is(err, ErrPairsInjoignables) && !errors.Is(err, ErrAjoutEnCours) {
			return Plan{}, err
		}
		if time.Now().After(limite) {
			return Plan{}, err
		}
		logger.Warn("cluster etcd pas encore rejoignable, nouvelle tentative", "raison", err, "pause", o.Pause)
		select {
		case <-ctx.Done():
			return Plan{}, ctx.Err()
		case <-time.After(o.Pause):
		}
	}
}

// rejoindre fait une passe sur les pairs : le premier joignable ajoute cette
// machine (si elle n'y est pas déjà) et fournit la liste des membres.
func rejoindre(ctx context.Context, o Options, pairs []string, api APIMembres, logger *slog.Logger) (Plan, error) {
	peer := o.peerURL()
	for _, ip := range pairs {
		membres, err := api.Lister(ctx, ip)
		if err != nil {
			logger.Debug("pair etcd injoignable", "ip", ip, "err", err)
			continue
		}
		if !contientPeer(membres, peer) {
			membres, err = api.Ajouter(ctx, ip, peer)
			if err != nil {
				logger.Warn("ajout au cluster etcd refusé", "ip", ip, "err", err)
				continue
			}
		}
		return planifier(o.Nom, peer, membres)
	}
	return Plan{}, ErrPairsInjoignables
}

// planifier construit ETCD_INITIAL_CLUSTER à partir des membres du cluster
// (cette machine comprise). Fonction pure.
func planifier(nom, peer string, membres []MembreEtcd) (Plan, error) {
	var parties []string
	soi := false
	for _, m := range membres {
		if len(m.PeerURLs) == 0 {
			continue
		}
		n := m.Nom
		if contient(m.PeerURLs, peer) {
			n, soi = nom, true
		} else if n == "" {
			return Plan{}, ErrAjoutEnCours
		}
		parties = append(parties, n+"="+m.PeerURLs[0])
	}
	if !soi {
		return Plan{}, fmt.Errorf("cette machine (%s) absente de la liste des membres etcd", peer)
	}
	sort.Strings(parties)
	return Plan{Etat: "existing", Cluster: strings.Join(parties, ",")}, nil
}

// EcrireConfig écrit les variables ETCD_* lues par le conteneur etcd au
// démarrage (`. etcd.env && exec etcd`).
func EcrireConfig(o Options, p Plan) error {
	ip := net.JoinHostPort(o.IP, "2379")
	variables := [][2]string{
		{"ETCD_NAME", o.Nom},
		{"ETCD_DATA_DIR", o.DossierDonnees},
		{"ETCD_LISTEN_CLIENT_URLS", "http://0.0.0.0:2379"},
		{"ETCD_ADVERTISE_CLIENT_URLS", "http://" + ip},
		{"ETCD_LISTEN_PEER_URLS", "http://0.0.0.0:2380"},
		{"ETCD_INITIAL_ADVERTISE_PEER_URLS", o.peerURL()},
		{"ETCD_INITIAL_CLUSTER", p.Cluster},
		{"ETCD_INITIAL_CLUSTER_STATE", p.Etat},
		{"ETCD_INITIAL_CLUSTER_TOKEN", o.Jeton},
		// Accepte le keepalive gRPC des orchestrateurs (consensus/client.go).
		{"ETCD_GRPC_KEEPALIVE_MIN_TIME", consensus.KeepAliveTime.String()},
	}
	return ecrireEnv(o.FichierConfig, variables, 0o644)
}

// ecrireEnv écrit un fichier de variables chargeable par `sh` (`. fichier`),
// de façon atomique.
func ecrireEnv(chemin string, variables [][2]string, mode os.FileMode) error {
	var b strings.Builder
	b.WriteString("# Généré par l'orchestrateur AMANE (amorçage) — ne pas modifier.\n")
	for _, v := range variables {
		if strings.ContainsAny(v[1], "'\n") {
			return fmt.Errorf("valeur non exportable pour %s", v[0])
		}
		fmt.Fprintf(&b, "export %s='%s'\n", v[0], v[1])
	}
	if err := os.MkdirAll(filepath.Dir(chemin), 0o755); err != nil {
		return err
	}
	tmp := chemin + ".tmp"
	if err := os.WriteFile(tmp, []byte(b.String()), mode); err != nil {
		return err
	}
	return os.Rename(tmp, chemin)
}

func autresIP(pairs []string, soi string) []string {
	vus := map[string]bool{soi: true, "": true}
	var out []string
	for _, p := range pairs {
		p = strings.TrimSpace(p)
		if !vus[p] {
			vus[p] = true
			out = append(out, p)
		}
	}
	return out
}

func contient(liste []string, v string) bool {
	for _, x := range liste {
		if x == v {
			return true
		}
	}
	return false
}

func contientPeer(membres []MembreEtcd, peer string) bool {
	for _, m := range membres {
		if contient(m.PeerURLs, peer) {
			return true
		}
	}
	return false
}

// APIClient est l'implémentation réelle d'APIMembres (client etcd v3).
type APIClient struct {
	Delai time.Duration // délai de chaque appel
}

// appeler ouvre un client vers le etcd de cette IP le temps d'un appel.
func (a APIClient) appeler(ctx context.Context, ip string, f func(context.Context, *clientv3.Client) ([]*etcdserverpb.Member, error)) ([]MembreEtcd, error) {
	cli, err := consensus.NewClient([]string{"http://" + net.JoinHostPort(ip, "2379")})
	if err != nil {
		return nil, err
	}
	defer cli.Close()
	ctx, cancel := context.WithTimeout(ctx, a.Delai)
	defer cancel()
	membres, err := f(ctx, cli)
	if err != nil {
		return nil, err
	}
	out := make([]MembreEtcd, 0, len(membres))
	for _, m := range membres {
		out = append(out, MembreEtcd{Nom: m.Name, PeerURLs: m.PeerURLs})
	}
	return out, nil
}

// Lister renvoie les membres du cluster etcd joint par cette IP.
func (a APIClient) Lister(ctx context.Context, ip string) ([]MembreEtcd, error) {
	return a.appeler(ctx, ip, func(ctx context.Context, cli *clientv3.Client) ([]*etcdserverpb.Member, error) {
		r, err := cli.MemberList(ctx)
		if err != nil {
			return nil, err
		}
		return r.Members, nil
	})
}

// Ajouter déclare cette machine comme nouveau membre et renvoie la liste à jour.
func (a APIClient) Ajouter(ctx context.Context, ip, peerURL string) ([]MembreEtcd, error) {
	return a.appeler(ctx, ip, func(ctx context.Context, cli *clientv3.Client) ([]*etcdserverpb.Member, error) {
		r, err := cli.MemberAdd(ctx, []string{peerURL})
		if err != nil {
			return nil, err
		}
		return r.Members, nil
	})
}
