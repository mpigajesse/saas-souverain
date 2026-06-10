#!/bin/bash
# EL BARAA CONSULT — Installateur du relais zero-knowledge
# Serveur 2 : Ubuntu Server 24.04
# Exécuter : sudo bash relay-deploy/setup.sh
set -e

RELAY_DIR="/opt/elbaraa-relay"
BLOBS_DIR="$RELAY_DIR/blobs"
PROJECT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )/.." && pwd )"

echo ""
echo "========================================================"
echo "  EL BARAA CONSULT — Relais zero-knowledge"
echo "  Serveur : $(hostname -I | awk '{print $1}')"
echo "========================================================"
echo ""

# ── Droits root ───────────────────────────────────────────
if [ "$EUID" -ne 0 ]; then
    echo "ERREUR : exécuter avec sudo : sudo bash relay-deploy/setup.sh"
    exit 1
fi

# ── Étape 1 : Docker ──────────────────────────────────────
echo "[1/5] Installation de Docker..."
apt-get update -qq
apt-get install -y ca-certificates curl gnupg lsb-release

# Clé GPG officielle Docker
install -m 0755 -d /etc/apt/keyrings
curl -fsSL https://download.docker.com/linux/ubuntu/gpg \
    | gpg --dearmor -o /etc/apt/keyrings/docker.gpg
chmod a+r /etc/apt/keyrings/docker.gpg

# Dépôt Docker stable
echo \
  "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] \
  https://download.docker.com/linux/ubuntu \
  $(. /etc/os-release && echo "$VERSION_CODENAME") stable" \
  > /etc/apt/sources.list.d/docker.list

apt-get update -qq
apt-get install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin

systemctl enable docker
systemctl start docker
echo "      Docker $(docker --version) installé."

# ── Étape 2 : Répertoires ─────────────────────────────────
echo "[2/5] Création des répertoires..."
mkdir -p "$BLOBS_DIR"
chmod 750 "$BLOBS_DIR"
echo "      Stockage blobs : $BLOBS_DIR"

# ── Étape 3 : Configuration .env ─────────────────────────
echo "[3/5] Configuration..."
ENV_FILE="$PROJECT_DIR/relay-deploy/.env"
if [ ! -f "$ENV_FILE" ]; then
    cp "$PROJECT_DIR/relay-deploy/.env.example" "$ENV_FILE"
    echo "      .env créé depuis .env.example"
else
    echo "      .env existant conservé."
fi

# ── Étape 4 : Build ───────────────────────────────────────
echo "[4/5] Build de ss-relay (compilation Rust — 3 à 5 min)..."
cd "$PROJECT_DIR"
docker compose -f relay-deploy/docker-compose.yml --env-file relay-deploy/.env build ss-relay
echo "      Build terminé."

# ── Étape 5 : Démarrage ───────────────────────────────────
echo "[5/5] Démarrage du relais..."
docker compose -f relay-deploy/docker-compose.yml --env-file relay-deploy/.env up -d

# ── Pare-feu ──────────────────────────────────────────────
echo "      Ouverture du port 8080..."
if command -v ufw &>/dev/null && ufw status | grep -q "active"; then
    ufw allow 8080/tcp comment "ElBaraaConsult-Relay" 2>/dev/null || true
fi

# ── Vérification ──────────────────────────────────────────
sleep 3
RELAY_IP=$(hostname -I | awk '{print $1}')
if curl -sf "http://localhost:8080/health" >/dev/null 2>&1; then
    echo ""
    echo "========================================================"
    echo "  Relais demarré avec succès !"
    echo "  Health : http://$RELAY_IP:8080/health"
    echo "  Nœuds  : http://$RELAY_IP:8080/api/nodes"
    echo "  Blobs  : http://$RELAY_IP:8080/api/blobs/{tenant}/{key}"
    echo ""
    echo "  Mettre a jour le .env du SaaS editeur :"
    echo "  RELAY_URL=http://$RELAY_IP:8080"
    echo "========================================================"
else
    echo "AVERTISSEMENT : health check echoue — logs :"
    echo "  docker compose -f relay-deploy/docker-compose.yml logs ss-relay"
fi
echo ""
