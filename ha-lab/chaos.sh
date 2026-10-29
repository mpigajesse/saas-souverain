#!/usr/bin/env bash
# Scénarios de chaos du laboratoire HA (refonte V3, sous-étape 2a).
#
# Chaque scénario prouve une promesse d'ADR-001 / ADR-004 sur le laboratoire
# trois machines de ha-lab/docker-compose.yml, et rend un verdict OK / ÉCHEC.
#
# Usage : bash ha-lab/chaos.sh [temoin|trois-pg|tout] [filtre]   (défaut : tout)
#   temoin    2 serveurs PostgreSQL + témoin etcd, synchrone non strict
#   trois-pg  3 serveurs PostgreSQL, synchrone strict
#   filtre    ne joue que les scénarios dont le nom contient ce texte
#             (ex. : isolement_primaire)
#
# Chaque topologie est recréée à neuf (docker compose down -v) : les
# scénarios détruisent des conteneurs et des données de laboratoire.
set -uo pipefail
export MSYS_NO_PATHCONV=1   # Git Bash : ne pas réécrire /service/... en chemin Windows
cd "$(dirname "$0")"

LAN=amane-ha-lab_lan
SCOPE=amane-lab
CIBLE_BASCULE_MS=5000      # promesse : bascule automatique en moins de 5 s
DELAI_ACQUIT_S=5           # au-delà, une écriture est réputée non acquittée
COOLDOWN_MS=32000          # pause de l'orchestrateur après une bascule forcée (30 s) + marge
PG_NODES=()
DERNIERE_BASCULE_MS=0
RESULTATS=()
FILTRE="${2:-}"

now_ms() { date +%s%3N; }
log()    { printf '%s  %s\n' "$(date +%H:%M:%S)" "$*"; }
resultat() { RESULTATS+=("$1|$2|$3"); log "→ $1 : $2 — $3"; }

# ── Observation ─────────────────────────────────────────────────────────────

# Rôle Patroni vu par le nœud lui-même (docker exec : fonctionne même isolé).
role() {
  docker exec "$1" curl -sfm 1 localhost:8008/patroni 2>/dev/null \
    | python -c 'import json,sys; print(json.load(sys.stdin).get("role","?"))' 2>/dev/null \
    || echo down
}

# Leader selon la MAJORITÉ etcd : un membre minoritaire refuse la lecture
# linéarisable, on passe alors au suivant.
leader() {
  local e v
  for e in etcd-m1 etcd-m2 etcd-m3; do
    v=$(docker exec "$e" etcdctl --command-timeout=1s get "/service/$SCOPE/leader" --print-value-only 2>/dev/null) \
      && [ -n "$v" ] && { echo "$v"; return 0; }
  done
  return 1
}

sql() { docker exec "$1" psql -U postgres -tAc "$2" 2>/dev/null; }

# Écriture acquittée par PostgreSQL dans le délai : en mode synchrone, un
# COMMIT n'est acquitté qu'après confirmation du standby synchrone.
ecriture_acquittee() {
  timeout "${3:-$DELAI_ACQUIT_S}" docker exec "$1" psql -U postgres -v ON_ERROR_STOP=1 -qtAc \
    "insert into ha_probe(id) values ('$2') on conflict do nothing" >/dev/null 2>&1
}

# PostgreSQL accepte-t-il encore des écritures ? Critère de rétrogradation :
# Patroni privé d'etcd arrête PostgreSQL (« demoting self, offline ») tandis
# que son API REST peut encore renvoyer le dernier rôle connu.
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

# Sain = tous les membres présents, un leader, au moins un standby synchrone,
# tous en flux.
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

# Attend qu'un nœud de la liste devienne primaire ET acquitte une écriture.
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

# IP fixe d'un conteneur sur `lan` (voir docker-compose.yml) : 10.77.0.<machine><rôle>.
ip_lan() {
  local n=${1##*-m} r
  case "$1" in etcd-*) r=1 ;; pg-*) r=2 ;; orch-*) r=3 ;; esac
  echo "10.77.0.$n$r"
}

isoler()      { local c; for c in $(conteneurs "$1"); do docker network disconnect "$LAN" "$c" >/dev/null; done; }
reconnecter() { local c; for c in $(conteneurs "$1"); do docker network connect --ip "$(ip_lan "$c")" "$LAN" "$c" >/dev/null; done; }
tuer()        { docker kill -s KILL $(conteneurs "$1") >/dev/null; }
relancer()    { docker start $(conteneurs "$1") >/dev/null; }

