# Groupe N — Nœud PME (quorum, fencing, enrôlement, streaming, restore, proxy)

Tests `N-01` à `N-06` du plan `plan-tests-exhaustif-v3.md` (§ Groupe N).
Périmètre : `orchestrator-go/` (consensus, dcs), `spike/node`, `spike/crates/*`
(`ss-consensus`, `ss-crypto`, `ss-journal`) et `core-rust`.

Date d'exécution : 2026-11-25. Aucune modification du code source : les preuves
sont des sorties brutes dans `docs/tests/artifacts-n01/` et des rapports de test
existant. Les scénarios non couverts par les tests unitaires ont été exercés par
deux harnais jetables (dans `/tmp/opencode`, hors dépôt) qui **incluent le code
réel** des crates (`ss-consensus`, `ss-crypto`) et du relais
(`spike/node/src/application.rs`), sans le modifier.

## Environnement

- Go 1.27.3 (`/home/jose/go-dev/go/bin/go`), Cargo/Rust 1.99.1.
- `go test ./...` dans `orchestrator-go/` → **tous les paquets OK**
  (`docs/tests/artifacts-n01/go-test-orchestrator.log`).
- `cargo test` du workspace `spike/` et de `core-rust/` → OK pour `ss-consensus`,
  `node`, `relay`, `ss-client` ; **échecs préexistants hors groupe N** documentés
  au groupe C : `ss-crypto::tests_c01::c02` (FAIL), `ss-journal::tests_c01::c05`
  (FAIL), `ss-journal::tests_c01::c06` (BLOQUE). Ils ne concernent pas les
  invariants N (voir `docs/tests/C-CRYPTO.md`).

---

## N-01 — Split-brain dans la garde de quorum — **FAIL**

- **Stimulus** : `evaluer_quorum()` observant deux membres `Role=Leader`.
- **Attendu (plan)** : `Etat = Perdu(Motif::SplitBrain)`, aucune écriture.
- **Commande** :
  `cargo run --offline` dans `/tmp/opencode/n-harness` (dépend de `ss-consensus`
  par chemin ; appelle `ss_consensus::evaluer_quorum`).
- **Résultat réel** :
  `N-01 deux Leaders -> SousLeSeuil(AucunStandbySynchrone) (code=sous_le_seuil)`.
  La variante `Motif::SplitBrain` **n'existe pas** dans `spike/crates/ss-consensus/src/quorum.rs`
  (l'enum `Motif` ne contient que `PatroniInjoignable`, `AucunPrimaire`,
  `AucunStandbySynchrone`). Avec deux leaders et aucun standby synchrone, la
  fonction renvoie `SousLeSeuil`, pas `Perdu` : le split-brain n'est pas détecté.
- **Preuve** : `docs/tests/artifacts-n01/harness-n01-n05.log` (lignes N-01),
  `docs/tests/artifacts-n01/cargo-test-spike.log` (19 tests `ss-consensus` OK,
  aucun ne couvre deux leaders).

## N-02 — Fencing continu par timeline PostgreSQL — **PASS**

- **Stimulus** : tick `run.rs` : machine locale primaire à `timeline=2`, cluster
  à `timeline=3`.
- **Attendu** : `decider()` → `fence:true`, `ecriture_autorisee:false`, rôle
  basculé à `standby`, sauvegarde annulée.
- **Commande** :
  `cargo test --offline -p node -- --nocapture cmd::run::tests::ancien_primaire_revenu_est_fence`
  puis `cargo run --offline` dans `/tmp/opencode/n-harness` (N-02).
- **Résultat réel** :
  - Test unitaire `ancien_primaire_revenu_est_fence` **OK** : `decider(Primaire, Some(3), 4, Sain)`
    = `Decision { role: "standby", ecriture_autorisee: false, fence: true }`.
  - Harnais : `check_fencing(2,3) -> fence=true ecriture_autorisee=false`.
  - Côté `boucle_supervision` (`run.rs` l. 347-350) : `(false, _) => prochaine_sauvegarde = None`
    → la sauvegarde planifiée est **annulée** dès que l'écriture n'est plus
    autorisée.
