<!--
═══════════════════════════════════════════════════════════════════════
  INSTRUCTIONS POUR LA GÉNÉRATION PPTX (à lire par l'assistant générateur)
═══════════════════════════════════════════════════════════════════════
  - FORMAT DE SORTIE : PowerPoint .pptx (déposé sur Moodle sous le nom
    MPIGA_Jesse_FE_Promo2026_Soutenance.pptx — max 8 Mo).
  - UNE SLIDE = un bloc séparé par "---". Le titre de slide est la 1re
    ligne "#" ou "##" du bloc.
  - Les blocs "<!-- SPEAKER NOTES -->" vont dans les COMMENTAIRES DU
    PRÉSENTATEUR (speaker notes), PAS sur la diapo visible.
  - Les emplacements "[CAPTURE : …]" et "[LOGO …]" sont des zones images
    à insérer (laisser un cadre vide si l'image n'est pas fournie).

  THÈME VISUEL — clair, sobre, académique (EIGSI)
  -----------------------------------------------
  - Fond            : blanc #FFFFFF (slides de section : #F4F6F9)
  - Couleur primaire: bleu EIGSI #003B71 (titres, filets, en-têtes de tableau)
  - Accent          : bleu clair #0072B5 (mots-clés, icônes)
  - Texte courant   : gris anthracite #1F2933
  - Texte secondaire: gris #5B6770
  - Succès / validé : vert #2E7D32 (✅) ; alerte : rouge sobre #C0392B
  - Police titres   : sans-serif (Montserrat ou Calibri Light), bleu #003B71
  - Police corps    : sans-serif lisible (Calibri / Source Sans), 18–22 pt
  - Tableaux        : en-tête fond #003B71 texte blanc ; lignes alternées #F4F6F9
  - Style général   : épuré, beaucoup de blanc, un filet bleu fin en pied de page
                      avec « Jesse MPIGA-ODOUMBA · EIGSI 2026 » + numéro de slide.
═══════════════════════════════════════════════════════════════════════
-->

---

# Framework SaaS Souverain

### Conception et Implémentation d'un Framework SaaS Souverain pour Logiciels Métier Distribués

**Expérience Professionnelle de Fin d'Études**
EIGSI Casablanca — Spécialité Big Data & Intelligence Artificielle — Promotion 2026

| | |
|---|---|
| **Étudiant** | Jesse MPIGA-ODOUMBA |
| **Encadrante entreprise** | Mme Soumia CHOKRI — AL BARAA CONSULTING |
| **Tuteur EIGSI** | M. Ayoub AMRANI |
| **Soutenance** | 01 juillet 2026 — EIGSI Casablanca |

`[LOGO EIGSI — haut gauche]`  ·  `[LOGO AL BARAA CONSULTING — haut droite]`

<!-- SPEAKER NOTES
Sourire, regarder le jury. Pause 3 secondes. Commencer par :
"Mesdames, Messieurs, je vous remercie de m'accorder ce temps pour vous présenter mon projet de fin d'études : la conception d'un framework qui permet à un éditeur de vendre un logiciel métier en mode SaaS, sans jamais voir les données de ses clients."
-->

---

## Plan de la présentation

| # | Partie | ⏱ |
|---|--------|---|
| 1 | AL BARAA CONSULTING & contexte du stage | 2 min |
| 2 | Problématique : la souveraineté des données métier | 3 min |
| 3 | La solution : un framework à trois acteurs | 3 min |
| 4 | Architecture & garantie zero-knowledge | 4 min |
| 5 | Le cœur technique : réplication & résilience | 4 min |
| 6 | **Démonstration live** — cluster PME 2 nœuds | 5 min |
| 7 | **Supervision intelligente — agent IA** | 3 min |
| 8 | Difficultés & posture ingénieur | 3 min |
| 9 | Bilan, compétences & perspectives | 3 min |

<!-- SPEAKER NOTES
"La présentation suit le fil du projet : d'abord le problème de souveraineté, puis la solution architecturale, le cœur technique que j'ai prouvé sur un banc réel, une démonstration live, et enfin la dimension intelligence artificielle qui supervise le système — au cœur de ma spécialité."
-->

---

## AL BARAA CONSULTING — un cabinet de conseil en ingénierie numérique

`[LOGO AL BARAA CONSULTING — centré]`

| 📅 Fondé | Mars 2017 |
|---------|-----------|
| 📍 Siège | Ain Sebaa, Casablanca |
| ⚖️ Statut | SARL AU — 100 000 MAD |
| 👤 DG | Mme Soumia CHOKRI |
| 🎯 Missions | Développement logiciel, Architecture SI, Transformation numérique |
| 🤝 Clients | Secteur public & privé (B2B) |

**Ma position :** Développeur & Architecte principal du projet — responsabilité totale sur l'ensemble du cycle (conception → développement → déploiement → validation).

<!-- SPEAKER NOTES
"AL BARAA CONSULTING est un cabinet à taille humaine, ce qui m'a placé en situation de forte responsabilité dès le premier jour : une mission d'ingénieur avec une autonomie réelle, pas des tâches d'exécution."
-->

---

## La problématique — le dilemme de l'éditeur de logiciel métier

Pour vendre en **SaaS**, l'éditeur héberge les données…
→ donc il **VOIT** les données métier de ses clients (stock, factures, paie, clients)
→ et souvent stockées sur un **cloud étranger**.

L'alternative actuelle — le client installe tout lui-même :
→ ❌ pas de mises à jour, pas de licences, pas de support.

**L'AUDPF — Union Africaine, Déc. 2025**
> *« Les données des organisations africaines doivent rester sous contrôle local et ne pas transiter sans consentement par des serveurs étrangers. »*

### ❓ La question centrale
> **Comment un éditeur peut-il vendre un logiciel métier en SaaS (comptes, licences, mises à jour) tout en garantissant qu'il ne pourra JAMAIS lire les données métier de ses clients ?**

<!-- SPEAKER NOTES
"Le problème n'est pas que technique, il est commercial. Un éditeur veut le modèle SaaS — abonnement, parc géré, mises à jour. Mais le SaaS classique implique de voir les données du client. Mon projet résout cette contradiction."
-->

---

## La solution — un framework à trois acteurs

Séparer ce qui **peut être vu** de ce qui **ne doit jamais l'être** :

<!-- DIAGRAMME À DESSINER (élément central de la slide — grand, pleine largeur, NE PAS afficher ce commentaire) :
3 grandes boîtes rectangulaires côte à côte, alignées horizontalement, reliées par des flèches.
  • Boîte 1 — "🏢 SaaS ÉDITEUR" : remplissage bleu EIGSI #003B71, texte blanc.
      Sous-texte : "Comptes · Licences · Suivi du parc". Étiquette dessous : "voit : compte / licence".
  • Boîte 2 — "🔒 RELAIS ZERO-KNOWLEDGE" : remplissage gris #5B6770, texte blanc.
      Sous-texte : "Blobs CHIFFRÉS opaques". Étiquette dessous, en gras : "voit : RIEN".
  • Boîte 3 — "🏭 CLUSTER PME" : remplissage vert #2E7D32, texte blanc, bordure plus épaisse (zone souveraine).
      Sous-texte : "Logiciel métier · Données EN CLAIR · PostgreSQL répliqué". Étiquette dessous : "voit : tout".
  Une flèche horizontale traverse les 3 boîtes, annotée en rouge sobre #C0392B :
      "Les données métier ne sortent que CHIFFRÉES — jamais en clair".
  Mettre la boîte 3 (verte) légèrement plus mise en valeur : c'est le périmètre souverain du client.
-->

> Les données métier ne sortent que **CHIFFRÉES**, jamais en clair.

### Les 3 garanties fondatrices
- 🏛️ **Souveraineté** — les données restent sur les machines du client
- 🔑 **Zero-knowledge** — l'éditeur gère le parc mais ne détient aucune clé
- ⚙️ **Résilience** — cluster répliqué : panne d'une machine ≠ perte de données

<!-- SPEAKER NOTES
"L'idée clé : on découpe le rôle en trois. L'éditeur gère le commercial — comptes, licences. Le relais stocke des sauvegardes chiffrées qu'il ne peut pas lire. Et c'est uniquement chez le client, dans son périmètre, que les données existent en clair."
-->

---

## Positionnement vs alternatives

| Critère | SaaS cloud classique | On-premise installé | **✅ SaaS Souverain** |
|---------|:---:|:---:|:---:|
| Données invisibles à l'éditeur | ❌ | ✅ | ✅ Zero-knowledge |
| Comptes / licences / parc gérés | ✅ | ❌ | ✅ SaaS éditeur |
| Mises à jour centralisées | ✅ | ❌ | ✅ Image Docker |
| Résilience (réplication, failover) | ✅ | ⚠️ Manuel | ✅ PostgreSQL natif |
| Souveraineté infrastructurelle | ❌ | ✅ | ✅ Totale |
| Conformité AUDPF | ❌ | ⚠️ | ✅ Par design |

**Les 3 différenciateurs**
1. Modèle SaaS complet (commercial) **sans** accès aux données métier
2. Résilience de niveau base de données (réplication primaire/standby)
3. Souveraineté garantie **cryptographiquement**, pas contractuellement

<!-- SPEAKER NOTES
"Le SaaS classique est pratique mais expose les données. Le logiciel installé protège les données mais perd tout le confort du SaaS. Mon framework prend le meilleur des deux : le confort SaaS pour l'éditeur, la souveraineté totale pour le client."
-->

---

## Architecture & garantie zero-knowledge — la hiérarchie de clés

<!-- DIAGRAMME À DESSINER (arbre vertical centré, remplace le bloc de code, NE PAS afficher ce commentaire) :
En haut, un encadré proéminent "🔑 DEK" (bleu EIGSI #003B71, texte blanc, coins arrondis),
sous-titre "Clé symétrique — unique par entreprise".
De cette DEK partent 3 branches descendantes (flèches) vers 3 encadrés bleu clair #0072B5 :
  1. "Chiffre les données métier + journal CBOR"
  2. "Emballée en sealed box pour chaque appareil autorisé (X25519)"
  3. "Emballée sous un code de récupération (Argon2id)"
        → de l'encadré 3, une flèche vers un encadré gris #5B6770 :
          "Stocké CHIFFRÉ sur le relais éditeur (jamais déchiffrable)".
Idée visuelle : la DEK est la racine ; tout en découle. Garder grand et aéré.
-->

```
DEK (clé symétrique, unique par entreprise)
 ├─ chiffre : données métier + journal des écritures (CBOR)
 ├─ emballée par « sealed box » pour chaque appareil autorisé (X25519)
 └─ emballée sous un code de récupération (Argon2id)
       └─ stocké CHIFFRÉ sur le relais éditeur
```

> Le relais ne voit jamais la DEK ni aucune clé privée. **Même saisi ou compromis, il ne peut rien déchiffrer.**

**Crypto : aucune primitive réinventée — tout via libsodium**

| Usage | Primitive |
|-------|-----------|
| Données & journal | XChaCha20-Poly1305 |
| Identité appareil | X25519 |
| Dérivation (code de récupération) | Argon2id |
| Enrôlement d'un appareil | Sealed box |

**Cœur partagé en Rust** — un seul code, desktop & mobile (via UniFFI).

<!-- SPEAKER NOTES
"Tout repose sur la DEK, une clé unique par entreprise. Elle chiffre les données. Pour chaque nouvel appareil autorisé, on lui emballe la DEK avec sa clé publique. Le relais ne reçoit que des blobs chiffrés et une copie de la DEK scellée sous le code de récupération du client — qu'il ne connaît pas. La promesse zero-knowledge tient cryptographiquement."
-->

---

## Le cœur technique — réplication & résilience

Cluster PME — **PostgreSQL primaire / standby**

<!-- DIAGRAMME À DESSINER (2 cylindres de base de données, remplace le bloc de code, NE PAS afficher ce commentaire) :
Deux symboles de base de données (cylindres) côte à côte :
  • Gauche — "🖥️ NŒUD PRIMAIRE" : cylindre bleu plein #003B71, texte blanc.
      Sous-texte : "Écritures + lectures · PostgreSQL 16".
  • Droite — "🖥️ NŒUD STANDBY" : cylindre bleu clair #0072B5.
      Sous-texte : "Réplique lecture seule · PostgreSQL 16".
Une grosse flèche horizontale du PRIMAIRE vers le STANDBY, étiquette en gras au-dessus :
      "Réplication streaming WAL (TCP) — chaque écriture répliquée en < 1 s".
Optionnel : petite icône bouclier verte #2E7D32 sur le standby = "données déjà ailleurs si panne".
-->

```
🖥️ NŒUD PRIMAIRE                     🖥️ NŒUD STANDBY
(écritures + lectures)                (réplique lecture seule)
PostgreSQL 16                         PostgreSQL 16
      │   réplication streaming WAL (TCP)   ▲
      └──────────────────────────────────────┘
           chaque écriture répliquée en < 1 s
```

| Mécanisme | Garantie |
|-----------|----------|
| Réplication par opération | Synchrone pour les invariants forts (stock, facturation) |
| Journal append-only (CBOR) | Écritures sérialisées, chiffrées DEK avant disque |
| `standby.signal` | Le standby refuse toute écriture → ne peut pas diverger |
| Failover | 2 nœuds = bascule manuelle · ≥ 3 nœuds = quorum automatique |

> **Packaging :** image Docker multi-arch · annonce au relais au démarrage (pas de mDNS) · `docker-compose.yml` fourni.

<!-- SPEAKER NOTES
"Côté client, le logiciel tourne dans Docker avec une base PostgreSQL répliquée. Toute écriture sur le primaire est copiée sur le standby en moins d'une seconde. Si une machine tombe, les données sont déjà ailleurs. Je n'ai réinventé ni la crypto, ni le consensus — j'ai orchestré des briques éprouvées."
-->

---

## Enrôlement d'un appareil & récupération de sinistre

**Enrôlement (partage de la DEK)**
```
1. La PME souscrit une licence chez l'éditeur
2. Un appareil déjà autorisé emballe la DEK (sealed box)
   pour la clé publique du nouvel appareil
3. Le nouvel appareil ouvre le blob avec sa clé privée → obtient la DEK
4. Le jeton d'invitation est consommé (usage unique, courte durée)
```

**Récupération si la PME perd toutes ses machines**
```
1. La PME contacte l'éditeur
2. L'éditeur restitue le blob chiffré stocké sur le relais
   (qu'il n'a JAMAIS pu lire)
3. La PME l'ouvre avec SON code de récupération → récupère la DEK
   → redéchiffre ses données sur une nouvelle machine
```

> L'éditeur aide sans jamais accéder au contenu. La promesse zero-knowledge tient **même en sinistre total**.

<!-- SPEAKER NOTES
"Deux scénarios critiques. L'enrôlement : ajouter un appareil, c'est lui transmettre la DEK de façon chiffrée. La récupération : même si le client perd toutes ses machines, l'éditeur lui rend une sauvegarde chiffrée qu'il est le seul à pouvoir ouvrir, avec son code de récupération."
-->

---

## 🔴 Démonstration live — cluster PME 2 nœuds (tenant MPJ)

`[CAPTURE : Kali primaire (.128) + Ubuntu standby (.130) côte à côte + dashboard SaaS]`

| Étape | Action | Résultat attendu |
|-------|--------|-----------------|
| 1 | Connexion employé (Alice) → créer 2 articles | Articles enregistrés sur le primaire |
| 2 | Bob (autre employé) → sortie de stock | Base partagée : Bob agit sur les données d'Alice |
| 3 | Requête SQL sur le **standby** Ubuntu | Articles & stocks répliqués en < 1 s |
| 4 | Tableau de bord SaaS éditeur | « ✓ Cluster sain · Réplication streaming » |
| 5 | Couper le standby → observer le SaaS | « Réplication interrompue » détecté |
| 6 | Rallumer le standby | Rattrapage WAL automatique + retour « sain » |
| 7 | Tableau de bord | KPI temps réel : nœuds en ligne, réplication active, serveur éditeur |
| 8 | Agent IA | Verdict du cluster + diagnostic du serveur éditeur + rapport PDF |

`[CAPTURE : articles répliqués sur Ubuntu]` · `[CAPTURE : SaaS « Réplication interrompue »]` · `[CAPTURE : page Agent IA — verdict + jauges serveur]`

<!-- SPEAKER NOTES
"Ce ne sont pas des simulations : deux machines réelles, Kali et Ubuntu. Un employé crée des données, elles apparaissent sur la seconde machine en moins d'une seconde. Et quand je coupe la réplication, le tableau de bord de l'éditeur le détecte immédiatement — il ne ment jamais sur l'état réel."
-->

---

## Résultats — validation sur banc réel (tenant MPJ, 2 employés)

| # | Scénario | Garantie démontrée | Verdict |
|---|----------|--------------------|:-------:|
| 1 | Création de 2 articles + entrées | Réplication primaire → standby | ✅ |
| 2 | Sorties de stock (2 employés) | Base unique partagée | ✅ |
| 3 | Stocks calculés + alerte de seuil | Cohérence ACID | ✅ |
| 4 | Insertion de doublon | Contrainte `UNIQUE` avant réplication | ✅ |
| 5 | Écritures concurrentes | Sérialisation MVCC sans corruption | ✅ |
| 6 | Panne du standby → reprise | Rattrapage WAL + détection SaaS | ✅ |

### Résultat global : **6 / 6 scénarios validés — 0 échec**
> Banc : Kali (primaire) + Ubuntu (standby) · réplication streaming async confirmée par `pg_stat_replication`.

<!-- SPEAKER NOTES
"Six scénarios, tous validés sur deux machines physiques. Le plus important est le sixième : la résilience à la panne. C'est là que se joue la vraie promesse d'un système distribué."
-->

---

## Supervision intelligente — un agent IA (implémenté)

L'agent analyse en temps réel les métriques d'infrastructure et rend un verdict :

<!-- DIAGRAMME À DESSINER (pipeline horizontal de gauche à droite, remplace le bloc, NE PAS afficher ce commentaire) :
Chaîne de 4 étapes reliées par des flèches :
  [Métriques sources] → [Séries temporelles en base] → [Agent IA — Mistral] → [Verdict + rapport]
  • Étape 1 "Métriques" (bleu clair #0072B5) : Clusters PME (heartbeats, streaming, WAL lag, failover) ET Serveur éditeur (CPU, RAM, disque, base).
  • Étape 2 "Séries temporelles collectées (ClusterMetricSample)" (bleu #003B71).
  • Étape 3 "Agent IA — Mistral (LLM) + repli local" : encadré PLEIN (implémenté), accent #0072B5.
  • Étape 4 "Verdict : risque + score /100 + diagnostic + reco · rapport PDF" (vert #2E7D32).
Toutes les étapes en trait plein : la chaîne est fonctionnelle, pas conceptuelle.
-->

```
Clusters PME    ┐
(heartbeats,     │
 streaming, WAL) ├─▶ Séries temporelles ─▶ Agent IA (Mistral ─▶ Verdict : risque + score
Serveur éditeur  │     en base              + repli local)        + diagnostic + rapport PDF
(CPU, RAM, base)┘
```

| Implémenté et démontrable | Perspective (prochaine étape) |
|-------------------|----------------------|
| Agent LLM (Mistral) : risque + score /100 + diagnostic + reco | **Modèle entraîné** non supervisé (Isolation Forest / z-score) |
| Supervise clusters PME **et** serveur éditeur | **Prédiction** de panne avant franchissement de seuil |
| Page temps réel animée + rapport PDF détaillé | Apprentissage sur l'historique accumulé |
| Repli local si IA indisponible · zero-knowledge | — |

> L'agent **est branché** sur le SaaS : il supervise en direct les clusters et le serveur éditeur, et produit des rapports. Seules des **métriques d'infrastructure** lui sont transmises — la promesse zero-knowledge tient jusque dans la supervision.

<!-- SPEAKER NOTES
"Ma spécialité IA & Big Data ne reste pas théorique. J'ai implémenté un agent : il analyse en temps réel les métriques des clusters ET de mon propre serveur éditeur, via un modèle de langage Mistral, et rend un verdict — niveau de risque, score, diagnostic, recommandation — avec un rapport PDF. Il a un repli local pour rester démontrable même hors-ligne. Et il ne voit que des métriques d'infrastructure, jamais de données métier. La prochaine étape, que j'assume comme perspective, est de remplacer le raisonnement du LLM par un modèle entraîné sur l'historique pour passer du diagnostic à la prédiction."
-->

---

## Sécurité — souveraineté par design (défense en profondeur)

<!-- DIAGRAMME À DESSINER (5 couches empilées, style "défense en profondeur", remplace le bloc, NE PAS afficher ce commentaire) :
5 bandes horizontales empilées (ou pyramide/oignon), de la couche externe (haut) à la couche interne (bas),
dégradé de bleu (externe clair #0072B5 → interne foncé #003B71), texte blanc, chacune avec son icône :
  1. 🔑 ZERO-KNOWLEDGE — Le relais ne détient aucune clé déchiffrable
  2. 💾 AT-REST (données + journal CBOR) — XChaCha20-Poly1305 sous la DEK
  3. 📲 ENRÔLEMENT — Sealed box X25519, jeton à usage unique
  4. 🛡️ FENCING (failover) — Jeton d'époque monotone, isole le nœud déchu
  5. 🆔 IDENTITÉ PARC — UUID d'installation authentifié (pas la MAC)
Idée : plus on descend, plus on est au cœur du système → couches de protection emboîtées.
-->

```
┌─────────────────────────────────────────────────┐
│  🔑 ZERO-KNOWLEDGE                                │
│  Le relais ne détient aucune clé déchiffrable    │
├─────────────────────────────────────────────────┤
│  💾 AT-REST (données + journal CBOR)             │
│  XChaCha20-Poly1305 sous la DEK                  │
├─────────────────────────────────────────────────┤
│  📲 ENRÔLEMENT                                    │
│  Sealed box X25519 — jeton à usage unique        │
├─────────────────────────────────────────────────┤
│  🛡️ FENCING (failover)                           │
│  Jeton d'époque monotone — isole le nœud déchu   │
├─────────────────────────────────────────────────┤
│  🆔 IDENTITÉ PARC                                 │
│  UUID d'installation authentifié (pas la MAC)    │
└─────────────────────────────────────────────────┘
```

> La souveraineté n'est pas une promesse contractuelle, c'est une **propriété cryptographique** : l'éditeur ne *peut pas* lire les données, même s'il le voulait.

<!-- SPEAKER NOTES
"La sécurité est architecturale. Le point le plus subtil est le fencing : quand un standby est promu après une panne, l'ancien primaire doit être bloqué par un jeton d'époque pour éviter deux primaires en parallèle — un piège classique des systèmes distribués que j'ai rencontré en vrai."
-->

---

## Difficulté majeure — diagnostic d'un split-brain

| | |
|---|---|
| 🔴 **Symptôme** | Le SaaS affiche « Cluster sain », mais le nœud affiche « aucun standby connecté » |
| 🔍 **Diagnostic** | Deux sources de vérité divergentes : le SaaS croyait un rôle *déclaré*, PostgreSQL mesurait la réalité |
| ❓ **Cause racine** | Le standby s'était auto-promu pendant que le primaire tournait → **deux primaires** (split-brain) |
| 🔧 **Réflexe technicien** | Redémarrer les conteneurs (ne corrige rien : le nœud promu reste primaire) |
| ✅ **Posture ingénieur** | (1) Réparer par `pg_basebackup` · (2) **faire dire la vérité au SaaS** : le primaire transmet son nombre réel de standbys (`pg_stat_replication`) |
| 📐 **Principe** | Jamais de dégradation silencieuse — un tableau de bord n'affiche que ce qu'il a *mesuré* |

> **De l'incident à l'amélioration produit** : le correctif a fermé la dégradation silencieuse et a inspiré la perspective d'une supervision IA prédictive.

<!-- SPEAKER NOTES
"Cet incident résume la posture ingénieur. Le réflexe technicien : redémarrer. Mais ça ne réglait rien. J'ai compris que le tableau de bord mentait parce qu'il se fiait à une déclaration, pas à une mesure. J'ai corrigé la cause — et j'en ai tiré une fonctionnalité : la supervision intelligente."
-->

---

## Compétences développées

**Techniques**

| Domaine | Compétences acquises |
|---------|---------------------|
| 🦀 Cœur système | Rust, sérialisation CBOR, journal append-only |
| 🔐 Cryptographie | libsodium, XChaCha20-Poly1305, X25519, Argon2id, zero-knowledge |
| 🗄️ Systèmes distribués | PostgreSQL réplication, failover/quorum, fencing, MVCC |
| 📊 Big Data & IA | Séries temporelles de métriques, détection d'anomalies, agent de supervision |
| 🐳 DevOps | Docker multi-arch, docker-compose, registre privé, watchtower |
| ⚡ Backend SaaS | Django, API REST, modèle tenant/licence/parc |

**Transversales**
- 🎯 Diagnostic en production — split-brain résolu structurellement
- 📝 Rigueur documentaire — plan directeur, guide de test, journal technique
- 🧭 Autonomie — décisions d'architecture sans supervision permanente
- 💬 Communication — points réguliers avec Mme CHOKRI & M. Amrani

<!-- SPEAKER NOTES
"Au-delà des technologies, ce stage m'a appris à diagnostiquer un système distribué en production — là où chaque couche cache la suivante — et à relier ma spécialité IA & Big Data à un problème d'infrastructure concret."
-->

---

## Bilan et perspectives

**Ce qui a été accompli**

| ✅ | Résultat |
|----|---------|
| 6/6 | Scénarios métier validés sur banc réel |
| Zero-knowledge | Hiérarchie de clés DEK prouvée cross-OS |
| Réplication | PostgreSQL primaire/standby + détection de panne |
| Reporting véridique | Le SaaS reflète l'état réel de la réplication |
| Supervision IA | Agent implémenté (Mistral) : clusters + serveur éditeur, temps réel, rapport PDF |

**Valeur pour AL BARAA CONSULTING** — framework réutilisable pour vendre tout logiciel métier en SaaS souverain · argument commercial fort (conformité AUDPF par design) · différenciation sur le marché africain.

**Perspectives techniques**

| Horizon court | Horizon moyen |
|---------------|---------------|
| Fencing par jeton d'époque (failover sûr) | Modèle IA entraîné (Isolation Forest) → prédiction de panne |
| IP statiques sur réseau de réplication | Quorum ≥ 3 nœuds (failover automatique) |
| Module métier v1 (gestion de stock) | Frontend Tauri + mobile (UniFFI) |

<!-- SPEAKER NOTES
"C'est un socle prouvé, pas un produit fini. La priorité technique suivante est le fencing, pour sécuriser le failover automatique. Et l'agent IA, aujourd'hui en preuve de concept, a vocation à devenir une supervision prédictive en production."
-->

---

## Conclusion

**Techniquement** — un éditeur peut vendre un logiciel métier en **SaaS complet** (comptes, licences, parc, mises à jour) tout en étant **cryptographiquement incapable** de lire les données de ses clients.

**Stratégiquement**
- **6/6** scénarios validés sur banc réel multi-OS
- **Zero-knowledge** garanti par la hiérarchie de clés, pas par un contrat
- **Résilience** de niveau base de données + supervision intelligente
- **Conforme AUDPF** — par design, pas par configuration

**Personnellement** — ce projet m'a confirmé que la valeur d'un ingénieur réside dans la **qualité des décisions architecturales**, la **rigueur du diagnostic**, et la capacité à relier l'**IA & le Big Data** à un problème d'infrastructure réel.

### La souveraineté numérique africaine commence ici.

`[LOGO EIGSI + LOGO AL BARAA CONSULTING]`
*Jesse MPIGA-ODOUMBA — EIGSI Casablanca — Promotion 2026*

<!-- SPEAKER NOTES
Pause. Regarder le jury.
"Je vous remercie pour votre attention. Je suis à votre disposition pour répondre à vos questions."
(Ne pas dépasser 33 minutes — chronomètre personnel conseillé.)
-->

---

# Slides de réserve

*(Ne pas présenter — uniquement si le jury pose la question)*

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

## RÉSERVE D — Le split-brain : comment l'éviter définitivement

> **Fencing par jeton d'époque monotone** : à chaque promotion, l'époque s'incrémente. Un ancien primaire qui revient avec une époque inférieure est isolé — il ne peut plus accepter d'écritures. C'est la prochaine étape technique.

---

## RÉSERVE E — L'agent IA : comment il est implémenté

| Élément | Implémentation actuelle |
|---------|-------|
| Données collectées | Séries temporelles en base (`ClusterMetricSample`) : heartbeats, streaming_standby_count, WAL lag, failover_count + métriques serveur éditeur (CPU, RAM, disque, base) |
| Moteur | Agent LLM **Mistral** (free tier) ; repli local déterministe si indisponible |
| Sortie | Verdict JSON : risque (sain/surveiller/critique) + score /100 + diagnostic + recommandation + raisonnement détaillé |
| Périmètre | Clusters PME **et** serveur éditeur · page temps réel animée · rapport PDF |
| Zero-knowledge | Seules des métriques d'infrastructure sont transmises — aucune donnée métier |
| Perspective | Remplacer le LLM par un modèle entraîné (Isolation Forest / z-score) pour la **prédiction** sur longues séries |

> L'agent est fonctionnel et démontrable. La perspective honnête restante : passer du diagnostic (LLM) à la prédiction (modèle entraîné sur l'historique).

---

## RÉSERVE F — Sécurité : que se passe-t-il si le relais est saisi ?

> Rien d'exploitable. Le relais ne stocke que des **blobs chiffrés** et une copie de la DEK **scellée sous le code de récupération du client**. Sans ce code (que l'éditeur ne connaît pas), aucun déchiffrement n'est possible.

---

*Fin du support de soutenance — MPIGA-ODOUMBA Jesse · EIGSI Casablanca · Promotion 2026 · AL BARAA CONSULTING · Soutenance 01/07/2026*