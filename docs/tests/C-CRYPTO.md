# Groupe C — Socle cryptographique et journal (`C-01` à `C-06`)

Date : 2026-11-25. Dépôt : `/home/jose/school/amane/repo`, branche `main3`, HEAD `78014b9`.
Cible : workspace Rust `spike/crates/` (`ss-crypto`, `ss-journal`, `ss-consensus`).

## Synthèse des résultats

| Code | Intitulé | Cible | Attendu par le plan | Résultat réel | Registre | Preuve brute |
|---|---|---|---|---|---|---|
| **C-01** | Corruption d'une part Shamir interceptée par AEAD | `ss-crypto` (`shamir.rs`, `dek.rs`) | `Err(DecryptionFailed)` | `Err(DecryptionFailed)` | **PASS** | `docs/tests/artifacts-c01/c01.log` |
| **C-02** | Vecteurs RFC 7748 de petit ordre et point à l'infini | `ss-crypto` (`device_key.rs`) | Rejet, aucune DEK scellée avec secret tout-à-zéro | Aucun rejet : `seal_dek` scelle avec `shared = [0u8; 32]` | **FAIL** | `docs/tests/artifacts-c01/c02.log` |
| **C-03** | Troncatures (0/31/32/79) et altération de bit de la clé éphémère | `ss-crypto` (`device_key.rs`) | `Err(InvalidSealedBox)` ou `Err(DecryptionFailed)`, sans panic | `Err(InvalidSealedBox)` (0/31/32/79) et `Err(DecryptionFailed)` (bits altérés), sans panic | **PASS** | `docs/tests/artifacts-c01/c03.log` |
| **C-04** | Zéroïsation mémoire (`ZeroizeOnDrop`) inspectée en `unsafe` | `ss-crypto` (`dek.rs`, `shamir.rs`, `ak.rs`) | Tous les octets internes à `0x00` après `drop` | Tous les 32 octets de `Dek`, `Kek`, `ShamirShare` et `SecretAccessKey` à `0x00` | **PASS** | `docs/tests/artifacts-c01/c04.log` |
| **C-05** | Torn write (`len=256` + 40 octets) et DoS (`len=0xFFFFFFFF`) | `ss-journal` (`journal.rs`) | `JournalError::Corrupted` ou `count_frames` en erreur, sans OOM | Ni `Corrupted` ni erreur `count_frames` : trames tronquées silencieusement ignorées (`Ok`) | **FAIL** | `docs/tests/artifacts-c01/c05.log` |
| **C-06** | Régression d'époque (`epoch=4` après `epoch=5`) | `ss-journal` (`journal.rs`, `error.rs`) | `JournalError::EpochRegression` | Variante `EpochRegression` absente de `JournalError` ; `epoch=4` acceptée (`Ok(1)`) | **BLOQUE** | `docs/tests/artifacts-c01/c06.log` |

---

## Détail par test

### C-01 — Corruption part Shamir interceptée par AEAD (`PASS`)

- **Fichier et fonction** : `spike/crates/ss-crypto/src/lib.rs` (`tests_c01::c01_corruption_part_shamir_interceptee_par_aead`), testant `split_kek`, `reconstruct_kek`, `Kek::wrap_dek` et `Kek::unwrap_dek` (`spike/crates/ss-crypto/src/shamir.rs`).
- **Stimulus exact** :
  1. Génération d'une paire `Kek::generate()` et `Dek::generate()`, puis emballage de la DEK par `kek.wrap_dek(&dek)`.
  2. Découpage de la KEK en trois parts `split_kek(&kek)` (`s1`, `s2`, `s3`, seuil 2 sur 3).
  3. Altération d'un octet de la part `s2` (`s2.data[0] ^= 0x01`).
  4. Reconstruction d'une fausse KEK via `reconstruct_kek(&s1, &s2)`.
  5. Tentative de déballage de la DEK avec `faux_kek.unwrap_dek(&wrapped)`.
- **Assertion** :
  - La fausse KEK diffère de la KEK originale (`faux_kek.as_bytes() != kek.as_bytes()`).
  - `faux_kek.unwrap_dek(&wrapped)` retourne `Err(CryptoError::DecryptionFailed)`.
- **Résultat réel** :
  - `test tests_c01::c01_corruption_part_shamir_interceptee_par_aead ... ok`.
  - Le secret partagé de Shamir n'étant pas authentifié en lui-même (GF(256)), la reconstruction sur une part altérée produit une KEK erronée sans erreur mathématique, mais l'authentification AEAD (XChaCha20-Poly1305) de `unwrap_dek` intercepte immédiatement la fraude avec `Err(DecryptionFailed)`.
- **Commande de reproduction** :
  ```bash
  cd spike && cargo test -p ss-crypto tests_c01::c01 -- --nocapture
  ```

