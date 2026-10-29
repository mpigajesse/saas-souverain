# Architecture de la solution — AMANE V3

**Enveloppe d'exécution souveraine pour SaaS externes**

Ce document décrit l'architecture fonctionnelle d'AMANE V3 : les acteurs et leurs périmètres de confiance, ce qui est installé chez la PME, le parcours d'installation, la licence et la récupération après sinistre. Les décisions et leur justification sont consignées dans le journal de la refonte, [`docs/refonte-v3/AMANE-Refonte-V3.md`](docs/refonte-v3/AMANE-Refonte-V3.md) (ADR-001 à ADR-009) ; ce document en est la synthèse.

> **Ce qui a changé depuis la V1.** AMANE n'exécute plus son propre logiciel métier (l'ancien module « Gestion de stock » a été retiré) : il héberge **le SaaS d'un éditeur tiers**, sans modifier son code. Un quatrième acteur apparaît donc, l'éditeur de ce SaaS, et le zero-knowledge s'étend à lui.

---

## 1. Vue d'ensemble — quatre acteurs, une seule frontière

| Acteur | Hébergement | Voit le clair ? | Rôle |
| :--- | :--- | :--- | :--- |
| **Portail éditeur AMANE** | Chez l'éditeur AMANE | Non : comptes, licences, manifestes, métadonnées d'état | Tenants, licences, catalogue des applications hébergées, parc installé, génération de la pile et des installeurs |
| **Relais zero-knowledge** | Chez l'éditeur AMANE | **Jamais** | Rendez-vous des nœuds ; stockage des sauvegardes, chiffrées avant d'arriver |
| **Éditeur du SaaS externe** | Chez lui | **Jamais** | Livre son application en image Docker, épinglée par digest, avec un manifeste |
| **Cluster PME** | Machines de la PME | **Oui**, c'est son périmètre | Exécute l'application ; détient les données et la DEK |

Le principe de souveraineté : **les données vivent dans le cluster de la PME** et n'en sortent que **chiffrées**. Le code de l'éditeur tiers s'exécute à l'intérieur du cluster et manipule du clair : il est donc **confiné**, sans aucune sortie réseau (ADR-003, ADR-008).

---

## 2. Le portail éditeur AMANE

| Élément | Choix |
| :--- | :--- |
| Backend | Django (API REST seule, plus `/admin/`) |
| Frontend | React + TypeScript |
| Base | PostgreSQL |
| Registre d'images | Registre Docker de l'éditeur AMANE : pile PME et applications, par digest |

**Ce qu'il gère.**

- **Tenants et licences** : compte de la PME, nombre de postes, validité.
- **Catalogue des applications** (ADR-009) : une application est créée en brouillon avec son manifeste, **validée** si son image est publiée dans le registre (son manifeste est alors figé), puis **rattachée à la licence** des PME qui l'hébergent. Une application retirée disparaît des piles générées ensuite.
- **Parc installé** : chaque machine s'annonce avec son identifiant d'installation, son rôle, l'état du quorum, la santé de l'application et le chiffrement de son disque.
- **Pile et installeurs** : générés pour chaque PME, avec les images épinglées et le manifeste de son application.

**Ce qu'il ne voit jamais** : les données de la PME, la DEK, les mots de passe de ses bases (générés dans le cluster et partagés par etcd, ADR-005).

---

## 3. Le relais zero-knowledge

