#!/bin/bash
# Custom entrypoint for pg-standby containers.
# Waits for the primary, runs pg_basebackup, then starts PostgreSQL normally.
set -e

PRIMARY_HOST="${PG_PRIMARY_HOST:-pg-primary}"
PGDATA="${PGDATA:-/var/lib/postgresql/data}"
REPLICA_USER="replicator"
REPLICA_PASS="replpass"

echo "[standby] Waiting for primary at $PRIMARY_HOST..."
until pg_isready -h "$PRIMARY_HOST" -U postgres -q; do
    sleep 2
done
echo "[standby] Primary is ready."

if [ ! -f "$PGDATA/PG_VERSION" ]; then
    echo "[standby] Running pg_basebackup from $PRIMARY_HOST..."
    PGPASSWORD="$REPLICA_PASS" pg_basebackup \
        -h "$PRIMARY_HOST" \
        -U "$REPLICA_USER" \
        -D "$PGDATA" \
        -P --wal-method=stream -R
    echo "[standby] pg_basebackup complete. standby.signal created automatically (-R flag)."
fi

echo "[standby] Starting PostgreSQL in recovery (standby) mode..."
exec docker-entrypoint.sh postgres
