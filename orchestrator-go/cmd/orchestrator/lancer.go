package main

import (
	"errors"
	"flag"
	"fmt"
	"io"
	"os"
	"os/exec"
	"path/filepath"
	"time"

	"github.com/amane/orchestrator-go/lanceur"
)

// cmdLancer est le point d'entrée du conteneur de l'application hébergée
// (ADR-007) : `orchestrator lancer -secrets <fichier> -- <commande d'origine>`.
// Il attend les identifiants de la base, compose les variables décrites par
// le manifeste (AMANE_MODELES), puis remplace son processus par la commande
// d'origine de l'image.
func cmdLancer(args []string) int {
	fs := flag.NewFlagSet("lancer", flag.ExitOnError)
	secrets := fs.String("secrets", "/amane/secrets/application.env", "fichier des identifiants de la base de l'application")
	attente := fs.Duration("attente", 10*time.Minute, "attente maximale du fichier de secrets")
	fs.Parse(args)
	commande := fs.Args()
	if len(commande) == 0 {
		fmt.Fprintln(os.Stderr, "lancer : commande d'origine de l'application manquante (après --)")
		return 2
	}

	limite := time.Now().Add(*attente)
	for {
		if _, err := os.Stat(*secrets); err == nil {
			break
		}
		if time.Now().After(limite) {
			fmt.Fprintf(os.Stderr, "lancer : %s toujours absent après %s\n", *secrets, *attente)
			return 1
		}
		time.Sleep(time.Second)
	}
	valeurs, err := lanceur.ChargerEnv(*secrets)
	if err != nil {
		fmt.Fprintln(os.Stderr, "lancer :", err)
		return 1
	}
	modeles, err := lanceur.DecoderModeles(os.Getenv("AMANE_MODELES"))
	if err != nil {
		fmt.Fprintln(os.Stderr, "lancer :", err)
		return 1
	}
	rendus, err := lanceur.Rendre(modeles, valeurs)
	if err != nil {
		fmt.Fprintln(os.Stderr, "lancer :", err)
		return 1
	}
	chemin, err := exec.LookPath(commande[0])
	if err != nil {
		fmt.Fprintln(os.Stderr, "lancer : commande introuvable :", err)
		return 1
	}
	// En cas de succès, executer ne revient pas : l'application remplace le
	// lanceur (même PID, signaux reçus directement).
	err = executer(chemin, commande, lanceur.Environnement(os.Environ(), rendus))
	fmt.Fprintln(os.Stderr, "lancer : exécution impossible :", err)
	return 1
}

// copierBinaire dépose le binaire statique de l'orchestrateur dans un volume
// partagé, d'où le conteneur de l'application l'exécute comme point d'entrée.
func copierBinaire(dossier string) error {
	source, err := os.Executable()
	if err != nil {
		return err
	}
	if err := os.MkdirAll(dossier, 0o755); err != nil {
		return err
	}
	src, err := os.Open(source)
	if err != nil {
		return err
	}
	defer src.Close()
	cible := filepath.Join(dossier, "orchestrator")
	tmp := cible + ".tmp"
	dst, err := os.OpenFile(tmp, os.O_CREATE|os.O_TRUNC|os.O_WRONLY, 0o755)
	if err != nil {
		return err
	}
	if _, err := io.Copy(dst, src); err != nil {
		dst.Close()
		return err
	}
	if err := dst.Close(); err != nil {
		return err
	}
	return os.Rename(tmp, cible)
}

var errExecIndisponible = errors.New("remplacement de processus indisponible sur ce système")
