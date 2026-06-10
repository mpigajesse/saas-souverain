#!/bin/bash
# Point d'entrée PostgreSQL — gère l'initialisation primaire ET standby
set -e

DATA_DIR="${PGDATA:-/var/lib/postgresql/data}"

if [ "${NODE_ROLE}" = "standby" ]; then
    if [ ! -f "${DATA_DIR}/PG_VERSION" ]; then
        echo "[standby] Répertoire vide — pg_basebackup depuis ${PRIMARY_IP}:5432"

        # Attendre que le primaire accepte les connexions de réplication
        # (psql -U replicator confirme que primary-init.sh a bien tourné)
        until PGPASSWORD="${REPLICATION_PASSWORD}" psql \
                -h "${PRIMARY_IP}" -p 5432 \
                -U replicator -d postgres \
                -c "SELECT 1" >/dev/null 2>&1; do
            echo "[standby] Attente du primaire ${PRIMARY_IP} (user replicator)..."
            sleep 3
        done

        # Cloner le primaire (crée standby.signal + primary_conninfo via -R)
        PGPASSWORD="${REPLICATION_PASSWORD}" pg_basebackup \
            -h "${PRIMARY_IP}" -p 5432 \
            -U replicator \
            -D "${DATA_DIR}" \
            -Fp -Xs -P -R \
            --checkpoint=fast

        echo "[standby] pg_basebackup terminé — réplication configurée"
    else
        echo "[standby] Données existantes — reprise en mode standby"
    fi
fi

# Déléguer au docker-entrypoint officiel (gère initdb sur primaire vide,
# démarrage normal sinon — standby.signal déjà présent pour le standby)
exec /usr/local/bin/docker-entrypoint.sh "$@"
