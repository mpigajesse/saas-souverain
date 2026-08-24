#!/usr/bin/env bash
# Mesure du switchover CONTROLE (chemin planifié) Patroni — Mission C, Jalon 4.
#
# Usage : scripts/switchover_measure.sh
#
# A appeler avec un candidat cible (optionnel) : le nouvel exposé des phases
# reste identique à failover_measure.sh mais l'injection de panne est un
# handover propre via l'API Patroni /switchover (release volontaire de la
# lease) : c'est le seul chemin capable de tenir < 5 s sur le backend etcd,
# dont la détection de crash est bornée par le plancher de lease ttl >= 20 s.
#
# Vérifie zéro perte (marqueur pré-switchover présent sur le nouveau primary)
# et fenceing implicite (ancien primary = réplica).
set -euo pipefail

ETCD_CT="etcd-amane"
SCOPE="amane"
LEADER_KEY="/service/${SCOPE}/leader"
PROBE_TABLE="ha_probe"

declare -A CONT REST
CONT[postgres-primary]=postgres-primary REST[postgres-primary]=8008
CONT[postgres-replica]=postgres-replica REST[postgres-replica]=8009

log()  { printf '%s %s\n' "$(date +%H:%M:%S,%3N)" "$*"; }
fail() { printf 'FAIL: %s\n' "$*" >&2; exit 1; }
role_of() { curl -sfm 1 "localhost:${1}/patroni" 2>/dev/null | grep -oE '"role": "[a-z]+"' | cut -d'"' -f4 || echo down; }
wp() { docker exec "$1" psql -U postgres -tAc "$2"; }
find_primary() {
  for name in postgres-primary postgres-replica; do
    { [ "$(role_of "${REST[$name]}")" = "primary" ] && echo "$name" && return; }
  done
  fail "aucun primary détecté"
}
now_ms() { date +%s%3N; }

echo "=== Mission C — switchover contrôlé Patroni ==="
PRIMARY="$(find_primary)"
for name in postgres-primary postgres-replica; do
  [ "$name" = "$PRIMARY" ] && continue
  CANDIDATE="$name"
done
log "topologie : $PRIMARY = primary, candidat $CANDIDATE"

[ "$(wp "$PRIMARY" 'show synchronous_commit;')" = "on" ] || fail "synchronous_commit != on"
[ "$(wp "$PRIMARY" 'select sync_state from pg_stat_replication;')" = "sync" ] || fail "standby non synchrone"

wp "$PRIMARY" "create table if not exists $PROBE_TABLE (id text primary key, ts timestamptz default now()); insert into $PROBE_TABLE(id) values ('pre-switch') on conflict (id) do nothing;" >/dev/null
for i in $(seq 1 50); do
  { [ "$(wp "$CANDIDATE" "select count(*) from $PROBE_TABLE where id='pre-switch';")" = "1" ] && break; }
  sleep 0.1
done
[ "$(wp "$CANDIDATE" "select count(*) from $PROBE_TABLE where id='pre-switch';")" = "1" ] || fail "standby non à jour avant switchover"
log "précondition OK : répl. synchrone, marqueur 'pre-switch' rejoué sur $CANDIDATE"

T0="$(now_ms)"
curl -sfm 5 -X POST -d "{\"leader\":\"$PRIMARY\",\"candidate\":\"$CANDIDATE\"}" "localhost:${REST[$PRIMARY]}/switchover" || fail "switchover refusé"
log "switchover déclenché vers $CANDIDATE"

while :; do
  { [ "$(role_of "${REST[$CANDIDATE]}")" = "primary" ] && break; }
  { [ $(( $(now_ms) - T0 )) -gt 30000 ] && fail "switchover : timeout"; }
  sleep 0.05
done
SW_MS=$(( $(now_ms) - T0 ))

# le candidat accepte-t-il de vraies écritures ?
for i in $(seq 1 60); do
  if wp "$CANDIDATE" "insert into $PROBE_TABLE(id) values ('post-switch') on conflict (id) do nothing;" >/dev/null 2>&1; then
    WRITABLE_MS=$(( $(now_ms) - T0 ))
    break
  fi
  sleep 0.1
done

# fencing : l'ancien primary devient réplica
for i in $(seq 1 60); do
  { [ "$(role_of "${REST[$PRIMARY]}")" = "replica" ] && break; }
  sleep 0.5
done
if [ "$(role_of "${REST[$PRIMARY]}")" != "replica" ]; then
  fail "FENCING : $PRIMARY n'est pas revenu réplica"
fi

[ "$(wp "$CANDIDATE" "select count(*) from $PROBE_TABLE where id='pre-switch';")" = "1" ] || fail "ZÉRO PERTE : marqueur absent"
log "fencing OK, zéro perte OK"

echo
echo "=== RÉSULTATS (switchover contrôlé) ==="
printf 'switchover_ms    : %d\n' "$SW_MS"
printf 'writable_ms      : %d\n' "$WRITABLE_MS"
printf 'primary_avant : %s -> primary_apres: %s\n' "$PRIMARY" "$CANDIDATE"
printf 'fencing        : OK\n'
printf 'zero_data_loss : OK\n'
[ "$SW_MS" -le 5000 ] && printf 'cible <5s : ATTEINTE\n' || printf 'cible <5s : NON\n'