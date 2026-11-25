#!/usr/bin/env bash
# -----------------------------------------------------------------------------
# A-00 — Déploiement de la pile PME AMANE sur le banc de trois VM réelles.
#
# Prérequis : banc démarré (tools/bench/create-bench.sh).
#
# 1. Docker dans chaque VM (get.docker.com via le NAT de ens3) puis
#    `docker run --rm hello-world` ;
# 2. images de la pile chargées depuis l'hôte SANS registre :
#    `docker save <id> | gzip | ssh … 'gunzip | docker load'` — les images
#    locales portent leur référence épinglée (repo@sha256:…) comme tag, elle
#    survit à save/load, donc le compose du banc résout tout localement ;
# 3. portail factice sur m1 (le banc est isolé du vrai portail ; il imite
#    strictement /api/devices/cluster-status et /register) ;
# 4. pile déployée depuis templates/tenants/docker-compose.yml.tmpl
#    (rendu par render-compose.py, vérifié identique à la pile en service) :
#    m1 d'abord (NODE_MODE=active) puis m2, m3 (passive), l'état du portail
#    factice étant mis à jour entre deux amorçages.
#
# Idempotent : chaque étape est sautée si déjà faite (Docker présent, image
# déjà chargée avec le même ID, conteneurs déjà en place).
# -----------------------------------------------------------------------------
set -euo pipefail

BASE=/mnt/data/amane-bench
LOGS=$BASE/logs
PILE=$BASE/pile
REPO=/home/jose/school/amane/repo
CLE_USAGE=/tmp/opencode/amane-bench-cle
VM_LIST=(m1 m2 m3)

declare -A IP_LAN=([m1]=10.77.0.11 [m2]=10.77.0.12 [m3]=10.77.0.13)
declare -A PORT_SSH=([m1]=2211 [m2]=2212 [m3]=2213)
declare -A MODE=([m1]=active [m2]=passive [m3]=passive)

SSH_OPTS=(-o BatchMode=yes -o StrictHostKeyChecking=accept-new
          -o UserKnownHostsFile=$BASE/known_hosts -o ConnectTimeout=5 -o LogLevel=ERROR
          -i $CLE_USAGE)

log()  { printf '[deploy-pile] %s\n' "$*"; }
fail() { printf '[deploy-pile] ERREUR : %s\n' "$*" >&2; exit 1; }

vmssh() { local m=$1; shift; ssh -p "${PORT_SSH[$m]}" "${SSH_OPTS[@]}" etudiant@127.0.0.1 "$@"; }
vmscp() { local m=$1; shift; scp -P "${PORT_SSH[$m]}" "${SSH_OPTS[@]}" "$@" "etudiant@127.0.0.1:"; }

attendre() { # attendre <desc> <timeout_s> <machine> <commande distante>
    local desc=$1 limite=$2 m=$3 cmd=$4 debut=$SECONDS
    log "attente : $desc"
    until vmssh "$m" "$cmd" 2>/dev/null; do
        [ $((SECONDS - debut)) -ge "$limite" ] && fail "délai dépassé : $desc"
        sleep 5
    done
    log "OK : $desc ($((SECONDS - debut)) s)"
}

# Jeton du tenant (celui de la pile de test déjà installée ; jamais affiché).
TOKEN=$(grep -E '^REGISTRATION_TOKEN=' /opt/amane-pme/.env | head -1 | cut -d= -f2-)
[ -n "$TOKEN" ] || fail "REGISTRATION_TOKEN introuvable dans /opt/amane-pme/.env"

# ---------------------------------------------------------------- 0. rendu ---
mkdir -p "$PILE" "$LOGS"
"$REPO/tools/bench/render-compose.py" --verifier > "$PILE/verification-rendu.txt" 2>&1 \
    || fail "le rendu du gabarit diffère de la pile en service (cf. $PILE/verification-rendu.txt)"
"$REPO/tools/bench/render-compose.py" --sans-pgadmin > "$PILE/compose.yml"
for m in "${VM_LIST[@]}"; do
    cat > "$PILE/.env-$m" <<EOF
NODE_IP=${IP_LAN[$m]}
NODE_HOSTNAME=$m
NODE_MODE=${MODE[$m]}
WEB_PORT=9001
SAAS_URL=http://${IP_LAN[m1]}:5173
RELAY_URL=http://10.77.0.254:8080
REGISTRATION_TOKEN=$TOKEN
TENANT_ID=ea49c4c4-f46f-464d-b5db-4fb5928b7e49
PGADMIN_EMAIL=admin@pmeamane-test.com
CHIFFREMENT_DISQUE=non
EOF
done
docker compose -f "$PILE/compose.yml" --env-file "$PILE/.env-m1" config --quiet \
    || fail "compose du banc invalide"
