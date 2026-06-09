# Architecture de la solution — SaaS souverain à trois acteurs

**Framework de souveraineté des données pour logiciels métier en SaaS**

Ce document décrit l'architecture d'ensemble de la solution : les trois acteurs, leurs rôles, le parcours d'enrôlement d'une PME et le mécanisme de licence. Il complète la *Stack technique* et le *Plan de réalisation*.

> 🚨 **RÈGLE D'OR.** **Développer d'abord le SaaS côté éditeur (l'application web du SaaS).** C'est le point de départ du développement : la plateforme web (Django + React/TypeScript + PostgreSQL) qui gère les comptes tenants et les licences. Tout le reste vient ensuite.

---

## 1. Vue d'ensemble — trois acteurs

La solution repose sur trois composants distincts, aux périmètres de confiance séparés.

| Acteur | Hébergement | Voit le clair ? | Rôle |
| :--- | :--- | :--- | :--- |
| **Serveur 1 — SaaS éditeur** | Chez l'éditeur | Données de **compte / licence** uniquement | Comptes tenants, licences, suivi du parc. |
| **Serveur 2 — Relais zero-knowledge** | Chez l'éditeur | **Jamais** (pas la clé des données) | Stockage **chiffré** des données métier ; aide à la récupération sans lire (§3.1). |
| **Cluster PME** | Sur les machines de la PME | **Oui** (c'est son périmètre) | Exécute le logiciel métier, détient et sérialise les données métier. |

Le principe de souveraineté : **les données métier vivent dans le cluster de la PME** et ne sortent que **chiffrées** vers le relais. L'éditeur gère le commercial (comptes, licences) sans jamais accéder au contenu métier.

---

## 2. Serveur 1 — Le SaaS de l'éditeur

Application web centrale, cœur commercial et administratif.

| Élément | Choix |
| :--- | :--- |
| **Backend** | Django |
| **Frontend** | React + TypeScript |
| **Base de données** | PostgreSQL |

### 2.1 Modules de plateforme

- **Gestion des comptes tenants** — création et administration des comptes PME.
- **Gestion des licences logicielles** — souscription, nombre de postes autorisés, suivi du parc installé.

### 2.2 Modules métier

Pour la **version 1**, le module métier est la **Gestion de stock**. Les autres modules (facturation, paie, comptabilité…) viendront dans des versions ultérieures.

> **Emplacement.** La logique métier manipule des données métier, qui doivent rester **souveraines dans le cluster PME**. La Gestion de stock s'exécute donc **côté cluster PME**, pas sur le serveur SaaS. Le serveur SaaS reste la couche compte / licence / suivi.

### 2.3 Données vues par le SaaS

Le serveur SaaS ne stocke que des données de **plateforme** : identité du tenant (nom, adresse, téléphone), licence souscrite, et l'inventaire des appareils installés (cf. §5). **Aucune donnée métier en clair n'y transite.**

---

## 3. Serveur 2 — Le relais zero-knowledge

Le relais est hébergé par l'éditeur, mais il fonctionne en **zero-knowledge** : l'éditeur **stocke** les données chiffrées sans jamais pouvoir les **lire**.

| Élément | Détail |
| :--- | :--- |
| **Rôle** | Stocker les données métier des PME-tenants sous forme **chiffrée uniquement** (sauvegarde hors-site). |
| **Garantie** | **Zero-knowledge** : l'éditeur **n'a pas la clé de déchiffrement** des données métier. Même serveur compromis ou saisi, rien n'est lisible. |
| **Contenu** | Blobs opaques produits par le cluster (jamais en clair). |

Le relais n'est **jamais** dans le chemin d'une écriture métier : il reçoit du chiffré déjà produit par le cluster.

### 3.1 Récupération des données — modèle souverain (l'éditeur ne lit jamais)

Le besoin est réel : **si une PME perd toutes ses machines, elle doit pouvoir récupérer ses données.** La solution retenue permet cette récupération **sans donner à l'éditeur la capacité de lire les données** — c'est ce qui distingue ce produit d'un SaaS classique.

> **Principe clé.** Ce n'est pas l'éditeur qui détient le pouvoir de déchiffrer, c'est **la PME**, via un secret qu'elle seule connaît. L'éditeur ne stocke que des blobs qu'il ne peut pas ouvrir.

| Mécanisme | Fonctionnement | Garantie |
| :--- | :--- | :--- |
| **Code de récupération** (retenu) | À la création du compte, la PME reçoit un **code à haute entropie** (imprimé, « au coffre »). La DEK est emballée **sous ce code**. L'éditeur stocke ce blob chiffré mais **ne connaît pas le code**. | L'éditeur ne peut pas ouvrir le blob. Seule la PME, avec son code, déchiffre. |
| **Séquestre à parts (Shamir)** (option) | La clé de récupération est découpée en parts (ex. éditeur 1 part, PME 2 parts). Il faut **plusieurs parts réunies** pour reconstruire. | L'éditeur seul ne peut rien reconstruire. |

**Scénario « la PME a tout perdu » :**
1. La PME contacte l'éditeur.
2. L'éditeur lui restitue le **blob chiffré** stocké sur le relais (qu'il n'a jamais pu lire).
3. La PME ouvre le blob avec **son code de récupération** → récupère la DEK → redéchiffre ses données sur une nouvelle machine.

L'éditeur a **aidé** à la récupération (il conservait la sauvegarde) **sans jamais accéder** au contenu. La promesse zero-knowledge tient.

### 3.2 Remplacement d'une machine (panne, perte, vol)

Pleinement compatible avec le modèle souverain — l'éditeur ne touche pas aux données.

1. Une machine de la PME tombe en panne / est perdue.
2. La PME **contacte l'éditeur** pour mettre à jour son **abonnement** : retirer la machine concernée, en introduire une nouvelle.
3. Côté éditeur : mise à jour du **suivi de licence** (quelle machine compte dans l'abonnement) — sans aucun accès aux données métier.
4. Côté technique : **dé-enrôlement** de l'ancienne machine (**rotation de la DEK**, recalcul du quorum) puis **enrôlement** de la nouvelle (QR + sealed box de la DEK).

> La machine perdue détenait une copie de la clé : la **rotation de la DEK** est donc obligatoire pour qu'elle ne puisse plus déchiffrer les données futures.

---

## 4. Côté PME — Le cluster local

C'est là que vivent réellement les données métier.

### 4.1 Le livrable installé chez la PME

Le logiciel fourni par l'éditeur combine :
- le **logiciel métier** (la valeur d'usage pour la PME) ;
- les **couches SaaS** qui dialoguent avec le serveur central (licence, suivi) ;
- le tout **dockerisé** (image déployée sur chaque PC).

> **Docker — mode d'installation acté.** La distribution se fait par conteneur. Deux effets sont assumés explicitement :
> 1. **Découverte réseau locale** (mDNS / scan du §4.4) à configurer pour fonctionner depuis les conteneurs (réseau en mode `host` ou équivalent).
> 2. **Lecture de l'identité matérielle** masquée par le conteneur — sans conséquence ici, puisque l'identité de licence repose sur l'**identifiant d'installation**, pas sur la MAC (cf. §5.2).

### 4.2 Composition du cluster

La règle de dimensionnement déclarée à l'inscription :

> **nombre d'employés = nombre de PC = nombre de nœuds**

Exemple : une PME de 2 employés → 2 PC → 2 nœuds → un cluster de 2 nœuds.

### 4.3 Rôles des nœuds

À l'installation, la PME **désigne** :
- **un nœud actif** — sérialise les écritures, voit le clair, détient l'autorité ;
- **un ou plusieurs nœuds passifs** — répliques en lecture, prêts à prendre le relais.

> **Failover et quorum — règle actée.**
> - **2 nœuds → bascule manuelle uniquement.** Pas de majorité possible, donc pas de failover automatique sûr (risque de split-brain).
> - **≥ 3 nœuds → failover automatique** par quorum, en plus de la bascule manuelle.
>
> Le SaaS connaît le nombre de postes souscrits : il peut **signaler** à la PME si son cluster (ex. 2 nœuds) ne permet que la bascule manuelle.

### 4.4 Déroulé d'installation du cluster

L'installation part du PC actif et propage les autres nœuds depuis lui.

| Étape | Sur quel PC | Action |
| :--- | :--- | :--- |
| **1** | PC actif | La PME installe le logiciel sur le PC qu'elle choisit comme **nœud actif**. |
| **2** | PC actif | Le logiciel **scanne le réseau local** et affiche les machines détectées. |
| **3** | PC actif | La PME **choisit** les machines sur lesquelles installer (futurs passifs). |
| **4** | PC actif | Pour chaque machine choisie, le logiciel **génère un lien** de téléchargement **et de rattachement** au cluster. |
| **5** | PC passif | Le lien **télécharge le logiciel ET rattache la machine au cluster** — un seul geste. |

#### Le lien tout-en-un (téléchargement + enrôlement)

Le lien généré fait les deux à la fois : installer le logiciel sur le nouveau PC **et** le rattacher au cluster, sans étape manuelle séparée.

> 🔒 **Sécurité du lien — règle à respecter.** Le lien ne transporte **jamais la clé de chiffrement (DEK)**. Il porte un **jeton d'invitation à usage unique et à courte durée de vie**. Le déroulé réel :
> 1. Le nouveau PC télécharge le logiciel via le lien.
> 2. Il **génère sa propre paire de clés** et présente sa **clé publique** au nœud actif (via le jeton).
> 3. Le nœud actif **emballe la DEK** (sealed box) pour cette clé publique → le nouveau PC peut déchiffrer les données.
> 4. Le jeton est alors **consommé** (inutilisable une seconde fois) et expire automatiquement.
>
> Ainsi, un lien intercepté ou réutilisé est **sans valeur** : il ne contient aucune clé et ne fonctionne qu'une fois. Le geste reste unique pour l'utilisateur, la sécurité est préservée.

---

## 5. Mécanisme de licence et suivi du parc

C'est le lien de contrôle entre l'éditeur et chaque PME.

### 5.1 Principe

À l'installation, le serveur SaaS **vérifie le nombre d'appareils** ayant installé le logiciel et **enregistre leur identité**. Objectif : l'éditeur sait **combien** de machines et **lesquelles** ont installé le logiciel, pour contrôler le respect de la licence (N postes souscrits = N machines).

### 5.2 Identité de licence — décision actée

**Référence retenue : l'identifiant d'installation.** Chaque installation génère, au premier lancement, un **identifiant unique (UUID)** transmis de façon **authentifiée** au SaaS. C'est lui qui sert de référence pour le suivi du parc et le contrôle de licence. L'**adresse MAC** peut être enregistrée **en complément indicatif**, mais n'est jamais le verrou.

> **Pourquoi pas la MAC comme référence :**
> 1. **Falsifiable** — une adresse MAC se modifie en quelques manipulations ; ce n'est pas un verrou anti-fraude fiable.
> 2. **Masquée par Docker** — un conteneur voit une MAC virtuelle, pas celle de la carte réseau physique ; lire la vraie MAC exigerait une configuration réseau privilégiée, à rebours d'une installation simple.
>
> L'identifiant d'installation est **robuste et compatible Docker**, ce qui le rend cohérent avec le mode de distribution retenu.

---

## 6. Parcours complet d'enrôlement d'une PME

Du compte à l'exploitation, dans l'ordre :

| Étape | Acteur | Action |
| :--- | :--- | :--- |
| **1** | PME | Crée son **compte tenant** sur le web SaaS : nom de l'entreprise, adresse, téléphone, **nombre d'employés = nombre de PC**. |
| **2** | SaaS | Délivre une instruction : **installer Docker**. |
| **3** | PME | Télécharge le **logiciel dockerisé**. |
| **4** | PME | Installe le logiciel sur le PC choisi comme **nœud actif**. |
| **4b** | PC actif | **Scanne le réseau**, la PME choisit les autres machines ; un **lien tout-en-un** (téléchargement + rattachement) est généré pour chacune. |
| **4c** | PC passifs | Chaque machine ouvre son lien → **téléchargement + enrôlement** (jeton à usage unique, sealed box de la DEK). |
| **5** | SaaS | **Vérifie le nombre d'appareils** installés et **enregistre leur identité matérielle** → suivi du parc vs licence. |
| **6** | Cluster | Entre en exploitation : écritures sérialisées par l'actif, réplication vers les passifs, sauvegarde chiffrée vers le relais. |

---

## 7. Schéma logique des flux

```
                       ┌─────────────────────────────┐
                       │   SERVEUR 1 — SaaS éditeur   │
                       │   Django · React/TS · PG     │
                       │  • comptes tenants           │
                       │  • licences                  │
                       │  • suivi parc (identité      │
                       │    matérielle)               │
                       └──────────────┬──────────────┘
                                      │  compte / licence / suivi
                                      │  (PAS de données métier en clair)
                                      │
        ┌─────────────────────────────┴──────────────────────────────┐
        │                      CLUSTER PME                            │
        │                                                             │
        │     ┌───────────┐        réplication       ┌───────────┐   │
        │     │ NŒUD ACTIF│◀────────────────────────▶│  PASSIF   │   │
        │     │ (voit le  │        du journal         │ (lecture) │   │
        │     │  clair)   │                           │           │   │
        │     └─────┬─────┘                           └───────────┘   │
        │           │ données métier = ici, souveraines              │
        └───────────┼─────────────────────────────────────────────────┘
                    │  sauvegarde CHIFFRÉE uniquement
                    ▼
        ┌─────────────────────────────┐
        │ SERVEUR 2 — Relais zero-     │
        │ knowledge (ne voit jamais    │
        │ le clair, blobs chiffrés)    │
        └─────────────────────────────┘
```

---

## 8. Décisions actées

Les quatre points structurants sont tranchés et cohérents entre eux :

1. **Modules métier (§2.2)** — version 1 = **Gestion de stock**, exécutée **côté cluster PME**. Facturation, paie, comptabilité : versions ultérieures.
2. **Identité de licence (§5.2)** — **identifiant d'installation (UUID authentifié)** comme référence ; MAC en complément indicatif seulement.
3. **Taille des clusters et failover (§4.3)** — **2 nœuds → bascule manuelle** ; **≥ 3 nœuds → failover automatique** par quorum. Le SaaS le signale à la PME.
4. **Docker (§4.1)** — **mode d'installation confirmé**, avec ses deux effets assumés : découverte réseau locale à configurer, et lecture MAC masquée (sans impact, l'identité de licence reposant sur l'identifiant d'installation).

> Ces décisions n'empêchent pas de démarrer le développement du **SaaS côté éditeur** (l'application web : comptes tenants, licences), qui n'en dépend pas.
