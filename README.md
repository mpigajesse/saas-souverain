# Branche `feature-b-dev-integration` — Mission B : client gRPC et mode hors-ligne

> Branche de travail de **Pmabouba27** · 7 commits · basée sur `main` (`ca80881`)

---

## En une phrase

Cette branche construit le **client applicatif du nœud PME** : le contrat gRPC qui le relie au
cluster, un cache local SQLite qui permet de continuer à travailler en lecture quand le réseau
tombe, et la validation autonome de la licence sans jamais interroger l'éditeur.

---

## Contenu de la branche

```
proto/framework.proto              contrat gRPC AmaneService (4 RPC)

core-rust/                         NOUVEAU workspace Cargo (coexiste avec spike/)
├── ss-client/
│   ├── build.rs                   génération tonic + protoc vendoré
│   ├── src/invariants.rs          règles métier : stock, numérotation de factures
│   ├── src/license.rs             validation de licence Ed25519 hors-ligne
│   ├── src/offline.rs             cache SQLite + mode dégradé lecture seule
│   ├── src/pairing.rs             appairage QR + scellement de l'AK
│   └── tests/mock_server.rs       serveur gRPC factice pour les tests
├── ss-crypto/                     COPIE de spike/crates/ss-crypto  (voir avertissement)
└── ss-journal/                    COPIE de spike/crates/ss-journal (voir avertissement)

pme-deploy/docker-compose.yml      +1 ligne : NODE_MODE transmis au conteneur
```

**Volume réel de code neuf : environ 800 lignes.** Les 1 993 lignes de `Cargo.lock` et les
~660 lignes de `ss-crypto` / `ss-journal` sont des copies, pas du travail nouveau.

---

## Ce qui est solide

### `proto/framework.proto` — le contrat gRPC

C'est la pièce la mieux conçue de la branche. Package `amane.framework.v1`, service
`AmaneService`, quatre appels :

| RPC | Rôle | Points notables |
|---|---|---|
| `RegisterNode` | Un nœud rejoint le cluster | `installation_id` (UUID), clé publique X25519 ; réponse avec `current_epoch` |
| `WriteOperation` | Écriture journalisée | Porte l'`epoch` (**jeton de fencing**) et un `encrypted_cbor_payload` opaque |
| `ReadOperation` | Lecture | `allow_offline_stale` ; réponse marquée `is_offline_read` |
| `GetClusterStatus` | État du cluster | `active_nodes_count`, `read_only_mode`, `signed_license_token` |

Le contrat respecte l'architecture : l'époque de fencing circule à chaque écriture, les données
transitent chiffrées et opaques, et le mode dégradé est signalé explicitement par le serveur
plutôt que deviné par le client.

### `build.rs` — la chaîne protoc

```rust
std::env::set_var("PROTOC", protoc_bin_vendored::protoc_bin_path()?);
tonic_build::configure().build_server(true).build_client(true)
    .compile(&["../../proto/framework.proto"], &["../../proto"])?;
```

`protoc-bin-vendored` embarque le binaire `protoc` dans la dépendance : **aucune installation
manuelle n'est nécessaire**. C'est ce que résout le dernier commit de la branche, et c'est le
point le plus propre du lot.

### `offline.rs` — le mode dégradé

Deux états, `NominalReadWrite` et `GracefulReadOnly`, derrière un `Arc<Mutex<…>>`. Le cache est
une table SQLite unique :

```sql
offline_cache(collection, record_id, encrypted_payload BLOB, updated_at,
              PRIMARY KEY(collection, record_id))
```

`handle_grpc_result` intercepte les `tonic::Status` : sur `Unavailable`, `Unknown` ou
`DeadlineExceeded`, il bascule en lecture seule ; sur toute autre erreur il la remonte sans
changer d'état. En lecture seule, `read_record` continue de servir le cache et `save_record`
retourne `ReadOnlyViolation`. La machine à états est correcte et testée.

### `license.rs` — validation hors-ligne

Vérification Ed25519 d'un `SignedLicenseToken`, **puis** désérialisation du payload — l'ordre
est le bon, la signature est contrôlée avant tout parsing. Contrôles d'expiration et de quota
(`claimed_nodes_count > max_nodes`). La clé publique de l'éditeur est passée en paramètre :
aucune clé n'est codée en dur.

---

