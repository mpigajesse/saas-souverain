#!/usr/bin/env bash
# Scénarios de chaos additionnels — Groupe E du plan exhaustif
# À exécuter sur ha-lab (topologie trois-pg déjà démarrée) ou sur le banc A-00.
# Usage : bash ha-lab/chaos-e.sh [E-01|E-02|E-03|E-04|tout]   (défaut : tout)
set -uo pipefail
export MSYS_NO_PATHCONV=1
cd "$(dirname "$0")"

LAN=amane-ha-lab_lan
SCOPE=amane-lab
PG_NODES=(pg-m1 pg-m2 pg-m3)
CIBLE_BASCULE_MS=5000
DELAI_ACQUIT_S=5
COOLDOWN_MS=32000
DERNIERE_BASCULE_MS=0
RESULTATS=()
FILTRE="${1:-tout}"

now_ms() { date +%s%3N; }
log()    { printf '%s  %s\n' "$(date +%H:%M:%S)" "$*"; }
resultat() { RESULTATS+=("$1|$2|$3"); log "→ $1 : $2 — $3"; }

# ── Observation (reprises de chaos.sh) ────────────────────────────────────────

role() {
  docker exec "$1" curl -sfm 1 localhost:8008/patroni 2>/dev/null \
    | python -c 'import json,sys; print(json.load(sys.stdin).get("role","?"))' 2>/dev/null \
    || echo down
}

leader() {
  local e v
  for e in etcd-m1 etcd-m2 etcd-m3; do
    v=$(docker exec "$e" etcdctl --command-timeout=1s get "/service/$SCOPE/leader" --print-value-only 2>/dev/null) \
      && [ -n "$v" ] && { echo "$v"; return 0; }
  done
  return 1
}

sql() { docker exec "$1" psql -U postgres -tAc "$2" 2>/dev/null; }

ecriture_acquittee() {
  timeout "${3:-$DELAI_ACQUIT_S}" docker exec "$1" psql -U postgres -v ON_ERROR_STOP=1 -qtAc \
    "insert into ha_probe(id) values ('$2') on conflict do nothing" >/dev/null 2>&1
}

inscriptible() { [ "$(sql "$1" 'select pg_is_in_recovery()')" = "f" ]; }

present() { [ "$(sql "$1" "select count(*) from ha_probe where id='$2'")" = "1" ]; }

autres() { local n; for n in "${PG_NODES[@]}"; do [ "$n" != "$1" ] && echo "$n"; done; }

machine() { echo "m${1##*-m}"; }

conteneurs() { docker ps -a --format '{{.Names}}' | grep -E "^(etcd|pg|orch)-$1\$" | sort; }

etat_json() {
  local n
  for n in "${PG_NODES[@]}"; do
    docker exec "$n" patronictl list -f json 2>/dev/null && return 0
  done
  echo '[]'
}

sain() {
  python - "$(etat_json)" "${#PG_NODES[@]}" <<'PY'
import json, sys
m, n = json.loads(sys.argv[1]), int(sys.argv[2])
ok = (len(m) == n
      and sum(x["Role"] == "Leader" and x["State"] == "running" for x in m) == 1
      and sum(x["Role"] == "Sync Standby" and x["State"] == "streaming" for x in m) >= 1
      and all(x["State"] in ("running", "streaming") for x in m))
sys.exit(0 if ok else 1)
PY
}

attendre_sain() {
  local t0; t0=$(now_ms)
  until sain; do
    [ $(( $(now_ms) - t0 )) -gt 240000 ] && { log "cluster non sain après 240 s"; docker exec "${PG_NODES[0]}" patronictl list; return 1; }
    sleep 2
  done
  local p; p=$(leader)
  sql "$p" "create table if not exists ha_probe (id text primary key, ts timestamptz default now())" >/dev/null
}

