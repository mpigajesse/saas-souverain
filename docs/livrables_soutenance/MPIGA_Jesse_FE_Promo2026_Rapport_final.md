# Rapport de Stage — Expérience Professionnelle de Fin d'Études
<!-- NOM FICHIER MOODLE : MPIGA_Jesse_FE_Promo2026_Rapport_final.pdf -->

---

<!-- PAGE DE COUVERTURE -->

[PHOTO: Logo EIGSI Casablanca — à insérer en haut à gauche]

[PHOTO: Logo AL BARAA CONSULTING — à insérer en haut à droite]

---

**RAPPORT DE STAGE**
**EXPÉRIENCE PROFESSIONNELLE DE FIN D'ÉTUDES**
**EIGSI Casablanca — Spécialité Big Data & Intelligence Artificielle — Promotion 2026**

---

**Titre de la mission :**
**Conception et Implémentation d'un Framework SaaS Souverain pour Logiciels Métier Distribués — أمان (Amān)**

---

**Date de début :** 19 février 2026
**Date de fin :** 06 Aout 2026

---

| **Encadrant Entreprise** | **Tuteur EIGSI** | **Étudiant** |
|---|---|---|
| Mme Soumia CHOKRI | M. Ayoub AMRANI | Jesse MPIGA-ODOUMBA |
| Directrice Générale | Enseignant IABD | Élève-Ingénieur |
| AL BARAA CONSULTING | EIGSI Casablanca | Promo 2026 |
| chokri.soumaya90@gmail.com | amraniayoub88@gmail.com | jesse.mpiga.26@eigsica.ma |

---

**Année académique : 2025–2026**

---

> *Ce rapport a bénéficié de l'appui de Claude AI (Anthropic) pour la structuration de certains paragraphes et la reformulation d'extraits de documentation technique. L'analyse, la réflexion critique et l'ensemble des décisions techniques sont le fruit du travail de l'auteur.*

---

## RÉSUMÉ

Ce rapport présente les travaux réalisés lors d'un stage de fin d'études de 24 semaines au sein d'AL BARAA CONSULTING, portant sur la conception et l'implémentation d'un **Framework SaaS Souverain** pour logiciels métier distribués, dont le logiciel métier open source **أمان (Amān)** (de l'arabe *amān*, « sécurité, confiance »). Face à la dépendance critique des organisations africaines aux solutions cloud centralisées étrangères, la mission consistait à permettre à un éditeur de vendre un logiciel métier en **mode SaaS** (comptes, licences, mises à jour, suivi du parc) tout en étant **cryptographiquement incapable** de lire les données de ses clients — conformément aux directives de l'AU Data Policy Framework (AUDPF). La solution repose sur **trois acteurs** : un *SaaS éditeur* (Django) qui gère le commercial, un *relais zero-knowledge* qui ne stocke que des blobs **chiffrés opaques**, et des *clusters PME* qui exécutent le logiciel et détiennent les données **en clair, sur leurs propres machines**. Le cœur est en **Rust** ; la cryptographie repose sur **libsodium** (XChaCha20-Poly1305, X25519, Argon2id) avec une DEK unique par entreprise ; la persistance utilise **PostgreSQL** en réplication primaire/standby (slot de réplication, **fencing** par *timeline* contre le split-brain) ; le packaging est en **Docker** ; un **agent IA** (Mistral) supervise les trois acteurs sans jamais accéder aux données métier. Le prototype a été validé sur un banc multi-OS (Windows 11, Ubuntu, Kali Linux, Debian) : **6/6 scénarios métier** et **4/4 protections de résilience** (réplication par slot, anti-promotion, fencing, auto-réparation), deux tenants souverains indépendants, et un déploiement **auto-adaptatif** sans configuration manuelle.

**Mots-clés :** souveraineté des données, SaaS souverain, zero-knowledge, Rust, libsodium, PostgreSQL, réplication primaire/standby, fencing, Docker, Django, agent IA, AUDPF, Afrique.

---

## REMERCIEMENTS

Je tiens à adresser mes sincères remerciements à toutes les personnes qui ont contribué à la réussite de ce stage.

Je remercie tout d'abord **Mme Soumia CHOKRI**, Directrice Générale d'AL BARAA CONSULTING et encadrante entreprise, pour la confiance qu'elle m'a accordée en me confiant la responsabilité complète de ce projet stratégique. Sa vision et son encadrement ont été déterminants dans la conduite de la mission.

Je remercie **M. Ayoub AMRANI**, mon tuteur pédagogique à l'EIGSI, pour son suivi régulier, ses conseils méthodologiques et sa capacité à maintenir l'alignement entre l'expérience terrain et les exigences de la formation d'ingénieur.

Je remercie également l'ensemble du **corps enseignant de l'EIGSI Casablanca** pour la formation solide en Big Data et Intelligence Artificielle qui m'a permis d'aborder ce stage avec les compétences nécessaires à sa réalisation.

---

## TABLE DES MATIÈRES

<!-- Générer automatiquement dans Word avec les styles Titre -->

- RÉSUMÉ
- REMERCIEMENTS
- TABLE DES MATIÈRES
- TABLE DES ILLUSTRATIONS
- LISTE DES ABRÉVIATIONS
- INTRODUCTION GÉNÉRALE
- PARTIE I — Présentation de l'Entreprise et Contexte du Stage
  - I.1. AL BARAA CONSULTING : Présentation et positionnement
  - I.2. Cadre de travail et organisation de la mission
  - I.3. Analyse fonctionnelle
  - I.4. État de l'art et positionnement technologique
  - I.5. Objectifs SMART de la mission
- PARTIE II — Bilan Technique
  - II.1. Méthodologie de travail
  - II.2. Architecture technique globale — trois acteurs
  - II.3. Le cœur cryptographique (Rust + libsodium)
  - II.4. Le journal append-only CBOR chiffré
  - II.5. Enrôlement d'un appareil par QR code
  - II.6. Persistance et réplication PostgreSQL
  - II.7. Haute disponibilité : failover, quorum et fencing
  - II.8. Le relais zero-knowledge
  - II.9. Le SaaS éditeur (Django)
  - II.10. Le logiciel métier côté PME
  - II.11. Déploiement auto-adaptatif
  - II.12. Agent IA de supervision (Mistral)
  - II.13. Tests et validation sur banc réel
  - II.14. Difficultés rencontrées et solutions apportées
  - II.15. Conclusion technique
- PARTIE III — Bilan de l'Expérience
  - III.1. Compétences mobilisées et acquises
  - III.2. Valeur ajoutée pour AL BARAA CONSULTING
  - III.3. Réflexion sur la posture ingénieur
  - III.4. Bilan personnel et perspectives
- CONCLUSION GÉNÉRALE
- RÉFÉRENCES BIBLIOGRAPHIQUES
- ANNEXES

---

## TABLE DES ILLUSTRATIONS

<!-- À compléter au fil de la rédaction -->

| Figure | Titre | Page |
|--------|-------|------|
| Figure 1 | Diagramme Bête à Cornes — finalité du framework | — |
| Figure 2 | Diagramme Pieuvre — fonctions de service et contraintes | — |
| Figure 3 | Diagramme FAST — décomposition fonctionnelle | — |
| Figure 4 | Positionnement concurrentiel — trois modèles de SaaS | — |
| Figure 5 | Architecture à trois acteurs (SaaS · relais · cluster PME) | — |
| Figure 6 | Hiérarchie de clés (DEK, sealed box, récupération) | — |
| Figure 7 | Sealed box X25519 — enrôlement d'un appareil | — |
| Figure 8 | Format du journal append-only CBOR chiffré | — |
| Figure 9 | Réplication streaming PostgreSQL primaire → standbys | — |
| Figure 10 | Arbre de décision du failover (quorum) | — |
| Figure 11 | Le relais zero-knowledge (entrées / sorties) | — |
| Figure 12 | Topologie du banc de validation (2 PME, 4 OS) | — |
| Figure 13 | Flux de l'agent IA de supervision (Mistral) | — |
| Tableau 1 | Fonctions principales et contraintes | — |
| Tableau 2 | Campagne métier — 6/6 scénarios validés | — |
| Tableau 3 | Campagne haute disponibilité — 4/4 protections | — |
| Tableau 4 | Critères de succès du spike Phase 0 | — |

---

## LISTE DES ABRÉVIATIONS

