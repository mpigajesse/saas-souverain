# 🛡️ AMANE V2 — Solution Consolidée (Missions A, B & C)

> **Branche d'intégration V2** : `main2`  
> **Documentation de référence** : [`architecture_V2_crypto.md`](architecture_V2_crypto.md) & [`AGENTS_IA_EXPLOITATION.md`](AGENTS_IA_EXPLOITATION.md)

---

## 📌 En une phrase

Cette branche réunit l'ensemble des 3 missions de stage R&D du framework **SaaS Souverain Amane V2** : le socle cryptographique d'herméticité RAM (Mission A), la couche logicielle applicative et hors-ligne (Mission B), ainsi que l'orchestrateur de cluster Go à haute disponibilité et réplication CRDT (Mission C).

---

## 🏗️ Architecture d'Ensemble des 3 Missions (A + B + C)

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

## 🔑 Détail des 3 Piliers de la V2

### 🛡️ 1. Mission A — Cybersécurité & Souveraineté Cryptographique (`spike/crates/`)
- **Séparation AK / DEK** : Clé d'accès réseau X25519 (`AccessKeyPair`) séparée de la clé de chiffrement disque (`Dek`).
- **Révocation Instantanée CRL (< 1s)** : Révocation d'une machine dé-enrôlée en $O(1)$ sans aucun rechiffrement de disque.
- **Récupération d'Urgence Shamir (SSS K=2 / N=3)** : Découpage zero-knowledge de la KEK sur $GF(256)$.
- **Sûreté RAM Stricte** : Réécriture mémoire RAM à `0x00` via `ZeroizeOnDrop` et masque `fmt::Debug` (`[REDACTED]`).

### 💻 2. Mission B — Développements & Intégration Logicielle (`core-rust/` & Web)
- **Invariants Métier (`invariants.rs`)** : Protection stricte contre les stocks négatifs et la rupture de séquence des factures.
- **Mode Hors-Ligne (`offline.rs`)** : Bascule dynamique en mode dégradé `GracefulReadOnly` lors d'une coupure réseau (lectures autorisées sur cache local SQLite chiffré, écritures bloquées).
- **Appairage d'Appareil (`pairing.rs`)** : Jeton d'invitation à durée limitée et scellage par Sealed Box X25519 (`DeviceKeyPair`).
- **Dashboard & Django Web** : Télémétrie PME et contrôle des nœuds actifs.

### ⚙️ 3. Mission C — System Design, Haute Disponibilité & Orchestration Go (`orchestrator-go/`)
- **Consensus & Élection Leader (`consensus/`)** : Campagne d'élection automatisée via lease etcd v3 avec Fencing anti-split-brain (`Write` gated).
- **Failover Automatique (< 5s) (`supervisor/`)** : Superviseur rapide avec heartbeat (2s) réduisant le temps de basculement à **3.2s mesurées**.
- **Réplication CRDT Delta Multi-Site (`replication/`)** : Compteur PN-CRDT avec fusion max commutative/associative/idempotente pour la tolérance aux pannes réseau.
- **Mesh Réseau WireGuard & mTLS 1.3 (`mesh/` & `grpcserver/`)** : Tunneling privé `wg0.conf` et sécurité mTLS gRPC renforcée.

---

## 🛠️ Démarrage Rapide

```bash
# 1. Démarrer l'infrastructure locale (etcd + PostgreSQL Patroni HA)
docker compose -f docker-compose.yml up -d

# 2. Compiler et exécuter l'orchestrateur Go
cd orchestrator-go
go build ./...
go test ./... -count=1

# 3. Exécuter les tests du client Rust (Missions A & B)
cd ../core-rust
cargo test --package ss-client --lib
```

---

## 📜 Conventions de Commits

`feat` · `fix` · `refactor` · `docs` · `test` · `chore` · `perf` · `ci`.