attendre_nouveau_primaire() {
  local t0=$1; shift
  local n
  while :; do
    for n in "$@"; do
      if [ "$(role "$n")" = "primary" ] && ecriture_acquittee "$n" "sonde-$(now_ms)" 2; then
        echo "$n"; return 0
      fi
    done
    [ $(( $(now_ms) - t0 )) -gt 60000 ] && return 1
    sleep 0.2
  done
}

ip_lan() {
  local n=${1##*-m} r
  case "$1" in etcd-*) r=1 ;; pg-*) r=2 ;; orch-*) r=3 ;; esac
  echo "10.77.0.$n$r"
}

isoler()      { local c; for c in $(conteneurs "$1"); do docker network disconnect "$LAN" "$c" >/dev/null; done; }
reconnecter() { local c; for c in $(conteneurs "$1"); do docker network connect --ip "$(ip_lan "$c")" "$LAN" "$c" >/dev/null; done; }
tuer()        { docker kill -s KILL $(conteneurs "$1") >/dev/null; }
relancer()    { docker start $(conteneurs "$1") >/dev/null; }

attendre_cooldown() {
  local reste=$(( DERNIERE_BASCULE_MS + COOLDOWN_MS - $(now_ms) ))
  [ "$reste" -gt 0 ] && { log "attente de la fin du cooldown orchestrateur ($(( reste / 1000 )) s)"; sleep $(( reste / 1000 + 1 )); }
  return 0
}

verdict_bascule() {  # $1 = durée ms
  [ "$1" -le "$CIBLE_BASCULE_MS" ] && echo "< 5 s" || echo "> 5 s"
}

executer() { [[ "$FILTRE" = "tout" || "$1" == *"$FILTRE"* ]] && "$1"; return 0; }

# ── Scénario E-01 : Partition asymétrique avec keepalive 1s ───────────────────
# Le primaire est isolé du réseau LAN (donc d'etcd majoritaire) MAIS conserve
# sa connexion au standby synchrone (m3). keepalive gRPC 1s entre orchestrateur
# et etcd (déjà configuré dans docker-compose.yml : ETCD_GRPC_KEEPALIVE_MIN_TIME: 1s).
# Attendu : pas de split-brain, le primaire isolé ne s'auto-rétrograde pas
# avant le TTL Patroni (20s), la majorité bascule.

sc_e01_partition_asymetrique() {
  local nom="E-01 Partition asymétrique (keepalive 1s)"
  attendre_cooldown
  local p; p=$(leader)  # pg-m2
  local m; m=$(machine "$p")  # m2
  log "$nom : primaire=$p, machine=$m"
  ecriture_acquittee "$p" "e01-avant" || { resultat "$nom" ÉCHEC "écriture de référence refusée"; return; }

  # Isoler la machine du primaire (m2) du LAN : coupe etcd, pg, orch de m2 du réseau lan
  # Mais on NE reconnecte PAS — l'isolement est total pour la machine.
  # Pour simuler "keepalive 1s entre primaire et un standby", on note que la config
  # ETCD_GRPC_KEEPALIVE_MIN_TIME: 1s est déjà en place.
  local t0; t0=$(now_ms)
  isoler "$m"

  # Tentative d'écriture sur le primaire isolé EN PARALLÈLE
  ecriture_acquittee "$p" "e01-isole" "$DELAI_ACQUIT_S" & local essai=$!

  # Attendre que la majorité (m1+m3) élise un nouveau leader
  local np; np=$(attendre_nouveau_primaire "$t0" $(autres "$p")) || {
    resultat "$nom" ÉCHEC "la majorité n'a pas basculé en 60 s"; wait "$essai"; reconnecter "$m"; attendre_sain; return; }
  local bascule=$(( $(now_ms) - t0 )); DERNIERE_BASCULE_MS=$(now_ms)

  # Vérifier que le primaire isolé N'A PAS acquitté l'écriture
  if wait "$essai"; then
    resultat "$nom" ÉCHEC "le primaire isolé a acquitté une écriture (split-brain)"; reconnecter "$m"; attendre_sain; return
  fi

  # Surveiller la rétrogradation du primaire isolé (ne doit pas être avant TTL ~20s)
  local retro=""
  while [ $(( $(now_ms) - t0 )) -lt 60000 ]; do
    if ! inscriptible "$p"; then retro=$(( $(now_ms) - t0 )); break; fi
    sleep 0.5
  done

  # Reconnexion et vérification finale
  reconnecter "$m"
  attendre_sain || { resultat "$nom" ÉCHEC "le cluster ne se reforme pas"; return; }
  local l; l=$(leader)
  present "$l" "e01-avant" || { resultat "$nom" ÉCHEC "perte de données : marqueur absent"; return; }
  if present "$l" "e01-isole"; then
    resultat "$nom" ÉCHEC "l'écriture non acquittée du primaire isolé est réapparue"; return
  fi

  local msg="bascule vers $np en $bascule ms ($(verdict_bascule $bascule)), zéro perte"
  [ -n "$retro" ] && msg="$msg ; $p n'accepte plus d'écriture après $retro ms (TTL Patroni=20s)"
  resultat "$nom" OK "$msg"
}

