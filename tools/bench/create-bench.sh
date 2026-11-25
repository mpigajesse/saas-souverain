#!/usr/bin/env bash
# -----------------------------------------------------------------------------
# A-00 — Banc de trois VM KVM sans root (AMANE).
#
# Crée et démarre (idempotent) trois machines virtuelles réelles à partir de
# l'image Ubuntu 22.04 déjà validée (/mnt/data/vm-tp4, SHA256 vérifié) :
#
#   - disque : overlay qcow2 de 20 Go par VM dans /mnt/data/amane-bench/mN/
#     (rien sur /) ;
#   - 4096 Mo de RAM, 2 vCPU, KVM (/dev/kvm accessible sans root) ;
#   - cloud-init : utilisateur `etudiant`, clé SSH de l'hôte, ssh_pwauth: false ;
#   - ens3 : DHCP sur le NAT QEMU (apt, docker), SSH redirigé sur
#     127.0.0.1:22NN, port web 9001 redirigé sur 127.0.0.1:90NN ;
#   - ens4 : IP fixe 10.77.0.1N/24 sur le commutateur virtuel sans root
#     (-netdev socket en mode mcast sur 127.0.0.1:7777 — voir BANC-VM.md :
#     le mode listen du socket netdev ne tient qu'une seule connexion, le
#     multicast est l'équivalent sans root pour trois machines) ;
#   - MAC fixes + adresses PCI fixes (0x3, 0x4) : noms stables ens3/ens4 ;
#     le netplan généré vise ens3/ens4 (jamais enp0s3).
#
# Idempotent : un disque existant est conservé, une VM déjà démarrée n'est pas
# touchée, une VM arrêtée est redémarrée telle quelle.
# -----------------------------------------------------------------------------
set -euo pipefail

BASE=/mnt/data/amane-bench
IMAGE_BASE=/mnt/data/vm-tp4/jammy-server-cloudimg-amd64.img
LOGS=$BASE/logs
CLE=$BASE/cle-bench          # paire de clés dédiée au banc (générée si absente)
# /mnt/data est monté fuseblk (permissions forcées 0777) : ssh refuse une clé
# trop ouverte. La clé privée persistée est donc copiée en 0600 dans /tmp/opencode.
CLE_USAGE=/tmp/opencode/amane-bench-cle
PUBKEY=$(cat "$CLE.pub" 2>/dev/null || true)
MCAST=230.0.0.1:7777
VM_LIST=(m1 m2 m3)

declare -A IP_LAN=([m1]=10.77.0.11 [m2]=10.77.0.12 [m3]=10.77.0.13)
declare -A PORT_SSH=([m1]=2211 [m2]=2212 [m3]=2213)
declare -A PORT_WEB=([m1]=9011 [m2]=9012 [m3]=9013)
declare -A MAC_NAT=([m1]=52:54:00:10:77:11 [m2]=52:54:00:10:77:12 [m3]=52:54:00:10:77:13)
declare -A MAC_LAN=([m1]=52:54:00:20:77:11 [m2]=52:54:00:20:77:12 [m3]=52:54:00:20:77:13)

SSH_OPTS=(-o BatchMode=yes -o StrictHostKeyChecking=accept-new
          -o UserKnownHostsFile=$BASE/known_hosts -o ConnectTimeout=5 -o LogLevel=ERROR
          -i $CLE_USAGE)

log()  { printf '[create-bench] %s\n' "$*"; }
fail() { printf '[create-bench] ERREUR : %s\n' "$*" >&2; exit 1; }

[ -f "$IMAGE_BASE" ] || fail "image de base absente : $IMAGE_BASE"
if [ ! -f "$CLE" ]; then
    ssh-keygen -t ed25519 -f "$CLE" -N "" -C amane-bench >/dev/null
    log "clé SSH du banc générée : $CLE"
fi
mkdir -p "$(dirname "$CLE_USAGE")"
install -m 600 "$CLE" "$CLE_USAGE"
PUBKEY=$(cat "$CLE.pub")
mkdir -p "$LOGS"

