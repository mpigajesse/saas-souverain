#!/bin/sh
# Hook pre_start - Exécuté AVANT le démarrage de l'application principale
# Applique les migrations SQL dans l'ordre alphabétique
# Variables d'environnement attendues (injectées par le lanceur AMANE) :
#   AMANE_APP_DB_UTILISATEUR, AMANE_APP_DB_MOT_DE_PASSE, AMANE_APP_DB_HOTE,
#   AMANE_APP_DB_PORT, AMANE_APP_DB_BASE
#   DATABASE_URL est construite par le lanceur à partir du manifeste

set -e

echo "[pre_start] Début - Attente base de données..."

# Construire DATABASE_URL si pas déjà fait (fallback)
if [ -z "$DATABASE_URL" ]; then
  if [ -z "$AMANE_APP_DB_UTILISATEUR" ] || [ -z "$AMANE_APP_DB_MOT_DE_PASSE" ] || [ -z "$AMANE_APP_DB_HOTE" ] || [ -z "$AMANE_APP_DB_PORT" ] || [ -z "$AMANE_APP_DB_BASE" ]; then
    echo "[pre_start] ERREUR: Variables DB manquantes" >&2
    exit 1
  fi
  export DATABASE_URL="postgres://${AMANE_APP_DB_UTILISATEUR}:${AMANE_APP_DB_MOT_DE_PASSE}@${AMANE_APP_DB_HOTE}:${AMANE_APP_DB_PORT}/${AMANE_APP_DB_BASE}?sslmode=disable"
fi

# Attendre que PostgreSQL soit prêt (max 60s)
for i in $(seq 1 60); do
  if pg_isready -h "$AMANE_APP_DB_HOTE" -p "$AMANE_APP_DB_PORT" -U "$AMANE_APP_DB_UTILISATEUR" >/dev/null 2>&1; then
    echo "[pre_start] Base de données prête"
    break
  fi
  if [ $i -eq 60 ]; then
    echo "[pre_start] ERREUR: Timeout attente base de données" >&2
    exit 1
  fi
  sleep 1
done

# Appliquer les migrations dans l'ordre
MIGRATIONS_DIR="/migrations"
if [ ! -d "$MIGRATIONS_DIR" ]; then
  echo "[pre_start] AVERTISSEMENT: Dossier $MIGRATIONS_DIR absent, pas de migration à appliquer"
else
  echo "[pre_start] Application des migrations depuis $MIGRATIONS_DIR..."
  for migration in "$MIGRATIONS_DIR"/*.sql; do
    [ -f "$migration" ] || continue
    echo "[pre_start] Exécution: $(basename "$migration")"
    if ! psql "$DATABASE_URL" -f "$migration" -v ON_ERROR_STOP=1; then
      echo "[pre_start] ERREUR: Échec migration $(basename "$migration")" >&2
      exit 1
    fi
  done
  echo "[pre_start] Toutes les migrations appliquées avec succès"
fi

echo "[pre_start] Terminé - Lancement application principale..."