//go:build unix

package main

import "syscall"

// executer remplace le processus courant par la commande de l'application.
func executer(chemin string, argv, env []string) error {
	return syscall.Exec(chemin, argv, env)
}