- **Preuve** : `docs/tests/artifacts-n01/node-tests-nocapture.log`,
  `docs/tests/artifacts-n01/harness-n01-n05.log` (lignes N-02),
  `docs/tests/artifacts-n01/cargo-test-node.log`.

## N-03 — DoS codes d'enrôlement — **FAIL**

- **Stimulus** : 5 requêtes invalides `POST /amane/api/enrolement`.
- **Attendu** : blocage IP (rate-limit), invités légitimes conservés, comparaison
  en temps constant.
- **Commande** :
  `cargo test --offline -p node -- --nocapture enrolement::tests::trop_d_echecs_annulent_les_codes`
  + inspection de `spike/node/src/enrolement.rs` et `auth/routes.rs`.
- **Résultat réel** :
  - **Pas de blocage IP / rate-limit** : `api_enrolement` (`auth/routes.rs` l. 501-520)
    ne lit aucune adresse IP, aucun compteur par IP, aucun `429`. Le seul garde-fou
    est un compteur global d'échecs.
  - **Invités légitimes NON conservés** : après `ECHECS_MAX = 5` échecs,
    `Invitations::consommer` exécute `codes.clear()` (`enrolement.rs` l. 64-66) :
    **tous** les codes en cours, y compris légitimes, sont annulés. Le test
    `trop_d_echecs_annulent_les_codes` (qui passe) asserte précisément cet
    annulation — l'inverse de l'attendu du plan.
  - **Comparaison non constant-temps** : `codes.iter().position(|(c, _)| *c == code)`
    (`enrolement.rs` l. 57) est une comparaison de `String` qui court-circuite ;
    le crate `subtle` (constant-temps) n'est pas utilisé ici.
- **Preuve** : `docs/tests/artifacts-n01/node-tests-nocapture.log`,
  `docs/tests/artifacts-n01/cargo-test-node.log`.

## N-04 — Streaming mémoire backup (OCU crash) — **FAIL**

- **Stimulus** : `pg_dump` 5 Go en flux (ou plus gros volume disponible).
- **Attendu** : RSS < 128 Mo, authentification AEAD par morceau, offset seek correct.
- **Commande** : inspection de `spike/node/src/sauvegarde.rs` (pas de volume de
  5 Go disponible sur ce banc ; voir réserve ci-dessous).
- **Résultat réel** :
  - `sauvegarder()` lance `pg_dump … .output()` (`sauvegarde.rs` l. 162-168) :
    `.output()` **matérialise l'intégralité du dump dans un `Vec<u8>`** en
    mémoire. La RSS croît donc avec la taille du dump → l'assertion « RSS < 128 Mo »
    est **fausse par construction** pour un gros volume.
  - `emballer()` chiffre l'export **en un seul bloc** (`encrypt_avec_contexte(export, entete)`,
    l. 101-110) : aucun découpage par morceau, aucune authentification AEAD par
    chunk, aucun `seek`/offset.
  - Aucun `chunk`, `seek` ni streaming n'apparaît dans le module (grep négatif).
- **Réserve** : le dump 5 Go n'a pas été exécuté (aucun volume de cette taille sur
  le banc, et la pile en service ne doit pas être sollicitée). Le résultat est
  néanmoins déterminé par le code : le chemin d'exécution charge tout en mémoire.
- **Preuve** : `docs/tests/artifacts-n01/` (inspection `sauvegarde.rs` l. 101-178).

## N-05 — Atomicité restore — **PASS**

- **Stimulus** : blob altéré d'1 bit depuis le relais.
- **Attendu** : déchiffrement DEK en erreur **avant** `pg_restore --clean`, base
  locale intacte.
- **Commande** : `cargo run --offline` dans `/tmp/opencode/n-harness` (N-05, AEAD
  réel via `ss_crypto::Dek::encrypt_avec_contexte` / `decrypt_avec_contexte`) +
  inspection de `restaurer()`.
- **Résultat réel** :
  - 1 bit inversé dans le ciphertext → `decrypt_avec_contexte = Err("DecryptionFailed")`.
    Le blob intact se déchiffre (`OK = true`).
  - Dans `restaurer()` (`sauvegarde.rs` l. 182-208), `deballer()` (qui appelle
    `decrypt_avec_contexte`) est appelé **ligne 193, avant** le `spawn` de
    `pg_restore --clean` (ligne 195). Une erreur de déchiffrement renvoie donc
    `Err` **avant** toute commande `pg_restore` : la base locale n'est pas touchée.
