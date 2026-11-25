#!/usr/bin/env bash
# HA-04 : bascule HAProxy. Attendu : le backend suit le leader en moins de 5 s,
# aucune requête routée vers un nœud non leader.
#
# Protocole :
#   1. Rend la configuration HAProxy avec le VRAI code (haproxy.Rendre) à partir
#      des membres Patroni réels du ha-lab. Le `option httpchk GET /primary` +
#      `http-check expect status 200` fait que SEUL le leader passe le check :
#      HAProxy ne route QUE vers un serveur marqué UP, donc jamais vers un
#      non-leader.
#   2. Démarre un HAProxy RÉEL (image amane/haproxy) sur le LAN ha-lab, avec un
#      écouteur de statistiques (ajouté par le harnais, purement pour l'audit —
#      ne modifie pas le routage).
#   3. Lit l'état des serveurs (UP/DOWN) via /stats;csv : exactement un UP, et
#      c'est le leader Patroni (primary_code 200). Preuve directe qu'aucune
#      requête ne peut partir vers un non-leader (tous DOWN).
#   4. Déclenche une bascule CONTRÔLÉE (Patroni /switchover) et mesure le temps
#      avant que l'unique serveur UP devienne le NOUVEAU leader (< 5 s).
#   5. Restaure le leader d'origine.
#
# N'utilise QUE le projet ha-lab. Crée un conteneur jetable ha04-haproxy (retiré
# en fin). Ne touche aucun conteneur amane-*. Sorties dans artifacts-ha01/.
set -uo pipefail
cd "$(dirname "$0")/../.."

export PATH="$PATH:/home/jose/go-dev/go/bin"
export HA_ETCD="${HA_ETCD:-10.77.0.11:2379}"
export HA_SCOPE="${HA_SCOPE:-amane-lab}"
OUT="docs/tests/artifacts-ha01"
mkdir -p "$OUT"

LAN=amane-ha-lab_lan
HPX_IP=10.77.0.50
HPX_PORT=6432
HPX_STATS=8404
HPX_NAME=ha04-haproxy
IMAGE=43f51ba59dff   # amane/haproxy (HAProxy 3.0.30)
PG_M1=10.77.0.12
PG_M2=10.77.0.22

log() { printf '%s  %s\n' "$(date +%H:%M:%S)" "$*"; }
role() { curl -s -m 2 "http://$1:8008/patroni" 2>/dev/null | python3 -c 'import json,sys;print(json.load(sys.stdin).get("role","?"))' 2>/dev/null || echo down; }
primary_code() { curl -s -m 2 -o /dev/null -w '%{http_code}' "http://$1:8008/primary" 2>/dev/null || echo 000; }

# ip du membre
ip_de() { case "$1" in pg-m1) echo $PG_M1;; pg-m2) echo $PG_M2;; pg-m3) echo 10.77.0.32;; esac; }

# État UP/DOWN des serveurs depuis /stats;csv -> "pg-m1:UP pg-m2:DOWN ...".
# On ne garde que les lignes serveur du proxy "primaire" (champ 1 = primaire,
# champ 2 = nom du serveur, pas FRONTEND/BACKEND), statut = colonne 18.
etats() {
  curl -s -m 2 "http://$HPX_IP:$HPX_STATS/stats;csv" 2>/dev/null \
    | awk -F, '$1=="primaire" && $2!="" && $2!="FRONTEND" && $2!="BACKEND" {printf "%s:%s ", $2,$18}'
}
# Renvoie le nom du SEUL serveur UP (chaîne vide si 0 ou >1).
seul_up() {
  etats | tr ' ' '\n' | awk -F: 'NF==2 && $2=="UP"{print $1}'
}

cleanup() {
  if [ -n "${ORIG_LEADER:-}" ]; then
    # Restaure le leader d'origine : le candidat doit être le sync_standby, qui
    # tourne à chaque bascule. On boucle jusqu'à 6 tentatives en lisant le
    # sync_standby courant dans etcd.
    for _ in $(seq 1 6); do
      cur_up=$(seul_up)
      [ -z "$cur_up" ] && break
      [ "$cur_up" = "$ORIG_LEADER" ] && break
      ss=$(docker exec etcd-m1 etcdctl get /service/$HA_SCOPE/sync --print-value-only 2>/dev/null \
        | python3 -c 'import json,sys;print(json.load(sys.stdin).get("sync_standby") or "")' 2>/dev/null)
      [ -z "$ss" ] && ss=$(for c in pg-m1 pg-m2 pg-m3; do [ "$c" != "$cur_up" ] && echo "$c" && break; done)
      log "restauration: $cur_up -> $ss (puis vers $ORIG_LEADER)"
      curl -s -m 20 -o /dev/null -X POST "http://$(ip_de "$cur_up"):8008/switchover" \
        -H 'Content-Type: application/json' -d "{\"leader\":\"$cur_up\",\"candidate\":\"$ss\"}" 2>/dev/null || true
      sleep 4
    done
  fi
  docker rm -f "$HPX_NAME" >/dev/null 2>&1 || true
}
trap cleanup EXIT

