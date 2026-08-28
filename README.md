# Branche `crypto` — Mission A : socle cryptographique V2

> Branche de travail de **Siradiou** · basée sur `main` (`ca80881`)  
> Documentation de référence de la branche : [`architecture_V2_crypto.md`](architecture_V2_crypto.md)

---

## 📌 En une phrase

Cette branche contient la **V2 complète du socle cryptographique (Mission A)** : séparation de la clé d'accès au cluster (AK) de la clé de chiffrement des données (DEK), révocation d'une machine en moins d'une seconde via CRL sans aucun rechiffrement de disque, hiérarchie KEK avec découpage de secret de Shamir (SSS N=3 / K=2), journal CBOR immuable et étanchéité mémoire RAM stricte.

---

## 🎯 Le problème que la V2 résout

En V1, une seule clé jouait deux rôles : identité réseau **et** chiffrement disque. Conséquence directe : dé-enrôler une machine — un salarié qui part, un portable volé — obligeait à faire tourner la DEK et donc à **rechiffrer l'intégralité des données**. Sur une PME réelle, cela signifie plusieurs minutes à plusieurs heures d'indisponibilité pour un événement administratif banal.

La V2 scinde les deux responsabilités :

| Clé | Rôle | Révocation |
|---|---|---|
| **AK** — Access Key (X25519) | Identité et accès réseau d'un appareil | Ajout à une liste de révocation (CRL), **immédiat (< 1s)** |
| **DEK** — Data Encryption Key | Chiffrement des données et du journal | Inchangée lors d'une révocation |

Révoquer une machine devient une écriture dans un registre, pas une réécriture de disque.

---

## 💻 Contenu réel de la branche

Toutes les étapes (A1 à A5) du socle cryptographique V2 ainsi que les **6 travaux de durcissement** sont **entièrement implémentés et validés** dans le code Rust :

```
architecture_V2_crypto.md                 Spécification et documentation de la mission V2
spike/crates/ss-crypto/src/ak.rs          AccessKeyPair X25519 + AccessPublicKey + KDF BLAKE2b-512 + SecretAccessKey
spike/crates/ss-crypto/src/crl.rs         Registre de révocation CrlRegistry (implémenté + tests)
spike/crates/ss-crypto/src/shamir.rs      Hiérarchie KEK + Shamir Secret Sharing K=2/N=3 (GF(256) + audit)
spike/crates/ss-crypto/src/lib.rs         Exposition des modules et structures V2 (mod ak, crl, shamir)
spike/crates/ss-journal/src/entry.rs      JournalEntry + impl Debug masqué pour l'herméticité RAM
spike/crates/ss-journal/src/journal.rs    Journal chiffré CBOR + count_frames() anti-tronquage + read_range + tests
```

### État de réalisation du découpage V2

| Étape | Objet | Fichier principal | État dans le code |
|:---:|---|---|---|
| **A1** | DEK avec `Zeroize` | `spike/crates/ss-crypto/src/dek.rs` | **100% Implémenté** (`ZeroizeOnDrop` + `Debug` masqué) |
| **A2** | `AccessKey` (AK) | `spike/crates/ss-crypto/src/ak.rs` | **100% Implémenté** (X25519, KDF BLAKE2b-512, `SecretAccessKey`) |
| **A3** | Registre de révocation CRL | `spike/crates/ss-crypto/src/crl.rs` | **100% Implémenté** (`CrlRegistry`, O(1), séquence monotone) |
| **A4** | Hiérarchie KEK + Shamir SSS | `spike/crates/ss-crypto/src/shamir.rs` | **100% Implémenté** (`Kek`, `wrap_dek`, `split_kek` K=2/N=3) |
| **A5** | Journal + Herméticité RAM | `spike/crates/ss-journal/src/entry.rs` | **100% Implémenté** (`read_range`, `count_frames()`, `fmt::Debug` masqué) |

---

## 🔍 Ce qui est réellement implémenté (Détails)

### 1. `ak.rs` — Paire de clés d'accès (AK) & Dérivation KDF
- **`AccessPublicKey`** : Clé publique X25519 de 32 octets, sérialisable via Serde pour l'annuaire d'enrôlement et le registre CRL.
- **`AccessKeyPair`** : Clé secrète X25519 protégée en mémoire avec `ZeroizeOnDrop`.
- **`SecretAccessKey`** : Structure wrapper dédiée réécrivant automatiquement ses octets à `0x00` à la sortie du scope.
- **Herméticité RAM** : `impl Debug` masqué qui affiche `AccessKeyPair([REDACTED])` et `SecretAccessKey([REDACTED])`.
- **Échange ECDH + BLAKE2b-512 KDF** : Méthode `diffie_hellman(&AccessPublicKey)` appliquant la dérivation KDF BLAKE2b-512 sur la sortie X25519 brute et les identités des nœuds.