| Abréviation | Signification |
|-------------|--------------|
| API | Application Programming Interface |
| Argon2id | Fonction de dérivation de clé *memory-hard* (variante id) |
| AUDPF | African Union Data Policy Framework |
| CBOR | Concise Binary Object Representation (sérialisation binaire) |
| DEK | Data Encryption Key — clé de chiffrement des données, unique par entreprise |
| EIGSI | École d'Ingénieurs en Génie des Systèmes Industriels |
| HTTP / HTTPS | HyperText Transfer Protocol (Secure) |
| IABD | Intelligence Artificielle et Big Data |
| JSON | JavaScript Object Notation |
| MAD | Dirham marocain |
| mDNS | multicast Domain Name System (abandonné — remplacé par l'annonce au relais) |
| MVCC | Multi-Version Concurrency Control (PostgreSQL) |
| P2P | Pair-à-Pair (Peer-to-Peer) |
| pg_promote | Fonction PostgreSQL de promotion d'un standby en primaire |
| PFE | Projet de Fin d'Études |
| RACI | Responsible, Accountable, Consulted, Informed |
| REST | Representational State Transfer |
| SARL | Société à Responsabilité Limitée |
| SMART | Specific, Measurable, Achievable, Realistic, Time-bound |
| TLI | Timeline ID — compteur monotone de promotion PostgreSQL (= époque) |
| UUID | Universally Unique Identifier |
| WAL | Write-Ahead Log (journal des transactions PostgreSQL) |
| WBS | Work Breakdown Structure |
| X25519 | Échange de clés Diffie-Hellman sur Curve25519 |
| XChaCha20-Poly1305 | Chiffrement authentifié à nonce étendu (libsodium) |
| ZK | Zero-Knowledge (le relais stocke sans pouvoir déchiffrer) |

---

## INTRODUCTION GÉNÉRALE

Le continent africain traverse une transformation numérique profonde. Avec plus de 1,4 milliard d'habitants et une économie numérique en forte croissance, les organisations africaines — entreprises, administrations, PME — s'appuient massivement sur des solutions cloud proposées par des acteurs étrangers : Amazon Web Services, Microsoft Azure, Google Cloud. Cette dépendance, si elle offre des avantages indéniables en matière de disponibilité et de scalabilité, soulève une question stratégique fondamentale : **qui contrôle les données africaines ?**

En décembre 2025, l'Union Africaine a validé l'**AU Data Policy Framework (AUDPF)**, un cadre réglementaire qui affirme le principe de souveraineté numérique : les données des organisations africaines doivent rester sous contrôle local, ne pas transiter sans consentement explicite par des serveurs étrangers, et être protégées par des mécanismes de sécurité robustes. Ce cadre crée une opportunité et un impératif : développer des alternatives souveraines aux solutions cloud centralisées.

C'est précisément dans ce contexte stratégique que s'inscrit la mission confiée par **AL BARAA CONSULTING** : concevoir et implémenter un **Framework SaaS Souverain** pour logiciels métier distribués — dont le logiciel métier open source **أمان (Amān)**. Ce projet vise à démontrer qu'un éditeur peut commercialiser un logiciel métier en mode SaaS (comptes, licences, mises à jour) tout en garantissant **cryptographiquement** qu'il ne pourra jamais lire les données métier de ses clients, lesquelles restent sur les machines de la PME — sans aucune dépendance à un cloud étranger.

La problématique centrale de ce stage peut ainsi être formulée : *Comment un éditeur peut-il vendre un logiciel métier en mode SaaS — avec gestion des comptes, des licences et des mises à jour — tout en garantissant, par construction cryptographique et non par contrat, qu'il ne pourra jamais accéder aux données métier de ses clients ?*

Pour répondre à cette question, le présent rapport s'articule en trois parties. La **première partie** pose le cadre de la mission : présentation d'AL BARAA CONSULTING, analyse fonctionnelle du besoin, état de l'art et objectifs SMART. La **deuxième partie**, la plus substantielle, présente le bilan technique du framework (architecture à trois acteurs, cœur Rust, réplication PostgreSQL, fencing, relais zero-knowledge, agent IA) : modules implémentés, tests de validation et difficultés surmontées. La **troisième partie** propose un bilan de l'expérience : compétences développées, valeur ajoutée pour l'entreprise et réflexion sur la posture d'ingénieur.

> *Ce stage a été réalisé du 19 février au 01 juillet 2026 au sein d'AL BARAA CONSULTING, Casablanca, dans le cadre de la formation d'ingénieur spécialité Big Data & IA de l'EIGSI Casablanca, Promotion 2026.*

---

## PARTIE I — Présentation de l'Entreprise et Contexte du Stage

### I.1. AL BARAA CONSULTING : Présentation et Positionnement

[PHOTO: Logo AL BARAA CONSULTING — haute résolution]

[PHOTO: Façade ou bureau AL BARAA CONSULTING — Résidence Al Amane, Ain Sebaa, Casablanca]

**AL BARAA CONSULTING** est un cabinet de conseil et d'ingénierie numérique fondé en **mars 2017**, constitué sous la forme juridique de Société à Responsabilité Limitée à Associé Unique (SARL AU) avec un capital de **100 000 MAD**. Le cabinet est basé à **Casablanca** (Résidence Al Amane GH31, Imm. 253, Appartement 1, Ain Sebaa, 20410 Casablanca) et est dirigé par **Mme Soumia CHOKRI**, Directrice Générale.

AL BARAA CONSULTING intervient sur des problématiques complexes alliant développement logiciel, architecture de systèmes d'information et transformation numérique, pour une clientèle à la fois publique et privée. Le cabinet se distingue par une approche intégrée : chaque mission débute par une analyse approfondie du contexte client avant de produire une solution sur mesure, plutôt qu'une solution générique clé en main.

**Domaines d'intervention :**
- Développement d'applications web et mobiles sur mesure
- Architecture de systèmes d'information distribués
- Conseil en transformation numérique
- Systèmes d'Information Géographique (SIG) et géomatique appliquée
- Solutions de souveraineté numérique et infrastructure locale

**Données clés :**

| Paramètre | Valeur |
|-----------|--------|
| Statut juridique | SARL AU |
| Année de création | Mars 2017 |
| Capital | 100 000 MAD |
| Siège social | Ain Sebaa, Casablanca |
| Dirigeant | Soumia CHOKRI (DG) |
| Secteur | Ingénierie numérique, Conseil SI |
| Clientèle | Publique et privée (B2B) |

La structure à taille humaine du cabinet constitue à la fois un atout et un défi. Elle favorise la réactivité, la communication directe et une forte responsabilisation de chaque collaborateur sur ses missions. Pour chaque projet, le stagiaire ou collaborateur devient le référent technique principal, ce qui crée des conditions d'apprentissage exigeantes et particulièrement formatrices.

**Références clients :** AL BARAA CONSULTING a mené des missions pour des donneurs d'ordre publics et privés de premier plan, notamment dans l'aménagement du territoire, l'urbanisme et les systèmes d'information géographique :

| Secteur public | Secteur privé |
|----------------|---------------|
| Ministère de l'Intérieur | Société Marocaine d'Ingénierie Immobilière |
| Ministère de l'Urbanisme, de l'Habitat et de la Politique de la Ville | NavCities |
| Commune de Casablanca | Vector Engineering |
| Agence Urbaine de Rabat | YZ Prestations |
| | Top Service Telecom |

> Ce portefeuille — institutions publiques manipulant des données sensibles (urbanisme, foncier, citoyens) et entreprises privées — illustre la pertinence directe d'une solution de **souveraineté des données** : ces organisations ont une exigence forte de garder leurs données sous contrôle local. Le projet présenté ici répond à ce besoin de marché concret.

[PHOTO: Organigramme AL BARAA CONSULTING — structure organisationnelle simplifiée]

**Enjeu stratégique du projet pour AL BARAA :** Le **Framework SaaS Souverain — أمان (Amān)** représente pour le cabinet une première référence dans le domaine des logiciels métier distribués et souverains — un segment en forte croissance en Afrique, porté par la dynamique de l'AUDPF et la prise de conscience des organisations concernant la souveraineté de leurs données. La réussite du prototype offre au cabinet un produit démontrable et déployable chez ses clients publics et privés (cf. références ci-dessus) souhaitant un logiciel métier en mode SaaS **sans confier leurs données à un cloud** — l'éditeur gère comptes, licences et mises à jour sans jamais pouvoir lire les données métier.

---

### I.2. Cadre de Travail et Organisation de la Mission

Dès le premier jour, j'ai été positionné comme **développeur et architecte principal** du Framework SaaS Souverain — أمان (Amān), sous la supervision directe de Mme Soumia CHOKRI. Cette configuration, exigeante, s'est révélée très formatrice : elle m'a confronté à la réalité d'un projet de production avec toutes ses contraintes de responsabilité, de prise de décision autonome et de livraison. Concrètement, j'étais à la fois architecte (concevoir l'architecture à trois acteurs), développeur du cœur Rust, intégrateur (Docker, PostgreSQL, Django) et testeur sur banc réel.

**Conditions de travail :**
- Poste de travail : PC Windows 11 Pro (hôte du SaaS éditeur, du relais et du premier nœud de test)
- Environnement de développement : Visual Studio Code + chaîne Rust (Cargo, rustfmt, clippy) ; Python 3 pour le SaaS Django
- Versionnement : Git / GitHub — *workspace* Cargo `spike/` (crates `ss-crypto`, `ss-journal`, `ss-consensus`, binaires `ss-node` et `ss-relay`), déploiements `pme-deploy/` et `relay-deploy/`
- Conteneurisation : Docker Desktop (Windows) / Docker Engine (Linux)
- Infrastructure de test : PC physique (Win11) + VMs VMware (Ubuntu, Kali Linux, Debian)
- Réseaux de test : deux LAN PME distincts et réalistes — `192.168.200.0/24` (tenant « MPJ ») et `192.168.10.0/24` (tenant « Yasmine Argan ») — le SaaS éditeur étant joignable depuis chacun, comme un SaaS public

**Organisation du suivi :**
- Points hebdomadaires avec Mme CHOKRI sur l'avancement et les choix d'architecture
- Contacts pédagogiques avec M. Ayoub AMRANI (tuteur EIGSI) selon le calendrier EIGSI
- Documentation continue des décisions techniques dans un journal de déploiement — c'est cette traçabilité qui m'a permis de remonter à la cause racine le jour où le cluster a divergé (cf. II.14)

---

### I.3. Analyse Fonctionnelle

#### I.3.1. Bête à Cornes

L'analyse « Bête à Cornes » formalise la finalité du framework :

> 🖼️ **[FIGURE — Bête à Cornes]** *(à créer sous draw.io)* — À qui rend-il service ?
> Sur quoi agit-il ? Dans quel but ?

- **À qui rend-il service ?** À **deux acteurs à la fois** : à l'**éditeur de logiciel** (vendre un logiciel métier « en mode SaaS » : comptes, licences, mises à jour, suivi du parc) et à la **PME cliente africaine** (garder ses données métier sous son propre contrôle).
- **Sur quoi agit-il ?** Sur les **données métier** de la PME (stock, mouvements, comptes employés) et sur leur **clé de chiffrement** (la DEK).
- **Dans quel but ?** Permettre le modèle SaaS **tout en garantissant, par construction cryptographique et non par contrat, que l'éditeur ne pourra jamais lire les données métier** de ses clients.

#### I.3.2. Diagramme Pieuvre

> 🖼️ **[FIGURE — Pieuvre]** *(à créer sous draw.io)* — Le framework au centre, ses
> fonctions de service et de contrainte vers son environnement.

Le diagramme pieuvre identifie les interactions entre le framework et son environnement :

| ID | Fonction | Critère | Niveau atteint |
|----|----------|---------|----------------|
| FP1 | Chiffrer les données métier localement | DEK unique par entreprise, jamais en clair hors PME | Validé (tests crypto) |
| FP2 | Répliquer les écritures entre machines de la PME | < 1 s, 0 perte | Validé (banc MPJ) |
| FP3 | Gérer comptes, licences et parc côté éditeur | SaaS sans accès aux données | Implémenté (Django) |
| FC1 | Rendre l'éditeur **incapable** de lire les données | Zero-knowledge par construction | Garanti par design |
| FC2 | Résister à la panne d'une machine | Failover + fencing sans split-brain | 4/4 protections validées |
| FC3 | Fonctionner sur plusieurs OS | Windows + Linux | 4 OS validés |
| FC4 | Découvrir les nœuds du cluster | Annonce au relais éditeur | Fonctionnel |
| FC5 | Respecter la conformité AUDPF | Données 100 % dans le périmètre PME | Garanti par design |

*Tableau 1 : Fonctions principales et contraintes du framework*

#### I.3.3. Diagramme FAST

> 🖼️ **[FIGURE — FAST]** *(à créer sous draw.io)*

La décomposition FAST articule les fonctions autour de quatre axes, qui structurent
aussi la Partie II :
- **Souveraineté (FP1, FC1) :** clé DEK (XChaCha20-Poly1305) → sealed box d'enrôlement (X25519) → journal append-only chiffré (CBOR) → relais zero-knowledge (blobs opaques).
- **Disponibilité (FP2, FC2) :** réplication streaming PostgreSQL → bascule manuelle (< 3 nœuds) / failover par quorum (≥ 3) → fencing par *timeline* contre le split-brain.
- **Commercialisation (FP3) :** SaaS Django → comptes, licences, parc → installateur auto-adaptatif (Docker, 1 commande).
- **Supervision (transverse) :** agent IA (Mistral) sur les trois acteurs, sans jamais franchir la frontière souveraine.

---

### I.4. État de l'Art et Positionnement Technologique

Avant de concevoir l'architecture, j'ai positionné le projet face aux modèles
existants de SaaS métier. La question n'est pas « stocker des fichiers de façon
décentralisée », mais « vendre un logiciel en mode SaaS sans pouvoir lire les
données du client ». Trois modèles s'opposent.

> 🖼️ **[FIGURE — Positionnement]** *(version visuelle du tableau ci-dessous)*

