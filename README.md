# 🛡️ AMANE V2 — Framework SaaS Souverain Consolidé (Missions A, B & C)

> **Branche d'intégration V2** : `main2`  
> **Documentation de référence** : [`architecture_V2_crypto.md`](architecture_V2_crypto.md) & [`AGENTS_IA_EXPLOITATION.md`](AGENTS_IA_EXPLOITATION.md)

---

## 📌 1. Vue d'Ensemble & Périmètre Réalisé

La branche **`main2`** contient la **version V2 complète, consolidée et testée** du framework **SaaS Souverain Amane**. Elle réunit les apports des 3 missions de stage R&D :
- **Mission A (Cybersécurité & Souveraineté Cryptographique)** : Isolation RAM `ZeroizeOnDrop`, séparation AK/DEK, révocation CRL `< 1s` et récupération d'urgence Shamir SSS.
- **Mission B (Développements & Couche Logicielle)** : Encapsulation des invariants métier, mode hors-ligne dégradé SQLite (`GracefulReadOnly`), appairage et gestion des licences.
- **Mission C (System Design & Orchestration Go)** : Consensus etcd v3 (Raft), Fencing applicatif anti-split-brain, failover Patroni/Postgres `< 5s`, réplication multi-site CRDT et tunneling WireGuard mTLS 1.3.

---

## 🏗️ 2. Architecture d'Ensemble des 3 Missions (A + B + C)

```
                            ┌──────────────────────────────────────────┐
                            │    Mission B : Application & Client      │
                            │    - Invariants Métier (Stock, Factures) │
                            │    - Cache Offline SQLite (Read-Only)    │
                            └────────────────────┬─────────────────────┘
                                                 │
                                                 │ Contrat gRPC Protobuf
                                                 │ (framework.proto)
                                                 ▼
┌──────────────────────────────────┐        ┌──────────────────────────────────────────┐
│   Mission A : Socle Crypto V2    │        │  Mission C : Orchestrateur Go Cluster    │
│   - AK X25519 & DEK Symétrique   ├───────►│  - Consensus etcd v3 (Raft & Lease)      │
│   - Registre Révocation CRL <1s  │  CRL   │  - Failover Auto Patroni/Postgres <5s    │
│   - KEK & Shamir SSS (K=2/N=3)   │  <1s   │  - Réplication CRDT Delta Multi-Site     │
│   - Journal CBOR Append-Only     │        │  - Mesh Réseau WireGuard mTLS 1.3        │
└──────────────────────────────────┘        └──────────────────────────────────────────┘
```

---

## 🔗 3. Les 3 Contrats d'Interface d'Amane V2

| Interface | Missions Concernées | Rôle & Fonctionnement |
| :-: | :--- | :--- |
| **Interface 1** | **Crypto A ↔ Orchestrateur C** | Transmet la clé publique révoquée (`AccessPublicKey`) à l'orchestrateur via `NotifyRevocation`. Fermeture des sessions gRPC et recalcul du quorum en `< 1s`. |
| **Interface 2** | **Crypto A ↔ Dev B** | Fournit les primitives `dek.encrypt()`, `dek.decrypt()`, `kek.wrap_dek()` et le journal CBOR append-only pour l'étanchéité des écritures disque. |
| **Interface 3** | **Dev B ↔ Orchestrateur C** | Contrat gRPC unique d'autorité `framework.proto`. Les écritures `Write` du client Rust (B) sont gated par la lease de leader etcd du serveur Go (C). |

---

## 🔑 4. Détail des 3 Piliers R&D

### 🛡️ Mission A — Cybersécurité & Souveraineté Cryptographique (`spike/crates/`)
- **Séparation AK / DEK** : Clé d'accès réseau X25519 (`AccessKeyPair`) séparée de la clé de chiffrement disque (`Dek`).
- **Révocation Instantanée CRL (< 1s)** : Révocation d'une machine dé-enrôlée en $O(1)$ sans aucun rechiffrement de disque.
- **Récupération d'Urgence Shamir (SSS K=2 / N=3)** : Découpage zero-knowledge de la KEK sur $GF(256)$.
- **Sûreté RAM Stricte** : Réécriture mémoire RAM à `0x00` via `ZeroizeOnDrop` et masque `fmt::Debug` (`[REDACTED]`).