- **Preuve** : `docs/tests/artifacts-n01/harness-n01-n05.log` (lignes N-05),
  `docs/tests/artifacts-n01/cargo-test-node.log` (`en_tete_falsifie_refuse`,
  `le_blob_depose_est_illisible_sans_la_dek` OK).

## N-06 — Étanchéité reverse proxy applicatif — **PASS**

- **Stimulus** : chemin `/amane/../`, en-têtes chunked + Content-Length, HTTP
  Upgrade, Keep-Alive croisé.
- **Attendu** : hop-by-hop épurés, normalisation URI stricte, isolation SaaS ↔
  admin, 502 si application absente.
- **Commande** : serveur de test **local** (pas la pile en service) :
  `cargo run` dans `/tmp/opencode/n06-harness`, qui charge le **vrai**
  `spike/node/src/application.rs` comme module, monte un faux SaaS local et un
  relais sur `127.0.0.1:18080`, puis envoie des requêtes HTTP brutes.
  Complété par `cargo test -p node -- --nocapture application::tests`.
- **Résultat réel** (`docs/tests/artifacts-n01/harness-n06.log`) :
  - **Hop-by-hop épurés** : requête avec `Connection`, `Keep-Alive`,
    `Transfer-Encoding`, `TE`, `Trailer`, `Upgrade`, `Proxy-Authorization` →
    le SaaS reçoit `connection=<absent>`, `keep-alive=<absent>`,
    `transfer-encoding=<absent>`, `te=<absent>`, `trailer=<absent>`,
    `upgrade=<absent>`. `x-forwarded-host=poste-1:9001` conservé.
    (`SAUT_A_SAUT` + `relayable()`/`copier_entetes()`, `application.rs` l. 21-80.)
  - **Keep-Alive croisé** : deux requêtes sur la même connexion → **deux**
    réponses distinctes (`PATH=/a` puis `PATH=/b` avec `connection: close`),
    aucune fuite d'une réponse vers l'autre.
  - **502 si application absente** : `relayer` vers un port mort → `status = 502
    Bad Gateway` (test `application_absente` OK).
  - **Normalisation / isolation** : `/amane/../admin/utilisateurs` est normalisé
    par la pile HTTP (hyper) en `PATH=/admin/utilisateurs` et **relayé au SaaS**
    (fallback), donc il **n'atteint pas** la console `/amane/admin` (protégée par
    `admin_middleware`). La console reste sous `/amane`, l'application est le
    fallback : l'isolation SaaS ↔ admin tient.
- **Preuve** : `docs/tests/artifacts-n01/harness-n06.log`,
  `docs/tests/artifacts-n01/node-tests-nocapture.log`,
  `docs/tests/artifacts-n01/cargo-test-node.log`.

---

## Synthèse

| Test | Résultat | Résumé |
|---|---|---|
| N-01 | **FAIL** | Deux leaders → `SousLeSeuil`, pas `Perdu(SplitBrain)` ; motif absent. |
| N-02 | **PASS** | `decider` fence timeline 2 < 3, écriture interdite, sauvegarde annulée. |
| N-03 | **FAIL** | Pas de rate-limit IP, codes légitimes effacés après 5 échecs, comparaison non constant-temps. |
| N-04 | **FAIL** | `pg_dump` chargé en entier en mémoire (`.output()`), chiffrement en un bloc : RSS non borné, pas d'AEAD par morceau. |
| N-05 | **PASS** | 1 bit altéré → `DecryptionFailed` avant `pg_restore` ; base intacte. |
| N-06 | **PASS** | Hop-by-hop épurés, keep-alive isolé, 502 si app absente, isolation SaaS ↔ admin. |

Bilan N : **3 PASS (N-02, N-05, N-06)**, **3 FAIL (N-01, N-03, N-04)**.
Les trois échecs sont des **invariants non implémentés** (détection split-brain,
rate-limit/temps constant à l'enrôlement, streaming mémoire borné) — à traiter
comme des défauts du code, pas comme des problèmes de banc.