## Deux points bloquants avant toute intégration

### 1. L'appairage Sealed Box est un stub — la clé circule en clair

`pairing.rs`, fonction `seal_access_key_for_node` :

```rust
// 3. Emballage Sealed Box (simulé / format de scellement)
let mut dummy_sealed = vec![0xA1, 0xB2];              // "header" cosmétique
dummy_sealed.extend_from_slice(cluster_ak_bytes);      // AK EN CLAIR
dummy_sealed.extend_from_slice(recipient_public_key_bytes);
```

Il n'y a **aucune cryptographie**. La clé d'accès du cluster est concaténée en clair derrière
deux octets décoratifs. `unseal_access_key` se contente de relire `sealed_ak_bytes[2..34]`, la
clé privée du destinataire n'étant utilisée que pour un test factice
(`recipient_private_key_bytes[0] == 0xFF`). Aucun appel à X25519.

Le message de commit annonce « scellement AK (Sealed Box) » et le fichier
`paroles_explication_mission_b.txt` parle d'un scellement « en toute sécurité ». Ce n'est pas
le cas aujourd'hui, et le risque est qu'un lecteur pressé prenne ce code pour acquis.

**À faire** : remplacer par un vrai sealed box X25519. `ss-crypto::DeviceKeyPair` existe déjà
dans le même workspace et n'est pas appelé.

**Conséquence sur les tests** : `test_successful_pairing_workflow` passe précisément *parce
que* l'AK est en clair. Il verrouille le comportement factice au lieu de la propriété de
sécurité — il devra être réécrit en même temps.

### 2. Un second cœur cryptographique a été créé

`core-rust/ss-crypto/**` et `core-rust/ss-journal/**` sont **octet pour octet identiques** à
`spike/crates/ss-crypto/**` et `spike/crates/ss-journal/**`, déjà présents sur `main`. C'est un
fork par copier-coller du socle.

Le `CLAUDE.md` pose : **« un seul cœur, jamais deux »**.

Et le problème est déjà concret : la branche `crypto` fait évoluer **l'autre copie** — elle
ajoute `ak.rs`, `crl.rs`, `shamir.rs` et modifie `journal.rs` côté `spike/crates/`. Les deux
copies divergent, et **git ne signalera jamais de conflit** puisque les chemins diffèrent.

**À faire** : supprimer `core-rust/ss-crypto` et `core-rust/ss-journal`, et référencer
`spike/crates/` par chemin. `spike/crates/` est le socle historique, utilisé par `node/`,
`relay/` et les Dockerfiles.

Ironie du dossier : `ss-client` déclare `ss-crypto` et `ss-journal` en dépendances mais ne les
utilise **nulle part** — aucun `use ss_crypto` dans le code. Le client ne chiffre rien ; il
manipule des `encrypted_payload` qu'il ne produit pas.

---

## Les 9 tests unitaires

| # | Fichier | Test | Couvre |
|:---:|---|---|---|
| 1 | `invariants.rs` | `test_valid_stock_reduction` | 10 − 3 = 7 |
| 2 | `invariants.rs` | `test_invalid_negative_stock` | 2 − 5 → erreur |
| 3 | `invariants.rs` | `test_valid_sequence` | numéro 104 == attendu |
| 4 | `license.rs` | `test_valid_offline_license_verification` | signature Ed25519 valide |
| 5 | `license.rs` | `test_tampered_license_signature_fails` | payload falsifié → rejet |
| 6 | `offline.rs` | `test_offline_cache_crud` | écriture + lecture SQLite |
| 7 | `offline.rs` | `test_graceful_read_only_switch` | bascule complète du mode dégradé |
| 8 | `pairing.rs` | `test_successful_pairing_workflow` | seal → unseal |
| 9 | `pairing.rs` | `test_consumed_invitation_fails` | jeton réutilisé → rejet |

Plus un test d'intégration hors décompte : `tests/mock_server.rs`.

**Lecture honnête de cette couverture :**

- **Vraiment utiles** : les tests 5 et 7. Le 5 prouve qu'une falsification après signature est
  rejetée ; le 7 vérifie la machine à états complète (lecture autorisée, écriture bloquée).
- **Triviaux** : 1 à 3 sont de l'arithmétique. Le test 2 se contente de `is_err()` sans
  vérifier quelle erreur. Le cas `total_amount < 0.0` et la variante `SequenceBreak` ne sont
  jamais testés.
