#!/bin/sh
# Démarre PostgreSQL local, initialise le nœud, crée les comptes de démo puis
# lance l'interface web sur $PORT (fourni par Render ; 3000 par défaut).
set -e
: "${DEMO_PASSWORD:?DEMO_PASSWORD manquant (mot de passe des comptes de démo)}"
PGBIN=$(ls -d /usr/lib/postgresql/*/bin | head -1)
PGDATA=/tmp/pgdata

mkdir -p "$PGDATA" "$SS_NODE_DIR" && chown -R postgres "$PGDATA"
su postgres -c "$PGBIN/initdb -D $PGDATA -A trust -U postgres" >/dev/null
su postgres -c "$PGBIN/pg_ctl -D $PGDATA -o '-c listen_addresses=127.0.0.1' -w start" >/dev/null
psql -h 127.0.0.1 -U postgres -qc "CREATE USER metier PASSWORD 'metier';"
psql -h 127.0.0.1 -U postgres -qc "CREATE DATABASE metier OWNER metier;"

export PG_URL="postgres://metier:metier@127.0.0.1:5432/metier"
export WEB_PORT="${PORT:-3000}"
export RELAY_URL="${RELAY_URL:-http://127.0.0.1:9}"   # relais absent en démo

ss-node init --first >/dev/null
# Nom affiché dans l'interface, même si le portail éditeur dort encore au
# démarrage (l'enregistrement auprès du SaaS le fournit sinon).
echo "tenant_name = \"${TENANT_NAME:-Atlas Distribution}\"" >> "$SS_NODE_DIR/config.toml"
ss-node adduser --username admin --name "Direction (démo)" --role admin --password "$DEMO_PASSWORD"
ss-node adduser --username employe --name "Magasinier (démo)" --role employee --password "$DEMO_PASSWORD"

# Les tables articles/mouvements sont créées par le nœud au démarrage : on
# l'attend, puis on charge le stock fictif.
ss-node run --mode active &
NOEUD=$!
for i in $(seq 1 60); do
  psql -h 127.0.0.1 -U metier -d metier -tAc "SELECT to_regclass('mouvements_stock')" | grep -q mouvements && break
  sleep 1
done
psql -h 127.0.0.1 -U metier -d metier -q -f /seed.sql
echo "Démo logiciel PME prête sur le port $WEB_PORT."
wait $NOEUD
