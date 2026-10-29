// Package lanceur démarre l'application externe hébergée (refonte V3,
// étape 3, ADR-007) SANS modifier son image : il est le point d'entrée du
// conteneur de l'application, lit les identifiants de sa base (générés sur le
// réseau de la PME), compose les variables d'environnement que l'application
// attend (ex. DATABASE_URL, décrites par son manifeste), puis exécute la
// commande d'origine de l'image à sa place.
//
// Le binaire de l'orchestrateur est statique : il s'exécute dans n'importe
// quelle image Linux, avec ou sans shell.
package lanceur

import (
	"bufio"
	"encoding/json"
	"fmt"
	"os"
	"regexp"
	"sort"
	"strings"
)

// Espaces réservés utilisables dans les modèles de variables du manifeste,
// et variable du fichier de secrets qui leur donne leur valeur.
var espacesReserves = map[string]string{
	"utilisateur":  "AMANE_APP_DB_UTILISATEUR",
	"mot_de_passe": "AMANE_APP_DB_MOT_DE_PASSE",
	"hote":         "AMANE_APP_DB_HOTE",
	"port":         "AMANE_APP_DB_PORT",
	"base":         "AMANE_APP_DB_BASE",
}

var espaceReserve = regexp.MustCompile(`\{([a-z_]+)\}`)

// ChargerEnv lit un fichier écrit par l'amorçage (`export CLE='valeur'`).
func ChargerEnv(chemin string) (map[string]string, error) {
	f, err := os.Open(chemin)
	if err != nil {
		return nil, err
	}
	defer f.Close()
	vars := map[string]string{}
	lecteur := bufio.NewScanner(f)
	for lecteur.Scan() {
		ligne := strings.TrimSpace(lecteur.Text())
		if ligne == "" || strings.HasPrefix(ligne, "#") {
			continue
		}
		ligne = strings.TrimPrefix(ligne, "export ")
		cle, valeur, ok := strings.Cut(ligne, "=")
		if !ok {
			return nil, fmt.Errorf("ligne illisible dans %s", chemin)
		}
		vars[cle] = strings.Trim(valeur, "'")
	}
	return vars, lecteur.Err()
}

// DecoderModeles lit AMANE_MODELES : {"DATABASE_URL": "postgres://{utilisateur}:…"}.
func DecoderModeles(brut string) (map[string]string, error) {
	if strings.TrimSpace(brut) == "" {
		return map[string]string{}, nil
	}
	modeles := map[string]string{}
	if err := json.Unmarshal([]byte(brut), &modeles); err != nil {
		return nil, fmt.Errorf("AMANE_MODELES illisible : %w", err)
	}
	return modeles, nil
}

// Rendre remplace les espaces réservés de chaque modèle. Un espace réservé
// inconnu est une erreur : mieux vaut ne pas démarrer qu'injecter une valeur
// fausse.
func Rendre(modeles, secrets map[string]string) (map[string]string, error) {
	sortie := make(map[string]string, len(modeles))
	for cle, modele := range modeles {
		var erreur error
		valeur := espaceReserve.ReplaceAllStringFunc(modele, func(m string) string {
			nom := espaceReserve.FindStringSubmatch(m)[1]
			source, connu := espacesReserves[nom]
			if !connu {
				erreur = fmt.Errorf("variable %s : espace réservé inconnu {%s}", cle, nom)
				return m
			}
			v, present := secrets[source]
			if !present {
				erreur = fmt.Errorf("variable %s : %s absent des secrets", cle, source)
			}
			return v
		})
		if erreur != nil {
			return nil, erreur
		}
		sortie[cle] = valeur
	}
	return sortie, nil
}

// Environnement ajoute (ou remplace) les variables rendues dans
// l'environnement courant. Résultat trié pour être déterministe.
func Environnement(courant []string, ajouts map[string]string) []string {
	vars := map[string]string{}
	for _, e := range courant {
		if cle, valeur, ok := strings.Cut(e, "="); ok {
			vars[cle] = valeur
		}
	}
	for cle, valeur := range ajouts {
		vars[cle] = valeur
	}
	sortie := make([]string, 0, len(vars))
	for cle, valeur := range vars {
		sortie = append(sortie, cle+"="+valeur)
	}
	sort.Strings(sortie)
	return sortie
}