| Critère | SaaS cloud classique (AWS/Azure/GCP) | SaaS « chiffré côté serveur » | **SaaS Souverain — أمان** |
|---------|--------------------------------------|-------------------------------|----------------------------|
| Où vivent les données | Serveurs de l'éditeur/cloud étranger | Serveurs de l'éditeur (chiffrées) | **Machines de la PME** |
| Qui détient la clé | L'éditeur / le cloud | **L'éditeur** | **La PME seule** |
| L'éditeur peut-il lire ? | Oui | Oui (il a la clé) | **Non — impossible par construction** |
| Garantie | Contractuelle | Contractuelle | **Cryptographique** |
| Disponibilité hors-ligne | Non | Non | **Oui (local-first)** |
| Résilience à la panne | Dépend du cloud | Dépend du cloud | **Réplication locale + failover** |
| Conformité AUDPF | Partielle / non | Partielle | **Totale** |

*Tableau de positionnement concurrentiel : trois modèles de SaaS métier*

Trois piliers différenciateurs émergent :

1. **Le zero-knowledge par construction, pas par promesse.** Un SaaS « chiffré côté serveur » rassure par contrat, mais l'éditeur détient la clé : sous contrainte légale ou compromission, il peut déchiffrer. Ici, la DEK ne quitte jamais la PME en clair — l'éditeur est techniquement incapable de lire.

2. **La souveraineté infrastructurelle.** Les données vivent sur les machines de la PME ; l'infrastructure *est* le parc de l'entreprise. Aucune dépendance à un cloud étranger, ce qui répond directement à l'AUDPF et au « Gap de connectivité » africain (continuité de service même réseau coupé).

3. **Ne réinventer ni la crypto ni le consensus.** Tout repose sur des briques éprouvées — libsodium pour la cryptographie, la réplication native et le *timeline* de PostgreSQL pour la résilience. L'effort d'ingénierie porte sur l'**architecture** (les trois acteurs, la hiérarchie de clés, le fencing), pas sur la réimplémentation de primitives risquées.

---

### I.5. Objectifs SMART de la Mission

Le Plan Directeur, déposé en mars 2026, définit 5 objectifs SMART structurant l'ensemble du stage. La règle d'or — **le socle d'abord, le métier ensuite** — impose que le cœur (crypto, journal, failover) soit prouvé avant toute logique métier.

| # | Objectif | Mesure | Résultat |
|---|----------|--------|----------|
| O1 | **Architecture & spécification** — trois acteurs, hiérarchie de clés, choix de stack actés | Plan Directeur validé avant 26/03/2026 | ✅ Livré |
| O2 | **Socle cryptographique prouvé** — DEK cross-OS, sealed box d'enrôlement, journal append-only CBOR chiffré | Tests `cargo test` verts, illisible sans DEK | ✅ Validé |
| O3 | **Réplication & résilience** — PostgreSQL primaire/standby, bascule manuelle / failover quorum, fencing | Banc réel : 6/6 scénarios + 4/4 protections, 0 split-brain | ✅ Validé |
| O4 | **SaaS éditeur** — comptes, licences, parc, déploiement auto-adaptatif | Inscription → installateur 1 commande, 2 tenants souverains | ✅ Validé |
| O5 | **Supervision IA & clôture** — agent Mistral sur 3 acteurs, rapport et soutenance | Diagnostic persisté, zero-knowledge préservé | ✅ Validé |

---

## PARTIE II — Bilan Technique

Cette partie constitue le cœur du rapport. Elle décrit ce que j'ai réellement
conçu et construit pendant le stage : un **framework SaaS souverain** organisé
autour de trois acteurs, dont un cœur cryptographique en Rust, une réplication
PostgreSQL durcie contre le *split-brain*, un relais *zero-knowledge*, un SaaS
éditeur Django et un agent de supervision intelligent. J'ai fait le choix d'aller
du général au détail : l'architecture d'ensemble d'abord, puis chaque brique, ses
décisions de conception, et enfin la campagne de validation sur banc réel.

> 📌 **Convention de lecture.** Les emplacements `📸 [CAPTURE — …]` et
> `🖼️ [FIGURE — …]` signalent les illustrations à insérer dans la version finale.
> Les captures d'écran référencées seront refaites proprement (terminaux lisibles,
> fond clair) avant remise. Les renvois `→ Annexe X` pointent vers la section
> ANNEXES en fin de rapport.

---

### II.1. Méthodologie de Travail

La règle qui a gouverné tout le stage tient en une phrase, posée dès le cadrage
avec Mme CHOKRI : **le socle d'abord, le métier ensuite**. Un framework de
souveraineté ne vaut que si sa fondation — la cryptographie, la sérialisation des
écritures, le basculement entre machines — est prouvée avant qu'une seule ligne de
logique métier ne s'appuie dessus. Si le socle bouge après coup, tout ce qui a été
bâti dessus est à réécrire. J'ai donc travaillé selon trois principes.

**Principe 1 — Dérisquer par un spike avant de produire.** Plutôt que d'écrire
directement un logiciel de gestion, j'ai commencé par un *spike* : un banc d'essai
minimal dont le seul but était de répondre à des questions binaires. Le
chiffrement traverse-t-il Windows, Ubuntu et Kali à l'identique ? Un standby
PostgreSQL se promeut-il proprement ? Un ancien primaire qui revient peut-il
provoquer une double écriture ? Tant que ces réponses n'étaient pas « oui »
démontré, je m'interdisais le métier. C'est exactement la discipline attendue d'un
ingénieur : ne pas confondre « ça compile » et « c'est prouvé ».

**Principe 2 — Ne jamais réimplémenter une primitive critique.** Deux familles de
problèmes sont des pièges classiques où l'amateurisme coûte cher : la cryptographie
et le consensus distribué. Pour la première, je m'appuie exclusivement sur
**libsodium** (via les crates Rust officielles `chacha20poly1305`, `x25519-dalek`,
`argon2`) — aucune primitive maison. Pour le second, je n'écris aucun algorithme
d'élection : je m'appuie sur la **promotion native de standby PostgreSQL** et sur
son *timeline* comme source de vérité. Ce choix, acté très tôt, m'a évité la classe
de bugs la plus dangereuse du projet.

**Principe 3 — Documenter comme un livrable de premier ordre.** Chaque décision
structurante, chaque incident et chaque solution ont été consignés au fil de l'eau.
Cette traçabilité n'est pas une corvée académique : c'est elle qui m'a permis, le
jour où le cluster a divergé (cf. II.14), de remonter à la cause racine en
relisant mon propre journal plutôt qu'en devinant.

**Outils de travail :**

| Catégorie | Outil | Usage |
|-----------|-------|-------|
| Cœur partagé | **Rust** (édition 2021), Cargo workspace | Crates `ss-crypto`, `ss-journal`, `ss-consensus`, binaires `ss-node`, `ss-relay` |
| Cryptographie | libsodium via `chacha20poly1305`, `x25519-dalek`, `argon2`, `blake2`, `zeroize` | Chiffrement, identité, dérivation |
| Base de données | **PostgreSQL 16** (réplication streaming), SQLite (réplique locale lecture seule) | Persistance + réplication |
| SaaS éditeur | **Django 5** + PostgreSQL + templates serveur | Comptes, licences, parc, supervision |
| Conteneurisation | **Docker** + Docker Compose (multi-arch) | Packaging livré à la PME |
| Versionnement | Git / GitHub | Workspace `spike/`, déploiements `pme-deploy/`, `relay-deploy/` |
| Banc de test | PC Windows 11 + VMs VMware (Ubuntu, Kali, Debian) | Validation cross-OS réelle |

---

### II.2. Architecture Technique Globale — Trois Acteurs

#### II.2.1. Le principe : séparer le commercial du secret

L'idée fondatrice du framework est une séparation stricte des rôles. Un éditeur
veut vendre un logiciel métier « comme un SaaS » : créer des comptes, vendre des
licences, suivre le parc installé, pousser des mises à jour. Mais il ne doit
**jamais** pouvoir lire les données métier de ses clients. La plupart des solutions
du marché règlent cette tension par un *contrat* (« nous promettons de ne pas
regarder »). Mon parti pris est de la régler par **construction cryptographique** :
l'éditeur est techniquement *incapable* de déchiffrer, même s'il le voulait, même
sous contrainte légale.

Cette exigence se traduit par trois acteurs nettement séparés :

| Acteur | Hébergement | Voit le clair ? | Rôle |
|--------|-------------|:---------------:|------|
| **① SaaS éditeur** (Django) | Chez l'éditeur | Données compte/licence **uniquement** | Comptes tenants, licences, suivi du parc, supervision |
| **② Relais zero-knowledge** (Rust) | Chez l'éditeur | **Jamais** | Stockage de blobs **chiffrés opaques** + topologie du cluster |
| **③ Cluster PME** (Rust + PostgreSQL) | Machines de la PME | **Oui** (périmètre souverain) | Exécute le logiciel métier, détient et chiffre les données |

> 🖼️ **[FIGURE 5 — Architecture à trois acteurs]** Schéma d'ensemble : le SaaS
> éditeur et le relais hébergés côté éditeur (zone « ne voit jamais le clair »), le
> cluster PME côté client (zone souveraine). Flèches : annonce HTTP nœud → relais,
> enregistrement → SaaS, dépôt de blob chiffré → relais. *(à créer sous draw.io)*

#### II.2.2. Le fil rouge : la DEK ne sort jamais en clair

Tout le système s'organise autour d'une seule clé : la **DEK** (*Data Encryption
Key*), une clé symétrique de 256 bits, **unique par entreprise**. Elle chiffre les
données métier et le journal. Elle est générée **sur la première machine de la
PME**, jamais chez l'éditeur. Pour qu'elle circule entre les appareils autorisés ou
soit sauvegardée chez l'éditeur, elle est toujours **emballée** (chiffrée) au
préalable. La hiérarchie de clés est la colonne vertébrale du projet :

```
DEK (symétrique 256 bits, unique par entreprise)
  ├─ chiffre  →  données métier + journal CBOR
  ├─ emballée (sealed box X25519)      →  pour chaque appareil autorisé
  └─ emballée (Argon2id + code de récup.) → blob opaque stocké sur le relais
```

> 🖼️ **[FIGURE 6 — Hiérarchie de clés]** La DEK au centre ; trois flèches
> sortantes : « chiffre les données », « scellée pour l'appareil B », « scellée
> sous le code de récupération ». Mention explicite : *le relais ne voit jamais la
> DEK ni aucune clé privée.* *(reprendre le poster de soutenance)*

#### II.2.3. Pourquoi Rust, pourquoi un workspace en crates

J'ai choisi **Rust** pour le cœur partagé pour trois raisons concrètes : la
sécurité mémoire sans ramasse-miettes (essentiel quand on manipule des clés en
mémoire), un écosystème cryptographique mature et audité, et la possibilité de
viser à terme desktop et mobile depuis un **cœur unique** (via UniFFI) — jamais
deux implémentations à maintenir en parallèle. Le cœur est organisé en *workspace*
Cargo, par domaine et non par couche technique :

```
spike/
├── crates/
│   ├── ss-crypto/      # DEK, sealed box X25519, dérivation Argon2id
│   ├── ss-journal/     # journal append-only CBOR chiffré
│   └── ss-consensus/   # époque, fencing, supervision PostgreSQL
├── node/               # binaire ss-node (logiciel PME : CLI + web métier)
└── relay/              # binaire ss-relay (relais zero-knowledge)
```

Cette découpe rend chaque brique testable isolément : `ss-crypto` a ses propres
tests de chiffrement, `ss-journal` ses tests de relecture, sans démarrer la moindre
base de données. C'est ce qui a permis de **prouver le socle avant le métier**.

---

### II.3. Le Cœur Cryptographique (Rust + libsodium)