### 2. `crl.rs` — Registre de révocation (CRL)
- **`CrlRegistry`** : Registre basé sur un `HashSet<[u8; 32]>` pour une vérification de révocation en $O(1)$.
- **Numéro de séquence monotone** (`sequence`) et horodatage UTC mis à jour à chaque révocation.
- Méthode `verify_access(&AccessPublicKey) -> Result<(), CryptoError>` retournant `CryptoError::RevokedAccessKey` si la clé est révoquée.

### 3. `shamir.rs` — Hiérarchie KEK & Shamir Secret Sharing (SSS)
- **`Kek` (Key Encryption Key)** : Clé de 256 bits (`ZeroizeOnDrop`) avec méthodes `wrap_dek(&Dek)` et `unwrap_dek(&blob)` pour emballer/déballer la DEK.
- **`ShamirShare`** : Fragment de secret avec identifiant d'abscisse `id` (1, 2 ou 3) et payload de 32 octets.
- **Arithmétique GF(256)** : Addition XOR, multiplication polynomiale AES/GF(256) et inversion d'Éléments par exponentiation Fermat ($a^{254}$).
- **Découpage & Reconstruction (K=2 / N=3)** : `split_kek(&kek)` génère 3 parts ; `reconstruct_kek(&s1, &s2)` reconstruit la KEK originale à partir de **n'importe quelles 2 parts parmi 3** via interpolation de Lagrange à $x=0$.
- **Audit de sécurité** : Documentation cryptographique complète en tête de module justifiant les calculs en temps constant.

### 4. `entry.rs` & `journal.rs` — Journal chiffré & Herméticité RAM
- **Protection Anti-Tronquage** : `count_frames()` vérifie que la position du curseur ne dépasse pas la longueur réelle du fichier (`file_len`) pour prévenir l'invalidation lors d'un arrêt brutal.
- **Lecture paginée** : `read_range(start_index, limit)` permettant la réplication incrémentale en sautant les trames via `seek`.
- **Herméticité RAM** : `impl Debug for JournalEntry` masquant le payload sous la forme `payload: <N bytes>`.
- **Tests historiques restaurés** : Inclusion des tests `reopen_restores_state` et `wrong_dek_fails`.

---

## ✅ Travaux de Durcissement & Finalisation Effectués (100% Complétés)

Tous les points de durcissement ont été traités avec succès et commités :

### 1. Robustesse du `count_frames()` contre les trames tronquées
- **Fichier** : [`spike/crates/ss-journal/src/journal.rs`](spike/crates/ss-journal/src/journal.rs)
- **Résolution** : Validation stricte de `pos + len <= file_len` dans `count_frames()` pour arrêter le comptage proprement si une trame est incomplète lors d'un arrêt système brutal.

### 2. Dérivation KDF (BLAKE2b-512) du secret Diffie-Hellman
- **Fichier** : [`spike/crates/ss-crypto/src/ak.rs`](spike/crates/ss-crypto/src/ak.rs)
- **Résolution** : Application de BLAKE2b-512 KDF sur le secret partagé X25519 brut et les clés publiques des deux nœuds dans `diffie_hellman()`.

### 3. Effacement RAM de la clé secrète exportée
- **Fichier** : [`spike/crates/ss-crypto/src/ak.rs`](spike/crates/ss-crypto/src/ak.rs)
- **Résolution** : Encapsulation dans la structure `SecretAccessKey([u8; 32])` dérivant `ZeroizeOnDrop` et `fmt::Debug` masqué.

### 4. Validations & Arbitrage Shamir
- **Fichier** : [`spike/crates/ss-crypto/src/shamir.rs`](spike/crates/ss-crypto/src/shamir.rs)
- **Résolution** : Documentation d'audit cryptographique complète rédigée en tête de module validant le calcul en temps constant sur GF(256).

### 5. Restauration des tests de sécurité historiques du journal
- **Fichier** : [`spike/crates/ss-journal/src/journal.rs`](spike/crates/ss-journal/src/journal.rs)
- **Résolution** : Réintégration complète des tests unitaires `reopen_restores_state` et `wrong_dek_fails`.

### 6. Nettoyage de la documentation d'architecture
- **Fichier** : [`architecture_V2_crypto.md`](architecture_V2_crypto.md)
- **Résolution** : Conversion des 5 liens absolus Windows en chemins relatifs Markdown standards.

---

## 🛠️ Attention : Dualité des dépôts (`spike/crates/` vs `core-rust/`)

La branche `feature-b-dev-integration` a dupliqué `ss-crypto` et `ss-journal` de `spike/crates/` vers `core-rust/`.
- Cette branche fait évoluer la version autoritaire historique située dans **`spike/crates/`**.
- Le `CLAUDE.md` impose **« un seul cœur, jamais deux »**. Il faudra fusionner/nettoyer le dossier `core-rust/` lors de la réintégration sur `main`.

---

## 📜 Conventions

Commits : `feat` · `fix` · `refactor` · `docs` · `test` · `chore` · `perf` · `ci`.

Voir le [README.md général du projet](https://github.com/mpigajesse/saas-souverain/blob/main/README.md) sur la branche `main` pour l'architecture d'ensemble, et [`CLAUDE.md`](CLAUDE.md) pour les décisions structurantes actées.
