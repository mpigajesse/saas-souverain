# A-01 — Baseline des tests existants (Go, contrats, HA)

Date : 2026-11-25. Dépôt : `/home/jose/school/amane/repo`, branche `main3`, HEAD `1021eae`.
Objectif : établir l'état **réel** de la suite de tests, sans modifier le code ni toucher au lab.

## Périmètre et méthode

- Fichiers lus avant action : `docs/STATE.md` (présent, 3 lignes, indique un précédent échec A-01) ; `AGENTS.md` **absent** du dépôt (seul `AGENTS_IA_EXPLOITATION.md` existe) ; `pilot.yaml` présent mais **non suivi** (`?? pilot.yaml`).
- Plans lus hors dépôt (lecture seule) : `/home/jose/school/amane/plan-tests-amane-v3.md` (section « État des Lieux des Tests Existants », catalogue C/N/R/HA/D/E, matrice, priorisation), `/home/jose/school/amane/plan-tests-charge-v3.md`, `/home/jose/school/amane/plan-tests-exhaustif-v3.md`.
- Toolchains observées : `go version go1.27.3 linux/amd64` à `/home/jose/go-dev/go/bin/go` (hors PATH) ; `cargo 1.99.1` / `rustc` présents ; `python3` 3.14.8 avec Django 6.0.6 importable, mais **aucun environnement virtuel** (`VIRTUAL_ENV` vide, aucun `.venv/`, `venv/`, `env/`, `backend/.venv/`, `backend/venv/`).
- Aucune installation, aucune modification de code, aucune commande Docker. Les conteneurs du lab n'ont pas été touchés. Variables `AMANE_TEST_ETCD` / `AMANE_TEST_PG` non définies : les tests d'intégration Go se skippent par garde.
- Sorties brutes et méta conservées dans `docs/tests/artifacts-a01/`.

## Tableau récapitulatif

| Suite | Commande exacte demandée | Exécution réelle | Résultat | Tests | Durée mesurée | Artefact |
|-------|--------------------------|------------------|----------|-------|---------------|----------|
| Go orchestrateur | `cd orchestrator-go && go test ./... -count=1` | Oui, avec `PATH=/home/jose/go-dev/go/bin:$PATH` préfixé (Go hors PATH) | **PASS**, exit 0 | 141 `RUN` / 129 `PASS` / 12 `SKIP` / 0 `FAIL` ; 14 paquets `ok`, 2 `[no test files]` | 7,95 s | `artifacts-a01/orchestrator-go.log`, `orchestrator-go-verbose.log` |
| Contrats Go | `cd tests && go test ./... -count=1` | Oui, avec PATH Go préfixé | **FAIL**, exit 1 avant compilation | 0 exécuté ; 4 fonctions de test présentes dans le module | 0,05 s | `artifacts-a01/tests-contracts.log` |
| Rust `core-rust` | `cd core-rust && cargo test --workspace` | Oui (toolchain présente) | **PASS**, exit 0 | 7 passed, 0 failed, 0 ignored ; 0 doctest | 0,56 s (cache chaud) | `artifacts-a01/core-rust.log` |
| Django | `python3 manage.py test` | **Non exécuté** — condition « si un environnement virtuel est disponible » non remplie | Non vérifié | 0 exécuté | — | — |
| HA lab chaos | Aucune commande demandée ; inventaire statique seul | **Non exécuté** (scripts destructifs, lab en service) | Non vérifié | 0 exécuté | — | — |

> Note : la commande Go littérale échouerait sans le préfixe PATH car `go` n'est pas dans l'environnement ; le préfixe n'a modifié aucun fichier du dépôt.

## Détail par suite

### 1. `orchestrator-go` — `go test ./... -count=1`

- **Commande** : `cd orchestrator-go && PATH=/home/jose/go-dev/go/bin:$PATH go test ./... -count=1`
- **Exit code** : 0. **Durée** : 7,95 s (verbose : 8 s).
- **Paquets** : 14 `ok`, 2 sans tests (`cmd/wgmesh`, `gen/amane/framework/v1`).

| Paquet | Résultat | Temps |
|--------|----------|-------|
| `bootstrap` | ok | 0.140s |
| `cmd/orchestrator` | ok | 6.155s |
| `cmd/wgmesh` | no test files | — |
| `consensus` | ok | 0.117s |
| `dcs` | ok | 0.014s |
| `gen/amane/framework/v1` | no test files | — |
| `grpcserver` | ok | 0.146s |
| `haproxy` | ok | 0.009s |
| `lanceur` | ok | 0.004s |
| `mesh` | ok | 0.017s |
| `portail` | ok | 0.008s |
| `provision` | ok | 0.005s |
| `replication` | ok | 0.005s |
| `supervisor` | ok | 0.008s |
| `synchrone` | ok | 0.007s |
| `telemetry` | ok | 0.004s |

