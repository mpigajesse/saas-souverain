#!/usr/bin/env bash
# -----------------------------------------------------------------------------
# A-00 — Arrêt et effacement du banc de VM AMANE.
#
# Arrête proprement les trois VM (poweroff via SSH, puis arrêt QEMU si
# nécessaire), supprime leurs disques qcow2. Les journaux (logs/) et les
# captures éventuelles (captures/) sous /mnt/data/amane-bench/ sont conservés,
# ainsi que les seeds et les .env pour permettre une recréation.
#
# Ne touche à rien d'autre : en particulier pas à la VM de TP
# (/mnt/data/vm-tp4) ni aux conteneurs de la pile en service (amane-*).
# -----------------------------------------------------------------------------
set -euo pipefail

BASE=/mnt/data/amane-bench
LOGS=$BASE/logs
CLE_USAGE=/tmp/opencode/amane-bench-cle
# Commande d'arrêt propre par SSH : si la clé 0600 de travail a disparu (tmp
# nettoyé), on la reconstitue depuis la copie persistée du banc.
if [ ! -f "$CLE_USAGE" ] && [ -f "$BASE/cle-bench" ]; then
    mkdir -p "$(dirname "$CLE_USAGE")"
    install -m 600 "$BASE/cle-bench" "$CLE_USAGE"
fi
VM_LIST=(m1 m2 m3)
declare -A PORT_SSH=([m1]=2211 [m2]=2212 [m3]=2213)

SSH_OPTS=(-o BatchMode=yes -o StrictHostKeyChecking=accept-new
          -o UserKnownHostsFile=$BASE/known_hosts -o ConnectTimeout=5 -o LogLevel=ERROR
          -i $CLE_USAGE)

log()  { printf '[destroy-bench] %s\n' "$*"; }

# Garde-fou : la suppression ne porte que sur le répertoire du banc, et chaque
# chemin effacé doit en être un sous-répertoire direct de la forme m1/m2/m3.
case "$BASE" in
    /mnt/data/amane-bench) ;;
    *) printf '[destroy-bench] ERREUR : BASE inattendu (%s) — refus\n' "$BASE" >&2; exit 1 ;;
esac
[ -d "$BASE" ] || { printf '[destroy-bench] ERREUR : %s absent\n' "$BASE" >&2; exit 1; }
garde_chemin() {
    case "$1" in
        "$BASE"/m[123]/?*) ;;
        *) printf '[destroy-bench] ERREUR : chemin hors banc refusé (%s)\n' "$1" >&2; exit 1 ;;
    esac
}
effacer() { # effacer <chemin> : rm -f après contrôle de chemin
    garde_chemin "$1"
    rm -f "$1"
}

vm_pid() {
    local p=$BASE/$1/qemu.pid
    [ -f "$p" ] || return 1
    local pid
    pid=$(cat "$p" 2>/dev/null) || return 1
    kill -0 "$pid" 2>/dev/null && echo "$pid"
}

for m in "${VM_LIST[@]}"; do
    pid=$(vm_pid "$m") || pid=""
    if [ -n "$pid" ]; then
        log "[$m] extinction (pid $pid)"
        ssh -p "${PORT_SSH[$m]}" "${SSH_OPTS[@]}" etudiant@127.0.0.1 \
            "sudo poweroff" >/dev/null 2>&1 || true
        # attente de l'extinction électrique (jusqu'à 45 s)
        for _ in $(seq 1 15); do
            kill -0 "$pid" 2>/dev/null || break
            sleep 3
        done
        if kill -0 "$pid" 2>/dev/null; then
            log "[$m] toujours active après poweroff — arrêt de QEMU (SIGTERM)"
            kill "$pid" 2>/dev/null || true
            sleep 3
            kill -0 "$pid" 2>/dev/null && kill -9 "$pid" 2>/dev/null || true
        fi
        log "[$m] éteinte"
    else
        log "[$m] déjà éteinte"
    fi
    effacer "$BASE/$m/disk.qcow2"
    effacer "$BASE/$m/qemu.pid"
    effacer "$BASE/$m/monitor.sock"
    effacer "$BASE/$m/.booted-once"
    # l'hôte SSH du port mapé correspondait à l'ancienne machine
    ssh-keygen -R "[127.0.0.1]:${PORT_SSH[$m]}" -f "$BASE/known_hosts" >/dev/null 2>&1 || true
done

log "disques supprimés ; journaux et captures conservés dans $LOGS et $BASE/captures"
log "banc détruit (les seeds et .env restent pour une recréation par create-bench.sh)"
