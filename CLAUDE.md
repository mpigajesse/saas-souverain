# SaaS Souverain — CLAUDE.md

Framework de souveraineté des données pour logiciels métier distribués. Les données métier restent sur les machines du client (cluster PME) et ne sortent que chiffrées. L'éditeur gère comptes et licences sans jamais lire les données métier.

---

## 🚨 Règle d'or — à respecter absolument

**Le socle d'abord, le métier ensuite.**

Le cœur crypto (libsodium), la sérialisation des écritures (journal CBOR) et le failover (PostgreSQL primaire/standby + quorum) doivent être **prouvés par le spike Phase 0** avant qu'une seule ligne de logique métier soit écrite. Si le socle change après, tout le métier bâti dessus est à réécrire.

**⛔ Interdit** : écrire de la logique métier réelle (stock, facturation, paie) tant que le spike Phase 0 n'est pas validé.

**✅ Autorisé dès maintenant** : spike Phase 0, SaaS éditeur (comptes tenants + licences), spec technique.

---

## Architecture — trois acteurs

| Acteur | Hébergement | Voit le clair ? | Rôle |
| --- | --- | --- | --- |
| **Serveur 1 — SaaS éditeur** | Chez l'éditeur | Données compte/licence uniquement | Comptes tenants, licences, suivi du parc |
| **Serveur 2 — Relais zero-knowledge** | Chez l'éditeur | **Jamais** | Stockage chiffré des données métier (blobs opaques) |
| **Cluster PME** | Machines de la PME | **Oui** (périmètre souverain) | Exécute le logiciel métier, détient et sérialise les données |

---

## Stack technique

