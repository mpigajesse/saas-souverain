#!/usr/bin/env bash
# L-10 — Latence réseau inter-machines du banc SOUS CHARGE.
# Mesure le RTT LAN (ens4, 10.77.0.0/24) entre les trois VM pendant qu'une
# charge applicative tourne, et rapporte min/avg/max/mdev + p95 par sens.
# Usage : bash l10_latence.sh <fichier_sortie>
set -u
BASE=/mnt/data/amane-bench
CLE=/tmp/opencode/amane-bench-cle
SSH_OPTS=(-o BatchMode=yes -o StrictHostKeyChecking=accept-new -o UserKnownHostsFile=$BASE/known_hosts -o ConnectTimeout=5 -o LogLevel=ERROR -i $CLE)
declare -A PORT=([m1]=2211 [m2]=2212 [m3]=2213)
declare -A IP=([m1]=10.77.0.11 [m2]=10.77.0.12 [m3]=10.77.0.13)
vmssh() { local m=$1; shift; ssh -p "${PORT[$m]}" "${SSH_OPTS[@]}" etudiant@127.0.0.1 "$@"; }
SORTIE="${1:-/dev/stdout}"

mesurer() { # de vers
  local de=$1 vers=$2
  local brut
  brut=$(vmssh "$de" "ping -c 100 -i 0.05 -W 2 ${IP[$vers]}" 2>&1)
  local rtt
  rtt=$(printf '%s\n' "$brut" | grep -oE 'min/avg/max/mdev = [0-9.]+/[0-9.]+/[0-9.]+/[0-9.]+' | head -1)
  local perte
  perte=$(printf '%s\n' "$brut" | grep -oE '[0-9]+% packet loss' | head -1)
  printf '%s -> %s : %s | %s\n' "$de" "$vers" "${rtt:-n/a}" "${perte:-n/a}"
  # p95 approximatif : tri des rtt par paquet (extrait "time=... ms").
  printf '%s\n' "$brut" | grep -oE 'time=[0-9.]+' | cut -d= -f2 | sort -n | \
    awk '{a[NR]=$1} END{if(NR>0){print "   p95="a[int(NR*0.95)]" ms (n="NR")"}}'
}

{
  echo "# L-10 latence inter-machines (LAN 10.77.0.0/24) — $(date -Is)"
  echo "# charge applicative en cours (L-03/L-11) au moment de la mesure"
  mesurer m1 m2
  mesurer m2 m3
  mesurer m3 m1
  mesurer m1 m3
} | tee "$SORTIE"