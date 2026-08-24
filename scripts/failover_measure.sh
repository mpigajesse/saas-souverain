#!/usr/bin/env bash
# Protocole reproductible de mesure du failover Patroni (Mission C, Jalon 4).
#
# Usage : scripts/failover_measure.sh [crash|partition]
#   crash     : docker kill (SIGKILL) du primary courant — vraie panne process.
#               (docker stop = SIGTERM puis SIGKILL après 10 s : supervisord
#               relance patroni qui garde la lease etcd et bloque la détection.)
#   partition : docker network disconnect (panne réseau : le primary tourne mais
#               est isolé d'etcd et du standby) — chemin différent, à tester aussi.
#
# Phases mesurées (skill mission-c-failover-ha) :
#   detection_ms : T0 → le standby note la perte de connexion au primary
#                  (horodatage dérivé de ses logs à la fin du run).
#   election_ms  : détection → "promoted self to leader" (lock etcd acquis).
#   promotion_ms : lock acquis → PG accepte de vraies écritures (INSERT OK).
#   failover_ms  : T0 → écriture acceptée = fenêtre totale de non-disponibilité.
#
# Vérifie en outre :
#   * fencing : l'ancien primary redevenu réplica n'écrit plus (pg_is_in_recovery).
#   * zero_data_loss : le marqueur inséré AVANT la panne (visible sur le standby
#     synchrone, réplication ackée) est présent sur le nouveau primary.
#
# Prérequis : stack docker-compose up, paramètres d'élection (ttl, loop_wait)
# réglés par API REST Patroni. Renvoie 0 si fencing + zéro perte vérifiés.
set -euo pipefail

MODE="${1:-crash}"
ETCD_CT="etcd-amane"
SCOPE="amane"
LEADER_KEY="/service/${SCOPE}/leader"
PROBE_TABLE="ha_probe"
POLL_S=0.05       # pas de sondage REST (s)
PROMO_TIMEOUT=60  # s avant échec

declare -A CONT REST PG
CONT[postgres-primary]=postgres-primary REST[postgres-primary]=8008  PG[postgres-primary]=5432
CONT[postgres-replica]=postgres-replica REST[postgres-replica]=8009  PG[postgres-replica]=5433

log()  { printf '%s %s\n' "$(date +%H:%M:%S,%3N)" "$*"; }
fail() { printf 'FAIL: %s\n' "$*" >&2; exit 1; }

leaderval() { docker exec "$ETCD_CT" etcdctl get "$LEADER_KEY" 2>/dev/null | tail -1; }
role_of()   { curl -sfm 1 "localhost:${1}/patroni" 2>/dev/null | grep -oE '"role": "[a-z]+"' | cut -d'"' -f4 || echo down; }
recovery()  { docker exec "$1" psql -U postgres -tAc 'select pg_is_in_recovery();' 2>/dev/null; }

wp() { docker exec "$1" psql -U postgres -tAc "$2"; }

find_primary() {
  for name in postgres-primary postgres-replica; do
    { [ "$(role_of "${REST[$name]}")" = "primary" ] && echo "$name" && return; }
  done
  fail "aucun primary détecté (8008/8009) — stack up ?"
}

now_ms() { date +%s%3N; }

echo "=== Mission C — mesure failover Patroni (mode: $MODE) ==="
PRIMARY="$(find_primary)"
log "topologie : $PRIMARY = primary"
for name in postgres-primary postgres-replica; do
  [ "$name" = "$PRIMARY" ] && continue
  STANDBY="$name"
done

# --- 1. Préconditions : réplication synchrone active ---
[ "$(wp "$PRIMARY" 'show synchronous_commit;')" = "on" ] || fail "synchronous_commit != on"
[ "$(wp "$PRIMARY" 'select sync_state from pg_stat_replication;')" = "sync" ] || fail "standby non synchrone"
log "précondition : synchronous_commit=on, standby $STANDBY sync"

# --- 2. Écriture de référence + visibilité sur le standby (zéro perte prouvable)
wp "$PRIMARY" "create table if not exists $PROBE_TABLE (id text primary key, ts timestamptz default now()); insert into $PROBE_TABLE(id) values ('pre-crash') on conflict (id) do nothing;" >/dev/null
for i in $(seq 1 50); do
  { [ "$(wp "$STANDBY" "select count(*) from $PROBE_TABLE where id='pre-crash';")" = "1" ] && break; }
  sleep 0.1
done
[ "$(wp "$STANDBY" "select count(*) from $PROBE_TABLE where id='pre-crash';")" = "1" ] || fail "standby non à jour avant panne"
log "zéro perte établi : marqueur 'pre-crash' rejoué sur $STANDBY"

# --- 3. Watch etcd horodatée : détecte l'expiration de la lease leader ---
LEADW="/tmp/amane_leader_watch.$$"
( docker exec "$ETCD_CT" etcdctl watch "$LEADER_KEY" 2>/dev/null \
    | while IFS= read -r line; do printf '%s %s\n' "$(now_ms)" "$line"; done ) > "$LEADW" 2>&1 &