vm_pid() {
    local p=$BASE/$1/qemu.pid
    [ -f "$p" ] || return 1
    local pid
    pid=$(cat "$p" 2>/dev/null) || return 1
    kill -0 "$pid" 2>/dev/null && echo "$pid"
}
vm_running() { [ -n "$(vm_pid "$1")" ]; }

# --- cloud-init : user-data, meta-data, network-config (par VM) ---------------
# N'écrit un fichier que si son contenu change : un second lancement ne doit
# pas réécrire les seeds (ni toucher aux médias déjà attachés aux VM).
ecrire_si_different() { # ecrire_si_different <fichier> ; contenu sur stdin
    local f=$1 tmp
    tmp=$(mktemp)
    cat > "$tmp"
    if cmp -s "$tmp" "$f"; then
        rm -f "$tmp"
        return 0
    fi
    mv "$tmp" "$f"
}

write_cloud_init() {
    local m=$1 ip=$2 mac_nat=$3 mac_lan=$4 dir=$BASE/$m
    ecrire_si_different "$dir/user-data" <<EOF
#cloud-config
hostname: $m
manage_etc_hosts: true
disable_root: true
ssh_pwauth: false
users:
  - name: etudiant
    groups: sudo
    shell: /bin/bash
    lock_passwd: true
    sudo: ALL=(ALL) NOPASSWD:ALL
    ssh_authorized_keys:
      - $PUBKEY
write_files:
  - path: /etc/ssh/sshd_config.d/99-bench-no-password.conf
    permissions: '0644'
    content: |
      PasswordAuthentication no
      KbdInteractiveAuthentication no
      ChallengeResponseAuthentication no
      PermitRootLogin no
runcmd:
  - [systemctl, restart, ssh]
EOF
    ecrire_si_different "$dir/meta-data" <<EOF
instance-id: amane-bench-$m
local-hostname: $m
EOF
    ecrire_si_different "$dir/network-config" <<EOF
# Netplan vise ens3/ens4 (MAC fixées + emplacements PCI 0x3/0x4 fixés par QEMU).
version: 2
ethernets:
  ens3:
    match:
      macaddress: "$mac_nat"
    set-name: ens3
    dhcp4: true
    dhcp4-overrides:
      route-metric: 100
  ens4:
    match:
      macaddress: "$mac_lan"
    set-name: ens4
    addresses: [$ip/24]
EOF
}

write_seed() {
    local dir=$BASE/$1
    if [ ! -f "$dir/seed.iso" ] || [ "$dir/user-data" -nt "$dir/seed.iso" ] \
       || [ "$dir/meta-data" -nt "$dir/seed.iso" ] \
       || [ "$dir/network-config" -nt "$dir/seed.iso" ]; then
        (cd "$dir" && xorriso -as mkisofs -quiet -V cidata -o seed.iso \
            -J -r user-data meta-data network-config)
        log "[$1] seed.iso régénéré"
    fi
}

ensure_disk() {
    local m=$1 dir=$BASE/$m
    if [ ! -f "$dir/disk.qcow2" ]; then
        qemu-img create -f qcow2 -F qcow2 -b "$IMAGE_BASE" "$dir/disk.qcow2" 20G >/dev/null
        log "[$m] disque qcow2 (20 Go, overlay) créé"
    fi
}

boot_vm() {
    local m=$1 dir=$BASE/$m
    if vm_running "$m"; then
        log "[$m] déjà démarrée (pid $(vm_pid "$m"))"
        return 0
    fi
    rm -f "$dir/qemu.pid" "$dir/monitor.sock"
    # Premier démarrage d'un disque neuf : l'hôte SSH précédent (même port)
    # n'a plus de rapport avec la nouvelle machine.
    if [ ! -f "$dir/.booted-once" ]; then
        ssh-keygen -R "[127.0.0.1]:${PORT_SSH[$m]}" -f "$BASE/known_hosts" >/dev/null 2>&1 || true
    fi
    qemu-system-x86_64 \
        -name amane-bench-$m \
        -enable-kvm -cpu host \
        -m 4096 -smp 2 \
        -drive file="$dir/disk.qcow2",if=virtio,format=qcow2 \
        -drive file="$dir/seed.iso",format=raw,if=virtio,readonly=on \
        -netdev user,id=n0,hostfwd=tcp:127.0.0.1:${PORT_SSH[$m]}-:22,hostfwd=tcp:127.0.0.1:${PORT_WEB[$m]}-:9001 \
        -device virtio-net-pci,netdev=n0,mac=${MAC_NAT[$m]},addr=0x3 \
        -netdev socket,id=n1,mcast=$MCAST,localaddr=127.0.0.1 \
        -device virtio-net-pci,netdev=n1,mac=${MAC_LAN[$m]},addr=0x4 \
        -display none \
        -monitor unix:$dir/monitor.sock,server,nowait \
        -pidfile "$dir/qemu.pid" \
        -serial file:$LOGS/$m-serial.log \
        -daemonize
    touch "$dir/.booted-once"
    log "[$m] QEMU démarré (pid $(vm_pid "$m"))"
}

