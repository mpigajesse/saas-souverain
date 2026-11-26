#!/usr/bin/env bash
# L-12 — Performance disque (chiffré attendu, brut mesuré).
# Le banc est déployé avec CHIFFREMENT_DISQUE=non : il n'y a PAS de volume
# chiffré à mesurer. On mesure donc le débit d'écriture puis de lecture du
# disque BRUT (sans chiffrement) comme référence, et on documente l'écart
# (cible chiffrée non tenue, cause racine : chiffrement désactivé).
# Usage : bash l12_disque.sh <fichier_sortie>
set -u
BASE=/mnt/data/amane-bench
CLE=/tmp/opencode/amane-bench-cle
SSH_OPTS=(-o BatchMode=yes -o StrictHostKeyChecking=accept-new -o UserKnownHostsFile=$BASE/known_hosts -o ConnectTimeout=5 -o LogLevel=ERROR -i $CLE)
vmssh() { ssh -p 2211 "${SSH_OPTS[@]}" etudiant@127.0.0.1 "$@"; }
SORTIE="${1:-/dev/stdout}"
F=/tmp/l12.bin

{
  echo "# L-12 performance disque — $(date -Is)"
  echo "# banc : CHIFFREMENT_DISQUE=non (aucun volume chiffré) ; mesure du disque brut"
  echo "# écriture 2 Gio (bs=1M, conv=fdatasync)"
  vmssh "dd if=/dev/zero of=$F bs=1M count=2048 conv=fdatasync 2>&1 | tail -1"
  echo "# vidage du cache puis lecture 2 Gio"
  vmssh "echo 3 | sudo tee /proc/sys/vm/drop_caches >/dev/null; dd if=$F of=/dev/null bs=1M 2>&1 | tail -1"
  vmssh "rm -f $F"
} | tee "$SORTIE"