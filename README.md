# SaaS Souverain

**Framework de souveraineté des données pour logiciels métier distribués.**

Les données métier d'une PME restent sur ses propres machines. Elles ne sortent du périmètre
de l'entreprise que **chiffrées**, sous une clé que l'éditeur ne détient pas. L'éditeur gère les
comptes et les licences ; il ne peut, à aucun moment et par aucun moyen technique, lire les
données de ses clients.

> Projet de fin d'études — EIGSI La Rochelle · Réalisé chez **AL BARAA CONSULTING**

---

## Sommaire

- [Le problème](#le-problème)
- [Architecture — trois acteurs](#architecture--trois-acteurs)
- [Arborescence du dépôt](#arborescence-du-dépôt)
- [Les composants en détail](#les-composants-en-détail)
- [Modèle cryptographique](#modèle-cryptographique)
- [Haute disponibilité et failover](#haute-disponibilité-et-failover)
- [Démarrage rapide](#démarrage-rapide)
- [Parcours d'une PME, de bout en bout](#parcours-dune-pme-de-bout-en-bout)
- [État d'avancement](#état-davancement)
- [Documentation](#documentation)

---

## Le problème

Un SaaS classique impose un arbitrage : soit la PME accepte que son éditeur héberge — et donc
puisse lire — son stock, sa facturation et sa paie, soit elle renonce au logiciel. Pour les
secteurs réglementés, les données sensibles ou simplement les entreprises soucieuses de leur
indépendance, cet arbitrage est inacceptable.

Ce framework le supprime : **l'exécution et le stockage du métier ont lieu chez le client**,
l'éditeur ne conserve que ce qui lui est strictement nécessaire (qui est client, combien de
postes) et un coffre de blobs qu'il est incapable d'ouvrir.

---

## Architecture — trois acteurs

| Acteur | Hébergement | Voit les données en clair ? | Rôle |
|---|---|:---:|---|
| **Serveur 1 — SaaS éditeur** | Chez l'éditeur | Compte / licence seulement | Comptes tenants, licences, suivi du parc installé |
| **Serveur 2 — Relais zero-knowledge** | Chez l'éditeur | **Jamais** | Annuaire des nœuds + stockage de blobs opaques |
| **Cluster PME** | Machines de la PME | **Oui** | Exécute le logiciel métier, détient et sérialise les données |

```
                    ┌──────────────────────────┐
                    │  SaaS éditeur (Django)   │   comptes, licences, parc
                    │  :8000                   │   <- jamais de données métier
                    └────────────┬─────────────┘
                                 │ token d'inscription
                                 │
   ┌─────────────────────────────┴─────────────────────────────┐
   │                                                           │
   │              ┌────────────────────────────┐               │
   │              │ Relais zero-knowledge      │               │
   │              │ ss-relay (Rust/axum) :8080 │               │
   │              │  · annuaire des nœuds      │               │
   │              │  · blobs chiffrés opaques  │               │
   │              └──────┬──────────────┬──────┘               │
   │      annonce /      │              │  annonce /           │
   │      découverte     │              │  découverte          │
   ├─────────────────────┼──────────────┼───────────────────────┤
   │  CLUSTER PME        │              │      (périmètre souverain)
   │                     ▼              ▼                       │
   │        ┌────────────────┐   ┌────────────────┐            │
   │        │ Nœud ACTIF     │   │ Nœud PASSIF    │            │
   │        │ ss-node :9001  │   │ ss-node :9001  │            │
   │        │ PostgreSQL     │──▶│ PostgreSQL     │            │
   │        │ (primaire)     │WAL│ (standby)      │            │
   │        └────────────────┘   └────────────────┘            │
   │           données EN CLAIR — ne sortent jamais d'ici       │
   └───────────────────────────────────────────────────────────┘
```

Ce que le relais voit passer : un identifiant de nœud, une adresse IP:port, un rôle, un numéro
d'époque, et des paquets d'octets chiffrés. Rien d'autre. Même saisi ou compromis, il ne
déchiffre rien.

---

## Arborescence du dépôt

```
SaaS souverain/
│
├── manage.py, config/            ── SaaS éditeur (Django 6)
│   ├── tenants/                     comptes PME + token d'inscription
│   ├── licenses/                    plans, nombre de postes, expiration
│   ├── devices/                     parc installé, rôles PG, état du failover
│   └── dashboard/                   vues de pilotage, architecture, relais
│
├── frontend/                     ── Interface éditeur (React + TypeScript + Vite)
│
├── spike/                        ── Cœur Rust (workspace Cargo)
│   ├── crates/ss-crypto/            DEK, clés appareil X25519, code de récupération
│   ├── crates/ss-journal/           journal append-only CBOR chiffré
│   ├── crates/ss-consensus/         époque, fencing, état du cluster, supervision
│   ├── node/                        binaire `ss-node` — logiciel métier PME
│   ├── relay/                       binaire `ss-relay` — relais zero-knowledge
│   ├── pg-config/                   init primaire + entrypoint standby PostgreSQL
│   └── docker-compose.yml           banc de test local
│
├── pme-deploy/                   ── Déploiement d'un nœud chez la PME
│   ├── docker-compose.yml           postgres + ss-node + pgAdmin
│   ├── setup-primary.sh             installe le premier nœud (actif)
│   ├── setup-standby.sh <IP>        rattache un nœud au primaire
│   └── .env.example
│
├── relay-deploy/                 ── Déploiement du relais chez l'éditeur
│
├── docs/livrables_soutenance/    ── Plan, slides et scénario de démonstration
└── tests/                        ── Guide de test métier (scénarios validés)
```

---

## Les composants en détail

### SaaS éditeur — Django 6 + PostgreSQL

Trois modèles portent tout le domaine :

- **`Tenant`** — la PME cliente. UUID, coordonnées, effectif, et un `registration_token`
  unique qui sert de sésame à l'installation. Expose `licence_status` : un tenant est
  conforme tant que ses appareils actifs ne dépassent pas les postes de sa licence.
- **`License`** — plan (`starter` / `pro` / `enterprise`), nombre de `seats`, dates de
  validité.
- **`Device`** — un nœud installé. Identifié par un **`installation_id` (UUID authentifié)**,
  jamais par son adresse MAC : une MAC est falsifiable et masquée par Docker. Le modèle suit
  aussi le `node_role` PostgreSQL (`primary` / `standby`), le `failover_count` et surtout
  `streaming_standby_count`, alimenté par `pg_stat_replication` — c'est la **source de vérité
  de la réplication**, pas une déclaration du nœud.

Routes : `/` (dashboard), `/tenants/`, `/licenses/`, `/devices/`, et une API REST miroir sous
`/api/…` consommée par le frontend React et par les nœuds.

### Nœud PME — binaire `ss-node` (Rust)

Une seule commande, six sous-commandes :

| Commande | Effet |
|---|---|
| `ss-node init [--first]` | Génère la paire X25519 du nœud. Avec `--first`, génère aussi la DEK et le code de récupération du cluster. |
| `ss-node run --mode active\|passive` | Démarre le nœud, s'annonce au relais, sert l'interface métier sur `:9001`. |
| `ss-node status` | État du nœud, de l'époque et du cluster. |
| `ss-node failover` | Promeut ce standby en primaire PostgreSQL et **incrémente l'époque**. |
| `ss-node delist --device-id <uuid>` | Dé-enrôle un appareil et déclenche la rotation de DEK. |
| `ss-node adduser …` | Crée un utilisateur métier (Argon2id). |

L'interface métier servie par le nœud (axum, rendu serveur) couvre l'authentification, un
module de stock (articles, catalogue, images produit) et une administration des utilisateurs.

### Relais zero-knowledge — binaire `ss-relay` (Rust/axum)

Quatre points d'entrée, volontairement minimalistes :

| Route | Rôle |
|---|---|
| `GET /health` | Sonde de vie et uptime |
| `POST /api/nodes/announce` | Un nœud publie `{node_id, tenant_id, addr, role, epoch}` |
| `GET /api/nodes` | Un nœud découvre ses pairs pour un tenant donné |
| `GET/PUT/DELETE /api/blobs/{tenant_id}/{key}` | Dépôt et retrait de blobs **opaques** |

C'est ce service qui remplace mDNS : la découverte réseau passe par une annonce HTTP au
démarrage, parce que Docker masque la topologie L2 et rend le multicast peu fiable.

### Les trois crates du cœur

- **`ss-crypto`** — `Dek` (XChaCha20-Poly1305), `DeviceKeyPair` / `DevicePublicKey` (X25519),
  dérivation du code de récupération (Argon2id).
- **`ss-journal`** — journal append-only. Chaque écriture est une opération CBOR sérialisée,
  chiffrée sous la DEK, puis ajoutée en trame `[u32 longueur ‖ blob chiffré]`.
- **`ss-consensus`** — `EpochToken`, `check_fencing`, `ClusterState` / `NodeRole`, supervision.

---

## Modèle cryptographique

```
DEK — clé symétrique, une par entreprise
 ├─ chiffre les données métier et le journal CBOR
 ├─ emballée en sealed box pour chaque appareil autorisé (X25519)
 └─ emballée sous le code de récupération (Argon2id) -> déposée chiffrée sur le relais
```

**Deux règles non négociables :**

1. Aucune primitive cryptographique n'est réimplémentée à la main.
2. Le relais ne détient ni la DEK, ni aucune clé privée.

### Enrôlement d'une machine

1. Le nouvel appareil génère sa paire X25519 et affiche sa clé publique.
2. Un appareil déjà autorisé la récupère et emballe la DEK en **sealed box** pour cette clé.
3. Le nouvel appareil ouvre le blob avec sa clé privée et obtient la DEK.
4. Le jeton d'invitation est **consommé** — usage unique, courte durée de vie.

### Récupération après perte de toutes les machines

1. La PME contacte l'éditeur.
2. L'éditeur lui restitue le blob chiffré conservé sur le relais — qu'il n'a jamais pu lire.
3. La PME l'ouvre avec **son code de récupération**, récupère la DEK, redéchiffre ses données
   sur une machine neuve.

L'éditeur assiste sans jamais accéder au contenu. C'est ce scénario qui prouve que la promesse
zero-knowledge tient jusqu'au cas limite.

---

## Haute disponibilité et failover

La réplication s'appuie sur le **streaming WAL natif de PostgreSQL** (`wal_level=replica`,
`synchronous_commit=remote_write`) — aucun algorithme de consensus n'est écrit à la main.

**Réplication choisie par opération :**

- **Synchrone** pour les invariants forts — stock, facturation, numérotation. Si aucun passif
  n'est joignable, l'écriture **bloque**. Jamais de dégradation silencieuse.
- **Asynchrone** pour les données tolérantes à la perte.

**Bascule selon la taille du cluster :**

| Nœuds | Comportement |
|:---:|---|
| 2 | Bascule **manuelle** uniquement — un quorum est impossible à départager |
| ≥ 3 | Failover **automatique** par quorum |

Le SaaS éditeur signale explicitement à la PME lorsque son cluster ne permet que la bascule
manuelle, et alerte au passage de 3 à 2 nœuds — le retrait d'une seule machine fait repasser
sous le seuil.

**Fencing.** Chaque promotion incrémente un **jeton d'époque monotone**. Un ancien primaire qui
revient après une panne présente une époque périmée : ses écritures sont rejetées et il est
isolé. C'est ce qui empêche le *split-brain* — deux nœuds se croyant simultanément primaires.

---

## Démarrage rapide

### Prérequis

- Python 3.11+ et PostgreSQL (SaaS éditeur)
- Node.js 18+ (frontend éditeur)
- Rust stable (compilation du cœur)
- Docker Engine ou Docker Desktop (nœuds PME et relais)

### 1 — SaaS éditeur

```bash
pip install -r requirements.txt
cp .env.example .env          # renseigner SECRET_KEY et la base
python manage.py migrate
python manage.py createsuperuser
python manage.py runserver 0.0.0.0:8000
```

Frontend éditeur :

```bash
cd frontend && npm install && npm run dev
```

### 2 — Relais zero-knowledge (chez l'éditeur)

```bash
cd relay-deploy
cp .env.example .env          # RELAY_AUTH_TOKEN, RELAY_NETWORKS
./setup.sh
```

Le relais écoute sur `:8080`. Vérification : `curl http://<IP_RELAIS>:8080/health`.

### 3 — Nœud PME

Premier nœud, celui qui crée le cluster :

```bash
cd pme-deploy
cp .env.example .env          # NODE_ADDR, RELAY_URL, SAAS_URL, REGISTRATION_TOKEN, mots de passe
./setup-primary.sh
```

Second nœud, rattaché au premier :

```bash
./setup-standby.sh <IP_DU_PRIMAIRE>
```

Chaque nœud expose ensuite :

- `http://<IP_NŒUD>:9001` — logiciel métier
- `http://<IP_NŒUD>:5050` — pgAdmin, données en clair (périmètre souverain uniquement)

> **`.env` ne doit jamais être commité.** Les fichiers `.env.example` listent toutes les
> variables attendues ; aucune valeur sensible n'est versionnée.

---

## Parcours d'une PME, de bout en bout

1. **Inscription** — la PME crée son compte sur le portail éditeur. Un `registration_token`
   unique lui est attribué.
2. **Souscription** — l'éditeur active une licence : plan et nombre de postes.
3. **Installation** — la PME récupère l'image Docker et le `docker-compose.yml`, renseigne son
   token, lance `setup-primary.sh`.
4. **Annonce** — au démarrage, le nœud s'enregistre auprès du relais et se déclare au SaaS avec
   son `installation_id`. Il apparaît dans le parc côté éditeur.
5. **Enrôlement** — chaque machine supplémentaire reçoit la DEK par sealed box depuis une
   machine déjà autorisée. Le SaaS vérifie que le nombre de postes reste dans la licence.
6. **Exploitation** — le métier tourne chez la PME. Le primaire réplique vers les standbys ; le
   SaaS affiche l'état du cluster en s'appuyant sur `pg_stat_replication`, pas sur une simple
   déclaration.
7. **Incident** — le primaire tombe. À 2 nœuds, l'opérateur lance `ss-node failover`. À 3 nœuds
   ou plus, le quorum promeut automatiquement. L'époque est incrémentée ; l'ancien primaire est
   fencé à son retour.

---

## État d'avancement

### Phase 0 — Spike de dérisquage

Validé sur un banc de machines réelles (Windows 11 · Ubuntu · Debian · Kali).

- [x] Chiffrement / déchiffrement DEK cross-OS
- [x] Journal append-only CBOR chiffré
- [x] Réplication streaming PostgreSQL actif -> passif
- [x] Bascule manuelle (2 machines)
- [x] Annonce de nœud au relais + découverte des pairs
- [x] Fencing par jeton d'époque
- [ ] Bascule automatique par quorum (3 machines)
- [ ] Dé-enrôlement + rotation DEK + alerte au passage 3 -> 2 nœuds
- [ ] Enrôlement par QR code

### Phase 1 — SaaS éditeur

Opérationnel : comptes tenants, licences, suivi du parc, inscription en ligne, génération de
l'installateur, tableau de bord de l'état des clusters.

### Phase 2 — Relais zero-knowledge

Opérationnel : annuaire des nœuds et stockage de blobs. Le durcissement (authentification des
dépôts, stockage objet S3-compatible, rendez-vous multi-sites) reste à faire.

### Phase 3 — Module métier v1

Gestion de stock exécutée côté cluster PME : articles, catalogue, images produit,
authentification et administration des utilisateurs.

### Hors périmètre à ce stade

Permissions fines entre opérateurs, découverte automatique complète, durcissement production,
audit de sécurité externe, modèle d'abonnement, mode dégradé complet.

---

## Règle de développement

> **Le socle d'abord, le métier ensuite.**

Le cœur cryptographique, la sérialisation du journal et le failover doivent être prouvés avant
qu'une logique métier ne soit bâtie dessus. Si le socle change après coup, tout ce qui repose
dessus est à réécrire.

Conventions de commit : `feat` · `fix` · `refactor` · `docs` · `test` · `chore` · `perf` · `ci`.

---

## Documentation

| Document | Contenu |
|---|---|
| [`CLAUDE.md`](CLAUDE.md) | Décisions structurantes actées, stack, phases |
| [`Architecture_SaaS_Souverain_Trois_Acteurs.md`](Architecture_SaaS_Souverain_Trois_Acteurs.md) | Le modèle à trois acteurs en détail |
| [`Stack_Technique_Framework_Souverainete_SaaS (1).md`](<Stack_Technique_Framework_Souverainete_SaaS (1).md>) | Justification des choix techniques |
| [`pme-deploy/COMPTES-ET-TESTS.md`](pme-deploy/COMPTES-ET-TESTS.md) | Comptes de test et procédures |
| [`tests/GUIDE-TEST-METIER-MPJ.md`](tests/GUIDE-TEST-METIER-MPJ.md) | Scénarios de test métier validés |
| [`docs/livrables_soutenance/`](docs/livrables_soutenance/) | Plan, slides et scénario de démonstration |
