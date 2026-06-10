#!/bin/bash
# VM1 — Nœud primaire PostgreSQL + logiciel métier PME
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# ── Docker ────────────────────────────────────────────────────────────────
if ! command -v docker &>/dev/null; then
    echo "=== Installation de Docker ==="
    curl -fsSL https://get.docker.com | sh
    systemctl enable --now docker
fi

# ── Répertoires persistants ───────────────────────────────────────────────
mkdir -p /opt/pme-node/pg-data /opt/pme-node/node-data

# ── Fichier .env ──────────────────────────────────────────────────────────
if [ ! -f "${SCRIPT_DIR}/.env" ]; then
    cp "${SCRIPT_DIR}/.env.example" "${SCRIPT_DIR}/.env"
    echo ""
    echo "=== IMPORTANT : configurez ${SCRIPT_DIR}/.env puis relancez ce script ==="
    echo "  Renseignez au minimum : PG_PASSWORD, REPLICATION_PASSWORD, NODE_ADDR,"
    echo "  RELAY_URL, SAAS_URL, REGISTRATION_TOKEN, PGADMIN_PASSWORD"
    exit 1
fi

update_env() {
    local key=$1 val=$2 file=$3
    if grep -q "^${key}=" "$file"; then
        sed -i "s|^${key}=.*|${key}=${val}|" "$file"
    else
        echo "${key}=${val}" >> "$file"
    fi
}

update_env NODE_ROLE  primary "${SCRIPT_DIR}/.env"
update_env NODE_MODE  active  "${SCRIPT_DIR}/.env"

# ── Démarrage ─────────────────────────────────────────────────────────────
cd "${SCRIPT_DIR}"
docker compose up -d --build

# ── Initialisation ss-node (premier démarrage uniquement) ────────────────
echo "=== Attente que ss-node soit prêt... ==="
sleep 8
if [ ! -f /opt/pme-node/node-data/config.toml ]; then
    docker exec ss-node ss-node init --first
    echo "=== Nœud initialisé (premier nœud du cluster — DEK générée) ==="
fi

MY_IP=$(hostname -I | awk '{print $1}')
SUBNET_PREFIX=$(echo "${MY_IP}" | cut -d. -f1-3)

echo ""
echo "=== Nœud PRIMAIRE opérationnel ==="
echo "  PostgreSQL  : ${MY_IP}:5432"
echo "  pgAdmin     : http://${MY_IP}:5050  (email: admin@pme.local)"
echo "  Interface   : http://${MY_IP}:9001"

# ── Découverte réseau — détecter les autres machines du LAN PME ───────────
echo ""
echo "=== Découverte réseau : scan de ${SUBNET_PREFIX}.0/24 ==="
echo "    (Assurez-vous que la VM2 est démarrée)"
CANDIDATES=()
for i in $(seq 1 254); do
    TARGET="${SUBNET_PREFIX}.${i}"
    [ "${TARGET}" = "${MY_IP}" ] && continue
    if ping -c 1 -W 1 "${TARGET}" &>/dev/null; then
        CANDIDATES+=("${TARGET}")
        echo "  Découvert : ${TARGET}"
    fi
done &
SCAN_PID=$!
wait "${SCAN_PID}" 2>/dev/null || true

if [ ${#CANDIDATES[@]} -eq 0 ]; then
    echo "  Aucune autre machine détectée sur ${SUBNET_PREFIX}.0/24"
    echo "  → Démarrez la VM2, puis notez son IP pour la prochaine étape"
else
    echo ""
    echo "  ${#CANDIDATES[@]} machine(s) candidate(s) pour le nœud standby :"
    for c in "${CANDIDATES[@]}"; do echo "    - ${c}"; done
fi

echo ""
echo "=== Prochaines étapes pour le nœud standby ==="
echo "  1. Sur cette machine    : docker exec -it ss-node ss-node enroll"
echo "     (affiche le QR code — laissez cette fenêtre ouverte)"
echo "  2. Sur la VM2 choisie   : sudo bash setup-standby.sh ${MY_IP}"
echo "     (scanne le QR code pour recevoir la DEK)"
echo ""
echo "  Licence : chaque PC comptera comme 1 poste dans le portail éditeur."
SAAS_URL_CLEAN=$(grep '^SAAS_URL=' "${SCRIPT_DIR}/.env" | cut -d= -f2-)
echo "  Le portail éditeur : ${SAAS_URL_CLEAN}/devices/"
