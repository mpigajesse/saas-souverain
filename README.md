# Mission C — Amane : orchestrateur du cluster distribué

Mission C couche l'**infrastructure & la résilience** de la plateforme Amane
(SaaS souverain) sur le travail des missions A et B : consensus distribué,
HA PostgreSQL, réplication multi-site (CRDT), mesh WireGuard intra-site et
serveur gRPC exposant le framework commun.

Le code vit dans **`orchestrator-go/`** (module Go `github.com/amane/orchestrator-go`),
le contrat partagé dans **`proto/`**, les tests d'interface dans **`tests/`**.

---

## 1. Ce que fait chaque brique

```
                    ┌────────────────────────────────────────────┐
   site distant ───►│ gRPC (TLS 1.3 / mTLS)                      │◄─── autre site
   (PushDelta)      │  amane.framework.v1.AmaneService           │    (Propagator)
                    ├────────────────────────────────────────────┤
                    │ telemetry : traceparent W3C propagé        │
                    ├───────────────┬────────────────────────────┤
                    │ replication   │ supervisor                 │
                    │ Relay CRDT +  │ heartbeat lease 2 s +      │
                    │ Propagator    │ POST /failover Patroni     │
                    ├───────────────┴────────────────────────────┤
                    │ consensus : etcd (Raft) élection+fencing   │
                    ├────────────────────────────────────────────┤
                    │ etcd v3 ◄──► Patroni/Postgres (Spilo)      │
                    └────────────────────────────────────────────┘
```

### `orchestrator-go/consensus` — etcd, élection & fencing

- `client.go` / `membership.go` / `quorum.go` : connexion etcd v3, membership du cluster, calcul de quorum.
- `lease.go` — `Leadership` : campagne d'élection via une **lease etcd**. Un seul nœud devient leader à un instant donné (anti split-brain).
- **Fencing applicatif** : le chemin d'écriture `Write` est *gated* par la lease — un non-leader reçoit `codes.FailedPrecondition` avant tout accès au journal. Relectures (`Read`, `Ping`, `Enroll`, `NotifyRevocation`) non gated.

### `orchestrator-go/grpcserver` — serveur gRPC du framework

- `server.go` : implémente `amane.framework.v1.AmaneService` (Ping, Write, Read, Enroll, PushDelta…). Câblage optionnel par builder : `WithMembership`, `WithLeadership` (fencing), `WithRelay` (CRDT). Sans relay → `PushDelta` répond `codes.Unavailable`.
- `credentials.go` : **mTLS TLS 1.3 obligatoire** (`MinVersion=1.3`, TLS 1.2 rejeté). La PKI de dev vit dans `tls/` — jamais committée.
- `interceptors.go` : logs structurés JSON (`log/slog`) + recovery ; jamais d'`error` brute, toujours `status.Error(codes.X, ...)`.
- `transport.go` : `Transport` gRPC utilisé par le propagateur pour pousser les deltas vers un pair.

### `orchestrator-go/replication` — CRDT delta multi-site

Garantie **AP** : convergence même avec doublons, désordre et pertes réseau.

- `counter.go` : compteur PN-CRDT. Un delta porte le **total cumulé** de l'émetteur (Inc/Dec depuis son origine, pas l'incrément) → fusion **max par nœud émetteur** commutative/associative/idempotente.
- `relay.go` : état local + pending de deltas ; `Accept(fromNode, deltas)` fusionne (réflexion ignorée), `Outgoing()`/`Confirm(ackSeq)` font le GC du pending.
- `propagator.go` : pousse le pending vers les pairs toutes les `-replicate-interval` (défaut 1 s) via `PushDelta` ; l'ack nettoie le pending. Sur échec réseau, rien n'est perdu : nouvelle tentative au tick suivant.
- `journal.go` : journal d'événements Write/Read du nœud.

### `orchestrator-go/supervisor` — failover < 5 s (option C)