---

### C-02 — Vecteurs RFC 7748 de petit ordre et point à l'infini (`FAIL`)

- **Fichier et fonction** : `spike/crates/ss-crypto/src/lib.rs` (`tests_c01::c02_petit_ordre_rfc7748_seal_open`), testant `DevicePublicKey::seal_dek` et `DeviceKeyPair::open_sealed_dek` (`spike/crates/ss-crypto/src/device_key.rs`).
- **Stimulus exact** :
  1. Passage des 7 vecteurs de petit ordre de la RFC 7748 § 5.2 à `DevicePublicKey::from_bytes(v).seal_dek(&dek)` :
     - Point à l'infini (`0x00...00`, ordre 1)
     - `0x01 00...00` (ordre 8)
     - `e0eb7a7c3b41b8ae1656e3faf19fc46ada098deb9c32b1fd866205165f49b800` (ordre 8)
     - `5f9c95bca3508c24b1d0b1559c83ef5b04445cc4581c8e86d8224eddd09f1157` (ordre 8)
     - `ecffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7f` ($p-1$, ordre 2)
     - `edffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7f` ($p$, ordre 4)
     - `eeffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7f` ($p+1$, ordre 4)
  2. Tentative d'ouverture de la DEK scellée avec `DeviceKeyPair::from_secret_bytes([0u8; 32])`.
  3. Tentative d'ouverture d'un sealed box dont les 32 premiers octets (`eph_public`) sont remplacés par le point à l'infini (`[0u8; 32]`).
- **Assertion attendue par le plan** :
  - Les fonctions `seal_dek` et `open_sealed_dek` doivent rejeter explicitement ces entrées (pas de scellement avec un secret partagé X25519 tout-à-zéro).