- **Trompeurs** : 8 et 9 valident un scellement qui ne chiffre rien.
- **Non couverts** : expiration de licence, dépassement de quota, `InvalidPublicKey`, signature
  de mauvaise longueur, expiration d'invitation, retour de `GracefulReadOnly` vers
  `NominalReadWrite`, et les RPC `RegisterNode` / `ReadOperation` / `GetClusterStatus` —
  implémentées dans le mock mais jamais appelées.

Le projet vise 80 % de couverture ; on en est loin, et aucun outil de mesure n'est configuré.

---

## Dette technique à solder

- **`total_amount: f64`** pour un montant monétaire. Incompatible avec des invariants de
  facturation stricts — utiliser des entiers en centimes ou un type décimal.
- **`uuid_simple()`** ne renvoie que `subsec_nanos()` sur 8 chiffres hex : collisions
  d'`invitation_id` probables, alors que la crate `uuid` est déjà en dépendance.
- **`PairingInvitationToken.consumed`** est un booléen en mémoire, sans persistance : l'usage
  unique n'est pas garanti entre deux redémarrages.
- **`.unwrap()` sur mutex** dans `offline.rs` (4 occurrences) — panique sur mutex empoisonné,
  dans du code destiné à la production.
- **Pas de reconnexion** : `handle_grpc_result` ne repasse en mode nominal que sur un `Ok`.
  Aucune sonde ; un cluster restauré peut rester bloqué en lecture seule.
- **Versions en retard** : `ss-client/Cargo.toml` n'utilise aucune des
  `[workspace.dependencies]` du parent et redéclare tout en dur. `thiserror` 1.0 et 2.0
  coexistent dans le `Cargo.lock` ; `tonic 0.10` et `prost 0.12` ont deux majeures de retard.
- **Deux bibliothèques CBOR** dans le même workspace : `ciborium` côté `ss-journal`,
  `cbor4ii` déclaré (et inutilisé) côté `ss-client`. À trancher.
- **Test gRPC fragile** : `mock_server.rs` utilise le port fixe 50051 et un `sleep(100 ms)`
  avant connexion — incompatible avec une exécution parallèle ou un port déjà occupé.
- **`paroles_explication_mission_b.txt`** : script de présentation orale versionné à la racine
  du dépôt, sans rapport avec le code, et décrivant des stubs comme opérationnels.

---

## Conformité au `CLAUDE.md`

| Règle | État |
|---|---|
| Identifiant d'installation UUID, jamais la MAC | **Respecté** — `installation_id` partout, aucune MAC |
| Découverte via le relais, pas mDNS | **Respecté** — `RegisterNode` va dans ce sens |
| Journal CBOR | Respecté côté `ss-journal` ; à clarifier côté client (deux libs) |
| PostgreSQL pour le nœud actif | Non concerné — le SQLite ajouté est un cache client, c'est légitime |
| Un seul cœur, jamais deux | **Violé** — voir le point bloquant n° 2 |
| Aucune primitive réimplémentée | Écart hérité : le socle utilise RustCrypto et non libsodium |
| Pas de logique métier avant validation du spike | **Zone grise** — `invariants.rs` (stock, factures) est de la logique métier |

Aucun secret ni clé privée en dur. En revanche, la clé publique de l'éditeur n'est ni embarquée
ni provisionnée : la validation de licence n'est pas branchable en l'état.

---

## Compiler et tester

```bash
git checkout feature-b-dev-integration
cd core-rust
cargo build          # protoc est vendoré, rien à installer
cargo test           # 9 tests unitaires + 1 test d'intégration gRPC
```

**Prérequis non documenté** : `rusqlite` est en mode `bundled` et compile SQLite depuis les
sources — un compilateur C est nécessaire (MSVC Build Tools sous Windows).

Le chemin `../../proto` dans `build.rs` impose que la crate reste à sa profondeur actuelle.

---

## Conventions

Commits : `feat` · `fix` · `refactor` · `docs` · `test` · `chore` · `perf` · `ci`.

Voir le [README.md général du projet](https://github.com/mpigajesse/saas-souverain/blob/main/README.md) sur la branche `main` pour l'architecture d'ensemble,
et [`CLAUDE.md`](CLAUDE.md) pour les décisions structurantes actées.
