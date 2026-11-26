# Groupe R — Relais zero-knowledge (isolation, atomicité, liaison cryptographique)

Tests `R-01` à `R-03` du plan `plan-tests-exhaustif-v3.md` (§ Groupe R),
sur `spike/relay` (binaire `ss-relay`).

> **Note de plan** : le fichier `plan-tests-exhaustif-v3.md` référencé par le
> brief n'est **pas présent** dans le dépôt (`find . -iname '*plan*test*'` →
> vide). Les énoncés R-01→R-03 ci-dessous reprennent **mot pour mot** ceux du
> brief de tâche, qui fait foi.

Date d'exécution : 2026-11-25. **Aucune modification du code source** de
`spike/relay` ni de `spike/node`. Le relais de test est un **binaire local
jetable** (`spike/target/debug/ss-relay`, build `cargo build -p relay`), lancé
sur `127.0.0.1:18099` avec une base de blobs jetable `/tmp/opencode/r01/blobs`
et un `RELAY_SECRET` aléatoire de 64 caractères. **Le relais de production n'a
jamais été touché** (aucun conteneur `amane-*` sollicité, aucun `docker`
invoqué).

## Environnement

- Rust 1.99.1, `cargo build -p relay` → `spike/target/debug/ss-relay` (10,8 s).
- Relais démarré : `RELAY_SECRET=<64 hex> BLOBS_DIR=/tmp/opencode/r01/blobs
  RELAY_PORT=18099 ss-relay`. `/health` → `"authentification":true`,
  `"zero_knowledge":true`.
- Jetons calculés avec `hmac` Python (3.14), **identique** au vecteur de test de
  `ss_crypto::jeton_relais` :
  `jeton('secret-de-test-du-relais-amane-0123456789','00000000-…-000000000000')
  = 1122cac8a5f6cce94d90f49b21d7aee4cb844f27725ab56a57ae4f80764df4a5`.
- Preuves brutes : `docs/tests/artifacts-r01/`.

---

## R-01 — Isolation multi-tenant — **FAIL** (statut 401 au lieu de 403)

- **Stimulus** : depuis un jeton du **tenant A**, appeler
  `GET` / `PUT` / `DELETE` sur `/api/blobs/{tenant_B}/sauvegarde-amane-1`.
- **Attendu (plan)** : `403`, validation HMAC du `tenant_id` authentifiée.
- **Commande** : `bash docs/tests/artifacts-r01/r01.sh` (requêtes `curl` avec
  en-tête `x-relay-token` du tenant A ; jetons `jeton.py`).
- **Résultat réel** :
  - Le blob du tenant B est d'abord déposé avec le jeton de B → `HTTP 200`
    (la cible est légitime, le refus de A n'est pas un 404 déguisé).
  - `GET`, `PUT`, `DELETE` du blob de B avec le **jeton de A** →
    **`HTTP 401`** (corps `Jeton du tenant manquant ou invalide`) pour les
    trois méthodes.
  - Sans jeton → `401` ; jeton aléatoire → `401`.
  - Après les tentatives, `GET` du blob de B avec le jeton de B → `200`
    `secret-de-B` : **aucune atteinte** (pas d'écrasement, pas de suppression,
    pas de fuite).
- **Écart avec le plan** : l'**isolation multi-tenant tient** (A ne peut ni lire,
  ni écrire, ni supprimer le blob de B) et la **validation HMAC du `tenant_id`
  est bien authentifiée** (`ss_crypto::verifier_jeton_relais`, temps constant).
  Mais le **code HTTP renvoyé est `401 Unauthorized`, pas `403 Forbidden`**.
  Le relais mappe tout échec d'autorisation sur `UNAUTHORIZED`
  (`spike/relay/src/main.rs`, `blob_autorise` l. 223-224 et `autorise` l. 65-72).
  Or A **est** authentifié (jeton valide pour A) : sémantiquement `403` serait
  exact. L'assertion littérale « 403 » du plan **n'est pas satisfaite**.
- **Verdict** : **FAIL** — l'invariant de sécurité (isolation + HMAC) est tenu,
  mais l'assertion du plan porte sur un statut `403` que le code ne produit pas
  (`401`). À traiter comme un défaut de conformité au contrat, pas comme une
  faille de confidentialité.
- **Preuve** : `docs/tests/artifacts-r01/r01-sortie.txt`, `r01.sh`, `jeton.py`.

## R-02 — Écriture atomique / coupure TCP à 20 % — **PASS**

- **Stimulus** : tuer la connexion TCP à 20 % du transfert d'un blob de 500 Mo
  (`524 288 000` octets), via un `PUT` HTTP/1.1 `Content-Length: 500 Mo` dont on
  ferme le socket par un **RST** (`SO_LINGER=0`) après 100 Mo émis.
