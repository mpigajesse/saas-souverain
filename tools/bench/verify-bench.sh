#!/usr/bin/env bash
# -----------------------------------------------------------------------------
# A-00 — Vérification de bout en bout du banc de trois VM (lecture seule).
#
# Ne modifie rien : interroge les trois VM (QEMU, réseau, etcd, Patroni,
# ss-node, application) et la pile locale chargée, puis imprime une ligne
# « OK » / « ÉCHEC » par contrôle. Sort 1 si un contrôle échoue.
#
# Sortie à conserver : docs/tests/artifacts-a00/verification-banc.log
# Usage : tools/bench/verify-bench.sh [--sortie FICHIER]
# -----------------------------------------------------------------------------
set -uo pipefail

BASE=/mnt/data/amane-bench
CLE_USAGE=/tmp/opencode/amane-bench-cle
VM_LIST=(m1 m2 m3)
declare -A IP_LAN=([m1]=10.77.0.11 [m2]=10.77.0.12 [m3]=10.77.0.13)
declare -A PORT_SSH=([m1]=2211 [m2]=2212 [m3]=2213)
declare -A PORT_WEB=([m1]=9011 [m2]=9012 [m3]=9013)
ECHECS=0

SSH_OPTS=(-o BatchMode=yes -o StrictHostKeyChecking=accept-new
          -o UserKnownHostsFile=$BASE/known_hosts -o ConnectTimeout=5 -o LogLevel=ERROR
          -i $CLE_USAGE)

if [ "${1:-}" = "--sortie" ] && [ -n "${2:-}" ]; then
    exec > >(tee "$2") 2>&1
fi

ok()    { printf 'OK     %s\n' "$*"; }
ko()    { printf 'ÉCHEC  %s\n' "$*"; ECHECS=$((ECHECS + 1)); }
titre() { printf '\n### %s\n' "$*"; }
vmssh() { local m=$1; shift; ssh -p "${PORT_SSH[$m]}" "${SSH_OPTS[@]}" etudiant@127.0.0.1 "$@"; }

rolerest() { # rôle Patroni d'une VM (via /patroni, valide sur primaire et réplica)
    vmssh "$1" "docker exec amane-postgresql python3 -c \"import urllib.request,json;print(json.load(urllib.request.urlopen('http://localhost:8008/patroni',timeout=5))['role'])\"" 2>/dev/null
}

printf 'Banc AMANE — vérification %s\n' "$(date -Is)"