log "compose du banc rendu depuis le gabarit et validé (sans pgadmin)"

# références épinglées attendues par la pile (reprises du compose rendu)
REFS=( \
  "100.64.0.6:5000/amane/etcd@sha256:9344cfb9cbe4df0635478b6a2b62765330128fbdf3ca8fc9f2edac262552f700" \
  "100.64.0.6:5000/amane/postgresql@sha256:17bf371c924196e4671f19d129fa86a80f7dcea7b2666cedca955ec0fba0b72b" \
  "100.64.0.6:5000/amane/orchestrateur@sha256:6a082c400a2809e927e98bc94f2494e80291df92aa987458f7cdcf351fa63c35" \
  "100.64.0.6:5000/amane/haproxy@sha256:43f51ba59dfffa4095c3dfd8363d11a3795369e985249c6d715f468c7def12d2" \
  "100.64.0.6:5000/amane/noeud@sha256:cea1862ce5bbcbbc78a111ed77ee22d19bf629c355ab8fc77393188d78387cd4" \
  "100.64.0.6:5000/editeurs/listmonk@sha256:3089042a786beddb096410da3818b98ce9fe35bdc0a4ad4550f92be674b17291" )

# ------------------------------------------------- 1-2. Docker + images -----
# Marqueur d'exécution : les journaux de préparation sont cumulatifs.
RUN_MARK="=== exécution $(date -Is) ==="

prep_vm() {
    local m=$1
    {
        if ! vmssh "$m" "docker info >/dev/null 2>&1"; then
            log "[$m] installation de Docker (get.docker.com)…"
            vmssh "$m" "curl -fsSL https://get.docker.com | sudo sh" \
                || { echo "FIN-VM-KO $m"; return 1; }
            vmssh "$m" "sudo usermod -aG docker etudiant"
            log "[$m] Docker installé"
        else
            log "[$m] Docker déjà présent"
        fi
        if ! vmssh "$m" "docker image inspect hello-world >/dev/null 2>&1"; then
            vmssh "$m" "docker pull hello-world" \
                || docker save hello-world | gzip | vmssh "$m" "gunzip | docker load" \
                || { echo "FIN-VM-KO $m"; return 1; }
        fi
        vmssh "$m" "docker run --rm hello-world 2>&1 | grep -q 'Hello from Docker!'" \
            || { echo "FIN-VM-KO $m"; return 1; }
        log "[$m] docker run --rm hello-world : OK"

        local ref id
        for ref in "${REFS[@]}"; do
            id=$(docker image inspect "$ref" --format '{{.Id}}')
            if ! vmssh "$m" "docker image inspect $ref --format '{{.Id}}' 2>/dev/null" | grep -q "$id"; then
                # L'image peut déjà avoir été transférée par une exécution
                # précédente sans sa référence épinglée : on la restitue
                # d'abord, sans re-transférer.
                vmssh "$m" "sudo ctr -n moby images tag 'moby-dangling@$id' '$ref' 2>/dev/null" || true
            fi
            if ! vmssh "$m" "docker image inspect $ref --format '{{.Id}}' 2>/dev/null" | grep -q "$id"; then
                log "[$m] chargement de ${ref%%@*}…"
                docker save "$id" | gzip | vmssh "$m" "gunzip | docker load" >/dev/null \
                    || { echo "FIN-VM-KO $m"; return 1; }
                # docker load ne recrée pas les références épinglées contenant
                # @sha256 (elles sont refusées comme tags) : l'image arrive
                # « moby-dangling@<id> » et la référence est restituée par
                # containerd, ce qui rend l'image identique à celle de l'hôte.
                vmssh "$m" "sudo ctr -n moby images tag 'moby-dangling@$id' '$ref'" \
                    || { echo "FIN-VM-KO $m"; return 1; }
            fi
            # vérification : l'image chargée répond à la référence épinglée,
            # avec exactement l'ID de l'hôte (aucun écart de contenu).
            local id_vm
            id_vm=$(vmssh "$m" "docker image inspect $ref --format '{{.Id}}'")
            [ "$id_vm" = "$id" ] || { echo "ECART $m $ref : $id_vm != $id"; echo "FIN-VM-KO $m"; return 1; }
            log "[$m] image OK : ${ref##*/}"
        done
        echo "FIN-VM-OK $m"
    } >> "$LOGS/$m-prep.log" 2>&1
}