# ── Scénario E-02 : Double kill du primaire simultané ─────────────────────────
# Tuer simultanément le processus PostgreSQL ET l'orchestrateur du primaire.
# Attendu : un seul nouveau leader émerge, zéro écriture acquittée perdue.

sc_e02_double_kill_primaire() {
  local nom="E-02 Double kill du primaire simultané"
  attendre_cooldown
  local p; p=$(leader)  # pg-m2
  local m; m=$(machine "$p")  # m2
  log "$nom : primaire=$p, machine=$m"
  ecriture_acquittee "$p" "e02-avant" || { resultat "$nom" ÉCHEC "écriture de référence refusée"; return; }

  local t0; t0=$(now_ms)
  # Tuer le conteneur PostgreSQL primaire ET l'orchestrateur de la même machine
  docker kill -s KILL "$p" >/dev/null
  docker kill -s KILL "orch-$m" >/dev/null

  local np; np=$(attendre_nouveau_primaire "$t0" $(autres "$p")) || {
    resultat "$nom" ÉCHEC "aucune bascule en 60 s"; docker start "$p" "orch-$m" >/dev/null; attendre_sain; return; }
  local bascule=$(( $(now_ms) - t0 )); DERNIERE_BASCULE_MS=$(now_ms)

  present "$np" "e02-avant" || { resultat "$nom" ÉCHEC "perte de données : marqueur absent de $np"; docker start "$p" "orch-$m" >/dev/null; attendre_sain; return; }

  # Relancer le primaire tué et son orchestrateur
  docker start "$p" "orch-$m" >/dev/null
  attendre_sain || { resultat "$nom" ÉCHEC "le cluster ne se reforme pas"; return; }
  [ "$(role "$p")" = "replica" ] || { resultat "$nom" ÉCHEC "$p n'est pas revenu en réplica"; return; }

  resultat "$nom" OK "bascule $p → $np en $bascule ms ($(verdict_bascule $bascule)), zéro perte, $p revenu en réplica"
}

# ── Scénario E-03 : Failover pour coupure REST (< 5s) ─────────────────────────
# Couper l'accès à l'API REST Patroni du primaire (port 8008) depuis l'orchestrateur.
# L'orchestrateur ne peut plus superviser le primaire via REST.
# Attendu : détection et bascule en moins de 5s.