Patroni borne son TTL de lease à ≥ 20 s → un crash non contrôlé sans aide met ~21 s à basculer.
Le superviseur maison accélère la détection **sans prendre la décision de promotion** (Patroni reste l'autorité de fencing) :

1. **heartbeat propre** publié dans etcd sous une lease courte (~2 s), indépendante du TTL Patroni ;
2. probe REST Patroni toutes les 500 ms (`patroni.go`, `evaluator.go`) ;
3. au crash constaté : libération du lock `/service/<scope>/leader` (`beacon.go`) puis `POST /failover` REST — promotion ~3 s mesurées ;
4. **garde anti-partition** : si le lien REST tombe mais que le heartbeat est frais, jamais de forçage (le primary est vivant mais isolé — Patroni tranche) ;
5. chaque superviseur ne publie le heartbeat **que de son nœud local** (`AMANE_NODE_ID`) — sinon il effacerait celui du primary quand son propre lien tombe ;
6. haute disponibilité du superviseur lui-même : lock etcd « droit de forcer », une seule instance agit.

Mesuré sur la stack réelle : crash SIGKILL → failover 3,2 s / writable 4,0 s ; partition réseau → 3,0 s. Fencing + zéro perte vérifiés à chaque run (`scripts/failover_measure.sh`, `scripts/switchover_measure.sh`).

### `orchestrator-go/mesh` — WireGuard intra-site

- `mesh.go` : génération de `wg0.conf` (AllowedIPs en /32, keepalive 25 s) pour relier les nœuds d'un site.
- `runtime.go` : publication/découverte des pairs dans etcd (`cmd/wgmesh`) — clé privée en fichier 0600 local, **jamais** dans etcd.

### `orchestrator-go/telemetry` — trace distribuée W3C

- `tracecontext.go` : parse/génération du header `traceparent` (pas de dépendance OTel — un SDK OTLP se branche sans changer les points d'ancrage).
- `grpc.go` : intercepteurs client/serveur qui propagent le `traceparent` dans le metadata gRPC ; les logs `rpc` portent `trace_id`/`span_id` pour corréler un événement (PushDelta, Write, failover) d'un nœud à l'autre.

### `proto/amane/framework/v1/framework.proto` — contrat B ↔ C

Contrat partagé entre la mission B (applications) et C (infrastructure).
**Règle non négociable** : ne jamais renuméroter ni changer le type d'un champ ;
code généré commité (`buf generate`, config `buf.gen.yaml` à la racine) ;
`buf lint` + `buf breaking` vs main en CI.

### `tests/` — contrats d'interface (module isolé)

Module Go séparé (`github.com/amane/tests`) prouvant les interfaces entre missions :
contrats B↔C, interface 1 A↔C (etcd réel requis via `AMANE_TEST_ETCD` ou testcontainers), chemin d'écriture gated.

### Racine

- `docker-compose.yml` : environnement local — etcd v3.5 + Postgres HA Spilo/Patroni (`postgres-primary:8008`, `postgres-replica:8009`).
- `.ci/benchcheck.sh` + `docs/bench/baseline.json` : garde-fou anti-régression perf (médiane ns/op vs baseline).

---

## 2. Démarrage rapide

```bash
# environnement local (etcd + Postgres/Patroni)
docker compose -f docker-compose.yml up -d

# build & tests
cd orchestrator-go && go build ./... && go test ./... -count=1

# tests d'interface (etcd requis)
cd tests && AMANE_TEST_ETCD=localhost:2379 go test ./... -count=1 -race

# régénérer le code gRPC après modification du proto
buf generate

# serveur avec mTLS (PKI de dev uniquement)
go build -o /tmp/orch ./cmd/orchestrator && AMANE_NODE_ID=orch /tmp/orch \
  -tls-cert tls/server.crt -tls-key tls/server.key -tls-ca tls/ca.crt

# serveur complet : superviseur failover + réplication multi-site
/tmp/orch -supervisor -patroni-scope amane \
  -patroni-nodes postgres-primary@http://localhost:8008,postgres-replica@http://localhost:8009 \
  -replicate-to site-atelier@127.0.0.1:50052 -replicate-interval 1s
```

Mesures HA : `bash scripts/switchover_measure.sh` (~2,2 s),
`bash scripts/failover_measure.sh crash|partition` (< 5 s superviseur actif).

---

## 3. CI (GitHub Actions)

| Workflow | Rôle |
|---|---|
| `proto-contract.yml` | `buf lint` + `buf breaking` vs main + test du contrat |
| `build-go.yml` | build, vet, tests unitaires + intégration (etcd réel), couverture ≥ 70 % agrégée et ≥ 60 %/paquet |
| `integration-tests.yml` | contrats B↔C / A↔C / write-gated vs etcd réel (testcontainers, module `.ci/testcontainers`) |
| `chaos-tests.yml` | panne : failover crash, garde anti-partition, fencing, convergence CRDT |
| `benchmark.yml` | micro-benchmarks vs baseline + macro failover réel vs SLO < 5 s |
| `security-scan.yml` | `govulncheck` + `gosec` (high bloquante) |

---

## 4. Contenu local uniquement (non versionné)

Volontairement exclu du dépôt via `.gitignore` :

- `docs/` sauf `docs/bench/baseline.json` (requis par la CI benchmark) :
  scripts de génération des documents (ReportLab/Graphviz, `docs/generate_*.py`),
  notes détaillées et PDF générés — tout reste en local ;
- `*.pdf` : aucun PDF n'est poussé (documents générés comme notes) ;
- `.opencode/` : agents IA et skills opencode utilisés pendant la mission ;
- `AGENTS.md` : instructions destinées aux agents IA (contexte, commandes, règles) ;
- `tls/` : PKI de développement (règle absolue : jamais de clé en clair dans le dépôt).

Pour retrouver la logique complète côté local : `docs/MISSION_C_FONCTIONNEMENT.md`,
`docs/TESTS_CI.md` et les scripts `docs/generate_mission_c_*.py` (PDF architecture /
fonctionnement / tests / logique).
