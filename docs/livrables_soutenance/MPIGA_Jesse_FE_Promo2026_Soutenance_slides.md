# Support de Soutenance — Slides
<!-- NOM FICHIER MOODLE : MPIGA_Jesse_FE_Promo2026_Soutenance.ppt -->
<!-- DEADLINE DEPOT MOODLE : 30/06/2026 (veille de soutenance) -->
<!-- DUREE CIBLE : 30 minutes (27–33 min tolérés) -->
<!-- FORMAT : PowerPoint .ppt — max 8 Mo -->
<!-- THÈME ACTUEL : Framework SaaS Souverain pour logiciels métier distribués -->

---
<!-- ═══════════════════════════════════════════════════════════ -->
<!-- SLIDE 1 — PAGE DE GARDE -->
<!-- ═══════════════════════════════════════════════════════════ -->

# SLIDE 1 — Page de Garde

**[Design : fond sombre #0D0A07, motif zellige subtil en transparence, accent doré #C79A1B]**

---

**[LOGO EIGSI — haut gauche]** &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp; **[LOGO AL BARAA CONSULTING — haut droite]**

---

## 🔐 Framework SaaS Souverain

### Conception et Implémentation d'un Framework SaaS Souverain
### pour Logiciels Métier Distribués

---

**Expérience Professionnelle de Fin d'Études**
EIGSI Casablanca — Spécialité Big Data & Intelligence Artificielle
**Promotion 2026**

---

| | |
|---|---|
| **Étudiant** | Jesse MPIGA-ODOUMBA |
| **Encadrante entreprise** | Mme Soumia CHOKRI — AL BARAA CONSULTING |
| **Tuteur EIGSI** | M. Ayoub AMRANI |
| **Soutenance** | 01 juillet 2026 — EIGSI Casablanca |

---

*Notes présentateur :*
> Sourire, regarder le jury. Pause 3 secondes. Commencer par : "Mesdames, Messieurs, je vous remercie de m'accorder ce temps pour vous présenter mon projet de fin d'études : la conception d'un framework qui permet à un éditeur de vendre un logiciel métier en mode SaaS, sans jamais voir les données de ses clients."

---
<!-- ═══════════════════════════════════════════════════════════ -->
<!-- SLIDE 2 — PLAN DE LA PRÉSENTATION -->
<!-- ═══════════════════════════════════════════════════════════ -->

# SLIDE 2 — Plan de la Présentation

**[Design : fond sombre, titre en doré, bullets avec icônes]**

---

## Plan

| # | Partie | ⏱ |
|---|--------|---|
| 1 | AL BARAA CONSULTING & contexte du stage | 2 min |
| 2 | Problématique : la souveraineté des données métier | 3 min |
| 3 | La solution : un framework à trois acteurs + **modèle économique** | 4 min |
| 4 | Architecture & garantie zero-knowledge | 3 min |
| 5 | Le cœur technique : réplication, résilience & **fencing** | 4 min |
| 6 | **Démonstration live** — cluster PME 2 nœuds | 5 min |
| 7 | **Supervision intelligente — agent IA (3 acteurs)** | 3 min |
| 8 | Difficultés & posture ingénieur | 3 min |
| 9 | Bilan, compétences & perspectives | 3 min |

---

*Notes présentateur :*
> "La présentation suit le fil du projet : d'abord le problème de souveraineté, puis la solution architecturale, le cœur technique que j'ai prouvé sur un banc réel, une démonstration live, et enfin la dimension intelligence artificielle qui supervise le système — au cœur de ma spécialité."

---
<!-- ═══════════════════════════════════════════════════════════ -->
<!-- SLIDE 3 — AL BARAA CONSULTING -->
<!-- ═══════════════════════════════════════════════════════════ -->

# SLIDE 3 — AL BARAA CONSULTING

**[Design : fond sombre, logo AL BARAA prominent, données clés en cartes]**

---

**[LOGO AL BARAA CONSULTING — centré, grand]**

---

## Un cabinet de conseil en ingénierie numérique

| 📅 Fondé | Mars 2017 |
|---------|-----------|
| 📍 Siège | Ain Sebaa, Casablanca |
| ⚖️ Statut | SARL AU — 100 000 MAD |
| 👤 DG | Mme Soumia CHOKRI |
| 🎯 Missions | Développement logiciel, Architecture SI, Transformation numérique |
| 🤝 Clients | Secteur public & privé (B2B) |

---

### Ma position dans l'entreprise

> Développeur & Architecte principal du projet — **responsabilité totale** sur l'ensemble du cycle (conception → développement → déploiement → validation).

---

*Notes présentateur :*
> "AL BARAA CONSULTING est un cabinet à taille humaine, ce qui m'a placé en situation de forte responsabilité dès le premier jour : une mission d'ingénieur avec une autonomie réelle, pas des tâches d'exécution."

---
<!-- ═══════════════════════════════════════════════════════════ -->
<!-- SLIDE 4 — PROBLÉMATIQUE -->
<!-- ═══════════════════════════════════════════════════════════ -->

# SLIDE 4 — La Problématique

**[Design : fond sombre, carte de l'Afrique en arrière-plan transparent, couleurs chaudes]**

---

## Le dilemme de l'éditeur de logiciel métier

```
┌──────────────────────────────────────────────────────────┐
│  Pour vendre en SaaS, l'éditeur héberge les données…      │
│        ↓                                                   │
│  ☁️  …donc il VOIT les données métier de ses clients      │
│        ↓                                                   │
│  ❌ Stock, factures, paie, clients → exposés à l'éditeur   │
│  ❌ Et souvent stockés sur un cloud étranger              │
│                                                            │
│  Alternative actuelle : le client installe tout lui-même  │
│        ↓                                                   │
│  ❌ Pas de mises à jour, pas de licences, pas de support  │
└──────────────────────────────────────────────────────────┘
```

---

### L'AUDPF — Union Africaine, Déc. 2025

> *"Les données des organisations africaines doivent rester sous contrôle local et ne pas transiter sans consentement par des serveurs étrangers."*

---

### ❓ La question centrale

> **Comment un éditeur peut-il vendre un logiciel métier en SaaS (comptes, licences, mises à jour) tout en garantissant qu'il ne pourra JAMAIS lire les données métier de ses clients ?**

---

*Notes présentateur :*
> "Le problème n'est pas que technique, il est commercial. Un éditeur veut le modèle SaaS — abonnement, parc géré, mises à jour. Mais le SaaS classique implique de voir les données du client. Mon projet résout cette contradiction."

---
<!-- ═══════════════════════════════════════════════════════════ -->
<!-- SLIDE 5 — LA SOLUTION : TROIS ACTEURS -->
<!-- ═══════════════════════════════════════════════════════════ -->

# SLIDE 5 — La Solution : un Framework à Trois Acteurs

**[Design : 3 blocs distincts reliés, fond sombre, accent doré ; PME en vert (zone souveraine)]**

---

## Séparer ce qui peut être vu de ce qui ne doit jamais l'être

```
  🏢 SaaS ÉDITEUR          🔒 RELAIS ZERO-KNOWLEDGE       🏭 CLUSTER PME
  (chez l'éditeur)          (chez l'éditeur)               (chez le client)
  ───────────────           ─────────────────              ──────────────
  Comptes tenants           Stockage de blobs              Logiciel métier
  Licences                  CHIFFRÉS opaques               Données EN CLAIR
  Suivi du parc             (jamais déchiffrables)         PostgreSQL répliqué
       │                          ▲                              │
       │  voit : compte/licence   │  voit : RIEN                 │  voit : tout
       └──────────────────────────┴──────────────────────────────┘
                       Les données métier ne sortent
                       que CHIFFRÉES, jamais en clair
```

---

### Les 3 garanties fondatrices

| 🏛️ Souveraineté | Les données métier restent sur les machines du client |
|-------------|----------------------------------------------|
| 🔑 Zero-knowledge | L'éditeur gère le parc mais ne détient aucune clé |
| ⚙️ Résilience | Cluster répliqué : panne d'une machine ≠ perte de données |

---

### Ce que le framework n'est PAS

❌ Un SaaS classique où l'éditeur voit les données
❌ Une installation isolée sans licences ni mises à jour
❌ Une solution dépendante d'un cloud étranger

---

*Notes présentateur :*
> "L'idée clé : on découpe le rôle en trois. L'éditeur gère le commercial — comptes, licences. Le relais stocke des sauvegardes chiffrées qu'il ne peut pas lire. Et c'est uniquement chez le client, dans son périmètre, que les données existent en clair."

---
<!-- ═══════════════════════════════════════════════════════════ -->
<!-- SLIDE 6 — POSITIONNEMENT -->
<!-- ═══════════════════════════════════════════════════════════ -->

# SLIDE 6 — Positionnement vs Alternatives

**[Design : tableau comparatif, colonnes colorées, notre solution en surbrillance]**

---

## Pourquoi pas les modèles existants ?

| Critère | SaaS cloud classique | Logiciel installé (on-premise) | **✅ Framework SaaS Souverain** |
|---------|:---:|:---:|:---:|
| Données invisibles à l'éditeur | ❌ | ✅ | ✅ Zero-knowledge |
| Comptes / licences / parc gérés | ✅ | ❌ | ✅ SaaS éditeur |
| Mises à jour centralisées | ✅ | ❌ | ✅ Image Docker |
| Résilience (réplication, failover) | ✅ | ⚠️ Manuel | ✅ PostgreSQL natif |
| Souveraineté infrastructurelle | ❌ | ✅ | ✅ Totale |
| Conformité AUDPF | ❌ | ⚠️ | ✅ Par design |

---

### Les 3 différenciateurs

> **1.** Modèle SaaS complet (commercial) **sans** accès aux données métier
> **2.** Résilience de niveau base de données (réplication primaire/standby)
> **3.** Souveraineté garantie cryptographiquement, pas contractuellement

---

*Notes présentateur :*
> "Le SaaS classique est pratique mais expose les données. Le logiciel installé protège les données mais perd tout le confort du SaaS. Mon framework prend le meilleur des deux : le confort SaaS pour l'éditeur, la souveraineté totale pour le client."

---
<!-- ═══════════════════════════════════════════════════════════ -->
<!-- SLIDE 6B — MODÈLE ÉCONOMIQUE & LICENCES -->
<!-- ═══════════════════════════════════════════════════════════ -->

# SLIDE 6B — Modèle Économique & Licences

**[Design : 3 cartes de plans, plan "Pro" en surbrillance ; tableau concurrents ; bandeau économie]**

---

## Une licence par poste — pensée pour la PME

| Plan | Postes | Prix / poste / mois |
|------|--------|:---:|
| Starter | 1 – 5 | **149 DH** HT |
| **Pro** ⭐ *(recommandé PME)* | 6 – 20 | **119 DH** HT |
| Enterprise | 21 et + | **89 DH** HT |

> Le prix **baisse** avec la taille du parc — l'inverse des SaaS classiques qui facturent souvent plus cher à mesure qu'on monte en gamme.

---

## Pourquoi structurellement moins cher ?

> L'éditeur **n'héberge pas** le cloud des données clients (elles restent chez la PME). Il n'a donc **pas** le coût d'infrastructure d'un SaaS classique → un prix par poste plus bas, **sans rogner la marge**.

| Solution (réf. marché) | Prix indicatif / poste / mois |
|------------------------|:---:|
| Odoo (Standard) | ≈ 250 DH |
| Zoho One | ≈ 370 DH |
| Sage Business Cloud | ≈ 280 – 550 DH |
| Microsoft Dynamics 365 BC | ≈ 700 – 1 100 DH |
| **✅ SaaS Souverain (Pro)** | **119 DH** |

---

### Exemple — PME de 10 postes (plan Pro)

> **14 280 DH/an** (souverain) **vs ≈ 48 000 DH/an** (SaaS classique milieu de gamme)
> → **≈ 70 % d'économie**, **et** les données ne quittent jamais la PME.

*Prix concurrents vérifiés par recherche web · page « Tarifs » intégrée à l'application SaaS éditeur.*

---

*Notes présentateur :*
> "Le modèle est simple : une licence par poste, dégressive — plus la PME grandit, moins le poste coûte. Et c'est viable parce que l'éditeur ne paie pas le cloud des données clients : elles restent chez la PME. Résultat, à 10 postes, une PME économise environ 70 % par rapport à un SaaS classique — tout en gardant la souveraineté. J'ai vérifié les prix concurrents par recherche, et la page Tarifs est intégrée directement dans l'application."

---
<!-- ═══════════════════════════════════════════════════════════ -->
<!-- SLIDE 7 — ARCHITECTURE & ZERO-KNOWLEDGE -->
<!-- ═══════════════════════════════════════════════════════════ -->

# SLIDE 7 — Architecture & Garantie Zero-Knowledge

**[Design : schéma hiérarchie de clés, fond sombre, cadenas dorés]**

---

## La hiérarchie de clés — le cœur de la promesse

```
  DEK (clé symétrique, unique par entreprise)
   ├─ chiffre : données métier + journal des écritures (CBOR)
   ├─ emballée par "sealed box" pour chaque appareil autorisé (X25519)
   └─ emballée sous un code de récupération (Argon2id)
         └─ stocké CHIFFRÉ sur le relais éditeur
```

> Le relais ne voit jamais la DEK ni aucune clé privée. **Même saisi ou compromis, il ne peut rien déchiffrer.**

---

## Crypto : aucune primitive réinventée — tout via libsodium

| Usage | Primitive |
|-------|-----------|
| Données & journal | XChaCha20-Poly1305 |
| Identité appareil | X25519 |
| Dérivation (code de récupération) | Argon2id |
| Enrôlement d'un appareil | Sealed box |

---

### Cœur partagé en **Rust** — un seul code, desktop & mobile (via UniFFI)

---

*Notes présentateur :*
> "Tout repose sur la DEK, une clé unique par entreprise. Elle chiffre les données. Pour chaque nouvel appareil autorisé, on lui emballe la DEK avec sa clé publique. Le relais, lui, ne reçoit que des blobs chiffrés et une copie de la DEK scellée sous le code de récupération du client — qu'il ne connaît pas. La promesse zero-knowledge tient cryptographiquement."

---
<!-- ═══════════════════════════════════════════════════════════ -->
<!-- SLIDE 8 — CLUSTER PME : RÉPLICATION & RÉSILIENCE -->
<!-- ═══════════════════════════════════════════════════════════ -->

# SLIDE 8 — Le Cœur Technique : Réplication & Résilience

**[Design : 2 nœuds PostgreSQL reliés par flux WAL, fond sombre]**

---

## Cluster PME — PostgreSQL primaire / standby

```
  🖥️ NŒUD PRIMAIRE                     🖥️ NŒUD STANDBY
  (écritures + lectures)                (réplique lecture seule)
  PostgreSQL 16                         PostgreSQL 16
       │                                      ▲
       │   réplication streaming WAL (TCP)    │
       └──────────────────────────────────────┘
            chaque écriture répliquée en < 1 s
```

| Mécanisme | Garantie |
|-----------|----------|
| Réplication par opération | Synchrone pour les invariants forts (stock, facturation) |
| **Slot de réplication** | Le primaire conserve le WAL pour le standby → pas de rupture |
| Journal append-only (CBOR) | Écritures sérialisées, chiffrées DEK avant disque |
| `standby.signal` | Le standby refuse toute écriture → ne peut pas diverger |
| Failover + **fencing** | 2 nœuds = manuel · ≥ 3 nœuds = quorum · ancien primaire clôturé |

---

## Découverte & packaging

> **Image Docker multi-arch** distribuée à la PME · les nœuds s'annoncent au relais éditeur au démarrage (pas de mDNS) · `docker-compose.yml` fourni.

---

*Notes présentateur :*
> "Côté client, le logiciel tourne dans Docker avec une base PostgreSQL répliquée. Toute écriture sur le primaire est copiée sur le standby en moins d'une seconde. Si une machine tombe, les données sont déjà ailleurs. Je n'ai réinventé ni la crypto, ni le consensus — j'ai orchestré des briques éprouvées."

---
<!-- ═══════════════════════════════════════════════════════════ -->
<!-- SLIDE 9 — ENRÔLEMENT & RÉCUPÉRATION -->
<!-- ═══════════════════════════════════════════════════════════ -->

# SLIDE 9 — Enrôlement d'un Appareil & Récupération de Sinistre

**[Design : 2 mini-schémas côte à côte, fond sombre]**

---

## Enrôlement (partage de la DEK, sans QR à scanner)

```
1. La PME souscrit une licence chez l'éditeur
2. Un appareil déjà autorisé emballe la DEK (sealed box)
   pour la clé publique du nouvel appareil
3. Le nouvel appareil ouvre le blob avec sa clé privée → obtient la DEK
4. Le jeton d'invitation est consommé (usage unique, courte durée)
```

---

## Récupération si la PME perd toutes ses machines

```
1. La PME contacte l'éditeur
2. L'éditeur restitue le blob chiffré stocké sur le relais
   (qu'il n'a JAMAIS pu lire)
3. La PME l'ouvre avec SON code de récupération → récupère la DEK
   → redéchiffre ses données sur une nouvelle machine
```

> L'éditeur aide sans jamais accéder au contenu. La promesse zero-knowledge tient même en sinistre total.

---

*Notes présentateur :*
> "Deux scénarios critiques. L'enrôlement : ajouter un appareil, c'est lui transmettre la DEK de façon chiffrée. La récupération : même si le client perd toutes ses machines, l'éditeur lui rend une sauvegarde chiffrée qu'il est le seul à pouvoir ouvrir, avec son code de récupération."

---
<!-- ═══════════════════════════════════════════════════════════ -->
<!-- SLIDE 10 — DÉMONSTRATION LIVE -->
<!-- ═══════════════════════════════════════════════════════════ -->

# SLIDE 10 — Démonstration Live

**[Design : fond vert foncé, badge "LIVE" rouge, captures terminal/SaaS en arrière-plan]**

---

## 🔴 LIVE — Cluster PME 2 nœuds (tenant MPJ)

**[CAPTURE : Kali primaire (.128) + Ubuntu standby (.130) côte à côte + dashboard SaaS]**

---

### Scénario de démonstration (5 min)

| Étape | Action | Résultat attendu |
|-------|--------|-----------------|
| **1** | Connexion employé (Alice) → créer 2 articles | Articles enregistrés sur le primaire |
| **2** | Bob (autre employé) → sortie de stock | Base partagée : Bob agit sur les données d'Alice |
| **3** | Requête SQL sur le **standby** Ubuntu | Articles & stocks répliqués en < 1 s |
| **4** | Tableau de bord SaaS éditeur | « ✓ Cluster sain · Réplication streaming » |
| **5** | Couper le standby → observer le SaaS | « ✗ Réplication interrompue » détecté |
| **6** | Rallumer le standby | Rattrapage WAL automatique + retour « sain » |

---

**[CAPTURE : articles répliqués sur Ubuntu]** · **[CAPTURE : SaaS « Réplication interrompue »]** · **[CAPTURE : retour « Cluster sain »]**

---

*Notes présentateur :*
> "Ce ne sont pas des simulations : deux machines réelles, Kali et Ubuntu. Un employé crée des données, elles apparaissent sur la seconde machine en moins d'une seconde. Et quand je coupe la réplication, le tableau de bord de l'éditeur le détecte immédiatement — il ne ment jamais sur l'état réel."

---
<!-- ═══════════════════════════════════════════════════════════ -->
<!-- SLIDE 11 — RÉSULTATS & VALIDATION -->
<!-- ═══════════════════════════════════════════════════════════ -->

# SLIDE 11 — Résultats : Validation sur Banc Réel

**[Design : tableau de résultats, vert dominant]**

---

## Campagne de test métier — tenant MPJ (2 employés)

| # | Scénario | Garantie démontrée | Verdict |
|---|----------|--------------------|:-------:|
| 1 | Création de 2 articles + entrées | Réplication primaire → standby | ✅ |
| 2 | Sorties de stock (2 employés) | Base unique partagée | ✅ |
| 3 | Stocks calculés + alerte de seuil | Cohérence ACID | ✅ |
| 4 | Insertion de doublon | Contrainte `UNIQUE` avant réplication | ✅ |
| 5 | Écritures concurrentes | Sérialisation MVCC sans corruption | ✅ |
| 6 | Panne du standby → reprise | Rattrapage WAL + détection SaaS | ✅ |

---

### Résultat global : **6 / 6 scénarios validés — 0 échec**

> Banc : Kali (primaire ⚡) + Ubuntu (standby) · réplication streaming async confirmée par `pg_stat_replication`.

---

*Notes présentateur :*
> "Six scénarios, tous validés sur deux machines physiques. Le plus important est le sixième : la résilience à la panne. C'est là que se joue la vraie promesse d'un système distribué."

---
<!-- ═══════════════════════════════════════════════════════════ -->
<!-- SLIDE 11B — TESTS DE RÉSILIENCE HA -->
<!-- ═══════════════════════════════════════════════════════════ -->

# SLIDE 11B — Résilience : 4 Protections Prouvées sur Banc

**[Design : 4 cartes/lignes vertes, bannière ⛔ "FENCED" pour le test 3]**

---

## Campagne haute disponibilité — Kali (primaire) + Ubuntu (standby)

| # | Protection | Garantie démontrée | Verdict |
|---|-----------|--------------------|:------:|
| 1 | **Slot de réplication** | Le primaire ne recycle plus le WAL du standby (`active=t`) | ✅ |
| 2 | **Anti auto-promotion** | À 2 nœuds, jamais de bascule auto → pas de split-brain | ✅ |
| 3 | **Fencing par timeline** | Ancien primaire déchu **se clôture** (timeline 1 < cluster 2) | ✅ |
| 4 | **Auto-réparation** | Tout re-clone recrée le slot **automatiquement** | ✅ |

---

### Preuve du fencing (logs réels du nœud déchu)

```
⛔ NŒUD CLÔTURÉ (FENCED)
Époque de ce nœud (timeline PG) : 1
Époque courante du cluster       : 2
→ REFUSE de servir pour éviter le split-brain.
```

> **4 / 4 protections validées — 0 échec.** Époque = `timeline_id` natif de PostgreSQL : aucun consensus réinventé.

---

*Notes présentateur :*
> "Au-delà des tests métier, j'ai mené une campagne de résilience. Quatre protections, toutes prouvées sur les deux machines réelles. La plus marquante : le fencing. Je promeus le standby, je rallume l'ancien primaire — et il se clôture tout seul, en lisant le timeline de PostgreSQL. Plus de split-brain. Et le cluster se répare automatiquement à chaque re-clone, sans intervention manuelle."

---
<!-- ═══════════════════════════════════════════════════════════ -->
<!-- SLIDE 12 — AGENT IA DE MONITORING ⭐ -->
<!-- ═══════════════════════════════════════════════════════════ -->

# SLIDE 12 — Supervision Intelligente : un Agent IA

**[Design : fond sombre, accent doré + bleu "IA", schéma flux métriques → modèle → alerte]**

---

## Un agent IA qui supervise les **3 acteurs** — implémenté

```
  Heartbeats (~60 s) ┐
  streaming_standby   ├─▶ Séries temporelles ─▶ AGENT IA ─▶ Score de risque 0–100
  WAL lag / époque    │     (SaaS, Big Data)    (Mistral)    + diagnostic + reco
  failover_count     ┘                                       sain / surveiller / critique
```

> L'agent analyse **l'éditeur SaaS, le relais zero-knowledge et les clusters PME**. Il produit un diagnostic en langage naturel via **Mistral AI**, persisté à chaque analyse (`AgentVerdict`).

---

## Garanties de l'agent

| Principe | Mise en œuvre |
|----------|---------------|
| 🔒 **Zero-knowledge préservé** | Seules des **métriques d'infrastructure** sont transmises au modèle — aucune donnée métier ne sort jamais |
| 🛰️ **Relais supervisé sans intrusion** | L'agent lit la **santé** du relais (`/health` : uptime, nb de tenants avec blobs) — **jamais le contenu** des blobs chiffrés |
| 🛟 **Fail-safe** | Clé API absente / quota épuisé / réseau coupé → **repli local déterministe** : le dashboard ne casse jamais |
| 📈 **Big Data** | Historique de métriques (`ClusterMetricSample`) → détection de dérives (lag, heartbeats espacés) |

---

### Au cœur de ma spécialité IA & Big Data

> L'agent ne se contente plus d'un seuil binaire : il **interprète la dynamique** des métriques et recommande une action préventive. La supervision intelligente est **implémentée et opérationnelle**, dans le respect strict de la souveraineté.

---

*Notes présentateur :*
> "Ma spécialité, c'est l'IA et le Big Data — et c'est ici qu'elle s'incarne. L'agent supervise les trois acteurs : l'éditeur, le relais, et les clusters PME. Point crucial : il respecte la souveraineté. Pour le relais, il ne lit que la santé du service — uptime, nombre de tenants ayant des blobs — jamais le contenu chiffré. Il s'appuie sur Mistral AI, avec un repli local si l'API est indisponible. C'est une supervision qui interprète, pas qui constate."

---
<!-- ═══════════════════════════════════════════════════════════ -->
<!-- SLIDE 13 — SÉCURITÉ -->
<!-- ═══════════════════════════════════════════════════════════ -->

# SLIDE 13 — Sécurité : Souveraineté par Design

**[Design : schéma en oignon — couches de sécurité emboîtées]**

---

## Défense en profondeur

```
┌─────────────────────────────────────────────────┐
│  🔑 ZERO-KNOWLEDGE                               │
│  Le relais ne détient aucune clé déchiffrable    │
├─────────────────────────────────────────────────┤
│  💾 AT-REST (données + journal CBOR)             │
│  XChaCha20-Poly1305 sous la DEK                  │
├─────────────────────────────────────────────────┤
│  📲 ENRÔLEMENT                                   │
│  Sealed box X25519 — jeton à usage unique        │
├─────────────────────────────────────────────────┤
│  🛡️ FENCING (failover)                          │
│  Jeton d'époque monotone — isole le nœud déchu   │
├─────────────────────────────────────────────────┤
│  🆔 IDENTITÉ PARC                                │
│  UUID d'installation authentifié (pas la MAC)    │
└─────────────────────────────────────────────────┘
```

---

> La souveraineté n'est pas une promesse contractuelle, c'est une **propriété cryptographique** : l'éditeur ne *peut pas* lire les données, même s'il le voulait.

---

*Notes présentateur :*
> "La sécurité est architecturale. Le point le plus subtil est le fencing : quand un standby est promu après une panne, l'ancien primaire doit être bloqué pour éviter deux primaires en parallèle. Je l'ai implémenté en m'appuyant sur le timeline natif de PostgreSQL — incrémenté à chaque promotion — et je l'ai prouvé sur le banc : l'ancien primaire revenu se clôture tout seul. Un piège classique des systèmes distribués, résolu sans réinventer de consensus."

---
<!-- ═══════════════════════════════════════════════════════════ -->
<!-- SLIDE 14 — DIFFICULTÉS : POSTURE INGÉNIEUR -->
<!-- ═══════════════════════════════════════════════════════════ -->

# SLIDE 14 — Difficulté Majeure : Diagnostic d'un Split-Brain

**[Design : avant/après — 2 sources de vérité qui se contredisent → résolution]**

---

## Un incident réel de système distribué

| | |
|---|---|
| 🔴 **Symptôme** | Le SaaS affiche « Cluster sain », mais le nœud affiche « aucun standby connecté » |
| 🔍 **Diagnostic** | Deux sources de vérité divergentes : le SaaS croyait un rôle *déclaré*, PostgreSQL mesurait la réalité |
| ❓ **Cause racine** | Le standby s'était auto-promu (failover) pendant que le primaire tournait → **deux primaires** (split-brain) |
| 🔧 **Technicien** | Redémarrer les conteneurs (ne corrige rien : le nœud promu reste primaire) |
| ✅ **Ingénieur** | (1) Réparer le cluster par `pg_basebackup` · (2) **faire dire la vérité au SaaS** : le primaire transmet son nombre réel de standbys en streaming (`pg_stat_replication`) |
| 📐 **Principe** | Jamais de dégradation silencieuse — un tableau de bord ne doit afficher que ce qu'il a *mesuré* |

---

> **De l'incident à l'amélioration produit** : le correctif a fermé la dégradation silencieuse, et a inspiré la perspective d'une supervision IA prédictive (slide 12).

---

*Notes présentateur :*
> "Cet incident résume la posture ingénieur. Le réflexe technicien : redémarrer. Mais ça ne réglait rien. J'ai compris que le tableau de bord mentait parce qu'il se fiait à une déclaration, pas à une mesure. J'ai corrigé la cause — et j'en ai tiré une fonctionnalité : la supervision intelligente."

---
<!-- ═══════════════════════════════════════════════════════════ -->
<!-- SLIDE 15 — COMPÉTENCES DÉVELOPPÉES -->
<!-- ═══════════════════════════════════════════════════════════ -->

# SLIDE 15 — Compétences Développées

**[Design : hexagones de compétences, couleurs par domaine]**

---

### Techniques

| Domaine | Compétences acquises |
|---------|---------------------|
| 🦀 Cœur système | Rust, sérialisation CBOR, journal append-only |
| 🔐 Cryptographie | libsodium, XChaCha20-Poly1305, X25519, Argon2id, zero-knowledge |
| 🗄️ Systèmes distribués | PostgreSQL réplication, failover/quorum, fencing, MVCC |
| 📊 Big Data & IA | Séries temporelles de métriques, détection d'anomalies, agent de supervision |
| 🐳 DevOps | Docker multi-arch, docker-compose, registre privé, watchtower |
| ⚡ Backend SaaS | Django, API REST, modèle tenant/licence/parc |

---

### Transversales

> 🎯 **Diagnostic en production** — split-brain résolu structurellement
> 📝 **Rigueur documentaire** — plan directeur, guide de test, journal technique
> 🧭 **Autonomie** — décisions d'architecture sans supervision permanente
> 💬 **Communication** — points réguliers avec Mme CHOKRI & M. Amrani

---

*Notes présentateur :*
> "Au-delà des technologies, ce stage m'a appris à diagnostiquer un système distribué en production — là où chaque couche cache la suivante — et à relier ma spécialité IA & Big Data à un problème d'infrastructure concret."

---
<!-- ═══════════════════════════════════════════════════════════ -->
<!-- SLIDE 16 — BILAN & PERSPECTIVES -->
<!-- ═══════════════════════════════════════════════════════════ -->

# SLIDE 16 — Bilan et Perspectives

**[Design : deux colonnes — Acquis (vert) / Perspectives (bleu/or)]**

---

## Ce qui a été accompli

| ✅ | Résultat |
|----|---------|
| 6/6 | Scénarios métier validés sur banc réel |
| 4/4 | Tests de résilience HA (slot, anti-promotion, **fencing**, auto-réparation) |
| Zero-knowledge | Hiérarchie de clés DEK prouvée cross-OS |
| Réplication | PostgreSQL primaire/standby + slot + détection de panne |
| Fencing | Timeline PostgreSQL — split-brain neutralisé, prouvé sur banc |
| Reporting véridique | Le SaaS reflète l'état réel de la réplication |
| Supervision IA | **Implémentée** — agent Mistral sur les 3 acteurs, zero-knowledge préservé |

---

## Valeur pour AL BARAA CONSULTING

> ✅ Framework réutilisable pour vendre tout logiciel métier en SaaS souverain
> ✅ Argument commercial fort : conformité AUDPF par design
> ✅ Différenciation sur le marché africain des solutions souveraines

---

## Perspectives techniques

| Horizon court | Horizon moyen |
|---------------|---------------|
| Quorum ≥ 3 nœuds (failover automatique sûr) | Couche IA **prédictive** (apprentissage des dérives) |
| IP statiques sur le réseau de réplication | Module métier v1 (gestion de stock) |
| Durcissement production & audit sécurité | Frontend Tauri + mobile (cœur Rust via UniFFI) |

---

*Notes présentateur :*
> "C'est un socle prouvé, pas un produit fini. Pendant le stage, j'ai poussé deux briques jusqu'au bout : le fencing — validé sur le banc — et l'agent IA de supervision. La suite logique, c'est le quorum à 3 nœuds pour un failover automatique sûr, et une couche IA prédictive qui apprend les dérives pour alerter encore plus tôt."

---
<!-- ═══════════════════════════════════════════════════════════ -->
<!-- SLIDE 17 — CONCLUSION -->
<!-- ═══════════════════════════════════════════════════════════ -->

# SLIDE 17 — Conclusion

**[Design : fond sombre, accent doré, texte centré, sobre et fort]**

---

## Ce que nous avons démontré

---

### Techniquement

> Un éditeur peut vendre un logiciel métier en **SaaS complet** (comptes, licences, parc, mises à jour) tout en étant **cryptographiquement incapable** de lire les données de ses clients.

---

### Stratégiquement

> **6/6** scénarios validés sur banc réel multi-OS
> **Zero-knowledge** garanti par la hiérarchie de clés, pas par un contrat
> **Résilience** de niveau base de données + supervision intelligente
> **Conforme AUDPF** — par design, pas par configuration

---

### Personnellement

> Ce projet m'a confirmé que la valeur d'un ingénieur réside dans la **qualité des décisions architecturales**, la **rigueur du diagnostic**, et la capacité à relier l'**IA & le Big Data** à un problème d'infrastructure réel.

---

## La souveraineté numérique africaine commence ici.

---

**[LOGO EIGSI + LOGO AL BARAA CONSULTING]**

*Jesse MPIGA-ODOUMBA — EIGSI Casablanca — Promotion 2026*

---

*Notes présentateur :*
> Pause. Regarder le jury.
> "Je vous remercie pour votre attention. Je suis à votre disposition pour répondre à vos questions."
> *(Ne pas dépasser 33 minutes — chronomètre personnel conseillé)*

---

<!-- ═══════════════════════════════════════════════════════════ -->
<!-- ANNEXE — SLIDES DE RÉSERVE (si questions jury) -->
<!-- ═══════════════════════════════════════════════════════════ -->

---
---

# SLIDES DE RÉSERVE — Pour les questions jury

*(Ne pas présenter sauf si le jury pose ces questions)*

---

## RÉSERVE A — Les trois acteurs en détail

| Acteur | Hébergement | Voit le clair ? | Rôle |
|--------|-------------|:---:|------|
| SaaS éditeur (Django) | Chez l'éditeur | Compte/licence seulement | Comptes tenants, licences, suivi parc |
| Relais zero-knowledge | Chez l'éditeur | **Jamais** | Stockage de blobs chiffrés opaques |
| Cluster PME | Chez le client | **Oui** | Exécute le logiciel, détient les données |

---

## RÉSERVE B — Pourquoi PostgreSQL et pas un consensus maison ?

> Aucun algorithme de consensus écrit à la main. On utilise la **promotion de standby PostgreSQL** + supervision type Patroni. Réinventer Raft/Paxos serait une faute d'ingénierie : risque élevé, valeur nulle face à des briques éprouvées.

---

## RÉSERVE C — Failover : 2 nœuds vs 3 nœuds

| Configuration | Comportement |
|---------------|--------------|
| 2 nœuds | Bascule **manuelle** uniquement (risque de split-brain sinon) |
| ≥ 3 nœuds | Failover **automatique** par quorum |

> Le SaaS signale à la PME si son cluster ne permet que la bascule manuelle.

---

## RÉSERVE D — Le split-brain : évité définitivement (implémenté)

> **Fencing par timeline PostgreSQL** : l'époque = le `timeline_id` natif de PostgreSQL, incrémenté à chaque `pg_promote()` (aucun consensus maison). Un ancien primaire revenu avec un timeline inférieur **se clôture lui-même** : il ne lance pas son serveur web → aucune écriture métier possible.
>
> **Défense en profondeur (2 couches)** : (1) le SaaS refuse qu'une époque périmée rétrograde le primaire légitime (`409 fenced`) ; (2) le nœud s'auto-clôture au démarrage et à chaud.
>
> **Validé sur banc réel** : ancien primaire (timeline 1) face au cluster (timeline 2) → bannière `⛔ NŒUD CLÔTURÉ`, service refusé. Voir l'annexe *Tests résilience HA* (4/4).

---

## RÉSERVE E — L'agent IA : comment il fonctionne *(implémenté)*

| Élément | Mise en œuvre |
|---------|-------|
| Périmètre | Supervise les **3 acteurs** : SaaS éditeur, relais zero-knowledge (santé seule), clusters PME |
| Données | Séries temporelles : heartbeats, `streaming_standby_count`, WAL/époque, `failover_count` (`ClusterMetricSample`) |
| Moteur | **Mistral AI** (`mistral-small-latest`) → diagnostic langage naturel + score de risque 0–100 |
| Sortie | Niveau `sain / surveiller / critique` + recommandation, persistée (`AgentVerdict`) |
| Zero-knowledge | **Aucune** donnée métier transmise — uniquement des métriques d'infrastructure |
| Fail-safe | Repli local déterministe si l'API Mistral est indisponible — le dashboard ne casse jamais |

> Implémenté et opérationnel. La **prochaine** étape est une couche prédictive (apprentissage des dérives) qui anticiperait la panne avant le franchissement de seuil.

---

## RÉSERVE F — Sécurité : que se passe-t-il si le relais est saisi ?

> Rien d'exploitable. Le relais ne stocke que des **blobs chiffrés** et une copie de la DEK **scellée sous le code de récupération du client**. Sans ce code (que l'éditeur ne connaît pas), aucun déchiffrement n'est possible.

---

*Fin du support de soutenance*

---

*MPIGA-ODOUMBA Jesse — EIGSI Casablanca — Promotion 2026*
*AL BARAA CONSULTING — Soutenance : 01/07/2026*