### 💻 Mission B — Développements & Intégration Logicielle (`core-rust/` & Web)
- **Invariants Métier (`invariants.rs`)** : Protection stricte contre les stocks négatifs et la rupture de séquence des factures.
- **Mode Hors-Ligne (`offline.rs`)** : Bascule dynamique en mode dégradé `GracefulReadOnly` lors d'une coupure réseau (lectures autorisées sur cache local SQLite chiffré, écritures bloquées).
- **Appairage d'Appareil (`pairing.rs`)** : Jeton d'invitation à durée limitée et scellage par Sealed Box X25519 (`DeviceKeyPair`).
- **Dashboard & Django Web** : Télémétrie PME et contrôle des nœuds actifs.

### ⚙️ Mission C — System Design, Haute Disponibilité & Orchestration Go (`orchestrator-go/`)
- **Consensus & Élection Leader (`consensus/`)** : Campagne d'élection automatisée via lease etcd v3 avec Fencing anti-split-brain (`Write` gated).
- **Failover Automatique (< 5s) (`supervisor/`)** : Superviseur rapide avec heartbeat (2s) réduisant le temps de basculement à **3.2s mesurées**.
- **Réplication CRDT Delta Multi-Site (`replication/`)** : Compteur PN-CRDT avec fusion max commutative/associative/idempotente pour la tolérance aux pannes réseau.
- **Mesh Réseau WireGuard & mTLS 1.3 (`mesh/` & `grpcserver/`)** : Tunneling privé `wg0.conf` et sécurité mTLS gRPC renforcée.

---

## 🛠️ 5. Guide Complet d'Initialisation & Commandes de Test

### 5.1. Démarrage de l'Infrastructure Locale (Docker Compose)

```bash
# 1. Démarrer etcd v3.5 + PostgreSQL HA Patroni/Spilo en arrière-plan
docker compose -f docker-compose.yml up -d

# 2. Vérifier l'état des conteneurs
docker compose ps
```

---

### 5.2. Compilation & Tests de l'Orchestrateur Go (Mission C)

```bash
cd orchestrator-go

# Compilation complète des packages Go
go build ./...

# Exécution des tests unitaires Go
go test ./... -count=1

# Exécution des tests d'intégration avec etcd réel
AMANE_TEST_ETCD=localhost:2379 go test ./consensus/... -v
```

---

### 5.3. Compilation & Tests des Crates Rust (Missions A & B)

```bash
cd core-rust

# 1. Tests des composants cryptographiques V2 (Mission A)
cargo test --package ss-crypto --lib

# 2. Tests du journal CBOR append-only (Mission A)
cargo test --package ss-journal --lib

# 3. Tests du client SDK, des invariants métier et du mode hors-ligne (Mission B)
cargo test --package ss-client --lib
```

---

### 5.4. Mesures de Haute Disponibilité & Failover (< 5s)

```bash
# 1. Mesure du Switchover contrôlé (~2,2 s)
bash scripts/switchover_measure.sh

# 2. Mesure du Failover automatique sur crash (SIGKILL -> Basculement ~3,2 s)
bash scripts/failover_measure.sh crash

# 3. Test de résilience sur partition réseau
bash scripts/failover_measure.sh partition
```

---

### 5.5. Régénération du Contrat gRPC Protobuf

Si vous modifiez le schéma d'autorité [`proto/amane/framework/v1/framework.proto`](proto/amane/framework/v1/framework.proto) :

```bash
# 1. Régénérer les fichiers Go (.pb.go) via Buf
buf generate

# 2. Régénérer les bindings Rust (tonic-build)
cd core-rust/ss-client
cargo build
```

---

## 📜 6. Conventions & Pratiques de Commits

 Commits : `feat` · `fix` · `refactor` · `docs` · `test` · `chore` · `perf` · `ci`.

Pour toute précision sur la gouvernance d'architecture : voir [`CLAUDE.md`](CLAUDE.md) et [`architecture_V2_crypto.md`](architecture_V2_crypto.md).