#### II.3.1. La DEK : chiffrement authentifié XChaCha20-Poly1305

La DEK est implémentée par le type `Dek` de la crate `ss-crypto`. Trois choix de
conception méritent d'être soulignés.

**Le chiffrement est authentifié.** J'utilise **XChaCha20-Poly1305** : le « Poly1305 »
est un tag d'authentification qui garantit qu'un octet modifié dans le chiffré rend
le déchiffrement impossible (et non silencieusement faux). Mon test
`tampered_ciphertext_fails` corrompt délibérément un bit du chiffré et vérifie que
le déchiffrement échoue — c'est la preuve que l'intégrité est protégée, pas
seulement la confidentialité.

**Le nonce est aléatoire et étendu (24 octets).** La variante « X » de ChaCha20
porte un nonce de 192 bits, suffisamment large pour être tiré aléatoirement sans
risque de collision. Concrètement, chiffrer deux fois le même message produit deux
chiffrés différents — ce que vérifie mon test `two_encryptions_differ`. Le format
de sortie est simple et auto-porteur : `nonce (24 octets) ‖ ciphertext`.

**La clé est effacée de la mémoire à sa libération.** Le type `Dek` dérive
`Zeroize` et `ZeroizeOnDrop` : dès qu'une DEK sort de portée, ses 32 octets sont
écrasés en mémoire. C'est une précaution contre la lecture de la clé dans un *dump*
mémoire ou un fichier d'échange.

```rust
// ss-crypto/src/dek.rs — extrait
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct Dek([u8; 32]);

impl Dek {
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let cipher = XChaCha20Poly1305::new(Key::from_slice(&self.0));
        let nonce  = XChaCha20Poly1305::generate_nonce(&mut OsRng);
        let ct     = cipher.encrypt(&nonce, plaintext)
                           .map_err(|_| CryptoError::EncryptionFailed)?;
        // sortie = nonce ‖ ciphertext
        let mut out = Vec::with_capacity(24 + ct.len());
        out.extend_from_slice(&nonce);
        out.extend_from_slice(&ct);
        Ok(out)
    }
}
```

> 📸 **[CAPTURE — tests crypto]** Sortie de `cargo test -p ss-crypto` montrant les
> 5 tests verts (`roundtrip_data`, `wrong_key_fails`, `tampered_ciphertext_fails`,
> `two_encryptions_differ`, `roundtrip_empty`). → Annexe B.

#### II.3.2. Le sealed box : transmettre la DEK à un nouvel appareil

Pour autoriser une nouvelle machine, il faut lui transmettre la DEK **sans que
quiconque sur le réseau ne puisse l'intercepter**, et **sans que l'émetteur ait
besoin de prouver son identité** (l'appareil receveur n'a pas encore de référence).
La primitive adaptée est le *sealed box* de la famille NaCl/libsodium, que
`ss-crypto` implémente sur **X25519** :

1. Chaque appareil possède une paire de clés X25519 (`DeviceKeyPair`).
2. Pour sceller la DEK vers l'appareil B, on génère une paire **éphémère**, on
   calcule un secret partagé par échange Diffie-Hellman (X25519) avec la clé
   publique de B, puis on dérive nonce + clé via **BLAKE2b-512** sur
   `secret_partagé ‖ clé_éphémère ‖ clé_publique_B`.
3. On chiffre la DEK avec XChaCha20-Poly1305 et on renvoie `clé_éphémère ‖ chiffré`.

Seul B, avec sa clé privée, peut reconstituer le même secret partagé et ouvrir le
paquet. L'émetteur reste anonyme (clé éphémère jetée), et deux scellés successifs
de la même DEK diffèrent (test `two_seals_differ`). Mes tests `seal_and_open` et
`wrong_key_cannot_open` prouvent les deux propriétés essentielles : B ouvre, un
tiers n'ouvre pas.

> 🖼️ **[FIGURE 7 — Sealed box X25519]** Diagramme des 5 étapes ci-dessus, de la
> paire éphémère jusqu'à l'ouverture par la clé privée de l'appareil receveur.

#### II.3.3. La récupération : Argon2id sur un code haute entropie

Si la PME perd toutes ses machines, la DEK doit pouvoir renaître. Le mécanisme :
au premier démarrage, le nœud génère un **code de récupération de 160 bits** (au
format groupé `XXXX-XXXX-…`, dérivé de deux UUID v4 cryptographiques), puis dérive
une clé via **Argon2id** (64 Mio de mémoire, 3 itérations, parallélisme 4) à partir
de ce code et d'un sel aléatoire de 16 octets. Cette clé emballe la DEK ; le blob
résultant (`sel ‖ nonce ‖ chiffré`) est déposé sur le relais. Argon2id est un KDF
*memory-hard* : il rend une attaque par force brute coûteuse même avec du matériel
spécialisé. Mes tests vérifient le déterminisme (`deterministic`) et la sensibilité
au code comme au sel (`different_passphrase_different_key`, `different_salt…`).

Le point souverain capital : **le relais reçoit un blob opaque**. Sans le code de
récupération — que seule la PME détient et que le relais ne connaît pas — ce blob
est indéchiffrable. L'éditeur peut donc *conserver* la sauvegarde et la *restituer*,
sans jamais pouvoir la lire.

---

### II.4. Le Journal Append-Only CBOR Chiffré

Toute écriture métier passe par un **journal append-only** : un fichier où l'on
ne fait qu'ajouter, jamais modifier ni supprimer. C'est la base d'une trace
auditable et d'une réplication fiable. La crate `ss-journal` l'implémente avec
trois caractéristiques.

**Sérialisation en CBOR.** Chaque entrée (`JournalEntry`) porte un index monotone,
l'époque, l'identifiant du nœud, l'horodatage, le type d'opération et la charge
utile. Elle est sérialisée en **CBOR** (via `ciborium`), un format binaire compact,
typé et versionnable — préférable au JSON pour un journal qui doit rester stable
dans le temps et économe en place.

**Chiffrement avant écriture disque.** L'entrée CBOR est chiffrée avec la DEK
*avant* d'atteindre le disque. Le format de fichier est une suite de trames
`u32 longueur (little-endian) ‖ blob chiffré`. Conséquence forte : **un journal volé
sans la DEK est illisible**. Mon test `wrong_dek_fails` ouvre un journal avec une
mauvaise DEK et vérifie que la relecture échoue.

**Relecture robuste et reprise d'état.** À l'ouverture, le journal compte ses
trames en lisant uniquement les en-têtes de longueur, sans déchiffrer — il peut
donc déterminer le prochain index même si la DEK n'est pas encore disponible. Les
index sont strictement monotones (test `indices_monotone`) et l'état se reconstruit
après réouverture (test `reopen_restores_state`).

```rust
// ss-journal/src/journal.rs — écriture d'une trame
let blob = self.dek.encrypt(&cbor_buf)?;        // CBOR chiffré par la DEK
let len  = blob.len() as u32;
writer.write_all(&len.to_le_bytes())?;          // en-tête longueur
writer.write_all(&blob)?;                        // puis le blob opaque
```

> 🖼️ **[FIGURE 8 — Format du journal]** Représentation des trames successives
> `len ‖ blob`, avec un zoom sur une entrée CBOR déchiffrée (index, époque,
> op_type, payload).

---

### II.5. Enrôlement d'un Appareil par QR Code

L'enrôlement est le moment où un nouvel appareil rejoint le cercle de confiance.
Le flux combine les briques précédentes :

1. Le nouvel appareil génère sa paire X25519 et **affiche sa clé publique sous
   forme de QR code**.
2. Un appareil déjà autorisé **scanne le QR**.
3. Il **scelle la DEK** (sealed box) pour la clé publique du nouvel appareil.
4. Le nouvel appareil ouvre le blob avec sa clé privée → il détient la DEK.
5. Le **jeton d'invitation est consommé** : usage unique, courte durée de vie.

Ce protocole a une propriété élégante : la DEK transite *chiffrée pour un seul
destinataire*, sur un canal qui peut être totalement public (un QR à l'écran).
Même filmé, le QR ne révèle que la clé *publique* du receveur — inutile à un
attaquant.

> 📸 **[CAPTURE — enrôlement QR]** Écran d'un appareil affichant son QR de clé
> publique, et l'écran de l'appareil autorisé en train de scanner. *(perspective :
> l'enrôlement par QR est une fonctionnalité cible ; le spike a prouvé la mécanique
> cryptographique sous-jacente — sealed box — par tests automatisés.)* → Annexe B.

> ⚠️ **Honnêteté technique.** Dans le périmètre du spike, j'ai prouvé la
> **mécanique cryptographique** de l'enrôlement (génération de paire, scellement,
> ouverture) par tests. L'enrôlement « scan de QR de bout en bout sur deux
> appareils » est une perspective produit, pas une fonctionnalité de production à
> ce stade. Je le présente comme tel pour ne pas survendre le prototype.

---

### II.6. Persistance et Réplication PostgreSQL

#### II.6.1. Pourquoi PostgreSQL plutôt qu'une base distribuée maison

Le besoin métier (stock, mouvements, comptes) exige des **invariants forts** :
unicité d'une référence article, cohérence d'un solde de stock, numérotation sans
trou. Réécrire une base distribuée transactionnelle serait la pire des idées —
c'est un domaine où des équipes entières échouent. J'ai donc retenu **PostgreSQL 16**
et sa **réplication streaming primaire/standby** native :

- le **primaire** accepte les écritures et journalise tout dans son WAL
  (*Write-Ahead Log*) ;
- chaque **standby** rejoue ce WAL en continu et reste une copie cohérente,
  accessible en lecture.

Dans le `docker-compose`, le cluster de validation comprend un `pg-primary` et deux
standbys (`pg-standby1`, `pg-standby2`), chacun adossé à un nœud `ss-node`.

> 🖼️ **[FIGURE 9 — Réplication streaming primaire → standbys]** Le primaire et son
> WAL au centre ; flèches de streaming vers deux standbys ; mention du *slot de
> réplication* qui empêche le primaire de recycler du WAL encore nécessaire.

#### II.6.2. Le slot de réplication : ne jamais perdre du WAL

Un piège classique : si le primaire recycle un segment de WAL avant qu'un standby
lent ne l'ait consommé, le standby décroche définitivement. La parade native est le
**slot de réplication** : le primaire conserve le WAL tant qu'un standby référencé
ne l'a pas confirmé. La validation vérifie que le slot est bien `active=t` (première
protection de la campagne HA, cf. II.13).

#### II.6.3. La cohérence est vérifiée à la source, donc jamais répliquée fausse

Un point que je tiens à expliciter, car il porte toute la promesse de cohérence :
une contrainte comme l'unicité de `articles.code` est vérifiée **sur le primaire,
avant l'écriture du WAL**. Un doublon est donc rejeté *à la source* — il n'entre
jamais dans le journal, donc n'est jamais répliqué. Le standby ne peut pas diverger
du primaire sur ce point. C'est du PostgreSQL natif, éprouvé : **je n'ai réécrit
aucune logique de cohérence à la main**. C'est précisément ce que la démonstration
montre (insertion d'un doublon rejetée, écritures concurrentes sérialisées sans
corruption).

---

### II.7. Haute Disponibilité : Failover, Quorum et Fencing

C'est la partie la plus délicate du projet, et celle dont je suis le plus fier,
parce qu'elle a exigé de comprendre *pourquoi* une solution naïve casse, puis de la
remplacer par une solution structurellement saine.