# L'orchestrateur s'interdit une nouvelle bascule forcée pendant 30 s après la
# précédente (anti-oscillation) : sans cette attente, la panne suivante ne
# serait rattrapée que par le ttl de Patroni et la mesure serait faussée.
attendre_cooldown() {
  local reste=$(( DERNIERE_BASCULE_MS + COOLDOWN_MS - $(now_ms) ))
  [ "$reste" -gt 0 ] && { log "attente de la fin du cooldown orchestrateur ($(( reste / 1000 )) s)"; sleep $(( reste / 1000 + 1 )); }
  return 0
}

verdict_bascule() {  # $1 = durée ms
  [ "$1" -le "$CIBLE_BASCULE_MS" ] && echo "< 5 s" || echo "> 5 s"
}

# ── Scénarios ───────────────────────────────────────────────────────────────

# Arrêt brutal du seul processus PostgreSQL du primaire (sa machine reste en
# vie, son orchestrateur aussi). Promesses : bascule, zéro perte, l'ancien
# primaire revient en réplica.
sc_crash_primaire() {
  local nom="Crash du PostgreSQL primaire ($TOPO)"
  attendre_cooldown
  local p; p=$(leader)
  ecriture_acquittee "$p" "avant-crash" || { resultat "$nom" ÉCHEC "écriture de référence refusée"; return; }
  local t0; t0=$(now_ms)
  docker kill -s KILL "$p" >/dev/null
  local np; np=$(attendre_nouveau_primaire "$t0" $(autres "$p")) || { resultat "$nom" ÉCHEC "aucune bascule en 60 s"; docker start "$p" >/dev/null; attendre_sain; return; }
  local bascule=$(( $(now_ms) - t0 )); DERNIERE_BASCULE_MS=$(now_ms)
  present "$np" "avant-crash" || { resultat "$nom" ÉCHEC "perte de données : marqueur absent de $np"; return; }
  docker start "$p" >/dev/null
  attendre_sain || { resultat "$nom" ÉCHEC "le cluster ne se reforme pas"; return; }
  [ "$(role "$p")" = "replica" ] || { resultat "$nom" ÉCHEC "$p n'est pas revenu en réplica"; return; }
  resultat "$nom" OK "bascule $p → $np en $bascule ms ($(verdict_bascule $bascule)), zéro perte, $p revenu en réplica"
}

# Panne complète de la machine du primaire (etcd, PostgreSQL, orchestrateur).
# Promesse : la majorité etcd restante (dont le témoin) suffit à basculer.
sc_panne_machine_primaire() {
  local nom="Panne de la machine du primaire ($TOPO)"
  attendre_cooldown
  local p; p=$(leader); local m; m=$(machine "$p")
  ecriture_acquittee "$p" "avant-panne" || { resultat "$nom" ÉCHEC "écriture de référence refusée"; return; }
  local t0; t0=$(now_ms)
  tuer "$m"
  local np; np=$(attendre_nouveau_primaire "$t0" $(autres "$p")) || { resultat "$nom" ÉCHEC "aucune bascule en 60 s"; relancer "$m"; attendre_sain; return; }
  local bascule=$(( $(now_ms) - t0 )); DERNIERE_BASCULE_MS=$(now_ms)
  present "$np" "avant-panne" || { resultat "$nom" ÉCHEC "perte de données : marqueur absent de $np"; return; }
  relancer "$m"
  attendre_sain || { resultat "$nom" ÉCHEC "le cluster ne se reforme pas"; return; }
  resultat "$nom" OK "machine $m perdue, bascule vers $np en $bascule ms ($(verdict_bascule $bascule)), zéro perte"
}