for m in "${VM_LIST[@]}"; do
    vmssh "$m" "true" 2>/dev/null || fail "[$m] injoignable — lancer create-bench.sh d'abord"
    # Journal cumulatif : les exécutions précédentes ne sont plus effacées
    # (preuve de l'installation d'origine) ; on cherche le résultat sous le
    # marqueur de l'exécution courante.
    printf '\n%s\n' "$RUN_MARK" >> "$LOGS/$m-prep.log"
    prep_vm "$m" &
done
wait || true
for m in "${VM_LIST[@]}"; do
    awk -v mark="$RUN_MARK" '$0 == mark { depuis = 1; next } depuis' "$LOGS/$m-prep.log" \
        | grep -q "FIN-VM-OK $m" || fail "[$m] préparation incomplète (cf. $LOGS/$m-prep.log)"
    log "[$m] Docker + 6 images : OK"
done

# ----------------------------------------------- 3. portail factice (m1) ----
vmscp m1 "$REPO/tools/bench/portail-mock.py"
cat > /tmp/opencode/amane-portail-mock.service <<'UNIT'
[Unit]
Description=AMANE banc — portail editeur factice (A-00)
After=network.target

[Service]
User=etudiant
ExecStart=/usr/bin/python3 /home/etudiant/portail-mock.py --port 5173 --state /home/etudiant/portail/etat.json --log /home/etudiant/portail/serveur.log
Restart=always

[Install]
WantedBy=multi-user.target
UNIT
vmscp m1 /tmp/opencode/amane-portail-mock.service
vmssh m1 "sudo mv ~/amane-portail-mock.service /etc/systemd/system/ \
          && sudo systemctl daemon-reload \
          && sudo systemctl enable --now amane-portail-mock >/dev/null" \
    && log "portail factice actif sur m1 (10.77.0.11:5173)"
vmssh m1 "mkdir -p ~/portail && echo '{\"active_nodes\": []}' > ~/portail/etat.json"

# ------------------------------------------------ 4. pile, machine par machine
for m in "${VM_LIST[@]}"; do
    vmssh "$m" "mkdir -p ~/amane-pme"
    scp -P "${PORT_SSH[$m]}" "${SSH_OPTS[@]}" "$PILE/compose.yml" "etudiant@127.0.0.1:amane-pme/docker-compose.yml"
    scp -P "${PORT_SSH[$m]}" "${SSH_OPTS[@]}" "$PILE/.env-$m" "etudiant@127.0.0.1:amane-pme/.env"
    log "[$m] compose + .env déposés (NODE_MODE=${MODE[$m]})"

    vmssh "$m" "cd ~/amane-pme && docker compose up -d --pull never >/dev/null" \
        || fail "[$m] docker compose up en échec"
    attendre "[$m] amane-init terminé" 720 \
        "$m" "[ \$(docker inspect -f '{{.State.ExitCode}}' amane-init) = 0 ]"
    attendre "[$m] etcd sain" 720 \
        "$m" "[ \"\$(docker inspect -f '{{.State.Health.Status}}' amane-etcd)\" = healthy ]"

    if [ "$m" = m1 ]; then
        # /patroni (et non /) : sur cette version, Patroni répond 503 sur / à un
        # réplica ; /patroni rend le rôle sur tous les membres.
        attendre "[m1] Patroni élit un primaire" 900 \
            m1 "docker exec amane-postgresql python3 -c \"import urllib.request,json;print(json.load(urllib.request.urlopen('http://localhost:8008/patroni',timeout=5))['role'])\" | grep -Eq '^(primary|leader)$'"
        # m2 doit voir m1 comme machine déjà installée
        vmssh m1 "echo '{\"active_nodes\": [{\"node_addr\": \"${IP_LAN[m1]}:9001\"}]}' > ~/portail/etat.json"
    else
        attendre "[$m] etcd joint le cluster (membre started)" 720 \
            m1 "docker exec amane-etcd etcdctl member list 2>/dev/null | grep -qE ', started, $m,'"
        attendre "[$m] Patroni réplique (pas primaire)" 900 \
            "$m" "docker exec amane-postgresql python3 -c \"import urllib.request,json;print(json.load(urllib.request.urlopen('http://localhost:8008/patroni',timeout=5))['role'])\" | grep -Eq '^(secondary|replica|standby_leader|standby)$'"
        if [ "$m" = m2 ]; then
            vmssh m1 "echo '{\"active_nodes\": [{\"node_addr\": \"${IP_LAN[m1]}:9001\"}, {\"node_addr\": \"${IP_LAN[m2]}:9001\"}]}' > ~/portail/etat.json"
        fi
    fi
done

log "déploiement terminé : vérifier l'état final avec les commandes de docs/tests/BANC-VM.md"
