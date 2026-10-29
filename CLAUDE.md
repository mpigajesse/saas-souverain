# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

**AMANE — Framework SaaS Souverain.** Monorepo polyglotte (Django + React, Go, Rust, Protobuf).
Les données métier restent sur les machines du client (cluster PME) et ne sortent que chiffrées ; l'éditeur gère comptes et licences sans jamais lire les données métier.
**V3 (branche `main3`)** : AMANE est une enveloppe d'exécution pour SaaS externes — l'image Docker d'un éditeur tiers s'exécute, inchangée, dans le cluster de la PME, confinée sans sortie réseau. Le journal `docs/refonte-v3/AMANE-Refonte-V3.md` (tri du dépôt, ADR-001 à ADR-009, plan, points ouverts) fait foi : le mettre à jour à chaque étape et régénérer le PDF avec `python docs/refonte-v3/generer_pdf.py`.
Documentation détaillée : `README.md`, `ACCES-INTERFACES.md` (toutes les URL et ports, éditeur et PME), `CONTRAT-API.md`, `Architecture_SaaS_Souverain_Trois_Acteurs.md`, `AGENTS_IA_EXPLOITATION.md`, `ha-lab/README.md`.

La langue du dépôt est le **français** : commentaires, messages de log, documentation, commits. S'y conformer.

---

## Carte du dépôt — qui fait quoi