# Primaire coupé du réseau mais vivant. Promesses : il n'acquitte plus
# aucune écriture, il se rétrograde lui-même, la majorité bascule, et une
# écriture non acquittée n'apparaît jamais dans le cluster.
sc_isolement_primaire() {
  local nom="Isolement réseau du primaire ($TOPO)"
  attendre_cooldown
  local p; p=$(leader); local m; m=$(machine "$p")
  ecriture_acquittee "$p" "avant-isolement" || { resultat "$nom" ÉCHEC "écriture de référence refusée"; return; }
  local t0; t0=$(now_ms)
  isoler "$m"
  # Tentative d'écriture sur le primaire isolé, EN PARALLÈLE de la surveillance
  # de la bascule : l'attendre d'abord ajouterait son délai à la mesure.
  ecriture_acquittee "$p" "ecrite-isole" "$DELAI_ACQUIT_S" & local essai=$!
  local np; np=$(attendre_nouveau_primaire "$t0" $(autres "$p")) || { resultat "$nom" ÉCHEC "la majorité n'a pas basculé en 60 s"; wait "$essai"; reconnecter "$m"; attendre_sain; return; }
  local bascule=$(( $(now_ms) - t0 )); DERNIERE_BASCULE_MS=$(now_ms)
  if wait "$essai"; then
    resultat "$nom" ÉCHEC "le primaire isolé a acquitté une écriture (split-brain)"; reconnecter "$m"; attendre_sain; return
  fi
  local retro=""
  while [ $(( $(now_ms) - t0 )) -lt 60000 ]; do
    if ! inscriptible "$p"; then retro=$(( $(now_ms) - t0 )); break; fi
    sleep 0.5
  done
  [ -n "$retro" ] || { resultat "$nom" ÉCHEC "$p accepte encore des écritures après 60 s d'isolement"; reconnecter "$m"; attendre_sain; return; }
  reconnecter "$m"
  attendre_sain || { resultat "$nom" ÉCHEC "le cluster ne se reforme pas"; return; }
  local l; l=$(leader)
  present "$l" "avant-isolement" || { resultat "$nom" ÉCHEC "perte de données : marqueur absent"; return; }
  if present "$l" "ecrite-isole"; then
    resultat "$nom" ÉCHEC "l'écriture non acquittée du primaire isolé est réapparue"; return
  fi
  resultat "$nom" OK "aucune écriture acquittée par le primaire isolé ; bascule vers $np en $bascule ms ($(verdict_bascule $bascule)) ; $p n'accepte plus d'écriture après $retro ms ; écriture non acquittée écartée au retour"
}

# Perte de la machine du standby avec 2 serveurs PostgreSQL (non strict) :
# les écritures doivent reprendre, le cluster passe en asynchrone.
sc_perte_standby_non_strict() {
  local nom="Perte du standby, non strict ($TOPO)"
  local p; p=$(leader); local s; s=$(autres "$p"); local m; m=$(machine "$s")
  local t0; t0=$(now_ms)
  tuer "$m"
  local reprise=""
  while [ $(( $(now_ms) - t0 )) -lt 60000 ]; do
    if ecriture_acquittee "$p" "sans-standby-$(now_ms)" 2; then reprise=$(( $(now_ms) - t0 )); break; fi
  done
  [ -n "$reprise" ] || { resultat "$nom" ÉCHEC "écritures bloquées plus de 60 s"; relancer "$m"; attendre_sain; return; }
  local sync; sync=$(etat_json | python -c 'import json,sys; print(sum(x["Role"]=="Sync Standby" for x in json.load(sys.stdin)))')
  relancer "$m"
  attendre_sain || { resultat "$nom" ÉCHEC "le cluster ne se reforme pas"; return; }
  resultat "$nom" OK "écritures reprises en $reprise ms sans standby (standby synchrone restant : $sync → le nœud remonte « sous le seuil » au portail, étape 2d) ; standby réintégré"
}

# Isolement du seul témoin etcd : rien ne doit changer.
sc_isolement_temoin() {
  local nom="Isolement du témoin etcd ($TOPO)"
  local p; p=$(leader)
  isoler m3
  local t0; t0=$(now_ms); local ok=1
  while [ $(( $(now_ms) - t0 )) -lt 15000 ]; do
    ecriture_acquittee "$p" "temoin-$(now_ms)" 2 || { ok=0; break; }
    [ "$(role "$p")" = "primary" ] || { ok=0; break; }
    sleep 1
  done
  reconnecter m3
  attendre_sain
  [ "$ok" = 1 ] && resultat "$nom" OK "primaire $p inchangé, écritures acquittées pendant 15 s" \
                || resultat "$nom" ÉCHEC "le primaire a été perturbé par la perte du témoin"
}