| Élément | Détail |
| :--- | :--- |
| Rôle | Point de rendez-vous (chaque nœud s'y annonce au démarrage et y découvre ses pairs) ; stockage des **sauvegardes chiffrées** |
| Garantie | Il ne détient aucune clé : un relais compromis ou saisi ne livre que des blobs opaques |
| Contenu | `pg_dump` de chaque base, chiffré par la DEK du cluster, rotation sur 7 jours (ADR-008) |

Le relais n'est jamais dans le chemin d'une écriture : il reçoit du chiffré déjà produit par le cluster.

---

## 4. Le cluster PME

### 4.1 Ce qui est installé

Chaque **serveur** exécute la pile générée par le portail :

| Service | Rôle |
| :--- | :--- |
| etcd | Consensus Raft : membership, élection, secrets partagés |
| PostgreSQL 17 / Patroni | Base répliquée ; Patroni promeut le standby en cas de panne |
| Orchestrateur Go | Amorçage, secrets, HAProxy, mode synchrone, rôle et base de l'application |
| HAProxy | Mène toujours au primaire |
| `ss-node` | Relaie l'interface de l'application ; console AMANE ; enrôlement ; sauvegardes ; garde de quorum |
| Application hébergée | L'image de l'éditeur tiers, inchangée, sur un réseau **interne sans sortie** |

Un **témoin** (etcd seul) sur une troisième machine, même modeste, complète le quorum : il ne stocke aucune donnée et ne consomme pas de poste de licence.

### 4.2 Comment l'application est hébergée (ADR-007)

- Son **interface** est relayée par le nœud à la racine de son port web ; la console AMANE passe sous `/amane/`.
- Sa **base** lui est dédiée : rôle et base `application` créés par l'orchestrateur, joints par HAProxy. Elle les reçoit par variables d'environnement (ex. `DATABASE_URL`), composées au démarrage par un lanceur qui ne modifie pas l'image.
- Son **réseau** ne mène qu'au nœud et à HAProxy : elle ne peut rien envoyer à son éditeur.

### 4.3 Haute disponibilité (ADR-001, ADR-004, ADR-006)

| Situation | Comportement |
| :--- | :--- |
| 2 serveurs + témoin, ou ≥ 3 serveurs | Bascule **automatique** par etcd + Patroni (quorum de trois membres etcd) ; panne détectée en moins de 5 s |
| ≥ 3 serveurs PostgreSQL | Réplication synchrone **stricte** : aucune écriture validée sans une seconde copie |
| 2 serveurs | Synchrone non strict : un serveur seul continue d'écrire, et la PME est **alertée** que ses données ne sont plus protégées |
| 2 serveurs sans témoin | Pas de quorum : la panne de l'un bloque les écritures ; le portail demande d'installer un témoin |
| Ancien primaire revenu | **Fencing** par la timeline PostgreSQL : il ne peut plus écrire |

> *Remplace la règle V1 « 2 nœuds → bascule manuelle uniquement » : avec un témoin, deux serveurs suffisent à une bascule automatique sûre.*

### 4.4 Clés

```
DEK (symétrique, une par cluster) — générée par le premier serveur
 ├─ chiffre le journal du nœud et les sauvegardes
 └─ scellée (sealed box X25519) pour chaque serveur enrôlé
```

Le socle `ss-crypto` fournit aussi la récupération d'urgence (code PME dérivé par Argon2id, découpage Shamir K=2/N=3 d'une KEK), mais **la pile Docker ne l'utilise pas encore** (§ 7.2).

Aucune primitive cryptographique maison (`ss-crypto` : RustCrypto, libsodium). Aucun composant hors du cluster — portail, relais, éditeur tiers, agent IA — ne touche la DEK. Le chiffrement **au repos** des données est délégué au chiffrement du disque (BitLocker, LUKS), vérifié par l'installeur et signalé à la PME (ADR-008).

---

## 5. Parcours d'installation d'une PME

