#!/bin/bash
# Executed by docker-entrypoint.sh inside the pg-primary container.
# Creates the replication user and configures streaming replication.
set -e

psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" <<-EOSQL
    CREATE USER replicator REPLICATION LOGIN ENCRYPTED PASSWORD 'replpass';
EOSQL

# Allow replication connections from the ss-net subnet
echo "host replication replicator 172.20.0.0/24 md5" >> "$PGDATA/pg_hba.conf"

# Streaming replication settings (appended to postgresql.conf)
cat >> "$PGDATA/postgresql.conf" <<-EOF

# -- ss-souverain Phase 0 spike --
wal_level = replica
max_wal_senders = 5
wal_keep_size = 64MB
synchronous_commit = local
EOF
