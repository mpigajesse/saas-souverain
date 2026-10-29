#!/usr/bin/env bash
# Installation PME machine par machine, sans configuration figée
# (refonte V3, sous-étape 2b, ADR-005). Chaque vérification rend OK / ÉCHEC.
#
# Usage : bash ha-lab/pme/installation.sh
# Le laboratoire est recréé à neuf (down -v).
set -uo pipefail
export MSYS_NO_PATHCONV=1
cd "$(dirname "$0")"

RESULTATS=()
now_ms() { date +%s%3N; }
log()    { printf '%s  %s\n' "$(date +%H:%M:%S)" "$*"; }
verdict() { RESULTATS+=("$1|$2|$3"); log "→ $1 : $2 — $3"; }
compose() { docker compose --profile m1 --profile m2 --profile m3 "$@"; }

# Requête SQL à travers le HAProxy d'une machine, avec le rôle amane. Bornée
# dans le temps : une connexion en cours pendant une bascule ne doit jamais
# bloquer le script.
via_haproxy() {  # $1 = machine (m1|m2), $2 = SQL
  local n=${1#m}
  timeout 8 docker exec "pg-$1" sh -c ". /amane/secrets/postgresql.env; PGPASSWORD=\$AMANE_PG_PASSWORD \
    PGCONNECT_TIMEOUT=3 PGOPTIONS='-c statement_timeout=5000' \
    psql -h 10.77.0.${n}4 -p 5000 -U amane -d amane -tAc \"$2\"" 2>/dev/null
}

attendre() {  # $1 = description, $2 = délai s, reste = commande
  local d=$1 delai=$2; shift 2
  local t0; t0=$(now_ms)
  until "$@" >/dev/null 2>&1; do
    [ $(( $(now_ms) - t0 )) -gt $(( delai * 1000 )) ] && { log "délai dépassé : $d"; return 1; }
    sleep 2
  done
}

leader_present()   { docker exec pg-m1 patronictl list 2>/dev/null | grep -q "Leader.*running"; }
sync_streaming()   { docker exec pg-m1 patronictl list 2>/dev/null | grep -q "Sync Standby.*streaming"; }
role_amane_pret()  { [ "$(via_haproxy "$1" "select 1")" = "1" ]; }
membres_etcd()     { [ "$(docker exec etcd-m1 etcdctl member list 2>/dev/null | grep -c started)" = "$1" ]; }

compose down -v --remove-orphans >/dev/null 2>&1

# ── Machine 1 : premier serveur ─────────────────────────────────────────────
log "installation de m1 (premier serveur)…"
docker compose --profile m1 up -d >/dev/null 2>&1
if attendre "leader m1" 180 leader_present && attendre "rôle amane via HAProxy m1" 120 role_amane_pret m1; then
  etat=$(docker logs init-m1 2>&1 | grep -o '"etat":"[a-z]*"' | tail -1)
  verdict "m1 : nouveau cluster, secrets, rôle amane, HAProxy" OK "amorçage $etat ; connexion amane@amane par HAProxy vers le primaire"
else
  verdict "m1 : nouveau cluster, secrets, rôle amane, HAProxy" ÉCHEC "voir docker logs init-m1 / orch-m1 / pg-m1"
fi

# ── Machine 2 : second serveur ──────────────────────────────────────────────
log "installation de m2…"
docker compose --profile m1 --profile m2 up -d >/dev/null 2>&1
if attendre "standby synchrone" 240 sync_streaming && attendre "HAProxy m2" 120 role_amane_pret m2; then
  s1=$(docker exec pg-m1 cat /amane/secrets/postgresql.env | sha256sum | cut -c1-12)
  s2=$(docker exec pg-m2 cat /amane/secrets/postgresql.env | sha256sum | cut -c1-12)
  if [ "$s1" = "$s2" ] && membres_etcd 2; then
    verdict "m2 : ajout au cluster etcd, mêmes secrets, standby synchrone" OK "member add sur le réseau local ; secrets identiques ($s1) ; HAProxy m2 mène au primaire"
  else
    verdict "m2 : ajout au cluster etcd, mêmes secrets, standby synchrone" ÉCHEC "secrets m1=$s1 m2=$s2"
  fi
else
  verdict "m2 : ajout au cluster etcd, mêmes secrets, standby synchrone" ÉCHEC "voir docker logs init-m2 / pg-m2"
fi

# ── Machine 3 : témoin etcd ─────────────────────────────────────────────────
log "installation de m3 (témoin)…"
docker compose --profile m1 --profile m2 --profile m3 up -d >/dev/null 2>&1
if attendre "3 membres etcd" 120 membres_etcd 3; then
  verdict "m3 : témoin etcd ajouté" OK "3 membres etcd démarrés (2 serveurs + témoin)"
else
  verdict "m3 : témoin etcd ajouté" ÉCHEC "voir docker logs init-m3 / etcd-m3"
fi

# ── Bascule de bout en bout à travers HAProxy ───────────────────────────────
via_haproxy m2 "create table if not exists sonde (id text primary key)" >/dev/null
via_haproxy m2 "insert into sonde values ('avant-bascule') on conflict do nothing" >/dev/null
primaire=$(docker exec etcd-m1 etcdctl get /service/amane-pme/leader --print-value-only 2>/dev/null)
autre=$([ "$primaire" = pg-m1 ] && echo m2 || echo m1)
log "arrêt brutal du primaire $primaire ; lecture par le HAProxy de $autre"
t0=$(now_ms)
docker kill -s KILL "$primaire" >/dev/null
ok=""
while [ $(( $(now_ms) - t0 )) -lt 60000 ]; do
  if [ "$(via_haproxy "$autre" "select count(*) from sonde where id='avant-bascule'")" = "1" ] \
     && via_haproxy "$autre" "insert into sonde values ('apres-bascule') on conflict do nothing" >/dev/null \
     && [ "$(via_haproxy "$autre" "select pg_is_in_recovery()")" = "f" ]; then
    ok=$(( $(now_ms) - t0 )); break
  fi
  sleep 0.5
done
if [ -n "$ok" ]; then
  verdict "Bascule vue par HAProxy" OK "même adresse (HAProxy de $autre:5000) : écritures reprises en $ok ms sur le nouveau primaire, zéro perte"
else
  verdict "Bascule vue par HAProxy" ÉCHEC "HAProxy de $autre ne mène pas au nouveau primaire en 60 s"
fi
docker start "$primaire" >/dev/null

echo
echo "=== Bilan de l'installation ==="
echecs=0
for r in "${RESULTATS[@]}"; do
  IFS='|' read -r n v d <<<"$r"
  printf '%-58s %-6s %s\n' "$n" "$v" "$d"
  [ "$v" = OK ] || echecs=$((echecs + 1))
done
exit "$echecs"
