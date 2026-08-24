# AGENTS.md — Mission C (orchestrator-go)

Instructions pour les agents IA travaillant sur Mission C d'Amane :
infrastructure & résilience du cluster distribué (consensus, réplication, mesh, serveur gRPC).

## Structure du repo

```
amane/
├── proto/                # Contrat partagé proto/framework.proto (B ↔ C) — buf
├── orchestrator-go/      # Code Go (module github.com/amane/orchestrator-go)
│   ├── consensus/        # etcd (Raft) + Patroni + fencing
│   ├── replication/      # CRDT delta multi-site (relay.go : PushDelta total-cumulé/max-merge) + journal Write/Read
│   ├── mesh/             # Config WireGuard intra-site (AllowedIPs /32, keepalive 25)
│   └── grpcserver/       # Serveur gRPC exposant le framework
├── infra/                # (la config docker-compose.yml vit à la racine)
├── scripts/              # Mesures HA : failover_measure.sh, switchover_measure.sh
├── tests/                # Module Go isolé github.com/amane/tests : contrats B↔C / Interface 1 A↔C (etcd réel requis)
├── docs/                 # Doc PDF (ReportLab) : generate_mission_c_pdf.py + mission_c_architecture.pdf
│                         # + MISSION_C_FONCTIONNEMENT.md (logique & schémas détaillés)
├── docker-compose.yml    # etcd + Postgres (Spilo/Patroni)
└── .opencode/
    ├── skills/           # Skills mission-c-* (contexte déclenché par description)
    └── agent/            # Agents subagents (docs-keeper, code-reviewer, cross-mission-checker)
```

## Commandes de développement

```bash
# Tests & build Go
cd orchestrator-go && go build ./...
cd orchestrator-go && go test ./... -count=1

# Environnement local (etcd + Patroni/Postgres)
docker compose -f docker-compose.yml up -d
docker compose ps
docker compose down -v   # wipe volumes (données de dev)

# Génération gRPC depuis proto/
buf generate              # (config buf.gen.yaml à la racine)

# TLS 1.3 / mTLS (dev)
go build -o /tmp/orch ./cmd/orchestrator && AMANE_NODE_ID=orch /tmp/orch \
  -tls-cert tls/server.crt -tls-key tls/server.key -tls-ca tls/ca.crt
~/go/bin/grpcurl -cacert tls/ca.crt -cert tls/client.crt -key tls/client.key \
  -d '{}' 127.0.0.1:50051 amane.framework.v1.AmaneService/Ping
openssl s_client -connect 127.0.0.1:50051 -tls1_2   # doit échouer (MinVersion=1.3)

# Inspection cluster Patroni
curl -s localhost:8008/patroni      # primary/replica (ports 8008/8009)

# Consensus — élection + fencing lease-based (Write gated)
AMANE_TEST_ETCD=localhost:2379 go test ./consensus/ -run TestLeadershipFencing -v   # élection + fencing lease-based vs etcd réel

# Réplication multi-site — relay delta-CRDT (jalon 4)
go test ./replication/ -count=1 -race   # convergence 3 nœuds réordonnés/doublés, ack gc pending, réflexion ignorée, trous de seq
go test ./grpcserver/ -run 'TestPushDelta' -v   # PushDelta entre 2 sites (bufconn) + sans relay
go test ./grpcserver/ -run 'TestPropagatorE2E' -v   # propagateur périodique : 2 serveurs gRPC TCP réels, convergence + pending vidé

# Observabilité — trace distribuée W3C (forward path sans dépendance OTel)
go test ./telemetry/ -v   # parse/round-trip traceparent, génération racine, propagation serveur↔client
go build -o /tmp/orch ./cmd/orchestrator   # relancer après tout changement (serveur + télémetry actifs)

# Mesh WireGuard intra-site — runtime (publication etcd + découverte + wg0.conf)
go build -o /tmp/wgmesh ./cmd/wgmesh && /tmp/wgmesh -etcd localhost:2379 -name node-1 -site A -index 1 \
  -pubkey <clé publique> -privkey-file /path/to/wg_private.key -endpoint 192.168.1.20:51820 \
  -conf /tmp/wg0-node1.conf -interval 1s   # clé privée 0600 locale, jamais dans etcd ; appliquer via wg-quick

# Mesures HA (jalon 4) — fencing + zéro perte vérifiés à chaque run
bash scripts/switchover_measure.sh          # contrôlé : < 5 s mesuré (~2,2 s)
bash scripts/failover_measure.sh crash      # SIGKILL : ~3,2 s superviseur actif (sans superviseur ~21 s plancher ttl>=20)
bash scripts/failover_measure.sh partition  # réseau (coupure complète) : ~3,0 s superviseur actif (sans superviseur ~21 s plancher ttl>=20)

# Superviseur de failover (option C) — crash < 5 s
AMANE_NODE_ID=orch /tmp/orch -supervisor -patroni-scope amane \
  -patroni-nodes postgres-primary@http://localhost:8008,postgres-replica@http://localhost:8009
AMANE_TEST_ETCD=localhost:2379 go test ./supervisor/ -run SupervisorAgainstEtcd -v   # intégration (faux Patroni + vrai etcd), failover forcé < 5 s
AMANE_TEST_ETCD=localhost:2379 go test ./supervisor/ -run PartitionGuardAgainstEtcd -v   # garde anti-partition : REST down + heartbeat frais → JAMAIS de forçage
go test ./supervisor/ -run TestDetector -v   # unitaires : crash, partition, candidat, re-arm

# Tests cross-mission (module isolé tests/) — contrats B↔C / A↔C / chemin d'écriture gated
cd tests && AMANE_TEST_ETCD=localhost:2379 go test ./... -count=1 -race   # contrats B↔C / A↔C / chemin d'écriture (etc-cluster requis)

# Doc PDF (ReportLab) — régénérer après tout changement proto/compose/code
.venv/bin/python docs/generate_mission_c_pdf.py   # sortie : docs/mission_c_architecture.pdf
# Schémas Graphviz (docs/diagrams/*.dot → PNG) puis markdown → PDF :
for f in docs/diagrams/*.dot; do dot -Tpng "$f" -o "${f%.dot}.png"; done
.venv/bin/python docs/diagrams/gen_seq.py   # diagrammes de séquence (lifelines rank=same)
.venv/bin/python docs/generate_mission_c_fonctionnement_pdf.py   # sortie : docs/mission_c_fonctionnement.pdf (markdown + PNG → PDF)
.venv/bin/python docs/generate_mission_c_tests_pdf.py             # sortie : docs/mission_c_tests.pdf (tests, logique CI, résultats)
.venv/bin/python docs/generate_mission_c_logique_pdf.py           # sortie : docs/mission_c_logique.pdf (logique globale, schémas vectoriels ReportLab)
```

