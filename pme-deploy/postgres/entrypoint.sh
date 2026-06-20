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

        # Créer le slot de réplication physique sur le primaire (idempotent).
        # Le slot force le primaire à CONSERVER le WAL tant que ce standby ne l'a
        # pas consommé. Sans lui, un primaire démarré seul (avant le standby)
        # recyclait les segments au-delà de wal_keep_size → réplication cassée
        # au redémarrage. Le slot supprime ce bug par construction.
        echo "[standby] Création du slot de réplication standby_1 (si absent)..."
        PGPASSWORD="${REPLICATION_PASSWORD}" psql \
            -h "${PRIMARY_IP}" -p 5432 \
            -U replicator -d postgres -tAc \
            "SELECT pg_create_physical_replication_slot('standby_1') WHERE NOT EXISTS (SELECT 1 FROM pg_replication_slots WHERE slot_name='standby_1');" \
            >/dev/null 2>&1 || echo "[standby] (slot déjà présent — on continue)"

        # Cloner le primaire (crée standby.signal + primary_conninfo + primary_slot_name via -R)
        PGPASSWORD="${REPLICATION_PASSWORD}" pg_basebackup \
            -h "${PRIMARY_IP}" -p 5432 \
            -U replicator \
            -D "${DATA_DIR}" \
            -Fp -Xs -P -R \
            --slot=standby_1 \
            --checkpoint=fast

        echo "[standby] pg_basebackup terminé — réplication configurée (slot standby_1)"
    else
        echo "[standby] Données existantes — reprise en mode standby"
    fi
fi

# Déléguer au docker-entrypoint officiel (gère initdb sur primaire vide,
# démarrage normal sinon — standby.signal déjà présent pour le standby)
exec /usr/local/bin/docker-entrypoint.sh "$@"
