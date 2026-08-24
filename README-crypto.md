# Branche `crypto` — Mission A : socle cryptographique V2

> Branche de travail de **Siradiou** · 5 commits · basée sur `main` (`ca80881`)
>
> Documentation de référence de la branche : [`architecture_V2_crypto.md`](architecture_V2_crypto.md)

---

## En une phrase

Cette branche prépare la **V2 du socle cryptographique** : séparer la clé qui donne *accès au
cluster* de la clé qui *chiffre les données*, afin de pouvoir révoquer une machine en moins
d'une seconde au lieu de rechiffrer tout le disque de la PME.

---

## Le problème que la V2 résout

En V1, une seule clé jouait deux rôles : identité réseau **et** chiffrement disque. Conséquence
directe : dé-enrôler une machine — un salarié qui part, un portable volé — obligeait à faire
tourner la DEK et donc à **rechiffrer l'intégralité des données**. Sur une PME réelle, cela
signifie plusieurs minutes à plusieurs heures d'indisponibilité pour un événement administratif
banal.

La V2 scinde les deux responsabilités :

| Clé | Rôle | Révocation |
|---|---|---|
| **AK** — Access Key (X25519) | Identité et accès réseau d'un appareil | Ajout à une liste de révocation, **immédiat** |
| **DEK** — Data Encryption Key | Chiffrement des données et du journal | Inchangée lors d'une révocation |

Révoquer une machine devient une écriture dans un registre, pas une réécriture de disque.

---

## Contenu de la branche

```
architecture_V2_crypto.md                 spécification de la mission (139 lignes)
spike/crates/ss-crypto/src/ak.rs          AccessKeyPair X25519 + AccessPublicKey
spike/crates/ss-crypto/src/crl.rs         registre de révocation      [VIDE]
spike/crates/ss-crypto/src/shamir.rs      partage de secret K=2/N=3   [VIDE]
spike/crates/ss-journal/src/journal.rs    pagination read_range + count_frames allégé
```

### Découpage annoncé par les commits

| Étape | Objet | Commit | État réel dans le code |
|:---:|---|---|---|
| A1 | DEK avec `Zeroize` | `5a2f9eb` | Déjà présent sur `main` — rien d'ajouté |
| A2 | `AccessKey` (AK) | `5a2f9eb` | **Implémenté** dans `ak.rs` |
| A3 | Registre de révocation CRL | `43d23f2` | Fichier créé, **0 octet** |
| A4 | Hiérarchie KEK + Shamir | `c393f96` | Fichier créé, **0 octet** |
| A5 | Journal + herméticité RAM | `88f153e` | Journal modifié ; herméticité **non faite** |

---

## Ce qui est réellement implémenté

### `ak.rs` — la paire de clés d'accès

```rust
pub struct AccessPublicKey(pub [u8; 32]);   // sérialisable (annuaire, CRL)

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct AccessKeyPair {
    secret_bytes: [u8; 32],
    pub public: AccessPublicKey,
}
```

- Génération via `StaticSecret::random_from_rng(OsRng)` (`x25519-dalek` v2).
- `ZeroizeOnDrop` : la clé secrète est effacée de la RAM à la libération.
- `impl Debug` masqué → affiche `AccessKeyPair([REDACTED])`, jamais la clé.
- `diffie_hellman(&AccessPublicKey) -> [u8; 32]` pour l'échange de secret.
- 3 tests unitaires : génération, ECDH, herméticité du `Debug`.

### `journal.rs` — lecture paginée

Le journal chiffré gagne une lecture par plage, nécessaire à la réplication incrémentale :

```rust
pub fn read_range(&self, start_index: u64, limit: usize) -> Result<Vec<JournalEntry>>
pub fn read_all(&self) -> Result<Vec<JournalEntry>>   // = read_range(0, usize::MAX)
```

Les trames antérieures à `start_index` sont sautées par `seek` au lieu d'être allouées et
déchiffrées. Le format de trame reste `[u32 longueur ‖ blob chiffré]`.

---

## Ce que la spécification prévoit et qui reste à écrire

Ces éléments sont décrits en détail dans `architecture_V2_crypto.md` mais **absents du code** :

- **Registre de révocation (CRL)** — `HashSet<AccessPublicKey>` avec numéro de séquence
  monotone, recherche en O(1), propagation vers l'orchestrateur pour couper les échanges
  inter-nœuds.
- **Hiérarchie KEK** — troisième niveau de clé : `Données <- DEK <- KEK`, la KEK emballant la
  DEK (`kek.wrap_dek(&dek)`).
- **Partage de secret de Shamir, K=2 / N=3** — répartition prévue : part 1 imprimée au coffre
  de la PME, part 2 en blob chiffré sur le relais éditeur, part 3 clé physique du dirigeant.
  Deux parts sur trois suffisent à reconstruire.
