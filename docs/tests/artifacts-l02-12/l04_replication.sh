#!/usr/bin/env bash
# L-04 — Réplication synchrone SOUS CHARGE (pgbench).
# Charge pgbench (4 clients, 25 s) sur le primaire m3 ; échantillonnage du lag
# de rejeu (pg_stat_replication) toutes les ~2 s depuis l'hôte (requête fournie
# par stdin, pas d'échappement). Cible : lag ~0 sous charge, mode synchrone
# maintenu, aucune écriture acquittée perdue.
# Usage : bash l04_replication.sh <fichier_sortie>
set -u
BASE=/mnt/data/amane-bench
CLE=/tmp/opencode/amane-bench-cle
SSH_OPTS=(-o BatchMode=yes -o StrictHostKeyChecking=accept-new -o UserKnownHostsFile=$BASE/known_hosts -o ConnectTimeout=5 -o LogLevel=ERROR -i $CLE)
PRIM=2213   # primaire = m3 (leader Patroni)
vmssh() { local p=$1; shift; ssh -p "$p" "${SSH_OPTS[@]}" etudiant@127.0.0.1 "$@"; }
SORTIE="${1:-/dev/stdout}"
Q=/tmp/opencode/l04/q.sql
cat > "$Q" <<'SQL'
select now()::time(0) as t, count(*) filter (where sync_state='sync') as sync,
       count(*) filter (where state='streaming') as stream,
       coalesce(max(extract(milliseconds from replay_lag))::int,0) as replay_ms
from pg_stat_replication;
SQL

echantillon() { vmssh $PRIM "docker exec -i amane-postgresql psql -U postgres -d postgres -tA" < "$Q" 2>/dev/null; }

{
  echo "# L-04 réplication synchrone sous charge (pgbench) — $(date -Is) (primaire m3)"
  vmssh $PRIM "docker exec amane-postgresql psql -U postgres -d postgres -c 'DROP DATABASE IF EXISTS pgl04;' -c 'CREATE DATABASE pgl04;'" 2>&1
  vmssh $PRIM "docker exec amane-postgresql pgbench -i -q -U postgres -d pgl04" 2>&1 | tail -1

  # Sampler en arrière-plan (12 échantillons espacés de ~2 s).
  ( for i in $(seq 1 12); do echantillon; sleep 1; done ) > /tmp/opencode/l04/lag.txt 2>&1 &
  LAGPID=$!

  # Charge pgbench : 4 clients, 4 threads, 25 s, sans vacuum.
  DEBUT=$(date +%s.%N)
  vmssh $PRIM "docker exec amane-postgresql pgbench -c 4 -j 4 -T 25 -n -U postgres -d pgl04" 2>&1 | grep -E 'tps|latency|number of transactions|clients|threads'
  FIN=$(date +%s.%N)
  wait $LAGPID 2>/dev/null

  echo "# durée charge : $(awk -v d="$DEBUT" -v f="$FIN" 'BEGIN{printf "%.1f s", f-d}')"
  echo "# échantillons (t|sync|stream|replay_ms) :"
  cat /tmp/opencode/l04/lag.txt
  echo "# max replay_lag observé (ms) :"
  awk -F'|' '{gsub(/ /,"",$4); if($4+0>m)m=$4+0} END{print m+0}' /tmp/opencode/l04/lag.txt 2>/dev/null
  echo "# état de réplication final :"
  vmssh $PRIM "docker exec -i amane-postgresql psql -U postgres -d postgres -tA -c 'select application_name,state,sync_state from pg_stat_replication;'" 2>&1
  vmssh $PRIM "docker exec amane-postgresql psql -U postgres -d postgres -c 'DROP DATABASE IF EXISTS pgl04;'" 2>&1
} | tee "$SORTIE"