- **Résultat réel** :
  - `test tests_c01::c02_petit_ordre_rfc7748_seal_open ... FAILED`.
  - `device_key.rs` appelle `eph_secret.diffie_hellman(&recipient_public)` et `my_secret.diffie_hellman(&eph_public)` sans jamais vérifier `shared.was_contributory()` ni filtrer les points de petit ordre.
  - Pour les 7 vecteurs (y compris le point à l'infini `[0u8; 32]`), `seal_dek` retourne `Ok(sealed)` (80 octets) en dérivant la clé XChaCha20-Poly1305 sur un secret partagé `shared = [0u8; 32]`, et `open_sealed_dek` déchiffre sans aucun rejet.
- **Commande de reproduction** :
  ```bash
  cd spike && cargo test -p ss-crypto tests_c01::c02 -- --nocapture
  ```

---

### C-03 — Sealed box tronquée et altération de la clé éphémère (`PASS`)

- **Fichier et fonction** : `spike/crates/ss-crypto/src/lib.rs` (`tests_c01::c03_sealed_box_tronquee_et_bit_ephemere`), testant `DevicePublicKey::seal_dek` et `DeviceKeyPair::open_sealed_dek`.
- **Stimulus exact** :
  1. Scellement d'une DEK de 32 octets produisant un blob `sealed` de 80 octets (`32` octets de clé publique éphémère + `48` octets de ciphertext et tag Poly1305).
  2. Troncatures du sealed box aux longueurs `0`, `31`, `32` et `79` octets passées à `open_sealed_dek`.
  3. Altération d'un bit sur la clé publique éphémère : `sealed[0] ^= 0x01` puis `sealed[31] ^= 0x80`.
- **Assertion** :
  - Pour `0`, `31`, `32`, `79` octets : `Err(CryptoError::InvalidSealedBox)`.
  - Pour les bits altérés dans la clé éphémère : `Err(CryptoError::DecryptionFailed)`.
  - Aucun `panic` (pas de `unwrap()` ou d'indexation hors bornes).
- **Résultat réel** :
  - `test tests_c01::c03_sealed_box_tronquee_et_bit_ephemere ... ok`.
  - La garde `if sealed.len() < SEALED_MIN_LEN` (`SEALED_MIN_LEN = 80`) intercepte toutes les troncatures avec `Err(InvalidSealedBox)`, et l'altération de la clé éphémère modifie le secret ECDH dérivé par BLAKE2b-512, provoquant un échec d'authentification AEAD `Err(DecryptionFailed)`.
- **Commande de reproduction** :
  ```bash
  cd spike && cargo test -p ss-crypto tests_c01::c03 -- --nocapture
  ```

---

### C-04 — Zéroïsation des secrets en mémoire après `drop` (`PASS`)

- **Fichier et fonction** : `spike/crates/ss-crypto/src/lib.rs` (`tests_c01::c04_zeroisation_apres_drop_inspection_unsafe`), testant `Dek`, `Kek`, `ShamirShare` et `SecretAccessKey`.
- **Stimulus exact** :
  1. Instanciation de `Dek::from_bytes([0xAA; 32])`, `Kek::from_bytes([0xBB; 32])`, `ShamirShare { id: 1, data: [0xCC; 32] }` et `SecretAccessKey::from_bytes([0xDD; 32])` confinés par `Box::into_raw`.
  2. Récupération d'un pointeur brut `*const u8` sur les 32 octets internes de chaque structure.
  3. Appel explicite à `std::ptr::drop_in_place(ptr)` pour forcer l'exécution du destructeur `Drop` en place sans désallouer le bloc mémoire.
  4. Lecture volatile en `unsafe` des 32 octets via `ptr.add(i).read_volatile()`.
- **Assertion** :
  - Avant `drop_in_place`, le premier octet est non nul (`0xAA`, `0xBB`, `0xCC`, `0xDD`).
  - Après `drop_in_place`, la totalité des 32 octets de chaque structure vaut `0x00`.
- **Résultat réel** :
  - `test tests_c01::c04_zeroisation_apres_drop_inspection_unsafe ... ok`.
  - Les 128 octets inspectés (4 structures × 32 octets) valent tous `0x00` suite à l'invocation de `ZeroizeOnDrop::drop`.
- **Commande de reproduction** :
  ```bash
  cd spike && cargo test -p ss-crypto tests_c01::c04 -- --nocapture
  ```

---

### C-05 — Torn write et en-tête DoS dans le journal (`FAIL`)

- **Fichier et fonction** : `spike/crates/ss-journal/src/lib.rs` (`tests_c01::c05_journal_torn_write_et_en_tete_dos`), testant `Journal::open` (`count_frames`) et `Journal::read_all` (`read_range`) dans `spike/crates/ss-journal/src/journal.rs`.
- **Stimulus exact** :
  1. **Torn write** : fichier binaire contenant un en-tête `256u32.to_le_bytes()` suivi de seulement 40 octets de données (et variante avec 1 frame valide suivie de ce fragment tronqué).
  2. **DoS par longueur massive** : fichier binaire contenant un en-tête `0xFFFFFFFFu32.to_le_bytes()` suivi de 40 octets.
- **Assertion attendue par le plan** :
  - `JournalError::Corrupted` ou erreur retournée par `count_frames`, sans OOM.
- **Résultat réel** :
  - `test tests_c01::c05_journal_torn_write_et_en_tete_dos ... FAILED`.
  - Dans `count_frames` (`journal.rs` l.162-165), la condition `if pos + len > file_len { break; }` stoppe le comptage silencieusement et retourne `Ok(0)` (ou `Ok(1)`), sans erreur.
  - Dans `read_range` (`journal.rs` l.113-117), `let mut blob = vec![0u8; len];` alloue un tampon de `0xFFFFFFFF` octets (4 Gio), puis `read_exact` rencontre `UnexpectedEof` et exécute `=> break`, retournant `Ok(vec![])` sans jamais produire `JournalError::Corrupted`.
- **Commande de reproduction** :
  ```bash
  cd spike && cargo test -p ss-journal tests_c01::c05 -- --nocapture
  ```

---

### C-06 — Régression d'époque (`epoch=4` après `epoch=5`) (`BLOQUE`)

- **Fichier et fonction attendus** : `spike/crates/ss-journal/src/lib.rs` (`tests_c01::c06_regression_epoch_apres_epoch_superieur`), `spike/crates/ss-journal/src/journal.rs` et `spike/crates/ss-journal/src/error.rs`.
- **Motif de blocage** :
  - La variante `JournalError::EpochRegression` **n'existe pas** et n'est pas exposée dans l'énumération `ss_journal::JournalError` (qui ne contient que `Io`, `Crypto`, `Cbor` et `Corrupted { index: u64 }`).
  - La structure `Journal` ne mémorise pas l'époque de la dernière entrée et `Journal::append(5, ...)` suivi de `Journal::append(4, ...)` est accepté sans contrôle (`Ok(0)` puis `Ok(1)`, et `read_all()` retourne deux entrées avec `epoch = 5` puis `epoch = 4`).
- **Assertion attendue par le plan** :
  - `Err(JournalError::EpochRegression)`.
- **Résultat réel** :
  - Conformément à la consigne du brief (*« Si un test ne peut pas être écrit parce que la fonction n'existe pas ou n'est pas exposée, le dire et le noter `BLOQUE` dans le registre »*), le test est noté **BLOQUE** dans `docs/tests/REGISTRE.md`.
- **Commande de reproduction** :
  ```bash
  cd spike && cargo test -p ss-journal tests_c01::c06 -- --nocapture
  ```