Les images des skills mission-c-* listent les commandes détaillées par domaine.

## CI — GitHub Actions (5 jobs + proto-contract)

| Workflow | Quand | Ce qu'il fait |
|---|---|---|
| `proto-contract.yml` | push main + PR | buf lint + breaking vs main + `TestProtoContract` |
| `build-go.yml` | push main + PR | build + vet + tests unitaires + **couverture ≥ 70 %** (plancher 60 %/paquet, etcd réel en amont, `gen/` exclu) |
| `integration-tests.yml` | push main + PR | contrats B↔C / A↔C / write gated vs etcd réel via **testcontainers** (harness `.ci/testcontainers`) |
| `chaos-tests.yml` | PR touchant `consensus/`, `replication/`, `supervisor/` | panne : failover crash, garde anti-partition, fencing, convergence CRDT (etcd réel) |
| `benchmark.yml` | PR chemins chauds + **cron 02:15** + dispatch | micro-benchmarks vs `docs/bench/baseline.json` (`.ci/benchcheck.sh`) ; macro failover réel vs SLO V2 (< 5 s) |
| `security-scan.yml` | push main + PR + **cron 02:30** | `govulncheck` (épinglé v1.7.0) + `gosec` (épinglé v2.28.0, high bloquante) |

Points de vigilance : seuil agrégé `MIN_COVERAGE` (70 %) + **plancher par paquet** 60 %
(anti-gaming : un paquet à 40 % n'est pas compensé par un autre à 90 %). Mesuré avec etcd
réel (intégration comptée, `gen/` exclu) : consensus 73,8 %, telemetry 93,2 %, mesh 83,2 %,
supervisor 80,2 %, grpcserver 85,3 %, replication 95,7 % (**agrégat 72 %**). **Plan de remontée
70 → 75 → 80 %** : grosse dette restante consentir via des PR de tests (consensus et telemetry
déjà rattrapés respectivement 73,8 et 93,2 ; prochain palier atteignable en couvrant les derniers
chemins lease/membership). Le harness `.ci/testcontainers` est un module Go isolé
(`github.com/amane/ci/testcontainers`) qui démarre etcd et exécute la commande passée
après `--` avec `AMANE_TEST_ETCD` pointé dessus.

## Règles transverses (non négociables)

1. **Jamais de clé en clair** : AK privée, DEK, KEK ne doivent jamais apparaître en Go,
   dans les logs, les vars d'env ou les messages (voir skills mission-c-interface-a-c, mission-c-observability).
2. **Réplication synchrone** : `synchronous_commit=on`, `max_slot_wal_keep_size` borné,
   mode synchrone Patroni actif (failover < 5s, zéro perte).
3. **Logs structurés** avec `log/slog` (JSON), corrélation de timing pour failover/lease.
4. **gRPC** : toujours `status.Error(codes.X, ...)` — jamais une `error` brute, ni `nil, nil`.
5. **Contrat proto** : ne jamais renuméroter/changer type d'un champ ; commiter le code généré ;
   `buf breaking` en CI.
6. **PKI `tls/` = développement uniquement** : clés 0600, ne jamais la commiter ni la réutiliser
   en prod ; les tests `go test ./grpcserver/ -run TestCredentials` prouvent mTLS + TLS 1.3.
   **Prod (hors scope POC, à planifier)** : PKI interne dédiée (AC racine hors ligne + ACs
   intermédiaires par site, révocation OCSP/CRL, rotation annuelle), jamais la CA dev ; validation
   **DNSSEC sortante** au résolveur (systemd-resolved/stubby/dnsmasq) pour les connexions sortantes
   TLS vers les services cloud — cf. skill mission-c-tls-security.
7. **Trace distribuée (observabilité)** : chaque nœud propage un `traceparent` W3C (package
   `telemetry/`) à travers le metadata gRPC ; les logs `rpc` portent `trace_id`/`span_id` pour
   corréler un même événement (PushDelta, Write, failover) d'un nœud à l'autre. Pas de dépendance
   OTel : le format est standard, un SDK OTLP se branche sans changer les points d'ancrage.

## Décisions d'architecture (JA)

- **HA / failover < 5 s (jalon 4)** : la cible < 5 s est tenue sur le chemin **contrôlé**
  (switchover planifié ≈ 2,2 s, mesuré) et sur le **crash non contrôlé** quand le superviseur
  est actif (3,2 s mesurées, voir ci-dessous). Sans superviseur, le crash reste borné par le
  plancher de lease Patroni `ttl >= 20 s` (validation `patroni/config.py`, indépendante du DCS —
  ZooKeeper/k8s ne le lèvent pas). **Choix retenu (v1) : chemin contrôlé + superviseur (option C)**.
- **Crash < 5 s via option C retenue — Superviseur maison (Go, `orchestrator-go/supervisor`) :
  décidé.** Un superviseur déployé sur les 3 nœuds détecte le crash vite et déclenche
  `POST /failover` REST Patroni. Pourquoi :
  1. Mono-repo Go, aucune nouvelle infrastructure (pas de K8s, pas de Pacemaker) ;
  2. etcd déjà présent — heartbeat **propre** du superviseur avec lease courte (~2-3 s),
     **indépendante** du plancher `ttl >= 20 s` de Patroni ;
  3. pas de fork de Patroni : `/failover` REST promeut sans attendre l'expiration de la lease
     (chemin déjà éprouvé à ~2,2 s lors de la mesure de switchover) ;
  4. le superviseur **ne décide pas la promotion** : Patroni reste l'autorité de fencing et de
     promotion (réplique synchrone et zéro perte préservées) — le superviseur n'est pas une
     couche HA superposée, juste un déclencheur rapide ;
  5. HA du superviseur lui-même : élection via un lock etcd « droit de forcer » (une seule
     instance agit), jamais de SPOF.
  **Gardes-fous** : jamais de forçage sur partition réseau (primary vivant mais isolé) — Patroni
  tranche alors ; baisser `ttl` (option B) reste écarté (affaiblit l'anti split-brain).
  **Mesuré (stack réelle) — crash SIGKILL, superviseur actif : failover 3,2 s, writable 4,0 s,
  détection 0,8 s, fencing + zéro perte OK — cible < 5 s ATTEINTE sur le chemin crash (avant : ~21 s) ;
  et la partition (coupure complète) : failover 3,0 s, writable 3,4 s, détection 0,7 s, fencing +
  zéro perte OK. La garde anti-partition n'empêche pas le forçage en coupure totale (primary
  injoignable + heartbeat stale).**
  Mécanisme final : heartbeat propre lease 2 s + probe 500 ms → libération du lock
  `/service/<scope>/leader` (fencing conditionné au heartbeat stale) → `POST /failover` sans le
  champ `leader` ; `force=true` seul insuffisant (sans libération du lock, la promotion reste
  bornée par l'expiration de la lease `ttl >= 20 s`).
  **Agent heartbeat local (`LocalNode`)** : chaque superviseur ne publie/supprime le heartbeat
  QUE de son **nœud local** (`AMANE_NODE_ID` dans `main.go`), jamais de ses pairs —
  sinon il supprimerait le heartbeat du primary quand SON lien REST tombe et la garde
  anti-partition serait inopérante en déploiement 3 nœuds. `LocalNode` vide = mode mono-hôte
  (tests). Prouvé en intégration etcd réel : `TestSupervisorPartitionGuardAgainstEtcd` (REST down
  + heartbeat frais → JAMAIS de forçage, lock Patroni intact) et `TestSupervisorAgainstEtcd`
  (#crash# → /failover < 5 s).
- **Réplication multi-site — Relay delta-CRDT (`PushDelta`, jalon 4)** : un delta porte le
  **total cumulé** du nœud émetteur (Inc/Dec depuis son origine, pas un incrément) → la fusion
  **max par nœud émetteur** est commutative/associative/idempotente : doublons, réordonnancements
  et trous de séquence ne compromettent jamais la convergence (garantie AP).
  `Add(inc, dec)` met à jour total + pending + seq ; `Accept(fromNode, deltas)` fusionne côté
  récepteur (réflexion d'un delta auto-émis ignorée) ; `Outgoing()`/`Confirm(ackSeq)` font le
  gc du pending (fuite de mémoire évitée). RPC **additif** `PushDelta` (numéros de champs
  stables, code généré committé, `buf generate` OK) ; handler `grpcserver` via
  `WithRelay(*replication.Relay)` — sans relay → `codes.Unavailable` ; câblé au démarrage dans
  `cmd/orchestrator/main.go`. Preuve live : site-eshop inc 5 puis 7 → ack seq 2, value 7 ;
  site-atelier dec 3 → value 4.
- **Propagation périodique multi-site (`-replicate-to site-id@host:port`, jalon 4)** : un
  `Propagator` (orchestrator-go/replication/propagator.go) tire le pending du Relay toutes les
  `-replicate-interval` (défaut 1 s) et pousse via `PushDelta` un `Transport` gRPC
  (`grpcserver/transport.go`) ; l'ack (dernière seq appliquée par le pair) est accusé au Relay
  (`Confirm`) → le pending est nettoyé. Sur échec réseau, les deltas ne sont jamais perdus :
  le propagateur retente au tick suivant jusqu'à ack. Prouvé par
  `TestPropagatorPushesAndConfirms`, `TestPropagatorRetriesOnErrorWithoutLosingDeltas`
  (fake pair ack = dernière seq reçue — jamais 0) et l'e2e `TestPropagatorE2EOverTCP`
  (2 serveurs gRPC réels, base 4 + deltas, convergence + pending vidé).
- **Le chemin d'écriture `Write` est gated par la lease etcd (fencing applicatif)** : seul le
  leader du cluster accepte d'écrire — un non-leader reçoit `codes.FailedPrecondition`
  (« write refusé : nœud non leader (fencing lease) ») avant tout accès au journal (aucun
  compromis de séquence). Câblé en prod via `WithLeadership(consensus.NewLeadership(...))`
  (`cmd/orchestrator/main.go`) ; sans `WithLeadership` le gating est désactivé (tests /
  rétro-compat), jamais en prod. Relectures (Read, Ping, Enroll, NotifyRevocation) non gated ;
  l'élection etcd ne produit jamais deux leaders en même temps (anti split-brain).

## Appartenance des rôles

- **docs-keeper** : mises à jour de ce fichier, des skills et de la doc PDF (ReportLab)
  quand le code change.
- **code-reviewer** : revue stricte du code Go (consensus, réplication, sécurité).
- **cross-mission-checker** : validation des 3 contrats d'interface par les tests `tests/`.

Sous-agents utiles existants (ne pas réimplémenter) : `explore` (recherche code), `plan` (plan mode).