# Partition minoritaire d'un standby (3 serveurs) : aucune promotion côté
# minorité, le primaire continue avec l'autre standby.
sc_partition_minoritaire() {
  local nom="Partition minoritaire d'un standby ($TOPO)"
  local p; p=$(leader); local s; s=$(autres "$p" | head -1); local m; m=$(machine "$s")
  isoler "$m"
  local t0; t0=$(now_ms); local promu=0 bloque=0 ecrit=0
  while [ $(( $(now_ms) - t0 )) -lt 30000 ]; do
    [ "$(role "$s")" = "primary" ] && promu=1
    ecriture_acquittee "$p" "minorite-$(now_ms)" 2 && ecrit=1
    sleep 1
  done
  [ "$(leader)" = "$p" ] || bloque=1
  reconnecter "$m"
  attendre_sain
  if [ "$promu" = 1 ] || [ "$bloque" = 1 ] || [ "$ecrit" = 0 ]; then
    resultat "$nom" ÉCHEC "promu=$promu leader_change=$bloque ecritures=$ecrit"
  else
    resultat "$nom" OK "$s isolé 30 s sans se promouvoir ; $p reste primaire et acquitte les écritures"
  fi
}

# Mode strict (3 serveurs) : sans aucun standby synchrone, le primaire ne
# doit plus acquitter d'écriture ; elles reprennent au retour d'un standby.
sc_strict_blocage() {
  local nom="Blocage strict sans standby ($TOPO)"
  local p; p=$(leader); local s1 s2; read -r s1 s2 <<<"$(autres "$p" | tr '\n' ' ')"
  docker kill -s KILL "$s1" "$s2" >/dev/null
  sleep 6   # laisse Patroni constater l'absence de standby
  local bloque=1
  ecriture_acquittee "$p" "strict-sans-standby" 8 && bloque=0
  docker start "$s1" "$s2" >/dev/null
  attendre_sain || { resultat "$nom" ÉCHEC "le cluster ne se reforme pas"; return; }
  local reprend=0
  ecriture_acquittee "$(leader)" "strict-retour" && reprend=1
  if [ "$bloque" = 1 ] && [ "$reprend" = 1 ]; then
    resultat "$nom" OK "aucune écriture acquittée sans standby synchrone ; reprise au retour des standbys"
  else
    resultat "$nom" ÉCHEC "bloque=$bloque reprise=$reprend"
  fi
}

# ── Topologies ──────────────────────────────────────────────────────────────

monter() {
  docker compose --profile trois-pg down -v --remove-orphans >/dev/null 2>&1
  if [ "$1" = temoin ]; then
    PG_NODES=(pg-m1 pg-m2)
    SYNC_STRICT=false docker compose up -d >/dev/null 2>&1
  else
    PG_NODES=(pg-m1 pg-m2 pg-m3)
    SYNC_STRICT=true LAB_PATRONI_NODES=pg-m1@http://10.77.0.12:8008,pg-m2@http://10.77.0.22:8008,pg-m3@http://10.77.0.32:8008 \
      docker compose --profile trois-pg up -d >/dev/null 2>&1
  fi
  TOPO=$1
  log "topologie $1 : démarrage…"
  attendre_sain || { log "topologie $1 : échec de démarrage"; exit 1; }
  log "topologie $1 : saine (leader $(leader))"
}

executer() { [[ -z "$FILTRE" || "$1" == *"$FILTRE"* ]] && "$1"; return 0; }

topologie_temoin() {
  monter temoin
  executer sc_crash_primaire
  executer sc_panne_machine_primaire
  executer sc_isolement_primaire
  executer sc_perte_standby_non_strict
  executer sc_isolement_temoin
}

topologie_trois_pg() {
  monter trois-pg
  executer sc_crash_primaire
  executer sc_panne_machine_primaire
  executer sc_isolement_primaire
  executer sc_partition_minoritaire
  executer sc_strict_blocage
}

case "${1:-tout}" in
  temoin)   topologie_temoin ;;
  trois-pg) topologie_trois_pg ;;
  tout)     topologie_temoin; topologie_trois_pg ;;
  *) echo "usage : $0 [temoin|trois-pg|tout] [filtre]" >&2; exit 2 ;;
esac

echo
echo "=== Bilan des scénarios ==="
echecs=0
for r in "${RESULTATS[@]}"; do
  IFS='|' read -r n v d <<<"$r"
  printf '%-48s %-6s %s\n' "$n" "$v" "$d"
  [ "$v" = OK ] || echecs=$((echecs + 1))
done
exit "$echecs"
