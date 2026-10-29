//go:build !unix

package main

// executer n'existe que sous Unix : le lanceur ne tourne que dans le
// conteneur Linux de l'application.
func executer(string, []string, []string) error {
	return errExecIndisponible
}