| Composant | Chemin | Rôle |
| --- | --- | --- |
| **SaaS éditeur (Django)** | `config/`, `tenants/`, `licenses/`, `devices/`, `applications/`, `dashboard/`, `api/` | API REST seule (plus `/admin/`) : tenants, licences, catalogue des applications hébergées (`applications/`), parc installé, annonce de nœud (`devices/api_register.py`), génération de la pile (`tenants/services.py`, `tenants/pile.py`, `tenants/manifeste.py`) |
| **Pile PME générée** | `templates/tenants/` | Gabarits compose serveur et témoin, installeurs Linux et Windows |
| **Console web (React/TS)** | `frontend/` | Vite + React 19 + react-router, consomme l'API DRF |
| **Orchestrateur cluster (Go)** | `orchestrator-go/` | Sous-commandes `init` (amorçage etcd, secrets), `temoin`, `lancer` (lanceur de l'application) ; superviseur, HAProxy, mode synchrone, provision de la base de l'application ; consensus etcd, réplication CRDT, mesh WireGuard, gRPC mTLS |
| **Socle crypto & journal (Rust)** | `spike/crates/ss-crypto`, `spike/crates/ss-journal`, `spike/crates/ss-consensus` | AK/DEK, CRL, Shamir, journal CBOR append-only, époque/fencing |
| **SDK client métier (Rust)** | `core-rust/ss-client/` | Invariants métier, mode hors-ligne SQLite, appairage, licence, client gRPC généré |
| **Nœud & relais PME (Rust)** | `spike/node/`, `spike/relay/` | Nœud `ss-node` (reverse proxy de l'application, console sous `/amane/`, enrôlement, sauvegardes chiffrées, garde de quorum) et relais zero-knowledge (axum **0.7**) |
| **Contrat d'autorité** | `proto/amane/framework/v1/framework.proto` | Interface gRPC unique B ↔ C, source des bindings Go et Rust |
| **Déploiement** | `docker-compose.yml` (dev complet), `relay-deploy/`, `scripts/publier-pile.sh` | Stack de développement, relais, publication des images de la pile par digest |
| **Laboratoire HA** | `ha-lab/` | Trois machines simulées, scénarios de chaos (`chaos.sh`), application d'exemple |
| **CI & mesures** | `.github/workflows/`, `.ci/`, `scripts/`, `tests/` | Build/couverture, chaos, contrats, benchmarks, mesures failover |

Les trois « missions » citées partout dans le code : **A** = crypto/souveraineté (`spike/crates/`), **B** = client & applicatif (`core-rust/`, Django/React), **C** = orchestration/HA (`orchestrator-go/`).

---

## Commandes

### Stack locale

```bash
docker compose up -d           # etcd + 2 Patroni/spilo + backend Django + frontend + Prometheus/Grafana
docker compose ps
```

Backend `:8000` · frontend `:5173` · Patroni REST `:8008`/`:8009` · PG `:5432`/`:5433` · etcd `:2379` · Grafana `:3000`.

### SaaS éditeur (Django)

```bash
python manage.py migrate
python manage.py runserver
python manage.py test                                   # toutes les apps
python manage.py test devices.tests.NomDuTest.test_x    # un seul test
```

Sans PostgreSQL disponible, la suite passe aussi sur SQLite avec un module de réglages qui importe `config.settings` et remplace `DATABASES` (`DJANGO_SETTINGS_MODULE=<module>`).

`config/settings.py` lit tout via `python-decouple` : un `.env` avec au minimum `SECRET_KEY`, `DB_NAME`, `DB_USER`, `DB_PASSWORD` est **obligatoire**, sinon Django refuse de démarrer.

### Frontend

```bash
cd frontend && npm install
npm run dev        # Vite
npm run build      # tsc -b && vite build
npm run lint       # eslint
```

### Orchestrateur Go

```bash
cd orchestrator-go
go build ./... && go vet ./...
go test ./... -count=1 -race
AMANE_TEST_ETCD=localhost:2379 go test ./consensus/... -v            # tests d'intégration (sinon skippés)
go test ./supervisor/ -run TestSupervisorAgainstEtcd -v -count=1     # un seul test
go run ./cmd/orchestrator -etcd-endpoints localhost:2379 -supervisor
```

Les tests d'intégration (`consensus`, `supervisor`, `mesh`) ne s'exécutent que si `AMANE_TEST_ETCD` pointe sur un etcd réel. Sans etcd déjà démarré, le harnais `.ci/testcontainers` en lance un et fixe la variable :

```bash
cd .ci/testcontainers && go run . -workdir ../../orchestrator-go -- go test ./supervisor/ -count=1
```

Les tests de contrats vivent dans le module **isolé** `tests/` (`replace` vers `orchestrator-go`) : `cd tests && go test ./contracts/ -v`.

### Rust

Deux workspaces distincts : `spike/` (socle A + nœud + relais) et `core-rust/` (SDK client B, qui dépend des crates du spike par chemin).

```bash
cd spike     && cargo test -p ss-crypto -p ss-journal -p ss-consensus
cd core-rust && cargo test -p ss-client
cargo test -p ss-crypto shamir      # filtre par nom de test
```

`core-rust/ss-client/build.rs` régénère les bindings tonic à partir du proto (protoc fourni par `protoc-bin-vendored`, aucune installation système requise).

### Contrat gRPC

Après toute modification de `proto/amane/framework/v1/framework.proto` :

```bash
buf generate                       # → orchestrator-go/gen/amane/framework/v1/
cd core-rust/ss-client && cargo build
cd tests && go test ./contracts/ -run TestProtoContract -v
```

### Laboratoire HA (lourd : trois machines simulées)

```bash
bash ha-lab/chaos.sh            # les deux topologies (2 PG + témoin, 3 PG)
bash ha-lab/chaos.sh temoin     # une seule
```

### Mesures HA (nécessitent la stack compose démarrée)

```bash
bash scripts/switchover_measure.sh          # switchover contrôlé (~2,2 s)
bash scripts/failover_measure.sh crash      # SIGKILL → bascule (~3,2 s)
bash scripts/failover_measure.sh partition
bash .ci/benchcheck.sh                      # micro-bench replication vs docs/bench/baseline.json
```

### Images Docker PME

```bash
bash scripts/publier-pile.sh                    # pousse les images de la pile et écrit media/node-images/pile.json (digests)
```

```powershell
powershell -ExecutionPolicy Bypass -File spike\build-node.ps1 [-NoCache] [-SkipPush]
.\relay-deploy\build-relay.ps1 [-NoCache]
```

Le contexte de build est la **racine du workspace `spike/`** (obligatoire pour le workspace Cargo), avec `-f node/Dockerfile`. L'image du nœud embarque le client PostgreSQL 17 (dépôt PGDG) pour les sauvegardes : il doit rester de la version de Spilo.

---

## Architecture — ce qu'il faut avoir en tête avant de coder

### Trois acteurs, une frontière de confiance

SaaS éditeur (voit comptes, licences, manifestes) · Relais zero-knowledge (ne voit **jamais** de clair, stocke des sauvegardes chiffrées) · Éditeur du SaaS hébergé (ne voit **jamais** de clair : son application tourne sur un réseau Docker `internal`, sans sortie) · Cluster PME (seul périmètre en clair). Le relais tient la topologie du cluster (metadata), pas les données. Découverte réseau = annonce HTTP au relais au démarrage (pas de mDNS — Docker le masque).

### L'application hébergée (ADR-007, ADR-009)

Décrite par un **manifeste** validé (`tenants/manifeste.py`), créée et validée dans le catalogue `applications/` (image présente dans le registre, manifeste alors figé), rattachée à `License.application`. La pile la lance en service `amane-application` ; le lanceur Go (`orchestrator lancer`) injecte les identifiants de sa base (rôle `application`, joint par HAProxy) sans modifier l'image ; `ss-node` relaie son interface à la racine et sert sa propre console sous `/amane/`.

### Les trois interfaces

1. **A ↔ C** : `NotifyRevocation` transmet la clé publique révoquée à l'orchestrateur → fermeture des sessions gRPC + recalcul du quorum en < 1 s, **sans rechiffrer le disque** (tout l'intérêt de la CRL).
2. **A ↔ B** : `Dek::encrypt/decrypt`, `Kek`/Shamir et le journal CBOR append-only. Aucun chemin d'écriture disque ne contourne le journal.
3. **B ↔ C** : `framework.proto` est le **seul** contrat. Les `Write` du SDK Rust sont *gated* par la lease de leader etcd côté Go (fencing anti-split-brain) : un ancien primaire revenu ne peut plus écrire.

### Hiérarchie de clés

```
AK (X25519, identité réseau/accès)   ≠   DEK (symétrique, chiffrement disque)
DEK ── sealed box par serveur enrôlé (code à usage unique, console /amane/admin/enrolement + ss-node enroler)
    └─ emballée sous KEK ── Shamir SSS K=2/N=3 (socle disponible, pas encore branché dans la pile Docker)
```

Seul le premier serveur génère la DEK ; elle chiffre le journal du nœud et les sauvegardes déposées sur le relais (`spike/node/src/sauvegarde.rs`). Le chiffrement au repos est délégué au disque (BitLocker, LUKS), vérifié par l'installeur.

Séparation AK/DEK stricte, `ZeroizeOnDrop` sur les secrets, `Debug` masqué `[REDACTED]`. Aucun composant Go, Django ou agent IA ne touche AK/DEK.

### Haute disponibilité

etcd v3 (Raft) pour membership/lease/quorum · Patroni (PostgreSQL 17, Spilo) promeut le standby · HAProxy mène au primaire · superviseur Go avec heartbeat 2 s. Bascule **automatique** avec un quorum de trois membres etcd : 2 serveurs + **témoin** (etcd seul, sans données, sans poste de licence) ou ≥ 3 serveurs ; sans témoin, 2 serveurs n'ont pas de quorum. Synchrone **strict** dès 3 serveurs PostgreSQL, non strict en dessous avec alerte « sous le seuil » au portail (ADR-004, ADR-006). Le nœud Rust ne bascule rien : il lit Patroni, et le fencing se fait par la timeline PostgreSQL. Réplication : **synchrone** pour les invariants forts (stock, facturation, numérotation) — elle **bloque** si aucun passif n'est joignable, jamais de dégradation silencieuse ; delta-CRDT (compteur PN, fusion max commutative/idempotente) pour le multi-site tolérant.

### Identité d'installation

La licence est liée à un **UUID d'installation authentifié** (`installation_id`), jamais à la MAC (falsifiable et masquée par Docker) — voir `devices/api_register.py`.

---

## Règles non négociables

1. **Aucune primitive crypto réimplémentée à la main.** Tout passe par le socle (`ss-crypto`) ; la référence est libsodium/RustCrypto, jamais du code maison.
2. **Aucun algorithme de consensus écrit à la main.** etcd (Raft) + Patroni, point.
3. **Le proto est un contrat figé** : ne jamais renuméroter un champ, changer son type ou renommer un RPC — cela casse silencieusement les SDK déployés. `buf breaking` et `TestProtoContractStable` (`tests/contracts/`) le verrouillent en CI ; tout message ajouté doit être répercuté dans ce test, sinon le contrat n'est plus audité.
4. **Le relais ne voit jamais de clair.** Toute donnée qui le traverse est déjà chiffrée côté PME.
5. **LLM : proposer, jamais décider.** Un agent IA ne déclenche aucune action critique d'infra (failover, promotion, rotation de clé) ; le code Go déterministe valide via liste blanche fermée et journalise dans `ss-journal`. Les opérations sur les clés restent **humaines**. Voir `AGENTS_IA_EXPLOITATION.md`.
6. **Le socle avant le métier.** La logique métier s'appuie sur crypto + journal + quorum, elle ne les contourne pas. (L'interdiction Phase 0 d'écrire du métier est levée : les invariants stock/facturation existent dans `core-rust/ss-client/src/invariants.rs` et `spike/node/src/stock/`.)

---

## Pièges connus

- **`pme-deploy/` n'existe plus** (pile V2) : la pile PME est générée par le portail depuis `templates/tenants/`. Les pages HTML Django aussi ont été retirées : seule la console React sert d'interface.
- **Relais en axum 0.7** : les paramètres de route s'écrivent `:nom`, pas `{nom}` (syntaxe 0.8, prise au pied de la lettre en 0.7 — les routes de blobs n'ont jamais répondu jusqu'à l'étape 4).
- **Un seul fichier proto** : `proto/amane/framework/v1/framework.proto` fait autorité (généré, testé, utilisé par Go et Rust). Le vestige V1 `proto/framework.proto` a été supprimé en V3 ; vérifier `buf lint` avant d'ajouter des fichiers sous `proto/`.
- **Workspaces Rust croisés** : `core-rust/ss-client` dépend de `spike/crates/*` par chemin relatif. Déplacer l'un casse l'autre ; un `cargo test` à la racine de `core-rust` ne couvre pas le socle A.
- **Seuils CI bloquants** : couverture Go agrégée ≥ 70 % **et** ≥ 60 % par paquet produit (`gen/` exclu) ; SLO failover et writable lus dans `docs/bench/baseline.json`. `gosec` et `govulncheck` bloquent en sévérité *high*.
- **Réseau Docker** : `--network host` côté Linux ; sur Windows Docker Desktop, bridge nommé à IP fixes (cf. `spike/docker-compose.yml`, sous-réseau `172.20.0.0/24`).
- **Le nœud PME est distribué en image Docker** ; si Docker est absent au premier lancement, le logiciel doit le détecter (`docker info`) et guider l'installation.
- Les scripts `.ps1` référencent un registre local (`192.168.200.1:5000`, `localhost:5000`) et des chemins de banc de test — les adapter avant usage hors banc.

---

## Commits

Préfixes utilisés : `feat` · `fix` · `refactor` · `docs` · `test` · `chore` · `perf` · `ci`, message en français. Branches : V3 `main3`, V2 `main2`.
