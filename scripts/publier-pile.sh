#!/usr/bin/env bash
# Publie les images de la pile PME dans le registre de l'éditeur et les épingle
# par digest (refonte V3, ADR-005 décision 3).
#
# Usage : bash scripts/publier-pile.sh [registre]   (défaut : localhost:5000)
#
# Pour chaque image : récupération (upstream) ou construction (orchestrateur),
# étiquetage sous <registre>/amane/<nom>, envoi au registre, puis lecture du
# digest attribué par le registre. Écrit media/node-images/pile.json, lu par le
# portail (tenants/pile.py) : les compose générés ne référencent les images que
# par ce digest, jamais par un tag. Sans ce manifeste, le portail refuse de
# générer la pile.
#
# Variables facultatives :
#   IMAGE_NOEUD           image locale du nœud (défaut ss-node:latest,
#                         construite par spike/build-node.ps1)
#   IMAGE_ORCHESTRATEUR   image locale de l'orchestrateur (défaut : construite
#                         ici depuis orchestrator-go/)
set -euo pipefail
export MSYS_NO_PATHCONV=1
cd "$(dirname "$0")/.."

REGISTRE="${1:-localhost:5000}"
SORTIE="media/node-images/pile.json"

if [ -z "${IMAGE_ORCHESTRATEUR:-}" ]; then
    echo "Construction de l'orchestrateur (orchestrator-go/)..."
    docker build -q -t amane-orchestrator:latest orchestrator-go >/dev/null
    IMAGE_ORCHESTRATEUR=amane-orchestrator:latest
fi

# nom logique | image source | à tirer d'un registre public (oui/non)
IMAGES="
noeud|${IMAGE_NOEUD:-ss-node:latest}|non
orchestrateur|${IMAGE_ORCHESTRATEUR}|non
postgresql|ghcr.io/zalando/spilo-17:4.0-p3|oui
etcd|quay.io/coreos/etcd:v3.5.5|oui
haproxy|haproxy:3.0-alpine|oui
pgadmin|dpage/pgadmin4:8|oui
"

declare -A REFS
while IFS='|' read -r nom source tirer; do
    [ -z "$nom" ] && continue
    echo "── ${nom} : ${source}"
    if [ "$tirer" = "oui" ]; then
        docker pull -q "$source" >/dev/null
    elif ! docker image inspect "$source" >/dev/null 2>&1; then
        echo "ERREUR : image locale ${source} absente (voir l'en-tête du script)." >&2
        exit 1
    fi
    cible="${REGISTRE}/amane/${nom}:publiee"
    docker tag "$source" "$cible"
    docker push -q "$cible" >/dev/null
    # Le digest fait foi tel que le REGISTRE l'a enregistré. Celui que donne
    # `docker image inspect` peut désigner l'index multi-architecture d'origine
    # alors que seule la plateforme locale a été poussée : la PME obtiendrait
    # alors « manifest unknown » (constaté à l'essai de l'étape 2b).
    digest=$(curl -fsSI \
        -H "Accept: application/vnd.oci.image.index.v1+json, application/vnd.oci.image.manifest.v1+json, application/vnd.docker.distribution.manifest.list.v2+json, application/vnd.docker.distribution.manifest.v2+json" \
        "http://${REGISTRE}/v2/amane/${nom}/manifests/publiee" \
        | tr -d '\r' | awk -F': ' 'tolower($1) == "docker-content-digest" {print $2}')
    if [ -z "$digest" ]; then
        echo "ERREUR : digest introuvable pour ${cible}." >&2
        exit 1
    fi
    REFS[$nom]="amane/${nom}@${digest}"
    echo "   ${REFS[$nom]}"
done <<< "$IMAGES"

mkdir -p "$(dirname "$SORTIE")"
{
    printf '{\n  "publie_le": "%s",\n  "registre": "%s",\n  "images": {\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$REGISTRE"
    premier=1
    for nom in noeud orchestrateur postgresql etcd haproxy pgadmin; do
        [ $premier = 1 ] || printf ',\n'
        premier=0
        printf '    "%s": "%s"' "$nom" "${REFS[$nom]}"
    done
    printf '\n  }\n}\n'
} > "$SORTIE"
echo
echo "Manifeste écrit : ${SORTIE}"
