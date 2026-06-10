#!/bin/bash
# VM2 — Nœud standby PostgreSQL (réplication streaming depuis le primaire)
set -e

PRIMARY_IP="${1:-}"
if [ -z "${PRIMARY_IP}" ]; then
    echo "Usage : $0 <IP_PRIMAIRE>"
    echo "Exemple : $0 192.168.200.130"
    exit 1
fi

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
    echo "=== IMPORTANT : configurez ${SCRIPT_DIR}/.env ==="
    echo "  PG_PASSWORD et REPLICATION_PASSWORD doivent être identiques au primaire"
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

update_env NODE_ROLE  standby       "${SCRIPT_DIR}/.env"
update_env NODE_MODE  passive       "${SCRIPT_DIR}/.env"
update_env PRIMARY_IP "${PRIMARY_IP}" "${SCRIPT_DIR}/.env"

MY_IP=$(hostname -I | awk '{print $1}')
update_env NODE_ADDR "${MY_IP}:9001" "${SCRIPT_DIR}/.env"

# ── Démarrage ─────────────────────────────────────────────────────────────
cd "${SCRIPT_DIR}"
docker compose up -d --build

echo ""
echo "=== PostgreSQL standby : pg_basebackup en cours depuis ${PRIMARY_IP}... ==="
echo "    Suivi : docker logs -f pg-node"
echo ""

# Attendre que pg-node passe healthy (pg_basebackup peut prendre 30-90 secondes)
until docker inspect pg-node --format='{{.State.Health.Status}}' 2>/dev/null | grep -q healthy; do
    sleep 5
    echo "    Attente du standby PostgreSQL..."
done

# ── Initialisation ss-node (enrôlement nécessaire) ───────────────────────
sleep 5
if [ ! -f /opt/pme-node/node-data/config.toml ]; then
    docker exec ss-node ss-node init
    echo ""
    echo "=== Enrôlement requis ==="
    echo "  Sur ce nœud  : docker exec -it ss-node ss-node enroll"
    echo "  Sur le primaire, scannez le QR code affiché pour transmettre la DEK"
fi

echo ""
echo "=== Nœud STANDBY opérationnel ==="
echo "  Réplication  : streaming depuis ${PRIMARY_IP}:5432"
echo "  pgAdmin      : http://${MY_IP}:5050  (données en lecture seule)"
echo "  Interface    : http://${MY_IP}:9001"
