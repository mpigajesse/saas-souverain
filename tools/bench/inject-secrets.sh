#!/usr/bin/env bash
# Charge les secrets du banc (tools/bench/.env, gitignore) et les rend
# disponibles aux outils, sans jamais les afficher.
#
# Usage :
#   # 1. lancer une commande avec les secrets dans l'environnement :
#   tools/bench/inject-secrets.sh python3 tools/bench/render-compose.py --sans-pgadmin
#   tools/bench/inject-secrets.sh docker compose -f ha-lab/docker-compose.yml up -d
#
#   # 2. exporter les secrets dans le shell courant (sans lancer de commande) :
#   source tools/bench/inject-secrets.sh
#
# Si tools/bench/.env est absent, le script s'arrete et indique comment le créer.
set -euo pipefail

RACINE="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
FICHIER="$RACINE/tools/bench/.env"
EXEMPLE="$RACINE/tools/bench/.env.example"

if [[ ! -f "$FICHIER" ]]; then
    echo "Erreur : $FICHIER est absent." >&2
    echo "  cp \"$EXEMPLE\" \"$FICHIER\"   puis remplir les valeurs." >&2
    exit 1
fi

# Charge le .env (format KEY=VALEUR, sans espaces autour de =).
set -a
# shellcheck disable=SC1090
. "$FICHIER"
set +a

# Verifie que les variables attendues sont bien definies (sans montrer les valeurs).
manquantes=()
for cle in AMANE_RELAY_AUTH_TOKEN AMANE_REGISTRATION_TOKEN AMANE_POSTGRES_PASSWORD; do
    if [[ -z "${!cle:-}" ]]; then
        manquantes+=("$cle")
    fi
done
if [[ ${#manquantes[@]} -gt 0 ]]; then
    echo "Attention : variables vides dans $FICHIER : ${manquantes[*]}" >&2
fi

# Sans argument : on a seulement exporte les variables (mode `source`).
if [[ $# -eq 0 ]]; then
    return 0 2>/dev/null || exit 0
fi

# Avec argument : on lance la commande avec les secrets charges.
exec "$@"