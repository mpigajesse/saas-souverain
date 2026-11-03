#!/bin/sh
# Hook pre_start pour app-multi - Applique migrations SQL
# Variables attendues : AMANE_APP_DB_* + DATABASE_URL (construite par lanceur)

set -e

echo "[pre_start] Début - Attente base de données..."

if [ -z "$DATABASE_URL" ]; then
  if [ -z "$AMANE_APP_DB_UTILISATEUR" ] || [ -z "$AMANE_APP_DB_MOT_DE_PASSE" ] || [ -z "$AMANE_APP_DB_HOTE" ] || [ -z "$AMANE_APP_DB_PORT" ] || [ -z "$AMANE_APP_DB_BASE" ]; then
    echo "[pre_start] ERREUR: Variables DB manquantes" >&2
    exit 1
  fi
  export DATABASE_URL="postgres://${AMANE_APP_DB_UTILISATEUR}:${AMANE_APP_DB_MOT_DE_PASSE}@${AMANE_APP_DB_HOTE}:${AMANE_APP_DB_PORT}/${AMANE_APP_DB_BASE}?sslmode=disable"
fi

for i in $(seq 1 60); do
  if pg_isready -h "$AMANE_APP_DB_HOTE" -p "$AMANE_APP_DB_PORT" -U "$AMANE_APP_DB_UTILISATEUR" >/dev/null 2>&1; then
    echo "[pre_start] Base de données prête"
    break
  fi
  if [ $i -eq 60 ]; then
    echo "[pre_start] ERREUR: Timeout attente base" >&2
    exit 1
  fi
  sleep 1
done

MIGRATIONS_DIR="/migrations"
if [ ! -d "$MIGRATIONS_DIR" ]; then
  echo "[pre_start] AVERTISSEMENT: Dossier $MIGRATIONS_DIR absent"
else
  echo "[pre_start] Application migrations depuis $MIGRATIONS_DIR..."
  for migration in "$MIGRATIONS_DIR"/*.sql; do
    [ -f "$migration" ] || continue
    echo "[pre_start] Exécution: $(basename "$migration")"
    if ! psql "$DATABASE_URL" -f "$migration" -v ON_ERROR_STOP=1; then
      echo "[pre_start] ERREUR: Échec $(basename "$migration")" >&2
      exit 1
    fi
  done
  echo "[pre_start] Migrations OK"
fi

echo "[pre_start] Terminé"