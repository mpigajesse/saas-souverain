#!/usr/bin/env bash
# Exécute les six épreuves HA-01..HA-06 contre l'etcd du laboratoire ha-lab.
#
# N'utilise QUE le projet ha-lab (conteneurs etcd-m*, pg-m*, orch-m*) et un
# keyspace jetable par test. Ne touche aucun conteneur amane-*. Sorties brutes
# dans docs/tests/artifacts-ha01/.
#
# Usage : bash tools/ha/run-ha.sh
set -uo pipefail
cd "$(dirname "$0")/../.."

export PATH="$PATH:/home/jose/go-dev/go/bin"
export HA_ETCD="${HA_ETCD:-10.77.0.11:2379}"
export GOFLAGS="-mod=mod"

OUT="docs/tests/artifacts-ha01"
mkdir -p "$OUT"

echo "== HA : etcd = $HA_ETCD =="
# Vérifie que l'etcd du lab est joignable (sinon BLOQUE).
if ! docker exec etcd-m1 etcdctl --command-timeout=3s endpoint health >/dev/null 2>&1; then
  echo "BLOQUE : etcd ha-lab injoignable"
  exit 2
fi
docker exec etcd-m1 etcdctl endpoint health 2>&1 | tee "$OUT/etcd-sante.txt"

# ── HA-01 : élection de lease (test INTERNE consensus, accès champs privés) ──
echo "== HA-01 : élection de lease etcd =="
( cd orchestrator-go && AMANE_TEST_ETCD="$HA_ETCD" go test ./consensus/ \
    -run 'TestLeadershipFencingAgainstEtcd' -v -count=1 ) \
    > "$OUT/ha01-election.log" 2>&1
grep -E '^(=== RUN|--- (PASS|FAIL)|PASS|FAIL|ok)' "$OUT/ha01-election.log" || cat "$OUT/ha01-election.log"

# ── Harness externe : HA-02, HA-03, HA-05, HA-06 ────────────────────────────
echo "== HA-02/03/05/06 : harness externe =="
( cd tools/ha && go test . -v -count=1 \
    -run 'TestHA02MutexForceRight|TestHA03KeepaliveSession|TestHA05HysteresisSync|TestHA06BootstrapConcurrent' ) \
    > "$OUT/ha02-06-harness.log" 2>&1
grep -E '^(=== RUN|--- (PASS|FAIL)|PASS|FAIL|ok|    )' "$OUT/ha02-06-harness.log" | head -80

# ── HA-04 : bascule HAProxy (HAProxy réel + bascule contrôlée) ──────────────
echo "== HA-04 : bascule HAProxy =="
bash tools/ha/ha04-haproxy.sh > "$OUT/ha04-bascule.log" 2>&1
grep -E 'OK|FAIL|BLOQUE|bascule|UP =' "$OUT/ha04-bascule.log" | tail -20

echo "== Fin =="