# AMANE — enveloppe d'exécution souveraine pour SaaS externes

> **Branche V3** : `main3` (V2 : `main2`).
> **Journal de la refonte, décisions d'architecture (ADR-001 à ADR-009) et plan** : [`docs/refonte-v3/AMANE-Refonte-V3.md`](docs/refonte-v3/AMANE-Refonte-V3.md) (PDF à côté).

Un éditeur de SaaS confie son application à AMANE : son image Docker, son interface et sa couche métier restent **inchangées**. AMANE l'exécute dans le **cluster de la PME cliente**, sur ses propres machines, en haute disponibilité. Les données ne sortent du cluster que **chiffrées**, et **personne d'autre que la PME** ne les voit en clair : ni l'éditeur AMANE, ni le relais, ni l'éditeur du SaaS hébergé, dont l'application tourne **sans accès à Internet**.

---

## 1. Les acteurs

| Acteur | Où | Voit le clair ? | Rôle |
| :--- | :--- | :--- | :--- |
| **Portail éditeur AMANE** (Django + React) | Chez l'éditeur AMANE | Non : comptes, licences, manifestes, métadonnées d'état | Tenants, licences, catalogue des applications hébergées, génération de la pile et des installeurs |
| **Relais zero-knowledge** (Rust) | Chez l'éditeur AMANE | **Jamais** | Rendez-vous des nœuds, stockage des sauvegardes chiffrées |
| **Éditeur du SaaS externe** | Chez lui | **Jamais** | Livre une image Docker épinglée par digest et son manifeste |
| **Cluster PME** | Machines de la PME | **Oui**, c'est son périmètre | Exécute l'application, détient les données et la DEK |

## 2. Ce qui tourne sur une machine de la PME

Une **pile Docker générée par le portail** pour chaque PME (`templates/tenants/docker-compose.yml.tmpl`), toutes images épinglées par digest :

| Service | Rôle |
| :--- | :--- |
| `amane-etcd` | Consensus (Raft) : membership, élection, verrou de leader |
| `amane-postgresql` | PostgreSQL 17 piloté par Patroni (Spilo) ; réplication synchrone selon la taille du cluster |
| `amane-orchestrateur` | Orchestrateur Go : amorçage, secrets partagés par etcd, HAProxy, mode synchrone, base de l'application |
| `amane-haproxy` | Mène toujours au primaire PostgreSQL |
| `ss-node` | Nœud Rust : reverse proxy de l'application, console d'administration (`/amane/`), enrôlement, sauvegardes chiffrées, garde de quorum |
| `amane-application` | Le SaaS hébergé, seul sur un réseau Docker **interne sans sortie** |

Un **témoin** (etcd seul, sans données, sans poste de licence) sur une troisième machine permet la bascule automatique avec deux serveurs.

```
                 Portail éditeur AMANE ──── registre d'images (digests)
                    ▲ métadonnées d'état        │ pile + installeurs
                    │                           ▼
   ┌──────────────── Cluster PME ───────────────────────────────────┐
   │  serveur 1                 serveur 2                témoin      │
   │  ┌───────────────┐         ┌───────────────┐       ┌────────┐   │
   │  │ application   │         │ application   │       │  etcd  │   │
   │  │ (sans sortie) │         │ (sans sortie) │       └────────┘   │
   │  │ ss-node       │         │ ss-node       │                    │
   │  │ HAProxy       │         │ HAProxy       │                    │
   │  │ Patroni/PG 17 │◀──WAL──▶│ Patroni/PG 17 │                    │
   │  │ etcd · orch.  │         │ etcd · orch.  │                    │
   │  └───────────────┘         └───────────────┘                    │
   └───────────────────────────────┬─────────────────────────────────┘
                                   │ sauvegardes chiffrées par la DEK
                                   ▼
                         Relais zero-knowledge
```

## 3. Garanties

- **Confinement** : l'application hébergée n'a aucune route vers Internet ; seuls le nœud (son interface) et HAProxy (sa base) la joignent (ADR-008).
- **Clés** : la DEK est générée par le premier serveur et transmise aux suivants **scellée pour leur clé publique**, par un code à usage unique saisi par un administrateur (ADR-008). Aucune primitive cryptographique maison : tout passe par `ss-crypto` (RustCrypto, libsodium).
- **Sauvegardes** : `pg_dump` chiffré par la DEK, déposé sur le relais en blob opaque, rotation sur 7 jours, restauration par `ss-node restaurer`.
- **Chiffrement au repos** : délégué au chiffrement du disque (BitLocker, LUKS), vérifié par l'installeur et signalé à la PME.
- **Haute disponibilité** : etcd + Patroni, aucun consensus écrit à la main. Bascule automatique avec un quorum de trois membres etcd ; synchrone strict à partir de trois serveurs PostgreSQL ; fencing par la timeline PostgreSQL ; alerte au portail sous le seuil (ADR-004, ADR-006).
- **Images** : la PME exécute exactement ce qui a été validé (digest), et une application n'est validée que si son image est publiée dans le registre (ADR-009).

