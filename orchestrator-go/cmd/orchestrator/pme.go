package main

// Sous-commandes de la pile PME (refonte V3, ADR-005).

import (
	"context"
	"crypto/sha256"
	"encoding/hex"
	"flag"
	"log/slog"
	"net"
	"os"
	"os/signal"
	"strings"
	"syscall"
	"time"

	clientv3 "go.etcd.io/etcd/client/v3"

	"github.com/amane/orchestrator-go/bootstrap"
	"github.com/amane/orchestrator-go/dcs"
	"github.com/amane/orchestrator-go/portail"
	"github.com/amane/orchestrator-go/provision"
)

func loggerJSON() *slog.Logger {
	l := slog.New(slog.NewJSONHandler(os.Stdout, &slog.HandlerOptions{Level: slog.LevelInfo}))
	slog.SetDefault(l)
	return l
}

// cmdInit amorce la machine avant le démarrage d'etcd : dossiers des volumes,
// puis appartenance au cluster etcd (création ou ajout). Exécuté une fois par
// démarrage de la pile, par un conteneur d'initialisation.
func cmdInit(args []string) int {
	fs := flag.NewFlagSet("init", flag.ExitOnError)
	nom := fs.String("nom", "", "nom de la machine (membre etcd et Patroni)")
	ip := fs.String("ip", "", "IP LAN de la machine, annoncée aux autres")
	urlPortail := fs.String("portail", "", "URL du portail éditeur (liste des machines déjà installées)")
	jeton := fs.String("jeton", "", "jeton d'enregistrement du tenant")
	pairs := fs.String("pairs", "", "IP des autres machines, séparées par des virgules (remplace la liste du portail)")
	nouveau := fs.Bool("nouveau-cluster", false, "créer un nouveau cluster etcd même si des machines sont connues (action explicite)")
	donnees := fs.String("donnees-etcd", "/etcd-data", "dossier de données etcd (volume partagé avec etcd)")
	config := fs.String("config-etcd", "/amane/etcd/etcd.env", "fichier de configuration etcd produit")
	secrets := fs.String("dossier-secrets", "", "dossier des secrets à préparer (serveur uniquement)")
	copier := fs.String("copier-binaire", "", "dossier où déposer le binaire statique de l'orchestrateur (lanceur de l'application, étape 3)")
	temoin := fs.String("dossier-temoin", "", "dossier de l'identifiant du témoin à préparer (témoin uniquement)")
	dossierHAProxy := fs.String("dossier-haproxy", "", "dossier HAProxy à préparer (serveur uniquement)")
	uid := fs.Int("uid-orchestrateur", 10001, "uid de l'orchestrateur, propriétaire des dossiers qu'il écrit")
	delai := fs.Duration("delai", 10*time.Minute, "attente maximale d'une machine joignable")
	fs.Parse(args)
	logger := loggerJSON()

	ctx, stop := signal.NotifyContext(context.Background(), os.Interrupt, syscall.SIGTERM)
	defer stop()

	if err := bootstrap.PreparerDossiers([]string{*secrets, *temoin}, *dossierHAProxy, *uid); err != nil {
		logger.Error("préparation des dossiers impossible", "err", err)
		return 1
	}
	if *copier != "" {
		if err := copierBinaire(*copier); err != nil {
			logger.Error("copie du lanceur impossible", "dossier", *copier, "err", err)
			return 1
		}
	}

	var listePairs []string
	switch {
	case *pairs != "":
		listePairs = strings.Split(*pairs, ",")
	case *urlPortail != "":
		// Sans réponse du portail, impossible de savoir s'il faut créer ou
		// rejoindre un cluster : on attend plutôt que de deviner.
		limite := time.Now().Add(*delai)
		for {
			l, err := portail.Nouveau(*urlPortail, *jeton).Pairs(ctx, *ip)
			if err == nil {
				listePairs = l
				break
			}
			if time.Now().After(limite) || ctx.Err() != nil {
				logger.Error("liste des machines indisponible", "err", err)
				return 1
			}
			logger.Warn("portail indisponible, nouvelle tentative", "err", err)
			time.Sleep(5 * time.Second)
		}
	}
	logger.Info("machines déjà installées", "pairs", listePairs)

	plan, err := bootstrap.PreparerEtcd(ctx, bootstrap.Options{
		Nom:            *nom,
		IP:             *ip,
		Pairs:          listePairs,
		Jeton:          jetonCluster(*jeton),
		DossierDonnees: *donnees,
		FichierConfig:  *config,
		NouveauCluster: *nouveau,
		Delai:          *delai,
		Pause:          5 * time.Second,
	}, bootstrap.APIClient{Delai: 5 * time.Second}, logger)
	if err != nil {
		logger.Error("amorçage etcd impossible", "err", err)
		return 1
	}
	logger.Info("amorçage etcd terminé", "etat", plan.Etat, "cluster", plan.Cluster, "deja_initialise", plan.DejaInitialise)
	return 0
}

// jetonCluster dérive l'initial-cluster-token etcd du jeton du tenant : deux
// PME sur un même réseau ne peuvent pas fusionner leurs clusters, et le jeton
// lui-même n'est pas recopié dans la configuration d'etcd.
func jetonCluster(jeton string) string {
	h := sha256.Sum256([]byte("amane-etcd:" + jeton))
	return "amane-" + hex.EncodeToString(h[:8])
}

