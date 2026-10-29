// Package haproxy maintient la configuration du HAProxy local de chaque
// machine PME (ADR-005, décision 5) : une adresse fixe qui mène toujours au
// primaire PostgreSQL du moment. La liste des serveurs suit les membres que
// Patroni enregistre dans etcd ; à chaque changement, la configuration est
// réécrite puis HAProxy est rechargé par son socket maître.
//
// Le primaire est désigné par Patroni lui-même : HAProxy interroge
// GET /primary sur l'API REST de chaque membre, qui ne répond 200 que sur le
// leader. Aucune décision de routage n'est prise ici.
package haproxy

import (
	"context"
	"fmt"
	"io"
	"log/slog"
	"net"
	"net/url"
	"os"
	"path/filepath"
	"regexp"
	"strings"
	"time"

	clientv3 "go.etcd.io/etcd/client/v3"

	"github.com/amane/orchestrator-go/dcs"
)

// FichierConfig et SocketMaitre sont relatifs au dossier partagé avec le
// conteneur HAProxy (volume de la machine).
const (
	FichierConfig = "haproxy.cfg"
	SocketMaitre  = "master.sock"
)

var nomInvalide = regexp.MustCompile(`[^A-Za-z0-9_.-]`)

// Rendre produit la configuration HAProxy pour ces membres. Le résultat est
// déterministe (membres triés par dcs.Membres) : une configuration identique
// ne provoque aucun rechargement.
func Rendre(membres []dcs.Membre, port int) string {
	var b strings.Builder
	b.WriteString(`# Généré par l'orchestrateur AMANE à partir des membres Patroni (etcd).
# Ne pas modifier : réécrit à chaque changement de membres.
global
    maxconn 1000
    # Après un rechargement, l'ancien processus garde ses connexions ouvertes
    # jusqu'à leur fin. Une connexion vers une machine morte ne reçoit jamais
    # de réponse : sans cette borne, le client resterait figé (constaté en
    # laboratoire pendant une bascule).
    hard-stop-after 30s

defaults
    mode tcp
    timeout connect 3s
    timeout client 30m
    timeout server 30m
    timeout check 2s
    # Connexion en cours d'établissement vers un serveur tombé : nouvel essai
    # sur un autre serveur sain (le nouveau primaire) plutôt qu'un échec.
    retries 2
    option redispatch

`)
	fmt.Fprintf(&b, "listen primaire\n    bind *:%d\n", port)
	b.WriteString(`    option httpchk GET /primary
    http-check expect status 200
    # Un primaire déclassé coupe ses sessions : les clients se reconnectent
    # aussitôt au nouveau primaire au lieu de rester sur un serveur en lecture.
    default-server inter 1s fall 2 rise 2 on-marked-down shutdown-sessions
`)
	for _, m := range membres {
		portAPI := "8008"
		if u, err := url.Parse(m.URLAPI); err == nil && u.Port() != "" {
			portAPI = u.Port()
		}
		fmt.Fprintf(&b, "    server %s %s check port %s\n", nomInvalide.ReplaceAllString(m.Nom, "_"), m.HotePG, portAPI)
	}
	return b.String()
}

// Gestionnaire tient la configuration HAProxy à jour.
type Gestionnaire struct {
	cli      *clientv3.Client
	scope    string
	dossier  string
	port     int
	periode  time.Duration
	logger   *slog.Logger
	actuelle string
}

// NouveauGestionnaire prépare un gestionnaire ; rien n'est écrit avant Run.
func NouveauGestionnaire(cli *clientv3.Client, scope, dossier string, port int, logger *slog.Logger) *Gestionnaire {
	return &Gestionnaire{cli: cli, scope: scope, dossier: dossier, port: port, periode: 2 * time.Second, logger: logger}
}

// Run écrit une première configuration (éventuellement sans serveur, pour que
// HAProxy puisse démarrer) puis la tient à jour jusqu'à l'annulation de ctx.
func (g *Gestionnaire) Run(ctx context.Context) error {
	if err := g.appliquer(ctx, nil); err != nil {
		return err
	}
	ticker := time.NewTicker(g.periode)
	defer ticker.Stop()
	for {
		select {
		case <-ctx.Done():
			return nil
		case <-ticker.C:
			lecture, cancel := context.WithTimeout(ctx, g.periode)
			membres, err := dcs.Membres(lecture, g.cli, g.scope)
			cancel()
			if err != nil {
				g.logger.Warn("haproxy : membres Patroni illisibles, configuration inchangée", "err", err)
				continue
			}
			if err := g.appliquer(ctx, membres); err != nil {
				g.logger.Error("haproxy : configuration non appliquée", "err", err)
			}
		}
	}
}

func (g *Gestionnaire) appliquer(ctx context.Context, membres []dcs.Membre) error {
	cfg := Rendre(membres, g.port)
	if cfg == g.actuelle {
		return nil
	}
	if err := ecrireAtomique(filepath.Join(g.dossier, FichierConfig), cfg); err != nil {
		return err
	}
	premiere := g.actuelle == ""
	g.actuelle = cfg
	g.logger.Info("haproxy : configuration écrite", "serveurs", len(membres))
	if premiere {
		// HAProxy n'est pas encore démarré : il lira ce fichier à son lancement.
		return nil
	}
	if err := Recharger(ctx, filepath.Join(g.dossier, SocketMaitre)); err != nil {
		// HAProxy relira de toute façon le fichier à son prochain démarrage.
		g.logger.Warn("haproxy : rechargement impossible", "err", err)
	}
	return nil
}

// Recharger demande au processus maître de HAProxy de relire sa configuration
// (commande « reload » du CLI maître, HAProxy lancé avec -W -S <socket>).
func Recharger(ctx context.Context, socket string) error {
	var d net.Dialer
	ctx, cancel := context.WithTimeout(ctx, 5*time.Second)
	defer cancel()
	conn, err := d.DialContext(ctx, "unix", socket)
	if err != nil {
		return fmt.Errorf("socket maître %s : %w", socket, err)
	}
	defer conn.Close()
	if dl, ok := ctx.Deadline(); ok {
		conn.SetDeadline(dl)
	}
	if _, err := io.WriteString(conn, "reload\n"); err != nil {
		return fmt.Errorf("commande reload : %w", err)
	}
	reponse, _ := io.ReadAll(conn)
	if strings.Contains(string(reponse), "Success=0") {
		return fmt.Errorf("HAProxy a refusé la configuration : %s", strings.TrimSpace(string(reponse)))
	}
	return nil
}

// ecrireAtomique remplace le fichier sans jamais l'exposer à moitié écrit.
func ecrireAtomique(chemin, contenu string) error {
	tmp := chemin + ".tmp"
	if err := os.WriteFile(tmp, []byte(contenu), 0o644); err != nil {
		return err
	}
	return os.Rename(tmp, chemin)
}