- **Herméticité RAM du journal** — `impl Debug` masqué sur `JournalEntry` pour que le payload
  s'affiche `<N bytes>` au lieu de son contenu.

---

## À savoir avant de reprendre cette branche

Quatre points à traiter, du plus au moins urgent.

### 1. `ak.rs` n'est pas compilé

`spike/crates/ss-crypto/src/lib.rs` n'a pas été modifié et déclare toujours :

```rust
mod error;  mod dek;  mod device_key;  mod recovery;
```

Il manque `mod ak;` et l'export correspondant. En Rust, un fichier non déclaré par un `mod` est
purement ignoré par le compilateur : le code d'`ak.rs` est mort, et ses 3 tests ne s'exécutent
jamais. C'est aussi pourquoi la branche « compile » malgré les deux fichiers vides.

**À faire** : ajouter `mod ak;` et `pub use ak::{AccessKeyPair, AccessPublicKey};`.

### 2. Régression dans `count_frames()`

L'ancienne version lisait chaque blob avec `read_exact`, ce qui levait `Corrupted` si le
fichier était tronqué. La nouvelle saute la trame avec `seek(SeekFrom::Current(len))`, qui
**réussit silencieusement au-delà de la fin du fichier**.

Conséquence : une trame tronquée — typiquement un crash pendant une écriture — est comptée
comme valide. `next_index` est faussé et les écritures suivantes se posent sur un journal
incohérent. Pour un journal append-only qui sert de source de vérité à la réplication, c'est le
point le plus sérieux de la branche.

**À faire** : vérifier que la position après `seek` ne dépasse pas la taille du fichier.

### 3. Le test `debug_hermeticity` est rouge

Il attend `debug_str.contains("<5 bytes>")`, mais `JournalEntry` dérive toujours un `Debug`
standard qui affiche `payload: [1, 2, 3, 4, 5]`. Le test échouera tant que l'`impl Debug`
masqué prévu par l'étape A5 n'aura pas été écrit dans `entry.rs`.

### 4. Shamir maison contre la règle du projet

Le `CLAUDE.md` pose : **« aucune primitive cryptographique n'est réimplémentée à la main »**.
La spécification acte pourtant une implémentation Shamir maison sur GF(256) avec interpolation
de Lagrange. Le fichier étant encore vide, la décision reste réversible sans coût — des crates
éprouvées existent (`sharks`, `vsss-rs`).

**À trancher avant d'écrire la première ligne.**

### Points secondaires

- `diffie_hellman()` retourne le secret ECDH **brut**. Utiliser directement une sortie X25519
  comme clé symétrique est déconseillé ; `device_key.rs` fait correctement le travail en
  dérivant via BLAKE2b-512. `ak.rs` devrait suivre le même schéma.
- `secret_bytes()` retourne une copie de la clé secrète qui, elle, n'est ni `Zeroize` ni
  `ZeroizeOnDrop` — la garantie d'herméticité s'arrête à ce point de sortie.
- Deux tests de sécurité ont été supprimés du journal : `reopen_restores_state` (persistance
  après réouverture) et `wrong_dek_fails` (rejet d'une mauvaise clé). À restaurer.
- `architecture_V2_crypto.md` contient 5 liens absolus `file:///c:/Users/Siradiou/...` à
  nettoyer avant tout partage.

---

## Attention : deux copies du socle existent

La branche `feature-b-dev-integration` a **dupliqué `ss-crypto` et `ss-journal`** de
`spike/crates/` vers un nouveau dossier `core-rust/`. Cette branche-ci fait évoluer la copie
`spike/crates/`, l'autre travaille sur la copie `core-rust/`.

Les deux divergent déjà, et **git ne signalera jamais de conflit** puisque les chemins
diffèrent. Le `CLAUDE.md` impose « un seul cœur, jamais deux ». Il faut trancher lequel des deux
emplacements fait autorité avant que les deux branches ne soient fusionnées — c'est le risque
d'intégration numéro un du projet.

`spike/crates/` est le socle historique, référencé par `node/`, `relay/` et les Dockerfiles.

---

## Vérifier l'état de la branche

```bash
git checkout crypto

# Ce que la branche apporte réellement
git diff --stat main...crypto

# Confirmer que les deux fichiers annoncés sont vides
wc -c spike/crates/ss-crypto/src/crl.rs spike/crates/ss-crypto/src/shamir.rs

# Confirmer que ak.rs n'est pas déclaré
cat spike/crates/ss-crypto/src/lib.rs

cd spike && cargo build && cargo test
```

---

## Conventions

Commits : `feat` · `fix` · `refactor` · `docs` · `test` · `chore` · `perf` · `ci`.

Voir le [`README.md`](README.md) général du projet sur `main` pour l'architecture d'ensemble,
et [`CLAUDE.md`](CLAUDE.md) pour les décisions structurantes actées.