- **Skips d'intégration (12)**, tous gardés par absence de service réel :
  - `TestObtenirSecretsAgainstEtcd` (`AMANE_TEST_ETCD`)
  - `TestProduireSecretsAgainstEtcd` (`AMANE_TEST_ETCD`)
  - `TestLeadershipFencingAgainstEtcd` (`AMANE_TEST_ETCD`)
  - `TestMembershipAgainstEtcd` (`AMANE_TEST_ETCD`)
  - `TestMembresAgainstEtcd` (`AMANE_TEST_ETCD`)
  - `TestEtcdKVPutAndList` (`AMANE_TEST_ETCD`)
  - `TestGarantirAgainstPostgres` (`AMANE_TEST_PG`)
  - `TestHeartbeatSessionRecreatedAfterLeaseLoss` (intégration)
  - `TestDiscoverNodesAgainstEtcd` (`AMANE_TEST_ETCD`)
  - `TestSupervisorAgainstEtcd` (`AMANE_TEST_ETCD`)
  - `TestSupervisorPartitionGuardAgainstEtcd` (`AMANE_TEST_ETCD`)
  - `TestGestionnaireAgainstEtcd` (`AMANE_TEST_ETCD`)
- **Lecture** : vert hors ligne ; aucun chemin HA réel exercé. Cela ne valide pas les scénarios HA-01..HA-06 du plan.

### 2. `tests` (contrats B↔C) — `go test ./... -count=1`

- **Commande** : `cd tests && PATH=/home/jose/go-dev/go/bin:$PATH go test ./... -count=1`
- **Exit code** : 1. **Durée** : 0,05 s. **Aucun test exécuté**.
- **Erreur** :
  ```
  go: updates to go.mod needed; to update it:
  	go mod tidy
  ```
- **Cause (diagnostic lecture seule, `go mod tidy -diff`)** : `tests/go.mod` déclare `golang.org/x/text v0.37.0` alors que le module local remplacé (`replace github.com/amane/orchestrator-go => ../orchestrator-go`) exige `v0.41.0` ; `go.sum` a aussi la paire v0.37.0. Diff limité à ces deux lignes.
- **Inventaire statique** (4 fonctions) :
  - `contracts/proto_contract_test.go`: `TestProtoContractStable`
  - `contracts/contracts_test.go`: `TestInterface1EnrollRevoke`, `TestWritePathGated`, `TestWriteRefusedOnNonLeader`
  - `TestInterface1EnrollRevoke` et `TestWritePathGated` se skippent sans `AMANE_TEST_ETCD` ; `TestProtoContractStable` et `TestWriteRefusedOnNonLeader` sont autonomes mais n'ont pas tourné à cause du blocage `go.mod`.
- **Contrainte** : correction non appliquée (écriture limitée à `docs/tests/`, pas de modification de code).

### 3. `core-rust` — `cargo test --workspace`

- **Commande** : `cd core-rust && cargo test --workspace`
- **Exit code** : 0. **Durée** : 0,56 s (cache compilé ; un run antérieur dans les artefacts existants indiquait ~50 s).
- **Résultat** : 1 cible `ss_client` unittests — **7 passed, 0 failed, 0 ignored** ; doctests `ss_client` — 0 test.
- **Tests passés** : `offline::tests::test_graceful_read_only_switch`, `offline::tests::test_offline_cache_crud`, `pairing::tests::test_consumed_invitation_fails`, `pairing::tests::test_successful_pairing_workflow_real_crypto`, `pairing::tests::test_wrong_keypair_cannot_unseal`, `license::tests::test_tampered_license_signature_fails`, `license::tests::test_valid_offline_license_verification`.
- **Avertissement** : `ss-journal` (lib) génère 1 warning `unused variable: e` (compilé comme dépendance).
- **Périmètre réel** : le workspace `core-rust/Cargo.toml` ne contient que `ss-client`. Les crates citées par le plan (`ss-crypto`, `ss-journal`, `ss-consensus`, `spike/node`, `spike/relay`) sont dans `spike/Cargo.toml`, **pas** dans ce workspace. Cette commande ne couvre donc ni C-01..C-06, ni N-01..N-06, ni R-01..R-03.

### 4. Django — `python3 manage.py test`

- **Non exécuté** : la condition du brief « si un environnement virtuel est disponible » n'est pas remplie (`VIRTUAL_ENV` vide, aucun venv trouvé).
- **Risque évité** : `config/settings.py` pointe PostgreSQL `127.0.0.1:5433` (lab) ; lancer les tests Django créerait/supprimerait une base de test sur le lab en service, ce qui contredit la contrainte « ne pas toucher aux conteneurs du lab ».
- **Inventaire statique** :
  - `tenants/tests.py` : 16 classes, 64 `def test_`.
  - `applications/tests.py` : 5 classes, 15 `def test_`.
  - `api/tests.py` : 7 classes, 42 `def test_`.
  - `devices/tests.py` : squelette vide, 0 test.
  - `licenses/tests.py` : squelette vide, 0 test.
