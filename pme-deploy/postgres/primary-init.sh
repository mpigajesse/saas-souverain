#!/bin/bash
# Script d'initialisation PostgreSQL — s'exécute une seule fois lors du premier démarrage
# Crée l'utilisateur de réplication et ouvre pg_hba.conf au sous-réseau PME
set -e

psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
    CREATE USER replicator
        WITH REPLICATION
        ENCRYPTED PASSWORD '${REPLICATION_PASSWORD}';
EOSQL

# Autoriser la réplication depuis le sous-réseau configuré (PG_REPLICATION_SUBNET)
# Défaut : 0.0.0.0/0 (tous — protégé par mot de passe scram-sha-256)
SUBNET="${PG_REPLICATION_SUBNET:-0.0.0.0/0}"

cat >> "${PGDATA}/pg_hba.conf" <<EOF

# Réplication streaming — standby PME
host    replication     replicator      ${SUBNET}        scram-sha-256
# Connexions applicatives depuis le LAN PME (pgAdmin, ss-node)
host    all             all             ${SUBNET}        scram-sha-256
EOF

psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" <<-EOSQL
    SELECT pg_reload_conf();
EOSQL

echo "=== Réplication configurée — utilisateur replicator créé ==="
