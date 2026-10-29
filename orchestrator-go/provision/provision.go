// Package provision crée, sur le primaire PostgreSQL, les rôles et bases dont
// la pile a besoin. Spilo ne fournit qu'un rôle `admin` sans connexion (son
// post_init force NOLOGIN, et Patroni 4 a retiré bootstrap.users) : le nœud
// AMANE se connecte donc avec un rôle `amane` créé ici, de façon idempotente.
//
// Exécuté par l'orchestrateur avec le mot de passe superutilisateur du
// cluster, qui ne quitte jamais le réseau de la PME (ADR-005, décision 4).
package provision

import (
	"context"
	"errors"
	"fmt"
	"net"
	"net/url"
	"regexp"

	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
)

// Rôle et base du nœud AMANE, et ceux de l'application hébergée (étape 3).
const (
	Role            = "amane"
	Base            = "amane"
	RoleApplication = "application"
	BaseApplication = "application"
)

// Noms de rôle et de base admis : minuscules, chiffres, souligné.
var identifiantSur = regexp.MustCompile(`^[a-z_][a-z0-9_]{0,62}$`)

// Les mots de passe générés par bootstrap sont en base64url : on refuse tout
// autre caractère plutôt que d'échapper un littéral SQL (ALTER ROLE n'accepte
// pas de paramètre lié).
var motDePasseSur = regexp.MustCompile(`^[A-Za-z0-9_-]{16,}$`)

// DSNSuperutilisateur construit la chaîne de connexion au primaire.
func DSNSuperutilisateur(hotePG, motDePasse string) string {
	u := url.URL{
		Scheme:   "postgres",
		User:     url.UserPassword("postgres", motDePasse),
		Host:     hotePG,
		Path:     "/postgres",
		RawQuery: "connect_timeout=5",
	}
	return u.String()
}

// Garantir crée le rôle et la base du nœud s'ils n'existent pas, et fixe le
// mot de passe du rôle. Idempotent : chaque machine peut l'exécuter.
func Garantir(ctx context.Context, dsn, motDePasseRole string) error {
	return GarantirRole(ctx, dsn, Role, Base, motDePasseRole)
}

// GarantirRole crée un rôle de connexion et sa base (dont il est
// propriétaire) s'ils n'existent pas, et fixe son mot de passe. Idempotent.
func GarantirRole(ctx context.Context, dsn, role, base, motDePasseRole string) error {
	if !identifiantSur.MatchString(role) || !identifiantSur.MatchString(base) {
		return fmt.Errorf("nom de rôle ou de base refusé : %q / %q", role, base)
	}
	if !motDePasseSur.MatchString(motDePasseRole) {
		return errors.New("mot de passe du rôle refusé : caractères hors base64url")
	}
	if _, _, err := net.SplitHostPort(hoteDe(dsn)); err != nil {
		return fmt.Errorf("adresse du primaire invalide : %w", err)
	}
	conn, err := pgx.Connect(ctx, dsn)
	if err != nil {
		return fmt.Errorf("connexion au primaire : %w", err)
	}
	defer conn.Close(ctx)

	// Noms validés par identifiantSur, puis échappés comme identifiants SQL.
	idRole, idBase := pgx.Identifier{role}.Sanitize(), pgx.Identifier{base}.Sanitize()
	if _, err := conn.Exec(ctx, `DO $$ BEGIN
		IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = '`+role+`') THEN
			CREATE ROLE `+idRole+` LOGIN;
		END IF;
	END $$`); err != nil && !codeErreur(err, "42710") { // duplicate_object : création concurrente
		return fmt.Errorf("création du rôle %s : %w", role, err)
	}
	if _, err := conn.Exec(ctx, "ALTER ROLE "+idRole+" WITH LOGIN PASSWORD '"+motDePasseRole+"'"); err != nil {
		return fmt.Errorf("mot de passe du rôle %s : %w", role, err)
	}
	var existe bool
	if err := conn.QueryRow(ctx, "SELECT EXISTS (SELECT 1 FROM pg_database WHERE datname = $1)", base).Scan(&existe); err != nil {
		return err
	}
	if !existe {
		if _, err := conn.Exec(ctx, "CREATE DATABASE "+idBase+" OWNER "+idRole); err != nil && !codeErreur(err, "42P04") { // duplicate_database
			return fmt.Errorf("création de la base %s : %w", base, err)
		}
	}
	return nil
}

func hoteDe(dsn string) string {
	u, err := url.Parse(dsn)
	if err != nil {
		return ""
	}
	return u.Host
}

func codeErreur(err error, code string) bool {
	var pgErr *pgconn.PgError
	return errors.As(err, &pgErr) && pgErr.Code == code
}
