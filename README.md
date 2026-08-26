# Branche `crypto` — Mission A : socle cryptographique V2

> Branche de travail de **Siradiou** · basée sur `main` (`ca80881`)
>
> Documentation de référence de la branche : [`architecture_V2_crypto.md`](architecture_V2_crypto.md)

---

## En une phrase

Cette branche prépare la **V2 du socle cryptographique** : séparer la clé qui donne *accès au cluster* (AK) de la clé qui *chiffre les données* (DEK), afin de pouvoir révoquer une machine en moins d'une seconde au lieu de rechiffrer tout le disque de la PME.

---

## Le problème que la V2 résout

En V1, une seule clé jouait deux rôles : identité réseau **et** chiffrement disque. Conséquence directe : dé-enrôler une machine — un salarié qui part, un portable volé — obligeait à faire tourner la DEK et donc à **rechiffrer l'intégralité des données**. Sur une PME réelle, cela signifie plusieurs minutes à plusieurs heures d'indisponibilité pour un événement administratif banal.

La V2 scinde les deux responsabilités :

| Clé | Rôle | Révocation |
|---|---|---|
| **AK** — Access Key (X25519) | Identité et accès réseau d'un appareil | Ajout à une liste de révocation (CRL), **immédiat** |
| **DEK** — Data Encryption Key | Chiffrement des données et du journal | Inchangée lors d'une révocation |

Révoquer une machine devient une écriture dans un registre, pas une réécriture de disque.

---

## Contenu réel de la branche

Toutes les étapes (A1 à A5) du socle cryptographique V2 sont **entièrement implémentées** dans le code Rust :

```
architecture_V2_crypto.md                 Spécification de la mission V2
spike/crates/ss-crypto/src/ak.rs          AccessKeyPair X25519 + AccessPublicKey (implémenté + tests)
spike/crates/ss-crypto/src/crl.rs         Registre de révocation CrlRegistry (implémenté + tests)
spike/crates/ss-crypto/src/shamir.rs      Hiérarchie KEK + Shamir Secret Sharing K=2/N=3 (implémenté + tests)
spike/crates/ss-crypto/src/lib.rs         Exposition des modules et structures V2 (mod ak, crl, shamir)
spike/crates/ss-journal/src/entry.rs      JournalEntry + impl Debug masqué pour l'herméticité RAM
spike/crates/ss-journal/src/journal.rs    Journal chiffré CBOR + lecture paginée (read_range) + tests
```

### État de réalisation du découpage V2

| Étape | Objet | Fichier principal | État dans le code |
|:---:|---|---|---|
| **A1** | DEK avec `Zeroize` | `spike/crates/ss-crypto/src/dek.rs` | **Implémenté** (`ZeroizeOnDrop`) |
| **A2** | `AccessKey` (AK) | `spike/crates/ss-crypto/src/ak.rs` | **Implémenté** (X25519, ECDH, `Debug` masqué) |
| **A3** | Registre de révocation CRL | `spike/crates/ss-crypto/src/crl.rs` | **Implémenté** (`CrlRegistry`, O(1), séquence monotone) |
| **A4** | Hiérarchie KEK + Shamir SSS | `spike/crates/ss-crypto/src/shamir.rs` | **Implémenté** (`Kek`, `wrap_dek`, `split_kek` K=2/N=3) |
| **A5** | Journal + Herméticité RAM | `spike/crates/ss-journal/src/entry.rs` | **Implémenté** (`read_range`, `fmt::Debug` masqué `<N bytes>`) |

---

## Ce qui est réellement implémenté (Détails)

### 1. `ak.rs` — Paire de clés d'accès (AK)
- **`AccessPublicKey`** : Clé publique X25519 de 32 octets, sérialisable via Serde pour l'annuaire d'enrôlement et le registre CRL.
- **`AccessKeyPair`** : Clé secrète X25519 protégée en mémoire avec `ZeroizeOnDrop`.
- **Herméticité RAM** : `impl Debug` masqué qui affiche `AccessKeyPair([REDACTED])`.
- **Échange ECDH** : Méthode `diffie_hellman(&AccessPublicKey)` pour le calcul de secret partagé entre nœuds.

### 2. `crl.rs` — Registre de révocation (CRL)
- **`CrlRegistry`** : Registre basé sur un `HashSet<[u8; 32]>` pour une vérification de révocation en $O(1)$.
- **Numéro de séquence monotone** (`sequence`) et horodatage UTC mis à jour à chaque révocation.
- Méthode `verify_access(&AccessPublicKey) -> Result<(), CryptoError>` retournant `CryptoError::RevokedAccessKey` si la clé est révoquée.