| Domaine | Choix acté |
| --- | --- |
| **Cœur partagé** | **Rust** — desktop + mobile via UniFFI (un seul cœur, jamais deux) |
| **Crypto** | libsodium : XChaCha20-Poly1305 (données/journal), X25519 (identité appareil), Argon2id (dérivation), sealed box (enrôlement) |
| **Base nœud actif** | **PostgreSQL** — réplication primaire/standby native, synchrone ou asynchrone selon l'opération |
| **Réplique locale passifs** | SQLite embarqué, lecture seule |
| **Journal** | Append-only, opérations **CBOR** (schéma minimal versionné), chiffré DEK avant écriture disque |
| **Consensus / failover** | Promotion de standby PostgreSQL (supervision type Patroni) + quorum ≥ 3 nœuds |
| **Fencing** | Jeton d'époque monotone + isolation du nœud déchu |
| **Découverte réseau** | **Annonce au relais éditeur** — les nœuds s'enregistrent au démarrage ; le relais maintient la topologie du cluster (metadata, pas données métier) |
| **Packaging** | **Image Docker multi-arch** (linux/amd64 + windows/amd64) distribuée à la PME. La PME installe Docker Desktop (ou le logiciel l'embarque). `docker-compose.yml` fourni avec le logiciel. |
| **Frontend métier** *(Phase 3, non acté)* | Tauri + React/TypeScript (desktop) · SwiftUI / Compose (mobile) |
| **SaaS éditeur** | Django + React/TypeScript + PostgreSQL |
| **Relais éditeur** | Service stateless (Rust/Go) + stockage objet S3-compatible (MinIO) |

---

## Phases de développement

### Phase 0 — Spike de dérisquage (PRIORITÉ ABSOLUE)

Prouver sur un banc de 3 machines réelles (Windows 11 · Ubuntu · Kali) :

- [ ] Chiffrement/déchiffrement DEK (libsodium) cross-OS
- [ ] Enrôlement par QR code (sealed box DEK vers clé publique appareil)
- [ ] Journal append-only CBOR chiffré
- [ ] Réplication synchrone actif → passif (PostgreSQL)
- [ ] Bascule manuelle (2 machines)
- [ ] Bascule automatique par quorum (3 machines)
- [ ] Fencing — retour de l'ancien actif bloqué par jeton d'époque
- [ ] Dé-enrôlement + rotation DEK + recalcul quorum + alerte 3 → 2 nœuds
- [ ] Annonce de nœud au relais éditeur + découverte des pairs via le relais

### Phase 1 — SaaS éditeur (peut démarrer en parallèle)

Application web Django + React/TypeScript + PostgreSQL :
- Gestion des comptes tenants (nom, adresse, téléphone, nb d'employés)
- Gestion des licences (souscription, nb de postes autorisés)
- Suivi du parc installé (identifiant d'installation UUID authentifié — **pas la MAC**)
- Inscription PME → instruction installation → téléchargement image Docker + docker-compose.yml

### Phase 2 — Relais zero-knowledge

Service stateless + stockage objet de blobs chiffrés + rendez-vous multi-sites.

### Phase 3 — Module métier v1 (APRÈS spike validé)

Gestion de stock exécutée **côté cluster PME**, pas sur le SaaS.

---

## Décisions structurantes actées

1. **Identité de licence** : identifiant d'installation (UUID authentifié). La MAC est en complément indicatif seulement — elle est falsifiable et masquée par Docker.
2. **Failover** : 2 nœuds → bascule manuelle uniquement ; ≥ 3 nœuds → failover automatique par quorum. Le SaaS signale à la PME si son cluster ne permet que la bascule manuelle.
3. **Docker pour le nœud PME** : le logiciel est distribué sous forme d'image Docker. La PME installe Docker Desktop (Windows) ou Docker Engine (Linux). Si Docker est absent au premier lancement, le logiciel détecte l'absence et guide l'installation (ou l'embarque en mode offline). Conséquence : mDNS supprimé — la découverte passe par le relais éditeur (annonce HTTP à chaque démarrage de nœud).
4. **Réplication par opération** : synchrone pour les invariants forts (stock, facturation, numérotation) ; asynchrone pour les données tolérantes à la perte. Le synchrone **bloque** si aucun passif n'est joignable — jamais de dégradation silencieuse.
5. **Crypto** : aucune primitive réimplémentée à la main. Tout passe par libsodium.
6. **Consensus** : aucun algorithme de consensus écrit à la main. On utilise la promotion de standby PostgreSQL + supervision type Patroni.

---

## Hiérarchie de clés

```
DEK (symétrique, unique par entreprise)
  └─ chiffre : données métier + journal CBOR
  └─ emballée via sealed box pour chaque appareil autorisé (X25519)
  └─ emballée sous code de récupération haute entropie (Argon2id) → stocké chiffré sur relais
```

Le relais ne voit jamais la DEK ni aucune clé privée. Même compromis ou saisi, il ne peut rien déchiffrer.

---

## Enrôlement d'une machine

1. Le nouvel appareil génère sa paire X25519 et affiche sa clé publique (QR code).
2. Un appareil déjà autorisé scanne le QR.
3. Il emballe la DEK (sealed box) pour la clé publique du nouvel appareil.
4. Le nouvel appareil ouvre le blob avec sa clé privée → récupère la DEK.
5. Le jeton d'invitation est **consommé** (usage unique, courte durée de vie).

---

## Récupération si la PME perd toutes ses machines

1. La PME contacte l'éditeur.
2. L'éditeur lui restitue le blob chiffré stocké sur le relais (qu'il n'a jamais pu lire).
3. La PME ouvre le blob avec **son code de récupération** → récupère la DEK → redéchiffre ses données sur une nouvelle machine.

L'éditeur aide sans jamais accéder au contenu. La promesse zero-knowledge tient.

---

## Points d'attention cross-OS (Phase 0)

- **Docker networking** : utiliser `--network host` sur Linux pour que les conteneurs voient le réseau LAN. Sur Windows Docker Desktop, utiliser un bridge nommé avec IPs fixes dans `docker-compose.yml`.
- **Détection Docker** : au premier lancement, tester `docker info` — si absent, afficher les instructions d'installation ou embarquer Docker dans le package.
- **Portabilité du cœur Rust** : chemins de fichiers, volumes Docker, services réseau — à valider sur Windows 11 dès Phase 0.
- **Quorum à exactement 3** : le retrait d'une seule machine fait repasser sous le seuil — le produit doit le détecter et le signaler.
- **PostgreSQL en conteneur** : volumes persistants pour les données PG ; la réplication streaming fonctionne normalement entre conteneurs sur le même bridge Docker ou réseau overlay.

---

## Ce qui est hors périmètre Phase 0

Logique métier réelle, interface soignée, permissions fines entre opérateurs, découverte automatique complète, durcissement production, audit sécurité, modèle d'abonnement, mode dégradé.
