#!/usr/bin/env bash
# L-06 / L-07 — Sauvegarde chiffrée puis restauration d'un GROS VOLUME.
# Crée une base ~1 Gio sur le primaire, la sauvegarde (pg_dump | AES-256-CBC)
# en mesurant débit et taille, puis la restaure dans une base neuve en mesurant
# le débit. Le relais étant injoignable, on utilise pg_dump + chiffrement
# (comme A-07), pas le chemin ss-node.
# Usage : bash l06_07_sauvegarde_restauration.sh <fichier_sortie>
set -u
BASE=/mnt/data/amane-bench
CLE=/tmp/opencode/amane-bench-cle
SSH_OPTS=(-o BatchMode=yes -o StrictHostKeyChecking=accept-new -o UserKnownHostsFile=$BASE/known_hosts -o ConnectTimeout=5 -o LogLevel=ERROR -i $CLE)
vmssh() { local p=$1; shift; ssh -p "$p" "${SSH_OPTS[@]}" etudiant@127.0.0.1 "$@"; }
role() { vmssh "$1" "docker exec amane-postgresql python3 -c \"import urllib.request,json;print(json.load(urllib.request.urlopen('http://localhost:8008/patroni',timeout=5))['role'])\"" 2>/dev/null; }
SORTIE="${1:-/dev/stdout}"
PRIM=""
for m in 2211 2212 2213; do [ "$(role $m)" = primary ] && PRIM=$m; done
[ -n "$PRIM" ] || { echo "aucun primaire"; exit 1; }
echo "# primaire = port $PRIM"
PGPW=$(vmssh "$PRIM" "docker exec amane-postgresql sh -c '. /amane/secrets/postgresql.env; echo \$PGPASSWORD_SUPERUSER'")
# psql / pg_dump via docker exec -e PGPASSWORD (pas de sh -c imbriqué).
P() { vmssh "$PRIM" "docker exec -e PGPASSWORD=$PGPW amane-postgresql psql -U postgres -d $1 -tA -c \"$2\"" 2>&1; }
PASS="amane-test-l06"

{
  echo "# L-06/L-07 sauvegarde+restauration chiffrée gros volume — $(date -Is)"
  echo "# création base l06 (~1 Gio)…"
  P postgres "DROP DATABASE IF EXISTS l06;" 
  P postgres "CREATE DATABASE l06;"
  P l06 "CREATE TABLE gros(id int primary key, v text);"
  T0=$(date +%s.%N)
  P l06 "INSERT INTO gros SELECT g, md5(g::text)||md5((g+1)::text) FROM generate_series(1,6000000) g;" | tail -1
  T1=$(date +%s.%N)
  TAILLE=$(P l06 "select pg_size_pretty(pg_database_size(current_database()));")
  echo "# base l06 créée : $TAILLE en $(awk -v a=$T0 -v b=$T1 'BEGIN{printf "%.1f",b-a}') s"

  echo "# L-06 : sauvegarde chiffrée (pg_dump | AES-256-CBC)"
  mkdir -p /tmp/opencode/l06
  T2=$(date +%s.%N)
  vmssh "$PRIM" "docker exec -e PGPASSWORD=$PGPW amane-postgresql pg_dump -U postgres -d l06" | \
    openssl enc -aes-256-cbc -salt -pass "pass:$PASS" > /tmp/opencode/l06/l06.dump.enc 2>/dev/null
  T3=$(date +%s.%N)
  TAILLE_ENC=$(stat -c%s /tmp/opencode/l06/l06.dump.enc)
  awk -v a=$T2 -v b=$T3 -v s=$TAILLE_ENC 'BEGIN{printf "# L-06 : %d octets chiffrés en %.1f s -> %.1f MiB/s\n", s, b-a, s/(b-a)/1048576}'

  echo "# L-07 : restauration dans une base neuve l07"
  P postgres "DROP DATABASE IF EXISTS l07;"
  P postgres "CREATE DATABASE l07;"
  T4=$(date +%s.%N)
  openssl enc -d -aes-256-cbc -pass "pass:$PASS" -in /tmp/opencode/l06/l06.dump.enc 2>/dev/null | \
    vmssh "$PRIM" "docker exec -i -e PGPASSWORD=$PGPW amane-postgresql psql -U postgres -d l07" 2>&1 | tail -2
  T5=$(date +%s.%N)
  N_L07=$(P l07 "select count(*) from gros;")
  awk -v a=$T4 -v b=$T5 -v n=$N_L07 'BEGIN{printf "# L-07 : %d lignes restaurées en %.1f s\n", n, b-a}'
  echo "# vérification intégrité : l06=$(P l06 'select count(*) from gros;') l07=$N_L07"

  P postgres "DROP DATABASE IF EXISTS l06;"
  P postgres "DROP DATABASE IF EXISTS l07;"
  rm -f /tmp/opencode/l06/l06.dump.enc
} | tee "$SORTIE"