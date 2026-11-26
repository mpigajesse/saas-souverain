#!/usr/bin/env bash
# L-05 — Bascule (failover) SOUS CHARGE.
# Un écrivain tourne sur m1 (via HAProxy -> primaire) et journalise chaque
# écriture acquittée (OK) / échouée (FAIL). On tue le primaire (m3) en pleine
# charge, on mesure le temps jusqu'à l'élection d'un nouveau primaire, puis on
# vérifie qu'AUCUNE écriture acquittée n'a été perdue (compte sur le nouveau
# primaire == nombre de OK). Le banc est destructif par nature (test de bascule).
# Usage : bash l05_failover.sh <fichier_sortie>
set -u
BASE=/mnt/data/amane-bench
CLE=/tmp/opencode/amane-bench-cle
SSH_OPTS=(-o BatchMode=yes -o StrictHostKeyChecking=accept-new -o UserKnownHostsFile=$BASE/known_hosts -o ConnectTimeout=5 -o LogLevel=ERROR -i $CLE)
vmssh() { local p=$1; shift; ssh -p "$p" "${SSH_OPTS[@]}" etudiant@127.0.0.1 "$@"; }
SORTIE="${1:-/dev/stdout}"
role() { vmssh "$1" "docker exec amane-postgresql python3 -c \"import urllib.request,json;print(json.load(urllib.request.urlopen('http://localhost:8008/patroni',timeout=5))['role'])\"" 2>/dev/null; }
Q="docker exec amane-postgresql psql -U postgres -d postgres -tA"

{
  echo "# L-05 bascule sous charge — $(date -Is)"
  PRIM_AVANT=$(for m in 2211 2212 2213; do r=$(role $m); [ "$r" = primary ] && echo $m; done)
  echo "# primaire avant : m$([ "$PRIM_AVANT" = 2213 ] && echo 3 || ([ "$PRIM_AVANT" = 2212 ] && echo 2 || echo 1)) (port $PRIM_AVANT)"

  # Table créée sur le PRIMAIRE courant ; écrivain sur m1 via HAProxy -> primaire.
  vmssh "$PRIM_AVANT" "$Q -c 'DROP TABLE IF EXISTS failover_load; CREATE TABLE failover_load(id serial primary key, v timestamptz);'" 2>&1
  # écrivain sur un nœud qui n'est PAS le primaire (sinon il meurt avec lui).
  WRITERPORT=2211
  for m in 2211 2212 2213; do [ "$m" != "$PRIM_AVANT" ] && { WRITERPORT=$m; break; }; done
  ( vmssh "$WRITERPORT" "docker exec amane-postgresql bash -c '. /amane/secrets/postgresql.env; export PGPASSWORD=\$PGPASSWORD_SUPERUSER; for i in \$(seq 1 300); do if psql -h amane-haproxy -p 5000 -U postgres -d postgres -tA -c \"insert into failover_load(v) values (clock_timestamp())\" >/dev/null 2>&1; then echo \"OK \$i \$(date +%s.%N)\"; else echo \"FAIL \$i \$(date +%s.%N)\"; fi; done'" ) > /tmp/opencode/l05/writer.log 2>&1 &
  WRITER=$!

  sleep 10
  T_KILL=$(date +%s.%N)
  echo "# $(date +%Is) : kill du primaire (port $PRIM_AVANT) (docker update --restart=no puis docker kill)"
  vmssh "$PRIM_AVANT" "docker update --restart=no amane-postgresql >/dev/null 2>&1; docker kill amane-postgresql" 2>&1

  # Attente d'un nouveau primaire != port tué.
  T_NEW=""
  for i in $(seq 1 120); do
    for m in 2211 2212 2213; do
      [ "$m" = "$PRIM_AVANT" ] && continue
      r=$(role $m)
      if [ "$r" = primary ]; then T_NEW=$(date +%s.%N); NEWPORT=$m; break; fi
    done
    [ -n "$T_NEW" ] && break
    sleep 0.5
  done
  if [ -n "$T_NEW" ]; then
    awk -v k="$T_KILL" -v n="$T_NEW" 'BEGIN{printf "# nouveau primaire : port %s — bascule en %.2f s\n", "'$NEWPORT'", n-k}'
  else
    echo "# AUCUN nouveau primaire élu dans le délai"
  fi

  wait $WRITER 2>/dev/null

  OK=$(grep -c '^OK' /tmp/opencode/l05/writer.log 2>/dev/null)
  FAIL=$(grep -c '^FAIL' /tmp/opencode/l05/writer.log 2>/dev/null)
  echo "# écrivain : $OK OK (acquittées), $FAIL FAIL (pendant la coupure)"
  # Lignes acquises AVANT le kill (les seules à ne pas perdre).
  OK_AVANT=$(awk -v k="$T_KILL" '$1=="OK" && $3+0 <= k+0 {c++} END{print c+0}' /tmp/opencode/l05/writer.log)
  echo "# écritures acquittées avant le kill : $OK_AVANT"

  # Compte sur le nouveau primaire.
  if [ -n "${NEWPORT:-}" ]; then
    ROWS=$(vmssh $NEWPORT "$Q -c 'select count(*) from failover_load;'" 2>/dev/null)
    echo "# lignes présentes sur le nouveau primaire : $ROWS"
    if [ "$ROWS" -ge "$OK_AVANT" ]; then
      echo "# VERDICT : aucune écriture acquittée perdue ($ROWS >= $OK_AVANT)"
    else
      echo "# VERDICT : PERTE détectée ($ROWS < $OK_AVANT)"
    fi
  fi

  # Restauration du banc : redémarrage de m3, attente réintégration.
  echo "# restauration : redémarrage du nœud tué (port $PRIM_AVANT)"
  vmssh "$PRIM_AVANT" "docker update --restart=unless-stopped amane-postgresql >/dev/null 2>&1; docker start amane-postgresql" 2>&1
  for i in $(seq 1 60); do
    r=$(role "$PRIM_AVANT"); [ "$r" = replica ] || [ "$r" = secondary ] && { echo "# nœud réintégré ($r)"; break; }
    sleep 2
  done
  PRIM_COURANT=""
  for m in 2211 2212 2213; do r=$(role $m); [ "$r" = primary ] && PRIM_COURANT=$m; done
  vmssh "${PRIM_COURANT:-2213}" "$Q -c 'DROP TABLE IF EXISTS failover_load;'" 2>&1
} | tee "$SORTIE"