// cmdTemoin signale régulièrement le témoin etcd au portail (il n'a pas de
// nœud AMANE pour le faire à sa place). Seules des métadonnées sont envoyées.
func cmdTemoin(args []string) int {
	fs := flag.NewFlagSet("temoin", flag.ExitOnError)
	urlPortail := fs.String("portail", "", "URL du portail éditeur")
	jeton := fs.String("jeton", "", "jeton d'enregistrement du tenant")
	ip := fs.String("ip", "", "IP LAN du témoin")
	nom := fs.String("nom", "", "nom de la machine affiché dans la console")
	fichierID := fs.String("identifiant", "/amane/temoin/installation-id", "fichier de l'UUID d'installation")
	periode := fs.Duration("periode", 30*time.Second, "intervalle entre deux signalements")
	fs.Parse(args)
	logger := loggerJSON()

	ctx, stop := signal.NotifyContext(context.Background(), os.Interrupt, syscall.SIGTERM)
	defer stop()

	id, err := portail.IdentifiantInstallation(*fichierID)
	if err != nil {
		logger.Error("identifiant d'installation indisponible", "err", err)
		return 1
	}
	client := portail.Nouveau(*urlPortail, *jeton)
	annonce := portail.Annonce{
		InstallationID: id,
		Adresse:        net.JoinHostPort(*ip, "2379"),
		Hostname:       *nom,
		Type:           "temoin",
	}
	logger.Info("témoin etcd démarré", "installation_id", id, "adresse", annonce.Adresse)
	signalerTemoin(ctx, client, annonce, *periode, logger)
	return 0
}

// signalerTemoin annonce le témoin au portail à intervalle régulier, jusqu'à
// l'annulation de ctx. Un portail injoignable n'arrête pas le témoin : etcd
// n'en dépend pas.
func signalerTemoin(ctx context.Context, client *portail.Client, annonce portail.Annonce, periode time.Duration, logger *slog.Logger) {
	for {
		if err := client.Annoncer(ctx, annonce); err != nil {
			logger.Warn("signalement au portail impossible", "err", err)
		}
		select {
		case <-ctx.Done():
			return
		case <-time.After(periode):
		}
	}
}

// produireSecrets obtient les secrets du cluster (créés par la première
// machine) et les écrit pour les conteneurs locaux. Réessaie tant qu'etcd
// n'est pas prêt : Spilo, le nœud et pgAdmin attendent ce fichier.
func produireSecrets(ctx context.Context, cli *clientv3.Client, chemin string, logger *slog.Logger) (bootstrap.Secrets, bool) {
	for {
		lecture, cancel := context.WithTimeout(ctx, 5*time.Second)
		s, cree, err := bootstrap.ObtenirSecrets(lecture, cli)
		cancel()
		if err == nil {
			if err := bootstrap.EcrireEnvSecrets(chemin, s); err != nil {
				logger.Error("écriture des secrets impossible", "fichier", chemin, "err", err)
				return bootstrap.Secrets{}, false
			}
			logger.Info("secrets PostgreSQL disponibles", "crees_par_cette_machine", cree)
			return s, true
		}
		logger.Warn("secrets du cluster indisponibles, nouvelle tentative", "err", err)
		select {
		case <-ctx.Done():
			return bootstrap.Secrets{}, false
		case <-time.After(2 * time.Second):
		}
	}
}

// produireSecretsApplication obtient le mot de passe de la base de
// l'application hébergée, crée son rôle et sa base sur le primaire, puis écrit
// le fichier lu par le lanceur. L'application rejoint sa base par HAProxy.
func produireSecretsApplication(ctx context.Context, cli *clientv3.Client, chemin, scope string, s bootstrap.Secrets, logger *slog.Logger) {
	for {
		lecture, cancel := context.WithTimeout(ctx, 10*time.Second)
		mdp, err := bootstrap.ObtenirSecretApplication(lecture, cli)
		if err == nil {
			var primaire dcs.Membre
			if primaire, err = dcs.Primaire(lecture, cli, scope); err == nil {
				err = provision.GarantirRole(lecture, provision.DSNSuperutilisateur(primaire.HotePG, s.Superutilisateur),
					provision.RoleApplication, provision.BaseApplication, mdp)
			}
			if err == nil {
				err = bootstrap.EcrireEnvApplication(chemin, provision.RoleApplication, mdp, "amane-haproxy", "5000", provision.BaseApplication)
			}
		}
		cancel()
		if err == nil {
			logger.Info("base de l'application prête", "role", provision.RoleApplication, "base", provision.BaseApplication)
			return
		}
		logger.Warn("base de l'application en attente", "err", err)
		select {
		case <-ctx.Done():
			return
		case <-time.After(3 * time.Second):
		}
	}
}

// provisionner crée sur le primaire le rôle et la base du nœud (idempotent),
// en réessayant tant qu'aucun primaire n'est disponible.
func provisionner(ctx context.Context, cli *clientv3.Client, scope string, s bootstrap.Secrets, logger *slog.Logger) {
	for {
		lecture, cancel := context.WithTimeout(ctx, 10*time.Second)
		primaire, err := dcs.Primaire(lecture, cli, scope)
		if err == nil {
			err = provision.Garantir(lecture, provision.DSNSuperutilisateur(primaire.HotePG, s.Superutilisateur), s.Amane)
		}
		cancel()
		if err == nil {
			logger.Info("rôle et base du nœud prêts sur le primaire", "role", provision.Role, "base", provision.Base)
			return
		}
		logger.Warn("provision PostgreSQL en attente", "err", err)
		select {
		case <-ctx.Done():
			return
		case <-time.After(3 * time.Second):
		}
	}
}