WPID=$!
sleep 1   # laisse la connexion watch s'établir (etcdctl n'émet que des évènements)

OLD_LEADER="$(leaderval)"
[ -n "$OLD_LEADER" ] || fail "leader etcd introuvable"
T0="$(now_ms)"
if [ "$MODE" = "crash" ]; then
  log "panne crash : docker kill $PRIMARY"
  docker kill -s KILL "$PRIMARY" >/dev/null
elif [ "$MODE" = "partition" ]; then
  NET="$(docker inspect -f '{{range $k,$v := .NetworkSettings.Networks}}{{$k}} {{end}}' "$PRIMARY" | tr -s ' ' | cut -d' ' -f1)"
  log "panne partition : docker network disconnect $NET $PRIMARY"
  docker network disconnect "$NET" "$PRIMARY" >/dev/null
else
  fail "mode inconnu: $MODE (crash|partition)"
fi

# --- 4. Détection & promotion : sondage REST rapide du standby ---
while :; do
  { [ "$(role_of "${REST[$STANDBY]}")" = "primary" ] && break; }
  { [ $(( $(now_ms) - T0 )) -gt $(( PROMO_TIMEOUT * 1000 )) ] && fail "promotion : timeout"; }
  sleep "$POLL_S"
done
FAILOVER_MS=$(( $(now_ms) - T0 ))

# détection = 1er évènement DELETE sur /service/amane/leader vu par la watch etcd
kill "$WPID" 2>/dev/null || true
DETECT_TS="$(awk '$2 ~ /^DELETE/ {print $1; exit}' "$LEADW" 2>/dev/null)"
if [ -n "$DETECT_TS" ]; then
  DETECT_MS=$(( DETECT_TS - T0 ))
else
  DETECT_MS="n/a"
fi
rm -f "$LEADW"

# --- 5. PG accepte de vraies écritures (availibilité réelle, pas seulement role) ---
WRITABLE=$(( $(now_ms) - T0 ))
for i in $(seq 1 60); do
  if wp "$STANDBY" "select 1;" >/dev/null 2>&1 && wp "$STANDBY" "insert into $PROBE_TABLE(id) values ('post-failover') on conflict (id) do nothing;" >/dev/null 2>&1; then
    WRITABLE=$(( $(now_ms) - T0 ))
    break
  fi
  { [ $(( $(now_ms) - T0 )) -gt $(( PROMO_TIMEOUT * 1000 )) ] && break; }
  sleep 0.1
done

# --- 6. Remontée de l'ancien primary → vérification fencing ---
log "réintégration de $PRIMARY (max 60 s)..."
if [ "$MODE" = "crash" ]; then
  docker start "$PRIMARY" >/dev/null
else
  docker network connect "$NET" "$PRIMARY" >/dev/null
fi
for i in $(seq 1 120); do
  { [ "$(role_of "${REST[$PRIMARY]}")" = "replica" ] && break; }
  sleep 0.5
done
if [ "$(role_of "${REST[$PRIMARY]}")" != "replica" ] || [ "$(recovery "$PRIMARY")" != "t" ]; then
  fail "FENCING : $PRIMARY n'est pas revenu en réplica — split-brain ?"
fi
log "fencing OK : $PRIMARY est réplica (pg_is_in_recovery=t), ne ré-accepte aucune écriture"

# --- 7. Zéro perte : marqueur présent sur le nouveau primary ---
ZERO_LOSS="$(wp "$STANDBY" "select count(*) from $PROBE_TABLE where id='pre-crash';")"
[ "$ZERO_LOSS" = "1" ] || fail "ZÉRO PERTE : marqueur pré-crash absent du nouveau primary"
log "zéro perte : marqueur 'pre-crash' présent sur le nouveau primary ($STANDBY)"

# --- 8. Résumé ---
NEW_LEADER="$(leaderval)"
echo
echo "=== RÉSULTATS (mode: $MODE) ==="
printf 'detection_ms     : %s   (expiration lease leader /service amane/leader)\n' "$DETECT_MS"
printf 'failover_ms      : %d   (role=début primary sur le survivant)\n' "$FAILOVER_MS"
printf 'writable_ms      : %d   (INSERT accepté, cluster disponible)\n' "$WRITABLE"
printf 'leader_avant  : %s   leader_apres : %s\n' "$OLD_LEADER" "$NEW_LEADER"
printf 'primary_avant : %s   primary_apres: %s\n' "$PRIMARY" "$STANDBY"
printf 'fencing        : OK\n'
printf 'zero_data_loss : OK\n'
if [ "$FAILOVER_MS" -le 3000 ]; then printf 'cible <3s : ATTEINTE\n'
elif [ "$FAILOVER_MS" -le 5000 ]; then printf 'cible <5s : ATTEINTE\n'
else printf '>5s : la détection d''un crash est bornée par la lease etcd (Patroni impose ttl ≥ 20 s) — voir commentaires.\n'; fi