- **Écart plan** : le plan signale « devices/tests.py : 3 tests seulement » et « tenants 9 classes » ; l'état réel diffère (devices vide, tenants 16 classes) — le plan est partiellement obsolète.

### 5. HA lab — inventaire statique

- `ha-lab/chaos.sh` contient **12 fonctions `sc_*`** (10 scénarios historiques + 2 egress applicatifs), et utilise `docker kill` / `docker network disconnect`.
- **Non exécuté** : explicitement interdit (lab en service, commandes destructives). La couverture HA du plan reste documentaire, pas mesurée par ce baseline.

## Analyse : ce qui manque au regard du plan

| Domaine / IDs plan | État mesuré | Ce qui manque |
|--------------------|-------------|---------------|
| HA-01..HA-06 (lease, force-right, keepalive gRPC, HAProxy cut, hystérésis, bootstrap) | Unitaires Go PASS ; 12 intégrations SKIP | Intégrations avec etcd/PG réelles non exécutées ; pas de tests dédiés nommés HA-01..HA-06 couvrant les stimuli/assertions du catalogue. |
| Contrats B↔C (`tests/`) | **Bloqué** `go.mod` | 0 contrat vérifié. Manque correction `golang.org/x/text v0.37.0 -> v0.41.0` dans `tests/go.mod`/`go.sum`, puis relance (et idéalement `AMANE_TEST_ETCD` avec accord). |
| C-01..C-06 (Shamir AEAD, X25519 petit ordre, sealed box tronqué, ZeroizeOnDrop, torn write/DoS journal, régression époque) | Non exécutés par `core-rust --workspace` | Couverture P0 manquante dans la baseline ; crates `ss-crypto`/`ss-journal` non testées ici (elles vivent dans `spike/`). |
| N-01..N-06 (quorum split-brain, fencing timeline, DoS enrôlement, streaming backup, restore atomique, proxy étanche) | Non exécutés | P0/P1 manquants ; `ss-consensus` et `spike/node` non ciblés par la commande baseline exécutée. |
| R-01..R-03 (isolation multi-tenant relais, écriture atomique blob, liaison crypto AAD) | Non exécutés | P1 manquants ; `spike/relay` non testé. |
| D-01..D-06 (usurpation appareil, expiration licence, course sièges, confinement réseau, injection manifeste, registry 401/500/302) | Django non lancé ; `devices/tests.py` et `licenses/tests.py` vides | P1/P2 manquants ; écart entre inventaire plan et état réel à corriger. |
| E2E-01..E2E-03 (onboarding, catalogue/validation, témoin dynamique) | Hors scope commandes baseline | P2 manquant, aucun test E2E automatisé. |
| Couverture CI (Go ≥70% agrégé, ≥60% par paquet) | Aucune mesure produite | Manque rapport de couverture (`go test -coverprofile`, `cargo llvm-cov` ou équivalent), seuils non vérifiables. |
| Chaos HA (`ha-lab/chaos.sh`) | Inventaire seul | Manque exécution contrôlée sur fenêtre dédiée avec accord d'exploitation ; pas de preuve fraîche d'exécution. |

## Écarts plan ↔ dépôt observés

- Le plan indique `haproxy/` « Aucun test visible » ; en réalité `orchestrator-go/haproxy/haproxy_test.go` existe et le paquet passe.
- Le plan indique `devices/tests.py` « 3 tests seulement » ; le fichier actuel est un squelette vide (0 test). `tenants/tests.py` compte 16 classes / 64 tests, pas 9 classes.
- Le plan cible des tests dans `spike/crates/*` et `spike/node|relay` ; le baseline demandé via `core-rust --workspace` ne les exécute pas. Il manque une commande baseline explicite pour `spike/` (ex. `cd spike && cargo test --workspace`).
- Le bloc de claims JSON décrit dans `AGENTS.md` ne peut pas être lu depuis `AGENTS.md` (fichier absent) ; le format utilisé est celui imposé par le brief de tâche.

## Risques / points humains

- **Contrats B↔C bloqués** : corriger `tests/go.mod`/`go.sum` (diff connu, `golang.org/x/text v0.41.0`) puis relancer `go test ./... -count=1`.
- **Intégrations Go skippées** : 12 tests nécessitent `AMANE_TEST_ETCD`/`AMANE_TEST_PG` et un lab disponible ; ne pas lancer sans accord car ils parlent au lab.
- **Django non exécutable sans risque** : nécessite un venv et une DB de test isolée ; ne pas pointer sur le Postgres du lab.
- **Baseline Rust incomplète** : ajouter `cd spike && cargo test --workspace` pour couvrir ss-crypto/ss-journal/ss-consensus/node/relay.
- **docs/ ignoré par git** : `docs/tests/BASELINE.md` et `artifacts-a01/` existent sur disque mais ne seront pas suivis par Git sans ajustement volontaire de `.gitignore` (hors scope de cette tâche).
