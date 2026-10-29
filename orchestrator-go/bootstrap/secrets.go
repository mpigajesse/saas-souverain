package bootstrap

import (
	"context"
	"crypto/rand"
	"encoding/base64"
	"encoding/json"
	"errors"
	"fmt"
	"os"

	clientv3 "go.etcd.io/etcd/client/v3"
)

// CleSecrets est la clé etcd des mots de passe PostgreSQL du cluster.
const CleSecrets = "/amane/secrets/postgresql"

// Secrets sont les mots de passe partagés par toutes les machines de la PME
// (ADR-005, décision 4). Générés aléatoirement par la première machine et
// transmis aux suivantes par etcd : ils ne quittent jamais le réseau de la
// PME, l'éditeur ne peut ni les connaître ni les recalculer.
type Secrets struct {
	Superutilisateur string `json:"superutilisateur"` // rôle postgres (Spilo)
	Replication      string `json:"replication"`      // rôle standby (Spilo)
	Amane            string `json:"amane"`            // rôle amane : nœud AMANE, pgAdmin (provision)
	PgAdmin          string `json:"pgadmin"`          // connexion web à pgAdmin
}

func (s Secrets) complets() bool {
	return s.Superutilisateur != "" && s.Replication != "" && s.Amane != "" && s.PgAdmin != ""
}

func motDePasse() (string, error) {
	b := make([]byte, 24)
	if _, err := rand.Read(b); err != nil {
		return "", err
	}
	return base64.RawURLEncoding.EncodeToString(b), nil
}

func genererSecrets() (Secrets, error) {
	var s Secrets
	for _, champ := range []*string{&s.Superutilisateur, &s.Replication, &s.Amane, &s.PgAdmin} {
		m, err := motDePasse()
		if err != nil {
			return Secrets{}, err
		}
		*champ = m
	}
	return s, nil
}

// ObtenirSecrets lit les secrets du cluster, ou les crée s'ils n'existent pas
// encore. La création est une transaction etcd « si absent » : deux machines
// qui démarrent ensemble obtiennent les mêmes secrets. Renvoie true si cette
// machine les a créés.
func ObtenirSecrets(ctx context.Context, cli *clientv3.Client) (Secrets, bool, error) {
	nouveaux, err := genererSecrets()
	if err != nil {
		return Secrets{}, false, err
	}
	valeur, err := json.Marshal(nouveaux)
	if err != nil {
		return Secrets{}, false, err
	}
	resp, err := cli.Txn(ctx).
		If(clientv3.Compare(clientv3.CreateRevision(CleSecrets), "=", 0)).
		Then(clientv3.OpPut(CleSecrets, string(valeur))).
		Else(clientv3.OpGet(CleSecrets)).
		Commit()
	if err != nil {
		return Secrets{}, false, err
	}
	if resp.Succeeded {
		return nouveaux, true, nil
	}
	kvs := resp.Responses[0].GetResponseRange().Kvs
	if len(kvs) != 1 {
		return Secrets{}, false, errors.New("secrets du cluster introuvables")
	}
	var existants Secrets
	if err := json.Unmarshal(kvs[0].Value, &existants); err != nil || !existants.complets() {
		return Secrets{}, false, fmt.Errorf("secrets du cluster illisibles dans %s", CleSecrets)
	}
	return existants, false, nil
}

// EcrireEnvSecrets écrit les secrets sous forme de variables chargées par les
// conteneurs de la machine (Spilo, nœud, pgAdmin) au démarrage. Le fichier
// reste sur le volume local de la machine.
func EcrireEnvSecrets(chemin string, s Secrets) error {
	return ecrireEnv(chemin, [][2]string{
		{"PGPASSWORD_SUPERUSER", s.Superutilisateur},
		{"PGPASSWORD_STANDBY", s.Replication},
		{"AMANE_PG_PASSWORD", s.Amane},
		{"PGADMIN_DEFAULT_PASSWORD", s.PgAdmin},
	}, 0o644)
}

// PreparerDossiers crée les dossiers des volumes partagés et leur donne les
// droits attendus par chaque conteneur : l'orchestrateur (uid non privilégié)
// écrit secrets, identifiant du témoin et configuration HAProxy ; HAProxy
// (autre uid) crée son socket maître dans le dossier HAProxy. Exécuté en root
// par `init`.
func PreparerDossiers(dossiersOrchestrateur []string, dossierHAProxy string, uidOrchestrateur int) error {
	for _, d := range append(dossiersOrchestrateur, dossierHAProxy) {
		if d == "" {
			continue
		}
		if err := os.MkdirAll(d, 0o755); err != nil {
			return err
		}
		if err := os.Chown(d, uidOrchestrateur, uidOrchestrateur); err != nil && !errors.Is(err, errors.ErrUnsupported) {
			return fmt.Errorf("droits de %s : %w", d, err)
		}
	}
	if dossierHAProxy != "" {
		// Accessible en écriture à HAProxy pour son socket maître ; volume
		// local à la machine, jamais exposé.
		if err := os.Chmod(dossierHAProxy, 0o777); err != nil {
			return err
		}
	}
	return nil
}