wait_ssh() {
    local m=$1 limite=$((SECONDS + 900))
    log "[$m] attente de SSH (port ${PORT_SSH[$m]})…"
    until ssh -p "${PORT_SSH[$m]}" "${SSH_OPTS[@]}" etudiant@127.0.0.1 true 2>/dev/null; do
        [ $SECONDS -ge $limite ] && fail "[$m] SSH inaccessible après 15 min (voir $LOGS/$m-serial.log)"
        sleep 5
    done
    log "[$m] SSH opérationnel"
}

ensure_net() {
    local m=$1 ip=${IP_LAN[$1]}
    local etat
    etat=$(ssh -p "${PORT_SSH[$m]}" "${SSH_OPTS[@]}" etudiant@127.0.0.1 "ip -br addr show ens4 2>/dev/null || true")
    if ! printf '%s' "$etat" | grep -q "$ip"; then
        log "[$m] ens4 sans $ip — application du netplan"
        ssh -p "${PORT_SSH[$m]}" "${SSH_OPTS[@]}" etudiant@127.0.0.1 "sudo netplan apply"
        sleep 4
        etat=$(ssh -p "${PORT_SSH[$m]}" "${SSH_OPTS[@]}" etudiant@127.0.0.1 "ip -br addr show ens4 2>/dev/null || true")
        printf '%s' "$etat" | grep -q "$ip" || fail "[$m] ens4 n'a pas $ip : $etat"
    fi
    log "[$m] réseau OK (ens4=$ip)"
}

cross_ping() {
    local de=$1 vers=$2
    if ssh -p "${PORT_SSH[$de]}" "${SSH_OPTS[@]}" etudiant@127.0.0.1 \
        "ping -c 3 -W 2 $vers >/dev/null"; then
        local ms
        ms=$(ssh -p "${PORT_SSH[$de]}" "${SSH_OPTS[@]}" etudiant@127.0.0.1 \
             "ping -c 3 -W 2 $vers | tail -1 | cut -d/ -f5")
        log "ping $de -> $vers : OK (moy. ${ms} ms)"
    else
        fail "ping $de -> $vers : ÉCHEC (commutateur virtuel en cause)"
    fi
}

# --- exécution ----------------------------------------------------------------
for m in "${VM_LIST[@]}"; do
    mkdir -p "$BASE/$m"
    write_cloud_init "$m" "${IP_LAN[$m]}" "${MAC_NAT[$m]}" "${MAC_LAN[$m]}" "$BASE/$m"
    write_seed "$m"
    ensure_disk "$m"
    boot_vm "$m"
done
for m in "${VM_LIST[@]}"; do
    wait_ssh "$m"
    ensure_net "$m"
done
log "connectivité croisée du LAN du banc :"
cross_ping m1 "${IP_LAN[m2]}"
cross_ping m2 "${IP_LAN[m3]}"
cross_ping m3 "${IP_LAN[m1]}"
log "banc prêt : 3 VM démarrées (m1=${IP_LAN[m1]}, m2=${IP_LAN[m2]}, m3=${IP_LAN[m3]})"
log "SSH : 127.0.0.1:${PORT_SSH[m1]}/${PORT_SSH[m2]}/${PORT_SSH[m3]} — web : :${PORT_WEB[m1]}/${PORT_WEB[m2]}/${PORT_WEB[m3]}"