#### II.7.1. L'époque, c'est le timeline de PostgreSQL

Plutôt que d'inventer un compteur de mandat, j'utilise une grandeur que PostgreSQL
maintient déjà nativement : le **timeline** (TLI). À chaque promotion (`pg_promote`),
PostgreSQL incrémente le timeline. C'est donc un **compteur monotone gratuit et
fiable**, que je lis via `pg_control_checkpoint()`. Un ancien primaire qui revient
conserve son ancien (plus petit) timeline, tandis que le cluster a avancé vers un
timeline supérieur — exactement ce dont le *fencing* a besoin.

```rust
// ss-consensus/src/supervision.rs — l'époque = timeline natif
pub async fn current_timeline(pool: &PgPool) -> Result<i64> {
    let tli: i32 = sqlx::query_scalar(
        "SELECT timeline_id FROM pg_control_checkpoint()"
    ).fetch_one(pool).await?;
    Ok(tli as i64)
}
```

#### II.7.2. La règle du quorum : 2 nœuds ≠ 3 nœuds

La boucle de supervision de chaque standby (`run_supervision_loop`) surveille son
*WAL receiver*. S'il est inactif **3 tics consécutifs de 5 s** (soit 15 s), le
primaire est présumé injoignable. **Mais la décision de basculer dépend de la
taille du cluster :**

- **Moins de 3 nœuds → bascule MANUELLE uniquement.** À 2 nœuds, une auto-promotion
  est dangereuse : si le primaire n'est pas vraiment mort (simple coupure réseau),
  on obtient **deux primaires** — un *split-brain* qui casse définitivement la
  réplication. Le standby s'abstient donc et le SaaS continue d'afficher la perte de
  redondance.
- **3 nœuds ou plus → failover AUTOMATIQUE par quorum.** Le standby interroge le
  SaaS pour connaître la taille réelle du cluster ; si elle est ≥ 3, il appelle
  `pg_promote()`, devient primaire, et son timeline s'incrémente.

```rust
// run.rs — extrait de la logique de décision
if wal_miss >= WAL_FAILOVER_THRESHOLD {           // 3 × 5 s
    let node_count = cluster_node_count().await;  // vérité = SaaS
    if matches!(node_count, Some(n) if n >= 3) {
        // ≥ 3 nœuds → failover automatique par quorum
        supervision::promote_standby(&pool).await?;
    } else {
        // < 3 nœuds OU SaaS injoignable → bascule MANUELLE (anti split-brain)
    }
}
```

> 🖼️ **[FIGURE 10 — Arbre de décision du failover]** WAL inactif → seuil 15 s →
> branche « cluster ≥ 3 » (promotion auto) vs branche « < 3 » (alerte + bascule
> manuelle). Cas SaaS injoignable → fail-safe : pas de promotion.

#### II.7.3. Le fencing : un ancien primaire déchu se clôture tout seul

C'est la pièce maîtresse. Quand un nœud démarre (ou périodiquement « à chaud »), il
compare **son** timeline à celui du cluster, obtenu via le SaaS. La fonction
`check_fencing` tranche :

```rust
// ss-consensus/src/fencing.rs
pub fn check_fencing(claimed: EpochToken, current: EpochToken) -> FencingResult {
    if claimed >= current { FencingResult::Allowed }
    else { FencingResult::Fenced { claimed, current } }
}
```

Si l'époque du nœud est inférieure à celle du cluster, c'est un **ancien primaire
périmé**. Il entre alors dans une boucle de clôture (`run_fenced_idle_loop`) qui ne
lance **jamais** le serveur web métier : **aucune écriture n'est possible, donc pas
de split-brain.** Il se signale au portail comme « standby clôturé » et attend un
re-clone manuel. La preuve, tirée des logs réels du nœud déchu :

```
⛔ NŒUD CLÔTURÉ (FENCED)
Époque de ce nœud (timeline PG) : 1
Époque courante du cluster       : 2
→ REFUSE de servir pour éviter le split-brain.
```

> 📸 **[CAPTURE — fencing en action]** Terminal du nœud déchu affichant la bannière
> ⛔ NŒUD CLÔTURÉ, en regard du SaaS qui montre le cluster sur l'époque 2. → Annexe D.

---

### II.8. Le Relais Zero-Knowledge

Le relais (`ss-relay`, en Rust avec le framework **axum**) est volontairement
**stateless et aveugle**. Il rend trois services, et aucun ne nécessite de
comprendre les données.

**① Topologie du cluster.** Au démarrage et périodiquement, chaque nœud s'annonce
(`POST /api/nodes/announce` : `node_id`, `tenant_id`, adresse, rôle, époque). Le
relais maintient en mémoire la liste des nœuds par tenant et la restitue
(`GET /api/nodes`). C'est ce qui a remplacé la découverte mDNS : sous Docker, le
mDNS est peu fiable, donc **les nœuds se découvrent via le relais** (décision actée).

**② Stockage de blobs opaques.** `PUT/GET/DELETE /api/blobs/{tenant}/{key}` stockent
des octets chiffrés (par exemple le blob de récupération). Le relais écrit, relit,
supprime — sans jamais déchiffrer. Le chemin est assaini (anti-traversée de
répertoire) et l'accès protégé par un jeton optionnel.

**③ Métadonnées sans contenu.** `GET /api/blob-stats` expose, par tenant, le
*nombre* de blobs, leur *taille totale* et la *date* du dernier dépôt — **jamais le
contenu**. L'éditeur sait donc qu'une sauvegarde *existe*, sans pouvoir la *lire*.
La réponse porte explicitement `"zero_knowledge": true`.

```rust
// relay/src/main.rs — le relais ne renvoie que des métadonnées
Json(json!({ "tenants": tenants, "zero_knowledge": true }))
```

Cette discipline est le cœur de la promesse : **même compromis ou saisi, le relais
ne peut rien déchiffrer.** Il ne détient ni la DEK, ni aucune clé privée d'appareil.

> 🖼️ **[FIGURE 11 — Le relais aveugle]** Trois entrées (annonce, blob chiffré,
> requête santé) ; trois sorties (topologie, blob opaque, métadonnées). Au centre,
> un cadenas barré sur « contenu en clair ».

---

### II.9. Le SaaS Éditeur (Django)

Le SaaS éditeur est l'unique acteur « commercial ». Développé en **Django**, il
porte le cycle de vie d'un client sans jamais toucher aux données métier :

- **Inscription d'un tenant** : nom, e-mail, téléphone, nombre d'employés. À la
  validation, le compte est créé, une **licence d'essai** est émise
  automatiquement, et la PME est redirigée vers une page d'installation.
- **Licences** : type d'offre, nombre de postes autorisés, échéance.
- **Parc machines** : chaque nœud s'enregistre via `POST /api/devices/register/`
  avec un identifiant d'installation (**UUID authentifié — pas la MAC**, car la MAC
  est falsifiable et masquée par Docker). Le portail affiche le rôle (primaire /
  standby), l'adresse, l'époque et l'état en ligne.
- **Clusters** : à partir des époques et des métriques de réplication remontées par
  les nœuds, le SaaS calcule l'état du cluster (« sain », « réplication
  interrompue », « bascule manuelle requise ») et conserve le **timeline maximum**
  connu — ce qui sert d'arbitre au fencing.
- **Installateur personnalisé** : la page d'installation génère un script
  (`install-<tenant>.sh` / `.bat`) **embarquant déjà** le jeton du tenant, l'URL du
  SaaS, l'URL du relais, l'adresse du registre Docker et l'image à tirer.

> 📸 **[CAPTURE — SaaS, page Parc machines]** Vue listant les nœuds d'un tenant avec
> rôle, adresse, époque, statut. → Annexe C.
> 📸 **[CAPTURE — SaaS, page Clusters]** États « ✓ Cluster sain » (MPJ, 2 nœuds) et
> « ⚠ Bascule manuelle » (1 nœud). → Annexe C.

#### Point souveraineté : ce que le SaaS voit, et ce qu'il ne voit pas

Le SaaS reçoit des **métriques d'infrastructure** : qui est primaire, combien de
standbys streament, quelle époque, en ligne ou non. Il ne reçoit **aucune donnée
métier** — pas un article, pas un mouvement de stock, pas un nom d'employé en clair
(les comptes employés vivent dans la base PostgreSQL de la PME, répliqués entre ses
propres machines, jamais remontés au SaaS).

---

### II.10. Le Logiciel Métier Côté PME

Le binaire `ss-node` est à la fois la CLI d'administration et le serveur du
logiciel métier. Sa CLI (via `clap`) expose les commandes structurantes :

| Commande | Rôle |
|----------|------|
| `init --first` | Initialise le 1ᵉʳ nœud : génère la paire X25519 **et la DEK** |
| `run --mode active\|passive` | Démarre le nœud (primaire ou standby) |
| `status` | Affiche l'état du nœud |
| `failover` | Promeut ce standby en primaire et incrémente l'époque |
| `delist --device-id` | Dé-enrôle un appareil + rotation de DEK |
| `adduser` | Crée/maj un utilisateur métier |

Au démarrage en mode actif, le nœud applique les migrations (auth + stock), crée si
besoin un **administrateur initial** dont le mot de passe est affiché une seule fois
sur la page de connexion (souverain : il naît sur la machine PME), puis lance le
serveur web métier sur le port 9001 et entre dans sa boucle de supervision.