# ---------------------------------------------------------------- 1. QEMU -----
titre "1. machines (QEMU/KVM, sans root)"
n=0
for m in "${VM_LIST[@]}"; do
    pid=$(cat "$BASE/$m/qemu.pid" 2>/dev/null || true)
    if [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null; then
        n=$((n + 1))
        cmd=$(tr '\0' ' ' < "/proc/$pid/cmdline")
        ram=$(printf '%s' "$cmd" | grep -o '\-m [0-9]*' | head -1)
        cpu=$(printf '%s' "$cmd" | grep -o '\-smp [0-9]*' | head -1)
        printf '       %s pid=%s %s %s disque=%s\n' \
            "$m" "$pid" "$ram" "$cpu" "$(du -h "$BASE/$m/disk.qcow2" | cut -f1)"
    else
        ko "[$m] QEMU absent (pas de pidfile vivant)"
    fi
done
[ "$n" -eq 3 ] && ok "3 VM QEMU démarrées (-daemonize -pidfile)" || ko "seulement $n VM démarrées"
racine=$(df --output=target "$BASE" | tail -1)
printf '       stockage sous %s (%s), aucun disque hors /mnt/data\n' "$racine" "$(df -h "$BASE" | tail -1 | awk '{print $4" libres"}')"

# ------------------------------------------------------------- 2. réseau ------
titre "2. réseau (ens3 NAT DHCP, ens4 LAN fixe, ping croisé)"
for m in "${VM_LIST[@]}"; do
    info=$(vmssh "$m" "printf '%s|%s' \"\$(ip -4 -br addr show ens3 | awk '{print \$3}')\" \"\$(ip -4 -br addr show ens4 | awk '{print \$3}')\"" 2>/dev/null)
    ens3=${info%%|*}; ens4=${info##*|}
    if printf '%s' "$ens3" | grep -q '^10\.0\.2\.' && [ "$ens4" = "${IP_LAN[$m]}/24" ]; then
        ok "[$m] ens3=$ens3 ens4=$ens4"
    else
        ko "[$m] réseau inattendu : ens3=$ens3 ens4=$ens4"
    fi
done
ping_croise() {
    local de=$1 vers=${IP_LAN[$2]}
    local res
    res=$(vmssh "$de" "ping -c 5 -W 2 $vers" 2>&1 | tail -2 | tr '\n' ' ')
    if printf '%s' "$res" | grep -q '0% packet loss'; then
        ok "ping $de -> $vers : $(printf '%s' "$res" | grep -o 'min/avg/max[^ ]* = [^ ]*')"
    else
        ko "ping $de -> $vers : $res"
    fi
}
ping_croise m1 m2
ping_croise m2 m3
ping_croise m3 m1

# -------------------------------------------------------------- 3. etcd -------
titre "3. cluster etcd (3 membres)"
membres=$(vmssh m1 "docker exec amane-etcd etcdctl member list" 2>/dev/null)
printf '%s\n' "$membres" | sed 's/^/       /'
demarres=$(printf '%s\n' "$membres" | grep -c ', started, ')
[ "$demarres" -eq 3 ] && ok "etcd : 3 membres started" || ko "etcd : $demarres membre(s) started"
sante=$(vmssh m1 "docker exec amane-etcd etcdctl endpoint health --cluster" 2>/dev/null)
printf '%s\n' "$sante" | sed 's/^/       /'
[ "$(printf '%s\n' "$sante" | grep -c 'is healthy: successfully')" -eq 3 ] \
    && ok "etcd : 3 points de terminaison sains" || ko "etcd : santé incomplète"

# ------------------------------------------------------------ 4. Patroni ------
titre "4. Patroni : un primaire, deux réplicas"
primaires=0; roles=""
for m in "${VM_LIST[@]}"; do
    r=$(rolerest "$m")
    roles="$roles $m=$r"
    [ "$r" = primary ] && primaires=$((primaires + 1))
done
printf '      %s\n' "$roles"
[ "$primaires" -eq 1 ] && ok "Patroni : exactement 1 primaire" || ko "Patroni : $primaires primaire(s)"
vmssh m1 "docker exec amane-postgresql patronictl list" 2>/dev/null | sed 's/^/       /'

# ------------------------------------------------------------ 5. ss-node ------
titre "5. ss-node et application"
for m in "${VM_LIST[@]}"; do
    etat=$(vmssh "$m" "docker inspect -f '{{.State.Status}}' ss-node" 2>/dev/null)
    mode=$(vmssh "$m" "docker inspect -f '{{range .Config.Env}}{{println .}}{{end}}' ss-node | grep -E '^NODE_MODE=' | cut -d= -f2" 2>/dev/null)
    [ "$etat" = running ] && ok "[$m] ss-node running (NODE_MODE=$mode)" || ko "[$m] ss-node : $etat"
done
code=$(curl -s -o /tmp/opencode/amane-app-m1.html -w '%{http_code}' http://127.0.0.1:9011/ 2>/dev/null || true)
if [ "$code" = 200 ]; then
    ok "application servie par m1 (http://127.0.0.1:9011/ -> 200, $(wc -c < /tmp/opencode/amane-app-m1.html) octets)"
else
    ko "application : http://127.0.0.1:9011/ -> $code"
fi
sante=$(curl -sf http://127.0.0.1:9011/health 2>/dev/null || true)
[ -n "$sante" ] && ok "santé de l'application : $sante" || ko "santé de l'application injoignable (/health)"

# --------------------------------------------------- 6. images sans registre ---
titre "6. images chargées depuis l'hôte (docker save | ssh … docker load)"
for m in "${VM_LIST[@]}"; do
    n_img=$(vmssh "$m" "docker images --format '{{.Repository}}@{{.Digest}}' | grep -c '^100.64.0.6:5000/'" 2>/dev/null)
    [ "${n_img:-0}" -ge 6 ] && ok "[$m] $n_img références épinglées présentes" \
        || ko "[$m] $n_img références (6 attendues)"
done
printf '       ids hôte : %s\n' "$(docker images --no-trunc --format '{{.ID}}' 2>/dev/null | wc -l) images au total sur l'hôte"

# ------------------------------------------------------------- 7. Docker ------
titre "7. Docker dans chaque VM"
for m in "${VM_LIST[@]}"; do
    v=$(vmssh "$m" "docker version --format '{{.Server.Version}}'" 2>/dev/null)
    [ -n "$v" ] && ok "[$m] docker $v" || ko "[$m] docker absent"
done

# --------------------------------------------------------- 8. pile en service --
titre "8. pile en service de l'hôte (jamais modifiée)"
docker ps --filter name=amane- --format '       {{.Names}} : {{.Status}}' 2>/dev/null

printf '\n%s contrôle(s) en échec\n' "$ECHECS"
[ "$ECHECS" -eq 0 ]