# ── 1. Configuration HAProxy par le vrai code (+ écouteur stats d'audit) ────
( cd tools/ha && go run render-haproxy.go "$HPX_PORT" ) > "$OUT/ha04-haproxy.cfg" 2> "$OUT/ha04-render.err"
if [ ! -s "$OUT/ha04-haproxy.cfg" ] || ! grep -q "listen primaire" "$OUT/ha04-haproxy.cfg"; then
  log "BLOQUE : configuration HAProxy non produite"; cat "$OUT/ha04-render.err"; exit 2
fi
# Ajout d'un écouteur de statistiques (audit uniquement, aucun effet routage).
cat >> "$OUT/ha04-haproxy.cfg" <<EOF

listen stats
    mode http
    bind *:$HPX_STATS
    stats enable
    stats uri /stats
    stats refresh 1s
    stats show-legends
EOF
log "configuration HAProxy rendue (Rendre) :"
grep -E 'server |httpchk|bind ' "$OUT/ha04-haproxy.cfg" | sed 's/^/    /'

# ── 2. Démarrage HAProxy réel sur le lan ────────────────────────────────────
docker rm -f "$HPX_NAME" >/dev/null 2>&1 || true
docker run -d --name "$HPX_NAME" --network "$LAN" --ip "$HPX_IP" \
  -v "$PWD/$OUT/ha04-haproxy.cfg:/usr/local/etc/haproxy/haproxy.cfg:ro" \
  --entrypoint haproxy "$IMAGE" -f /usr/local/etc/haproxy/haproxy.cfg >/dev/null 2>&1
sleep 2
if ! docker ps --format '{{.Names}}' | grep -q "^$HPX_NAME$"; then
  log "BLOQUE : HAProxy n'a pas démarré"; docker logs "$HPX_NAME" 2>&1 | tail; exit 2
fi

# ── 3. Routage initial : exactement un UP = leader ──────────────────────────
ORIG_LEADER=""; for n in pg-m1 pg-m2 pg-m3; do [ "$(primary_code "$(ip_de $n)")" = 200 ] && ORIG_LEADER=$n; done
if [ -z "$ORIG_LEADER" ]; then log "BLOQUE : aucun leader Patroni"; exit 2; fi
log "leader d'origine : $ORIG_LEADER"

ok=0
for i in $(seq 1 20); do
  up=$(seul_up)
  if [ "$up" = "$ORIG_LEADER" ]; then ok=1; break; fi
  sleep 0.5
done
if [ "$ok" != 1 ]; then
  log "FAIL : état serveurs inattendu après 10 s : $(etats)"; exit 1
fi
log "OK : unique serveur UP = $ORIG_LEADER (leader) — états: $(etats)"
# Preuve : tous les autres serveurs sont DOWN -> aucune requête ne peut y aller.
nb_up=$(seul_up | wc -w); [ "$nb_up" = 1 ] || { log "FAIL : $nb_up serveurs UP"; exit 1; }

# ── 4. Bascule contrôlée et mesure de convergence HAProxy ───────────────────
# Le candidat doit être accepté par Patroni (parfois contrainte sync_standby) :
# on essaie chaque réplica jusqu'à ce qu'un switchover aboutisse.
CAND=""
for try in pg-m1 pg-m2 pg-m3; do
  [ "$try" = "$ORIG_LEADER" ] && continue
  code=$(curl -s -m 25 -o /dev/null -w '%{http_code}' -X POST "http://$(ip_de "$ORIG_LEADER"):8008/switchover" \
    -H 'Content-Type: application/json' -d "{\"leader\":\"$ORIG_LEADER\",\"candidate\":\"$try\"}" 2>/dev/null)
  if [ "$code" = 200 ]; then CAND=$try; break; fi
  log "switchover vers $try refusé (HTTP $code), essai suivant"
done
if [ -z "$CAND" ]; then log "FAIL : aucun réplica n'a accepté la bascule"; exit 1; fi

log "bascule contrôlée $ORIG_LEADER -> $CAND (Patroni /switchover accepté)"
t0=$(date +%s%3N)

conv_ms=-1
for i in $(seq 1 120); do
  now=$(date +%s%3N); el=$((now - t0))
  up=$(seul_up)
  if [ "$up" = "$CAND" ]; then
    # Confirmer que CAND est bien devenu le leader Patroni.
    if [ "$(primary_code "$(ip_de "$CAND")")" = 200 ]; then conv_ms=$el; break; fi
  fi
  [ $el -gt 8000 ] && break
  sleep 0.1
done

etats | sed 's/^/    /' > "$OUT/ha04-etats-final.txt"
if [ "$conv_ms" -lt 0 ]; then
  log "FAIL : HAProxy n'a pas suivi le nouveau leader en 8 s — états: $(etats)"
  docker logs "$HPX_NAME" 2>&1 | tail -5
  exit 1
fi
log "OK : HAProxy a suivi le nouveau leader ($CAND) en ${conv_ms} ms — états: $(etats)"
if [ "$conv_ms" -ge 5000 ]; then
  log "FAIL : convergence ${conv_ms} ms >= 5000 ms (cible < 5 s)"; exit 1
fi

# Après bascule, l'unique UP est le nouveau leader, jamais l'ancien.
up=$(seul_up)
if [ "$up" != "$CAND" ]; then log "FAIL : serveur UP=$up, attendu $CAND"; exit 1; fi
log "OK : après bascule, unique UP = $CAND (nouveau leader), ancien = DOWN"
log "HA-04 OK : backend suit le leader en ${conv_ms} ms (< 5 s), aucun routage non-leader"
exit 0