Le **module stock** est un module métier *minimal mais réel*, conçu pour
démontrer la chaîne complète (écriture → réplication → cohérence), pas pour être un
ERP : articles (avec `code` unique et seuil d'alerte), mouvements de stock
(entrées/sorties), calcul des stocks et alertes de seuil, le tout transactionnel.

> 📸 **[CAPTURE — interface métier]** Page de connexion (avec l'encart
> d'identifiants initiaux), puis liste d'articles + mouvements de stock + alerte de
> seuil. → Annexe C.

---

### II.11. Déploiement Auto-Adaptatif

L'un des apports dont je suis le plus satisfait est l'expérience d'installation :
**une seule commande**, aucune compétence Docker requise côté PME, et un nœud qui
**détecte son propre rôle**.

```bash
sudo bash install-yasmine-argan.sh
```

Le script (généré par le SaaS, donc pré-renseigné) enchaîne cinq étapes :

| Étape | Action | Intelligence embarquée |
|:-----:|--------|------------------------|
| 1/5 | Détecte l'IP réseau | Se configure seul |
| 2/5 | Installe Docker s'il manque + autorise le registre éditeur | Aucune action manuelle |
| 3/5 | **Détecte le rôle** : aucun nœud existant → **PRIMAIRE** ; sinon **STANDBY** | Décision automatique |
| 4/5 | Dérive les mots de passe du jeton, génère `.env` + `docker-compose.yml` | Tout pré-configuré |
| 5/5 | `docker compose up -d` (+ `init --first` sur le primaire → **génère la DEK**) | La clé naît chez la PME |

Le packaging est une **image Docker** tirée d'un registre privé de l'éditeur. Point
cross-OS appris à mes dépens : le tag d'image est **mutable** ; l'installateur fait
donc `docker compose pull` au démarrage pour garantir que la PME tourne bien le
dernier binaire (cf. II.14, incident P4). Sous Windows Docker Desktop, le réseau
utilise un **bridge nommé avec IP fixes** ; sous Linux, le mode hôte suffit.

> 📸 **[CAPTURE — installation 1 commande]** Terminal Debian montrant les 5 étapes
> qui défilent jusqu'à « Installation terminée ! (primary) ». → Annexe C.

> 🖼️ **[FIGURE 12 — Topologie du banc de validation]** Deux PME indépendantes :
> « Yasmine Argan » (1 nœud Debian, LAN 192.168.10.0/24) et « MPJ » (2 nœuds —
> Kali primaire + Ubuntu standby, LAN 192.168.200.0/24), toutes deux pointant vers
> le même SaaS/relais éditeur.

---

### II.12. Agent IA de Supervision (Mistral)

C'est ici que ma spécialité — IA et Big Data — s'incarne concrètement. J'ai
implémenté un **agent de supervision** qui observe les **trois acteurs** (SaaS,
relais, clusters PME) et produit un diagnostic en langage naturel.

**Du seuil binaire à l'interprétation.** Une supervision classique se contente d'un
seuil (« lag > X → alerte »). Mon agent collecte des **séries temporelles** de
métriques (`ClusterMetricSample` : heartbeats ~60 s, nombre de standbys streamant,
lag WAL, époque, compteur de failovers), puis les soumet à **Mistral AI**
(`mistral-small-latest`). Le modèle renvoie un **score de risque 0–100**, un
diagnostic et une recommandation préventive, classés *sain / à surveiller /
critique*. Chaque analyse est **persistée** (`AgentVerdict`), ce qui constitue un
historique exploitable.

**La souveraineté est préservée, par construction.** Trois garde-fous :

| Principe | Mise en œuvre |
|----------|---------------|
| 🔒 Zero-knowledge | Seules des **métriques d'infrastructure** sont transmises au modèle — **aucune donnée métier ne sort jamais** |
| 🛰️ Relais sans intrusion | L'agent lit la **santé** du relais (`/health` : uptime, nb de tenants avec blobs) — **jamais le contenu** des blobs |
| 🛟 Fail-safe | Clé API absente / quota épuisé / réseau coupé → **repli local déterministe** : le dashboard ne casse jamais |

> 🖼️ **[FIGURE 13 — Flux de l'agent IA]** Métriques des 3 acteurs → séries
> temporelles → Mistral → score 0–100 + diagnostic + reco, avec la branche de repli
> local en cas d'indisponibilité de l'API.

> 📸 **[CAPTURE — verdict de l'agent]** Carte de diagnostic dans le SaaS : score de
> risque, texte en langage naturel, recommandation. → Annexe C.

---

### II.13. Tests et Validation sur Banc Réel

La validation ne s'est pas faite sur une seule machine ni en simulation : elle s'est
faite sur un **banc multi-OS réel** — Windows 11, Ubuntu, Kali Linux et Debian — et
sur **deux tenants souverains indépendants** (« Yasmine Argan », 1 nœud ; « MPJ »,
2 nœuds). Deux campagnes complémentaires.

#### II.13.1. Campagne métier — 6/6 scénarios (tenant MPJ)

Banc : **Kali (primaire) + Ubuntu (standby)**, réplication streaming asynchrone
confirmée par `pg_stat_replication`.

| # | Scénario | Garantie démontrée | Verdict |
|---|----------|--------------------|:-------:|
| 1 | Création de 2 articles + entrées de stock | Réplication primaire → standby | ✅ |
| 2 | Sorties de stock par 2 employés | Base unique partagée | ✅ |
| 3 | Stocks calculés + alerte de seuil | Cohérence ACID | ✅ |
| 4 | Insertion d'un doublon de référence | Contrainte `UNIQUE` **avant** réplication | ✅ |
| 5 | Écritures concurrentes | Sérialisation MVCC sans corruption | ✅ |
| 6 | Panne du standby → reprise | Rattrapage WAL + détection honnête par le SaaS | ✅ |

**Résultat : 6 / 6 scénarios validés — 0 échec.**

> 📸 **[CAPTURE — réplication métier]** Une donnée créée sur Kali (primaire)
> apparaissant sur Ubuntu (standby) en < 1 s, via `psql`. → Annexe C / Annexe D.

#### II.13.2. Campagne haute disponibilité — 4/4 protections

Banc : **Kali (primaire) + Ubuntu (standby)**.

| # | Protection | Garantie démontrée | Verdict |
|---|-----------|--------------------|:------:|
| 1 | **Slot de réplication** | Le primaire ne recycle plus le WAL du standby (`active=t`) | ✅ |
| 2 | **Anti auto-promotion** | À 2 nœuds, jamais de bascule auto → pas de split-brain | ✅ |
| 3 | **Fencing par timeline** | Ancien primaire déchu **se clôture** (timeline 1 < cluster 2) | ✅ |
| 4 | **Auto-réparation** | Tout re-clone recrée le slot **automatiquement** | ✅ |

**Résultat : 4 / 4 protections validées — 0 échec.** L'époque étant le `timeline_id`
natif de PostgreSQL, **aucun consensus n'a été réinventé**.

> 📸 **[CAPTURE — campagne HA]** Séquence promotion du standby → réveil de l'ancien
> primaire → bannière ⛔ FENCED. → Annexe D.

#### II.13.3. Tests unitaires du socle

Indépendamment du banc, le cœur Rust est couvert par des tests automatisés
(`cargo test`) : chiffrement/déchiffrement, rejet de clé erronée, détection de
falsification, scellement/ouverture sealed box, dérivation Argon2id, relecture du
journal, monotonie des index, comportement du fencing aux bornes d'époque. Ces
tests sont ce qui m'autorisait à dire « le socle est prouvé » avant d'écrire le
métier. → Annexe B (liste complète et sortie de `cargo test`).

| Critère de succès (spike Phase 0) | Cible | Résultat | Statut |
|-----------------------------------|-------|----------|:------:|
| Chiffrement DEK cross-OS | Identique Win/Ubuntu/Kali | Vérifié par tests | ✅ |
| Journal append-only CBOR chiffré | Illisible sans DEK | `wrong_dek_fails` | ✅ |
| Réplication primaire → standby | < 1 s, 0 perte | Confirmé (banc MPJ) | ✅ |
| Bascule manuelle (2 machines) | Sans split-brain | Validé | ✅ |
| Failover auto par quorum (≥ 3) | Promotion `pg_promote` | Logique validée | ✅ |
| Fencing — retour de l'ancien actif | Bloqué par époque | Bannière ⛔ FENCED | ✅ |
| Annonce au relais + découverte des pairs | Topologie maintenue | Vérifié | ✅ |
| Blob de récupération zero-knowledge | Opaque sur le relais | `blob-stats` zero_knowledge | ✅ |

---

### II.14. Difficultés Rencontrées et Solutions Apportées

Cette section est, à mes yeux, le vrai cœur de la réflexion d'ingénieur. Un
technicien contourne ; un ingénieur remonte à la cause racine et conçoit une
solution qui *élimine* le problème. Voici les quatre incidents les plus
instructifs.

#### II.14.1. P1 — Le split-brain : deux primaires en même temps

**Symptôme.** Après une coupure du standby suivie d'un redémarrage, le cluster se
retrouvait incohérent : les deux nœuds se croyaient primaires, et la réplication ne
repartait plus.

**Cause racine.** Mon premier code de failover était trop zélé : dès que le standby
perdait le WAL, il s'auto-promouvait — **même à 2 nœuds**. Or si le primaire
n'était pas réellement mort (simple coupure réseau côté standby), on obtenait
**deux primaires** : un *split-brain*. Chacun acceptait des écritures divergentes,
et la réplication devenait irréconciliable.

**Solution structurelle (deux volets).** *(1)* La promotion automatique est désormais
**conditionnée au quorum** : strictement réservée aux clusters de **3 nœuds ou
plus** ; à moins de 3, la bascule est **manuelle** (décision actée n°2). *(2)*
J'ai ajouté le **fencing par timeline** : un ancien primaire qui revient lit le
timeline du cluster et, s'il est en retard, **se clôture lui-même** sans jamais
servir d'écriture (cf. II.7.3). Le problème n'est plus « rattrapé » : il est rendu
**structurellement impossible**.

**Enseignement.** Dans un système distribué, la disponibilité naïve est l'ennemie
de la cohérence. La bonne réponse n'était pas un correctif local, mais un
changement de modèle : s'appuyer sur une grandeur monotone *native* (le timeline)
plutôt que sur une heuristique maison.

> 📸 **[CAPTURE — diagnostic du split-brain]** Logs des deux nœuds se déclarant
> primaires, puis logs après correctif (un seul primaire, l'autre clôturé).
> → Annexe D.

#### II.14.2. P2 — La clé/identité de nœud écrasée par une config partagée

**Symptôme.** En recopiant la configuration d'un nœud vers un autre, le second
héritait de l'identité (et de secrets) du premier, ce qui faussait son
enregistrement et sa réplication.

**Cause racine.** Des éléments *spécifiques au nœud* (identité, secrets dérivés)
étaient traités comme de la configuration *partagée*. Toute copie propageait donc
les valeurs du nœud d'origine.

**Solution.** Séparer nettement ce qui est **partagé** (compose, image, URL du
relais) de ce qui est **propre au nœud** (identité, DEK scellée, mots de passe
dérivés du jeton). L'installateur **dérive localement** les secrets propres au nœud
au lieu de les copier, et la config sensible n'est jamais versionnée.

**Enseignement.** La même tension reviendra dans tout système multi-nœuds : un
secret *node-specific* ne doit jamais voyager dans un artefact *partagé*. La
solution durable n'est pas « penser à régénérer à la main », mais rendre le système
**auto-configurant** au démarrage.

#### II.14.3. P3 — L'autorité de rôle : qui décide qui est primaire ?

**Symptôme.** Après un redémarrage du serveur SaaS, un nœud pouvait se re-déclarer
avec un rôle erroné (`NODE_MODE` figé dans son environnement), en contradiction avec
son état PostgreSQL réel.

**Cause racine.** Deux sources de vérité concurrentes : la variable d'environnement
`NODE_MODE` (statique) et l'état réel de PostgreSQL (`pg_is_in_recovery()`).

**Solution.** **PostgreSQL fait autorité.** Au démarrage, le nœud interroge
`pg_is_in_recovery()` ; si le rôle réel diffère du rôle déclaré, il **se
re-déclare** au portail avec le bon rôle et son époque (timeline). Le SaaS, de son
côté, conserve le **timeline maximum** et refuse qu'un ancien primaire rétrograde le
primaire légitime.

**Enseignement.** Quand deux sources se contredisent, il faut désigner *une* autorité
— et choisir celle qui *mesure* la réalité (PostgreSQL) plutôt que celle qui la
*déclare* (une variable d'env).

#### II.14.4. P4 — Le tag d'image Docker mutable : un binaire périmé en production

**Symptôme.** Après une modification du cœur Rust, une PME continuait de tourner un
**ancien binaire** — par exemple, l'encart d'identifiants à la connexion
disparaissait sans raison apparente.

**Cause racine.** Le tag `ss-node:dev` est **mutable**. Si l'image du registre
n'était pas reconstruite/republiée, ou si la PME avait gardé une image en cache,
elle exécutait une version périmée.

**Solution.** *(1)* L'installateur effectue désormais un `docker compose pull`
avant de démarrer, garantissant la dernière image du registre. *(2)* Une procédure
explicite « reconstruire + republier l'image **avant** toute démonstration » a été
documentée dans le runbook, et la purge d'environnement re-tire l'image fraîche.

**Enseignement.** Un tag mutable est un piège silencieux : « ça marchait hier » peut
cacher un binaire différent. La rigueur de livraison (pull systématique, procédure
écrite) vaut mieux que la confiance dans un cache.

---

### II.15. Conclusion Technique

Le prototype démontre, sur banc réel multi-OS, qu'un éditeur peut vendre un logiciel
métier « en mode SaaS » tout en étant **cryptographiquement incapable** de lire les
données de ses clients. Le socle Phase 0 est prouvé : chiffrement DEK cross-OS,
journal append-only CBOR chiffré, sealed box d'enrôlement, réplication PostgreSQL
primaire/standby, bascule manuelle et failover par quorum, et — surtout — un
**fencing par timeline qui neutralise structurellement le split-brain**. Sur ce
socle, deux campagnes ont été validées sans aucun échec : **6/6 scénarios métier**
et **4/4 protections de haute disponibilité**. Un **agent de supervision Mistral**
observe les trois acteurs sans jamais franchir la frontière souveraine.

Plusieurs chantiers restent ouverts pour un passage en production industrielle, que
j'assume comme des perspectives et non des manques cachés :

- **Enrôlement QR de bout en bout** sur deux appareils physiques (la mécanique
  cryptographique est prouvée ; l'expérience produit reste à finaliser).
- **Durcissement des paramètres Argon2id** pour la production (le spike utilise des
  valeurs raisonnables mais non durcies).
- **Failover automatique éprouvé à grande échelle** (tests sur 5+ nœuds, scénarios
  de partition réseau).
- **UniFFI** pour dériver les clients desktop et mobile du même cœur Rust.
- **Chaîne CI/CD** avec scans de sécurité (`cargo audit`, `cargo deny`) et tests
  d'intégration automatisés du failover.

Ce qui compte, à ce stade, n'est pas l'exhaustivité fonctionnelle, mais la **preuve
du socle** : la partie risquée — crypto, sérialisation, basculement sans
split-brain — est faite et démontrée. Le métier peut désormais s'écrire dessus en
confiance.

---

## PARTIE III — Bilan de l'Expérience

### III.1. Compétences Mobilisées et Acquises

Ce stage a mobilisé et approfondi un spectre large de compétences, allant des fondamentaux des systèmes distribués jusqu'aux enjeux stratégiques de la souveraineté numérique.

**Compétences techniques développées :**

| Domaine | Compétences spécifiques |
|---------|------------------------|
| Cryptographie appliquée | Chiffrement authentifié (XChaCha20-Poly1305), sealed box (X25519), dérivation *memory-hard* (Argon2id), gestion et zéroïsation de clés via libsodium |
| Systèmes distribués | Réplication primaire/standby PostgreSQL, WAL streaming, quorum, fencing par *timeline*, neutralisation du split-brain |
| Programmation Rust | Workspace Cargo en crates, gestion mémoire sûre, `async`/Tokio, `sqlx`, axum, tests unitaires |
| Big Data & IA | Séries temporelles de métriques, supervision par modèle de langage (Mistral), détection de dérives |
| DevOps / packaging | Docker & Docker Compose multi-arch, déploiement auto-adaptatif, registre privé, cross-OS Windows/Linux |
| Développement web | SaaS Django (comptes, licences, parc), serveur web métier embarqué dans le nœud |
| Gestion de projet | WBS, RACI, SMART, « le socle d'abord », documentation technique continue |

**Compétences transversales renforcées :**
- Capacité à diagnostiquer un incident distribué en conditions réelles (le split-brain, remonté à sa cause racine puis éliminé structurellement)
- Discipline du dérisquage : prouver le socle par tests avant d'écrire le métier
- Rigueur documentaire : journal technique, runbook de démonstration, décisions actées
- Autonomie et prise de décision en situation incertaine
- Communication technique avec la tutrice entreprise et le tuteur EIGSI

---

### III.2. Valeur Ajoutée pour AL BARAA CONSULTING

Ce projet génère une valeur ajoutée concrète et mesurable pour AL BARAA CONSULTING, à plusieurs niveaux :

**Valeur technique :**
- Prototype fonctionnel, validé sur banc réel multi-OS, documenté et déployable — livrable immédiatement démontrable à des prospects
- Un cœur Rust réutilisable (crypto, journal, supervision) comme socle de futurs produits souverains
- Une architecture à trois acteurs transposable à d'autres logiciels métier (pas seulement le stock)

**Valeur commerciale et stratégique :**
- Première référence du cabinet sur le segment « solutions souveraines africaines » — marché en forte croissance post-AUDPF
- Argument différenciant fort pour les clients publics (urbanisme, foncier, données citoyens) : souveraineté **prouvée cryptographiquement**, pas seulement contractuelle
- Modèle économique clair : vendre un logiciel « en mode SaaS » sans héberger ni voir les données du client

**Valeur documentaire :**
- Documentation technique complète (architecture, déploiement, tests, runbook de démonstration)
- Journal chronologique des incidents et de leurs solutions — réutilisable pour les déploiements futurs
- Guide de démonstration — utilisable par AL BARAA pour présenter la solution à des clients

---

### III.3. Réflexion sur la Posture Ingénieur

Ce stage m'a confronté à une distinction fondamentale entre la posture de technicien et la posture d'ingénieur. Cette réflexion est au cœur de la valeur ajoutée que l'EIGSI me demande de démontrer.

**Un technicien** résout le problème immédiat : quand le cluster diverge, il redémarre les nœuds jusqu'à ce que « ça reparte ». Quand une PME tourne un binaire périmé, il vide le cache à la main.

**Un ingénieur** s'interroge sur la cause racine et conçoit une solution qui élimine le problème structurellement. Pourquoi le cluster diverge-t-il ? Parce qu'un standby s'auto-promeut à 2 nœuds pendant que le primaire vit encore — un split-brain. La solution n'est pas de redémarrer : c'est de **conditionner la promotion au quorum** et d'ajouter un **fencing par *timeline*** qui rend la double écriture *impossible*. Pourquoi une PME tourne-t-elle un vieux binaire ? Parce que le tag Docker est mutable. La solution n'est pas « penser à reconstruire » : c'est un `docker compose pull` systématique au démarrage.

La posture d'ingénieur exige de refuser les contournements au profit des solutions durables, même plus coûteuses au départ. Elle exige aussi de **ne pas survendre** : j'ai documenté honnêtement ce qui est prouvé (le socle, 6/6 + 4/4) et ce qui reste une perspective (l'enrôlement QR de bout en bout, le durcissement Argon2id production). Enfin, elle exige de documenter les décisions pour que la connaissance reste dans l'organisation, et non dans la tête d'un seul développeur.

---

### III.4. Bilan Personnel et Perspectives

Ce stage représente ma première expérience de conception et de déploiement d'un système distribué souverain en conditions réelles — sur quatre OS hétérogènes, deux tenants indépendants, avec des incidents authentiques à résoudre, des délais à respecter et une tutrice exigeante à satisfaire. Cette expérience a profondément ancré des compétences qui ne peuvent s'acquérir que par la pratique : diagnostiquer un incident distribué, concevoir une solution architecturale pérenne, et choisir de s'appuyer sur des primitives éprouvées (libsodium, PostgreSQL) plutôt que de réinventer le risque.

Sur le plan professionnel, ce projet a renforcé ma conviction que les enjeux de souveraineté numérique constituent un domaine d'ingénierie à la fois techniquement stimulant et stratégiquement crucial pour le continent africain. Je souhaite contribuer, dans ma carrière d'ingénieur, au développement de solutions numériques souveraines adaptées au contexte africain.

Sur le plan personnel, ce stage m'a appris que la rigueur — dans le code, dans la documentation, dans la communication — n'est pas une contrainte académique mais une nécessité professionnelle.

---

## CONCLUSION GÉNÉRALE

### Conclusion technique

L'objectif principal de ce stage — concevoir et implémenter un Framework SaaS Souverain où l'éditeur vend un logiciel métier « en mode SaaS » tout en étant **cryptographiquement incapable** de lire les données de ses clients — a été pleinement atteint. Le prototype démontre, sur banc réel multi-OS, qu'une alternative crédible aux SaaS cloud centralisés étrangers est techniquement réalisable, en s'appuyant exclusivement sur des briques éprouvées (libsodium, PostgreSQL, Docker) orchestrées avec rigueur.

Les indicateurs définis dans le Plan Directeur sont tous validés : un socle cryptographique prouvé (DEK XChaCha20-Poly1305 cross-OS, sealed box X25519 d'enrôlement, journal append-only CBOR chiffré, illisible sans la DEK) ; une réplication PostgreSQL primaire/standby avec bascule manuelle à 2 nœuds et failover par quorum à 3+ ; et — surtout — un **fencing par *timeline* qui neutralise structurellement le split-brain**. Deux campagnes ont été validées sans aucun échec : **6/6 scénarios métier** et **4/4 protections de haute disponibilité**, sur deux tenants souverains indépendants. Un **agent de supervision Mistral** observe les trois acteurs sans jamais franchir la frontière souveraine.

### Conclusion personnelle

Ce projet m'a confirmé que l'ingénierie logicielle, dans sa dimension la plus exigeante, consiste moins à écrire du code qu'à concevoir des systèmes fiables, documentés et maintenables — et à savoir *ne pas* réécrire ce qui existe déjà en mieux (la crypto, le consensus). Le diagnostic du split-brain remonté à sa cause racine puis éliminé par un changement de modèle, le choix discipliné de s'appuyer sur le *timeline* natif de PostgreSQL, et la production d'une documentation de qualité industrielle constituent les apprentissages les plus durables de cette expérience.

Ce stage m'a ouvert la perspective d'une carrière en ingénierie de systèmes distribués et de cybersécurité, deux domaines au cœur des enjeux de souveraineté numérique africaine. Je sors de cette expérience avec la conviction que les ingénieurs africains peuvent et doivent concevoir les solutions technologiques qui répondent aux défis spécifiques de leur continent.

---

## RÉFÉRENCES BIBLIOGRAPHIQUES

1. African Union. *AU Data Policy Framework (AUDPF)*. Union Africaine, décembre 2025. https://au.int/sites/default/files/documents/42078-doc-DATA-POLICY-FRAMEWORKS-2024-ENG-V2.pdf

2. Bernstein, D. J. *ChaCha, a variant of Salsa20* / *The Poly1305-AES message-authentication code*. 2008. https://cr.yp.to/chacha.html — base du chiffrement authentifié XChaCha20-Poly1305.

3. Bernstein, D. J. *Curve25519: new Diffie-Hellman speed records*. PKC 2006. https://cr.yp.to/ecdh.html — base de l'identité d'appareil X25519.

4. The Sodium Project (libsodium). *Sealed boxes & authenticated encryption*. doc.libsodium.org, 2024. https://doc.libsodium.org/

5. Biryukov, A., Dinu, D., & Khovratovich, D. *Argon2: the memory-hard function for password hashing and other applications*. 2015. https://www.password-hashing.net/argon2-specs.pdf

6. Bormann, C. & Hoffman, P. *Concise Binary Object Representation (CBOR)*. RFC 8949, IETF, 2020. https://datatracker.ietf.org/doc/html/rfc8949

7. The PostgreSQL Global Development Group. *PostgreSQL 16 Documentation — High Availability, Load Balancing, and Replication*. postgresql.org, 2024. https://www.postgresql.org/docs/16/high-availability.html

8. The PostgreSQL Global Development Group. *Continuous Archiving and Point-in-Time Recovery — Timelines & `pg_promote`*. postgresql.org, 2024. https://www.postgresql.org/docs/16/continuous-archiving.html

9. Docker Inc. *Docker Compose Documentation — Multi-container applications*. docs.docker.com, 2024. https://docs.docker.com/compose/

10. New America Foundation. *Africa's Digital Sovereignty Trap: The Data Center Dilemma*. newamerica.org, 2026. https://www.newamerica.org/planetary-politics/briefs/africas-digital-sovereignty-trap/

---

## ANNEXES

> Les annexes rassemblent les éléments de preuve et de référence trop volumineux
> pour le corps du rapport. Les captures d'écran y seront insérées dans la version
> finale (terminaux lisibles, fond clair). Chaque annexe est autoportante et peut
> être lue indépendamment.

---

### Annexe A — Glossaire des termes et primitives

| Terme | Définition |
|-------|-----------|
| **DEK** (*Data Encryption Key*) | Clé symétrique 256 bits, unique par entreprise. Chiffre les données métier et le journal. Générée chez la PME, jamais chez l'éditeur. |
| **Sealed box** | Schéma de chiffrement anonyme (X25519 + XChaCha20-Poly1305) permettant de chiffrer un message pour un destinataire connu par sa seule clé publique, sans authentifier l'émetteur. |
| **XChaCha20-Poly1305** | Chiffrement authentifié à nonce étendu (192 bits). Garantit confidentialité **et** intégrité. |
| **X25519** | Échange de clés Diffie-Hellman sur courbe elliptique Curve25519. Base de l'identité d'appareil. |
| **Argon2id** | Fonction de dérivation de clé *memory-hard*, résistante au force-brute matériel. Dérive une clé à partir du code de récupération. |
| **CBOR** | *Concise Binary Object Representation* — format binaire compact et typé pour sérialiser les entrées du journal. |
| **WAL** (*Write-Ahead Log*) | Journal des transactions de PostgreSQL, rejoué par les standbys pour rester à jour. |
| **Timeline (TLI)** | Compteur monotone interne de PostgreSQL, incrémenté à chaque promotion. Utilisé comme **époque** pour le fencing. |
| **Slot de réplication** | Mécanisme PostgreSQL empêchant le primaire de recycler du WAL qu'un standby n'a pas encore consommé. |
| **Fencing** | Isolation d'un nœud déchu : un ancien primaire dont l'époque est périmée refuse de servir, pour éviter le split-brain. |
| **Split-brain** | Situation où deux nœuds se croient primaires simultanément et acceptent des écritures divergentes. |
| **Quorum** | Nombre minimal de nœuds (ici ≥ 3) requis pour autoriser un failover automatique. |
| **Zero-knowledge** | Propriété du relais : il stocke et restitue des données sans pouvoir les déchiffrer. |
| **Tenant** | Une entreprise cliente de l'éditeur, identifiée par un UUID. |

---

### Annexe B — Tests automatisés du cœur Rust

> 📸 **[CAPTURE — sortie `cargo test --workspace`]** À insérer : capture complète
> montrant tous les tests verts, par crate.

**Crate `ss-crypto` :**

| Test | Ce qu'il prouve |
|------|-----------------|
| `roundtrip_data` | Chiffrer puis déchiffrer redonne le message d'origine |
| `roundtrip_empty` | Fonctionne même sur une charge utile vide |
| `wrong_key_fails` | Une mauvaise DEK ne peut pas déchiffrer |
| `tampered_ciphertext_fails` | Un seul bit corrompu fait échouer le déchiffrement (intégrité) |
| `two_encryptions_differ` | Deux chiffrements du même message diffèrent (nonce aléatoire) |
| `seal_and_open` | L'appareil destinataire ouvre le sealed box |
| `wrong_key_cannot_open` | Un autre appareil ne peut pas ouvrir le sealed box |
| `two_seals_differ` | Deux scellés de la même DEK diffèrent (éphémère) |
| `deterministic` (recovery) | Même code + même sel → même clé dérivée |
| `different_passphrase_different_key` | Un code différent → clé différente |
| `different_salt_different_key` | Un sel différent → clé différente |

**Crate `ss-journal` :**

| Test | Ce qu'il prouve |
|------|-----------------|
| `append_and_read` | Une entrée écrite est relue à l'identique |
| `indices_monotone` | Les index sont strictement croissants |
| `reopen_restores_state` | L'état (prochain index) se reconstruit après réouverture |
| `wrong_dek_fails` | Un journal ouvert avec la mauvaise DEK est illisible |

**Crate `ss-consensus` :**

| Test | Ce qu'il prouve |
|------|-----------------|
| `current_epoch_allowed` | Époque à jour → écriture autorisée |
| `stale_epoch_fenced` | Époque périmée → nœud clôturé |
| `fresh_epoch_allowed` | Époque supérieure tolérée |
| `fenced_contains_both_epochs` | Le verdict de clôture rapporte les deux époques |
| `increment_is_monotone` | L'incrément d'époque est monotone |

> Commande de référence : `cargo test --workspace`. Les tests d'intégration
> PostgreSQL sont marqués `#[ignore]` et se lancent avec une instance réelle :
> `TEST_PG_URL=postgres://… cargo test -p ss-consensus -- --ignored`.

---

### Annexe C — Captures du parcours utilisateur (SaaS + métier)

> Captures à refaire proprement pour la version finale. Ordre de lecture suggéré :

1. 📸 **[CAPTURE]** SaaS — formulaire d'inscription d'un tenant.
2. 📸 **[CAPTURE]** SaaS — page « Bienvenue » avec lien de téléchargement de
   l'installateur (Linux / Windows).
3. 📸 **[CAPTURE]** Terminal PME — exécution de `install-<tenant>.sh`, les 5 étapes
   jusqu'à « Installation terminée ! (primary) ».
4. 📸 **[CAPTURE]** Interface métier (port 9001) — page de connexion avec l'encart
   d'identifiants initiaux.
5. 📸 **[CAPTURE]** Interface métier — création d'un article + entrée de stock +
   alerte de seuil.
6. 📸 **[CAPTURE]** SaaS — page « Parc machines » : rôle, adresse, époque, statut en
   ligne.
7. 📸 **[CAPTURE]** SaaS — page « Clusters » : « ✓ Cluster sain » (MPJ) et « ⚠
   Bascule manuelle » (PME mono-nœud).
8. 📸 **[CAPTURE]** SaaS — carte de verdict de l'agent IA (score de risque +
   diagnostic Mistral).

---

### Annexe D — Journal de la campagne de résilience (logs réels)

> Extraits de logs et commandes de vérification utilisés pendant la campagne HA.
> Captures terminal à insérer.

**D.1 — Réplication métier vérifiée sur le standby (Ubuntu) :**

```bash
docker exec pg-node psql -U metier -d metier -c \
  "SELECT code, nom FROM articles ORDER BY created_at DESC LIMIT 3;"
# → la donnée créée sur le primaire (Kali) apparaît ici en < 1 s
```

**D.2 — Réplication des comptes employés (sans aucune action sur le standby) :**

```bash
docker exec pg-node psql -U metier -d metier -c \
  "SELECT username, role FROM users ORDER BY username;"
# → admin, alice.martin, bob.dupont — identiques sur le standby
```

**D.3 — Unicité vérifiée à la source (le doublon n'est jamais répliqué) :**

```bash
# 1ʳᵉ insertion : OK
INSERT INTO articles (code, nom, unite, seuil_alerte, actif)
VALUES ('REF-DEMO','Cahier A5','unité',5,true);
# 2ᵉ insertion même code : rejet
# → ERROR: duplicate key value violates unique constraint "articles_code_key"
```

**D.4 — Preuve du fencing (logs du nœud déchu) :**

```
⛔ NŒUD CLÔTURÉ (FENCED)
Époque de ce nœud (timeline PG) : 1
Époque courante du cluster       : 2
Un primaire plus récent existe. Ce nœud est un ancien primaire déchu :
il REFUSE de servir pour éviter le split-brain.
→ Action opérateur : re-cloner ce nœud en standby.
```

> 📸 **[CAPTURE — séquence HA complète]** Promotion du standby → réveil de l'ancien
> primaire → bannière ⛔ FENCED → SaaS affichant le cluster sur l'époque 2.

---

### Annexe E — Configuration du banc de validation

| Nœud | OS | Rôle | LAN | Tenant |
|------|----|------|-----|--------|
| Hôte | Windows 11 Pro | Poste de dev + SaaS/relais éditeur | 192.168.200.1 / 192.168.10.1 | — |
| node-debian | Debian | Primaire (PME mono-nœud) | 192.168.10.128 | Yasmine Argan |
| node-kali | Kali Linux | Primaire | 192.168.200.128 | MPJ |
| node-ubuntu | Ubuntu | Standby | 192.168.200.130 | MPJ |

**Cluster de référence (`docker-compose.yml` du spike) :** relais + 1 primaire
PostgreSQL + 2 standbys + 3 nœuds `ss-node`, sur un bridge `172.20.0.0/24` à IP
fixes (requis sous Windows Docker Desktop).

| Service | Adresse interne | Port hôte |
|---------|-----------------|-----------|
| `relay` | 172.20.0.10 | 8080 |
| `pg-primary` | 172.20.0.20 | 5433 |
| `pg-standby1` / `pg-standby2` | 172.20.0.21 / .22 | 5434 / 5435 |
| `node1` (actif) | 172.20.0.11 | 9001 |
| `node2` / `node3` (passifs) | 172.20.0.12 / .13 | 9002 / 9003 |

---

### Annexe F — Décisions d'architecture actées

| # | Décision | Justification |
|---|----------|---------------|
| 1 | **Identité = UUID d'installation authentifié** (pas la MAC) | La MAC est falsifiable et masquée par Docker |
| 2 | **< 3 nœuds → bascule manuelle ; ≥ 3 → failover auto** | Empêche le split-brain à 2 nœuds |
| 3 | **Nœud PME = image Docker ; découverte via le relais** | mDNS peu fiable sous Docker → annonce HTTP |
| 4 | **Réplication par opération** (synchrone pour les invariants forts) | Jamais de dégradation silencieuse |
| 5 | **Aucune primitive crypto maison** — tout via libsodium | Domaine où l'amateurisme coûte cher |
| 6 | **Aucun consensus maison** — promotion de standby + timeline | Le timeline PostgreSQL est une époque native fiable |

---

### Annexe G — Référentiel de code

| Composant | Emplacement | Langage |
|-----------|-------------|---------|
| Cœur cryptographique | `spike/crates/ss-crypto/` | Rust |
| Journal append-only | `spike/crates/ss-journal/` | Rust |
| Époque / fencing / supervision | `spike/crates/ss-consensus/` | Rust |
| Nœud PME (CLI + web métier) | `spike/node/` | Rust |
| Relais zero-knowledge | `spike/relay/` | Rust (axum) |
| SaaS éditeur | application Django | Python |
| Packaging / déploiement | `spike/docker-compose.yml`, `pme-deploy/`, `relay-deploy/` | Docker / shell |

---

*Rapport de Stage — Fin d'Études — MPIGA-ODOUMBA Jesse*
*EIGSI Casablanca — Spécialité Big Data & IA — Promotion 2026*
*AL BARAA CONSULTING — Mme Soumia CHOKRI*
*Soutenance : 01/07/2026 à 10h00 — EIGSI Casablanca*