### 3. `shamir.rs` — Hiérarchie KEK & Shamir Secret Sharing (SSS)
- **`Kek` (Key Encryption Key)** : Clé de 256 bits (`ZeroizeOnDrop`) avec méthodes `wrap_dek(&Dek)` et `unwrap_dek(&blob)` pour emballer/déballer la DEK.
- **`ShamirShare`** : Fragment de secret avec identifiant d'abscisse `id` (1, 2 ou 3) et payload de 32 octets.
- **Arithmétique GF(256)** : Addition XOR, multiplication polynomiale AES/GF(256) et inversion d'Éléments par exponentiation Fermat ($a^{254}$).
- **Découpage & Reconstruction (K=2 / N=3)** :
  - `split_kek(&kek)` génère 3 parts.
  - `reconstruct_kek(&s1, &s2)` reconstitue la KEK originale à partir de **n'importe quelles 2 parts parmi 3** via interpolation de Lagrange à $x=0$.

### 4. `entry.rs` & `journal.rs` — Journal chiffré & Herméticité RAM
- **Lecture paginée** : `read_range(start_index, limit)` permettant la réplication incrémentale en sautant les trames via `seek`.
- **Herméticité RAM** : `impl Debug for JournalEntry` masquant le payload sous la forme `payload: <N bytes>`.

---

## Attention : Dualité des dépôts (`spike/crates/` vs `core-rust/`)

La branche `feature-b-dev-integration` a dupliqué `ss-crypto` et `ss-journal` de `spike/crates/` vers `core-rust/`.
- Cette branche fait évoluer la version autoritaire historique située dans **`spike/crates/`**.
- Le `CLAUDE.md` impose **« un seul cœur, jamais deux »**. Il faudra fusionner/nettoyer le dossier `core-rust/` lors de la réintégration sur `main`.

---

## 📋 Travaux non effectués / Améliorations restantes

Les éléments suivants n'ont pas encore été complètement traités et restent à réaliser avant de finaliser la branche :

### 1. Robustesse du `count_frames()` contre les trames tronquées
- **Fichier** : [`spike/crates/ss-journal/src/journal.rs`](file:///c:/Users/Siradiou/saas-souverain/spike/crates/ss-journal/src/journal.rs#L142-L161)
- **Problème** : `reader.seek(SeekFrom::Current(len))` ne valide pas si le saut dépasse la taille réelle du fichier lors d'un arrêt brutal pendant l'écriture d'une trame.
- **À faire** : Vérifier que la position dans le fichier après `seek` ne dépasse pas la fin effective du fichier pour éviter de valider une trame tronquée.

### 2. Dérivation KDF (HKDF / BLAKE2b) du secret Diffie-Hellman
- **Fichier** : [`spike/crates/ss-crypto/src/ak.rs`](file:///c:/Users/Siradiou/saas-souverain/spike/crates/ss-crypto/src/ak.rs#L67-L72)
- **Problème** : `diffie_hellman()` retourne directement les octets X25519 bruts.
- **À faire** : Appliquer une dérivation (ex: BLAKE2b-512 ou HKDF) sur la sortie ECDH avant de l'utiliser comme clé symétrique de session, comme le fait `device_key.rs`.

### 3. Effacement RAM de la clé secrète exportée
- **Fichier** : [`spike/crates/ss-crypto/src/ak.rs`](file:///c:/Users/Siradiou/saas-souverain/spike/crates/ss-crypto/src/ak.rs#L61-L64)
- **Problème** : `secret_bytes()` retourne une copie brute `[u8; 32]` sans wrapper `Zeroize`.
- **À faire** : Évaluer l'encapsulation de la clé secrète exportée dans un type dédié implémentant `ZeroizeOnDrop`.

### 4. Validations & Arbitrage Shamir (Primitive sur-mesure vs Crate auditée)
- **Fichier** : [`spike/crates/ss-crypto/src/shamir.rs`](file:///c:/Users/Siradiou/saas-souverain/spike/crates/ss-crypto/src/shamir.rs)
- **Problème** : Shamir GF(256) a été réimplémenté manuellement. Le `CLAUDE.md` mentionne la règle *« aucune primitive cryptographique n'est réimplémentée à la hand »*.
- **À faire** : Décider si l'implémentation GF(256) actuelle est conservée ou remplacée par une crate auditée de la communauté Rust (`sharks` ou `vsss-rs`).

### 5. Restauration des tests de sécurité historiques du journal
- **Fichier** : [`spike/crates/ss-journal/src/journal.rs`](file:///c:/Users/Siradiou/saas-souverain/spike/crates/ss-journal/src/journal.rs)
- **À faire** : Réintégrer les tests unitaires de persistance `reopen_restores_state` et d'invalidation de clé `wrong_dek_fails`.

### 6. Nettoyage de la documentation d'architecture
- **Fichier** : [`architecture_V2_crypto.md`](architecture_V2_crypto.md)
- **À faire** : Remplacer les 5 liens absolus Windows (`file:///c:/Users/Siradiou/...`) par des chemins relatifs.

---

## Conventions


Commits : `feat` · `fix` · `refactor` · `docs` · `test` · `chore` · `perf` · `ci`.

Voir le [README.md général du projet](https://github.com/mpigajesse/saas-souverain/blob/main/README.md) sur la branche `main` pour l'architecture d'ensemble,
et [`CLAUDE.md`](CLAUDE.md) pour les décisions structurantes actées.