| Étape | Acteur | Action |
| :--- | :--- | :--- |
| **1** | PME | Crée son compte sur le portail : entreprise et nombre d'employés |
| **2** | Éditeur AMANE | Délivre la licence et y rattache l'application validée que la PME doit héberger |
| **3** | PME | Télécharge depuis son espace l'installeur de chaque **serveur** et celui du **témoin** (Linux ou Windows) |
| **4** | Premier serveur | L'installeur vérifie Docker et le chiffrement du disque, lance la pile ; l'amorçage forme etcd, génère les secrets et la **DEK** |
| **5** | Serveurs suivants | Ils rejoignent etcd et la réplication. Ils démarrent **sans DEK** |
| **6** | Administrateur PME | **Enrôle** chaque serveur suivant : code à usage unique généré dans la console d'un serveur enrôlé, saisi sur le nouveau ; la DEK lui est scellée pour sa clé publique |
| **7** | Témoin | Installé sur une troisième machine, il complète le quorum |
| **8** | Portail | Enregistre chaque machine (identifiant d'installation) et contrôle le nombre de postes |
| **9** | Cluster | En exploitation : l'application sert les utilisateurs sur chaque serveur, la base est répliquée, une sauvegarde chiffrée part chaque jour vers le relais |

> *Remplace le « lien tout-en-un » et le scan du réseau de la V1 : la découverte passe par l'annonce au relais (Docker masque mDNS), et la transmission de la DEK reste un geste humain explicite, conformément à la règle « les opérations sur les clés restent humaines ».*

---

## 6. Licence et suivi du parc

À l'installation, chaque machine s'annonce au portail avec un **identifiant d'installation (UUID)** généré au premier lancement. C'est la seule référence de licence : l'**adresse MAC** n'est conservée qu'à titre indicatif (falsifiable, et masquée par Docker). Un serveur consomme un poste ; un témoin n'en consomme pas.

---

## 7. Récupération et remplacement

### 7.1 La PME a perdu un serveur

Les autres continuent (bascule automatique). Le nouveau serveur est installé avec le même installeur, puis **enrôlé**. La licence est mise à jour par l'éditeur AMANE, qui ne touche à aucune donnée.

### 7.2 La PME a perdu toutes ses machines

Le relais conserve les sauvegardes chiffrées des sept derniers jours, que l'éditeur AMANE ne peut pas lire. Pour les restaurer (`ss-node restaurer`), il faut la DEK.

> ⚠️ **Limite actuelle.** Dans la pile Docker, la DEK n'existe que sur les serveurs enrôlés : si **tous** sont perdus, les sauvegardes du relais sont **irrécupérables**. La cible : à la création du cluster, la DEK est emballée sous une clé que seule la PME détient (code de récupération Argon2id, ou parts Shamir), et ce blob est déposé sur le relais ; une installation neuve reconstitue la DEK à partir du secret de la PME. Les primitives existent dans `ss-crypto` ; leur intégration est le premier point ouvert du journal. Tant qu'elle manque, une PME doit garder au moins deux serveurs enrôlés.

---

## 8. Schéma des flux

```
                 ┌─────────────────────────────┐         ┌──────────────────────┐
                 │   Portail éditeur AMANE      │◀────────│ Éditeur du SaaS      │
                 │ tenants · licences ·         │  image  │ externe (image par   │
                 │ catalogue · parc · registre  │  + man. │ digest, manifeste)   │
                 └──────────────┬──────────────┘         └──────────────────────┘
          pile, installeurs ▼   ▲ métadonnées d'état (jamais de données)
   ┌────────────────────────────┴─────────────────────────────────────────┐
   │                          CLUSTER PME                                   │
   │   serveur 1                    serveur 2                  témoin       │
   │   application ◀─ nœud          application ◀─ nœud         etcd        │
   │   (sans sortie)  HAProxy       (sans sortie)  HAProxy                  │
   │   PostgreSQL ◀────── WAL ──────▶ PostgreSQL                            │
   │   etcd ◀──────────── Raft ─────▶ etcd ◀─────────────────▶ etcd         │
   └────────────────────────────┬─────────────────────────────────────────┘
                                │ sauvegardes CHIFFRÉES par la DEK
                                ▼
                 ┌─────────────────────────────┐
                 │  Relais zero-knowledge       │
                 │  (blobs opaques, aucune clé) │
                 └─────────────────────────────┘
```

---

## 9. Décisions structurantes

| Décision | Référence |
| :--- | :--- |
| HA par etcd + Patroni, pilotée par l'orchestrateur Go ; aucun consensus maison | ADR-001 |
| Docker d'abord ; binaires et `.exe` plus tard ; images épinglées par digest | ADR-002 |
| Zero-knowledge strict, étendu à l'éditeur du SaaS hébergé | ADR-003 |
| Témoin etcd ; synchrone strict dès trois serveurs ; fencing par la timeline | ADR-004 |
| Installation : amorçage etcd, secrets générés dans le cluster, HAProxy | ADR-005 |
| Réglages et cibles de bascule | ADR-006 |
| Hébergement de l'application : service déclaratif, reverse proxy, base dédiée, lanceur | ADR-007 |
| Confinement réseau, enrôlement par code, sauvegardes chiffrées, chiffrement du disque | ADR-008 |
| Catalogue des applications, validation contre le registre, rattachement à la licence | ADR-009 |
| Identité de licence = identifiant d'installation, jamais la MAC | § 6 |
