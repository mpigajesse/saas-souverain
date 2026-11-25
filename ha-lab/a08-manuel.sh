#!/usr/bin/env bash
# A-08 — test manuel d'extinction du primaire (docker stop pg-m1).
# Un client écrit en boucle sur le primaire ; on mesure le délai de bascule,
# ce que le client voit pendant la panne, et l'absence de perte d'écriture
# acquittée au retour de l'ancien primaire.
set -uo pipefail
cd "$(dirname "$0")/.."   # racine du dépôt (depuis ha-lab/)

MAINT="${1:-pg-m1}"
LOG="$(mktemp)"
T0=$(date +%s%3N)

# Écritures acquittées : id monotone 'w-N' + horodatage. Chaque ligne ACK = une
# écriture COMMIT (donc acquittée par PostgreSQL). Rejouées après bascule.
client_ecrivain() {
  local i=0
  while :; do
    i=$((i+1))
    if timeout 5 docker exec "$MAINT" psql -U postgres -v ON_ERROR_STOP=1 -qtAc \
         "insert into a08_manual(id) values ('w-$i') on conflict do nothing" >/dev/null 2>&1; then
      echo "ACK w-$i $(date +%s%3N)" >> "$LOG"
    else
      echo "FAIL w-$i $(date +%s%3N)" >> "$LOG"
    fi
    sleep 0.2
  done
}

echo "Préparation : table a08_manual sur $MAINT"
docker exec "$MAINT" psql -U postgres -qtAc \
  "create table if not exists a08_manual(id text primary key, ts timestamptz default now())" >/dev/null
# Repartir d'une table vide pour un test propre
docker exec "$MAINT" psql -U postgres -qtAc "truncate a08_manual" >/dev/null

client_ecrivain &
CLIENT=$!
sleep 3
NB_AVANT=$(grep -c '^ACK' "$LOG")
echo "Écritures acquittées avant panne : $NB_AVANT"

# Extinction du primaire (docker stop : arrêt gracieux, borne ~10 s par défaut)
TSTOP=$(date +%s%3N)
echo "docker stop $MAINT à $(date +%H:%M:%S.%3N)"
docker stop "$MAINT" >/dev/null
TSTOPEND=$(date +%s%3N)
echo "primaire arrêté à $(date +%H:%M:%S.%3N) (docker stop a duré $(( TSTOPEND - TSTOP )) ms)"

# Attendre qu'un nouveau primaire acquitte une écriture (tous les pg sauf MAINT)
CANDIDATS=$(docker ps --format '{{.Names}}' | grep -E '^pg-m[0-9]+$' | grep -v "^$MAINT$")
NOUVEAU=""
while :; do
  for n in $CANDIDATS; do
    r=$(docker exec "$n" curl -sfm 1 localhost:8008/patroni 2>/dev/null | python -c 'import json,sys;print(json.load(sys.stdin).get("role","?"))' 2>/dev/null)
    if [ "$r" = "primary" ] && timeout 3 docker exec "$n" psql -U postgres -qtAc "insert into a08_manual(id) values ('sonde-$(date +%s%3N)') on conflict do nothing" >/dev/null 2>&1; then
      NOUVEAU=$n; break
    fi
  done
  [ -n "$NOUVEAU" ] && break
  [ $(( $(date +%s%3N) - TSTOPEND )) -gt 60000 ] && { echo "AUCUNE BASCULE 60s"; break; }
  sleep 0.2
done
TBASC=$(date +%s%3N)
if [ -n "$NOUVEAU" ]; then
  # Délai de bascule après extinction effective du conteneur (hors temps de
  # grâce de docker stop, qui n'est pas un temps de bascule du cluster).
  DELAI_APRES_STOP=$(( TBASC - TSTOPEND ))
  DELAI_TOTAL=$(( TBASC - TSTOP ))
  echo "Nouveau primaire : $NOUVEAU"
  echo "Bascule après arrêt effectif : $DELAI_APRES_STOP ms ; total (stop inclus) : $DELAI_TOTAL ms"
else
  DELAI_APRES_STOP=999999; DELAI_TOTAL=999999
fi

sleep 3
kill "$CLIENT" 2>/dev/null; wait "$CLIENT" 2>/dev/null

echo "Redémarrage de $MAINT…"
docker start "$MAINT" >/dev/null
r="?"
for k in $(seq 1 120); do
  r=$(docker exec "$MAINT" curl -sfm 1 localhost:8008/patroni 2>/dev/null | python -c 'import json,sys;print(json.load(sys.stdin).get("role","?"))' 2>/dev/null)
  [ "$r" = "replica" ] && break
  sleep 1
done
echo "Rôle de $MAINT au retour : $r"

# Vérifier qu'aucune écriture acquittée n'est perdue : toute ligne ACK doit
# être présente sur le nouveau primaire.
NB_ACK=$(grep -c '^ACK' "$LOG")
PERDU=0
LISTE_PERDU=$(mktemp)
while read -r _ id _; do
  present=$(docker exec "$NOUVEAU" psql -U postgres -tAc "select count(*) from a08_manual where id='$id'" 2>/dev/null | tr -d ' ')
  [ "$present" = "1" ] || { PERDU=$((PERDU+1)); echo "$id" >> "$LISTE_PERDU"; }
done < <(grep '^ACK' "$LOG")
NB_FAIL=$(grep -c '^FAIL' "$LOG" || true)

echo "==================================================="
echo "RÉSULTAT MANUEL A-08 (extinction $MAINT)"
echo "Écritures acquittées (ACK) au total : $NB_ACK"
echo "Écritures refusées (FAIL) pendant panne : $NB_FAIL"
echo "Écritures acquittées PERDUES au retour : $PERDU"
echo "Délai bascule après arrêt effectif : $DELAI_APRES_STOP ms (total stop inclus : $DELAI_TOTAL ms)"
if [ "$PERDU" = 0 ]; then
  echo "VERDICT : OK — zéro perte, bascule en ${DELAI_APRES_STOP} ms après extinction"
else
  echo "VERDICT : ÉCHEC — $PERDU écritures perdues"; echo "IDs perdus :"; cat "$LISTE_PERDU"
fi
echo "==================================================="
cp "$LOG" docs/tests/chaos/a08-manuel-ecritures.log
echo "$DELAI_APRES_STOP" > docs/tests/chaos/a08-manuel-delai-ms.txt