## 4. Carte du dépôt

| Chemin | Contenu |
| :--- | :--- |
| `config/`, `tenants/`, `licenses/`, `devices/`, `applications/`, `dashboard/`, `api/` | Portail éditeur Django (API REST seule, plus `/admin/`) |
| `templates/tenants/` | Gabarits de la pile PME (serveur, témoin) et des installeurs Linux et Windows |
| `frontend/` | Console React (Vite, React 19) — espaces éditeur et PME |
| `orchestrator-go/` | Orchestrateur : `init`, `temoin`, `lancer`, superviseur, HAProxy, synchrone, consensus, gRPC |
| `spike/node/`, `spike/relay/` | Nœud PME `ss-node` et relais `ss-relay` (Rust) |
| `spike/crates/` | Socle : `ss-crypto` (DEK, AK, sealed box, Shamir, CRL), `ss-journal` (journal CBOR chiffré), `ss-consensus` (garde de quorum) |
| `core-rust/ss-client/` | SDK client Rust (invariants, hors-ligne, client gRPC) |
| `proto/amane/framework/v1/` | Contrat gRPC, figé |
| `ha-lab/` | Laboratoire HA (trois machines simulées, scénarios de chaos) et application d'exemple |
| `relay-deploy/` | Déploiement du relais |
| `scripts/` | Publication de la pile (`publier-pile.sh`), mesures de bascule, ouverture du pare-feu Windows |

## 5. Démarrer

### Portail éditeur (développement)

```bash
docker compose up -d                 # etcd, Patroni, backend, frontend, Prometheus, Grafana
```

Ou hors conteneur, avec un `.env` à la racine (`SECRET_KEY`, `DB_NAME`, `DB_USER`, `DB_PASSWORD` au minimum) :

```bash
python manage.py migrate && python manage.py runserver
cd frontend && npm install && npm run dev
```

### Publier la pile et une application

```bash
bash scripts/publier-pile.sh         # images de la pile poussées et épinglées (media/node-images/pile.json)
```

L'image d'un SaaS externe est poussée dans le même registre, puis déclarée dans la console (**Applications**) : brouillon → validation contre le registre → rattachement à la licence de la PME.

### Installer une PME

La PME télécharge depuis son espace (`/bienvenue`) l'installeur Linux ou Windows de chaque **serveur**, et celui du **témoin**. Les serveurs suivants reçoivent la DEK par enrôlement : console du premier serveur, `/amane/admin/enrolement`, puis `ss-node enroler` sur le nouveau. Le détail des URL et des ports est dans [`ACCES-INTERFACES.md`](ACCES-INTERFACES.md).

## 6. Tests

```bash
python manage.py test                                    # portail
cd frontend && npm run lint && npm run build             # console
cd orchestrator-go && go test ./... -count=1 -race       # orchestrateur
cd spike && cargo test --workspace                       # socle, nœud, relais
cd core-rust && cargo test -p ss-client                  # SDK client
bash ha-lab/chaos.sh                                     # scénarios de panne (lourd : trois machines simulées)
```

Les tests d'intégration Go s'exécutent contre un etcd réel (`AMANE_TEST_ETCD`, ou le harnais `.ci/testcontainers`). Les commandes détaillées sont dans [`CLAUDE.md`](CLAUDE.md).

## 7. Documentation

| Document | Contenu |
| :--- | :--- |
| [`docs/refonte-v3/AMANE-Refonte-V3.md`](docs/refonte-v3/AMANE-Refonte-V3.md) | Journal de la refonte, ADR, points ouverts, procédure du banc |
| [`Architecture_SaaS_Souverain_Trois_Acteurs.md`](Architecture_SaaS_Souverain_Trois_Acteurs.md) | Architecture fonctionnelle : acteurs, installation, licence, récupération |
| [`CONTRAT-API.md`](CONTRAT-API.md) | Contrat de l'API REST du portail |
| [`ACCES-INTERFACES.md`](ACCES-INTERFACES.md) | Toutes les interfaces, URL et ports |
| [`AGENTS_IA_EXPLOITATION.md`](AGENTS_IA_EXPLOITATION.md) | Agents IA d'exploitation : proposer, jamais décider |
| [`ha-lab/README.md`](ha-lab/README.md) | Laboratoire HA |

Commits en français, préfixes `feat` · `fix` · `refactor` · `docs` · `test` · `chore` · `perf` · `ci`.