- **Attendu** : aucun fichier résiduel sur disque, `GET` = 404, le nœud réémet.
- **Commande** : `python3 docs/tests/artifacts-r01/r02-kill.py 524288000 0.20`.
- **Résultat réel** :
  - Coupure effective à `104 857 600` octets (20,0 %) en 0,11 s, puis RST.
  - `find blobs -type f` → **aucun fichier du tenant C** (seul subsistait le
    blob de test R-01 du tenant B, 11 octets, antérieur et légitime).
  - `GET` du blob du tenant C → **`HTTP 404`**.
  - Log relais : **aucune** ligne « Blob stocke » pour le tenant C — le relais
    n'a jamais écrit. Cause structurelle : `blob_put` prend un `body: Bytes`
    (extracteur d'axum) qui **matérialise le corps entier avant** l'appel du
    handler ; une connexion coupée empêche l'extraction, donc `tokio::fs::write`
    n'est jamais atteint → pas de fichier partiel.
  - **Ré-émission** : un `PUT` complet de 500 Mo (simulant le nœud qui réémet)
    → `HTTP 200` en 1,27 s ; `GET` → `200`, `taille=524288000` ; fichier final
    `524288000` octets sur disque.
- **Verdict** : **PASS** — aucun résidu, `GET`=404, ré-émission acceptée et
  intégrale.
- **Réserve** : l'atomicité tient parce que le corps est tamponné en mémoire par
  l'extracteur, **pas** par un schéma « fichier temporaire + rename ». Un crash
  du relais **pendant** `tokio::fs::write` (et non une coupure client) pourrait
  laisser un fichier tronqué ; ce cas n'est pas couvert par R-02 (voir risques).
- **Preuve** : `docs/tests/artifacts-r01/r02-kill-sortie.txt`,
  `r02-preuves-complementaires.md`, `r02-kill.py`.

## R-03 — Liaison cryptographique du blob (AAD) — **PASS**

- **Stimulus** : copier un blob chiffré de `sauvegarde-amane-1` vers
  `sauvegarde-application-1` (même tenant), puis restaurer.
- **Attendu** : déchiffrement en erreur (l'AAD inclut `tenant_id`, nom de base,
  date).
- **Commande** : harnais `cargo run` dans `/tmp/opencode/r03-harness`, qui
  **inclut le vrai** `spike/node/src/sauvegarde.rs` et `relay_client.rs` (via
  `#[path]`, sans les modifier) et pilote le relais **local** `127.0.0.1:18099`.
- **Résultat réel** (`docs/tests/artifacts-r01/r03-sortie.txt`) :
  - `[1]` blob `amane` emballé (4214 octets, jour 3) ;
  - `[2]` `PUT sauvegarde-amane-3` → `200` ;
  - `[3]` copie vers `sauvegarde-application-3` → `200` (le relais stocke un
    blob opaque, il ne « voit » pas le nom) ;
  - `[4]` restauration `application` → **refusée** :
    `blob de la base "amane" déposé sous le nom de la base "application" : refusé`
    (le nœud rejette avant tout `pg_restore`) ;
  - `[5]` restauration `amane` légitime → **OK** (date 2026-11-25, export 4101
    octets) : preuve que le refus de `[4]` vient de la **liaison**, pas d'un blob
    invalide ;
  - `[6]` liaison AAD directe : déchiffrement avec l'en-tête `amane` → **OK**,
    avec l'en-tête `application` → **ERREUR** (`DecryptionFailed`). L'AAD
    (`amane-sauvegarde/1 <tenant> <base> <date>`) authentifie bien le
    `tenant_id`, le nom de base et la date.
- **Verdict** : **PASS** — un blob déplacé sous un autre nom est refusé à la
  restauration, et l'AEAD (XChaCha20-Poly1305) échoue si l'en-tête/AAD ne
  correspond pas.
- **Preuve** : `docs/tests/artifacts-r01/r03-sortie.txt`, `r03-relais.log`,
  `main.rs`, `Cargo.toml`.

---

## Synthèse

| Test | Résultat | Résumé |
|---|---|---|
| R-01 | **FAIL** | Isolation + HMAC tenus (A ne touche pas B), mais statut **401** au lieu de **403** attendu. |
| R-02 | **PASS** | Coupure RST à 20 % de 500 Mo → aucun résidu, `GET`=404, ré-émission 500 Mo OK. |
| R-03 | **PASS** | Blob copié `amane`→`application` refusé à la restauration ; AAD (tenant/base/date) authentifiée par l'AEAD. |

Bilan R : **2 PASS (R-02, R-03)**, **1 FAIL (R-01)**. L'échec R-01 est un
**écart de statut HTTP** (401 vs 403), pas une faille d'isolation : la
confidentialité multi-tenant et l'authentification HMAC du `tenant_id` sont
réellement assurées.

## Risques / réserves

- `plan-tests-exhaustif-v3.md` absent du dépôt : énoncés repris du brief.
- R-01 : le contrat attend `403` ; le code renvoie `401` pour tout échec
  d'autorisation (`blob_autorise`). Recommandation : distinguer
  « jeton absent/invalide » (401) de « jeton valide mais autre tenant » (403).
- R-02 : l'atomicité repose sur le tamponnage mémoire du corps par l'extracteur
  d'axum, pas sur un `tmp + rename`. Un crash serveur pendant `fs::write` (hors
  périmètre R-02, qui teste une coupure client) pourrait laisser un fichier
  tronqué.
- R-03 : la copie est refusée **au niveau du nœud** (`deballer`), le relais
  restant zero-knowledge ; c'est bien le comportement attendu (le relais ne
  déchiffre jamais).