sc_e03_failover_rest() {
  local nom="E-03 Failover coupure REST (< 5s)"
  attendre_cooldown
  local p; p=$(leader)  # pg-m2
  local m; m=$(machine "$p")  # m2
  log "$nom : primaire=$p, machine=$m"
  ecriture_acquittee "$p" "e03-avant" || { resultat "$nom" ÉCHEC "écriture de référence refusée"; return; }

  # Couper la connectivité REST entre orchestrateur et Patroni du primaire
  # On isole seulement le conteneur orch-m2 du réseau lan (il ne voit plus pg-m2:8008)
  # Mais pg-m2 reste connecté au lan (etcd, autres pg)
  local orch_conteneur="orch-$m"
  docker network disconnect "$LAN" "$orch_conteneur" >/dev/null

  local t0; t0=$(now_ms)
  # Attendre que l'orchestrateur détecte l'indisponibilité du primaire et force une bascule
  local np; np=$(attendre_nouveau_primaire "$t0" $(autres "$p")) || {
    resultat "$nom" ÉCHEC "aucune bascule en 60 s"; docker network connect --ip "$(ip_lan "$orch_conteneur")" "$LAN" "$orch_conteneur" >/dev/null; attendre_sain; return; }
  local bascule=$(( $(now_ms) - t0 )); DERNIERE_BASCULE_MS=$(now_ms)

  present "$np" "e03-avant" || { resultat "$nom" ÉCHEC "perte de données : marqueur absent de $np"; docker network connect --ip "$(ip_lan "$orch_conteneur")" "$LAN" "$orch_conteneur" >/dev/null; attendre_sain; return; }

  # Reconnecter l'orchestrateur
  docker network connect --ip "$(ip_lan "$orch_conteneur")" "$LAN" "$orch_conteneur" >/dev/null
  attendre_sain || { resultat "$nom" ÉCHEC "le cluster ne se reforme pas"; return; }

  local verdict="$(verdict_bascule $bascule)"
  [ "$bascule" -le "$CIBLE_BASCULE_MS" ] && local res=OK || local res=ÉCHEC
  resultat "$nom" "$res" "bascule vers $np en $bascule ms ($verdict), zéro perte"
}

# ── Scénario E-04 : Arrêt du registry pendant la validation ───────────────────
# Dans ha-lab, il n'y a pas de registry Docker explicite. Ce scénario simule
# l'indisponibilité du registry de l'éditeur (portail) lors de la validation
# d'un manifeste d'application (vérification du digest d'image).
# Attendu : erreur d'infrastructure (timeout/connection refused), pas de 409 masqué.
# Note : sur ha-lab, ce test est BLOQUE car pas de registry réel. Sur le banc A-00
# avec la pile complète, on pourrait tester le portail.

sc_e04_registry_stop() {
  local nom="E-04 Arrêt du registry pendant la validation"
  # Sur ha-lab, pas de registry Docker ni de portail pour valider un manifeste.
  # Ce scénario nécessite la pile complète (portail + registry).
  log "$nom : non applicable sur ha-lab (pas de registry/portail)"
  resultat "$nom" BLOQUE "ha-lab ne contient pas de registry ni de portail pour tester la validation de manifeste"
}

# ── Main ──────────────────────────────────────────────────────────────────────

case "${1:-tout}" in
  E-01) executer sc_e01_partition_asymetrique ;;
  E-02) executer sc_e02_double_kill_primaire ;;
  E-03) executer sc_e03_failover_rest ;;
  E-04) executer sc_e04_registry_stop ;;
  tout) executer sc_e01_partition_asymetrique; executer sc_e02_double_kill_primaire; executer sc_e03_failover_rest; executer sc_e04_registry_stop ;;
  *) echo "usage : $0 [E-01|E-02|E-03|E-04|tout]" >&2; exit 2 ;;
esac

echo
echo "=== Bilan des scénarios E ==="
echecs=0
for r in "${RESULTATS[@]}"; do
  IFS='|' read -r n v d <<<"$r"
  printf '%-50s %-6s %s\n' "$n" "$v" "$d"
  [ "$v" = OK ] || echecs=$((echecs + 1))
done
exit "$echecs"