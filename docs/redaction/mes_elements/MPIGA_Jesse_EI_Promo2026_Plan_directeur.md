**Plan directeur**

Stage de Fin d'Études

Spécialité : Big Data & IA

19/02/2026

**AL BARAA CONSULTING**

**Conception et Implémentation d'une Architecture**

**Coffre-Fort Data P2P Souveraine**

|     |     |     |
| --- | --- | --- |
| **Tuteur Entreprise** | **Tuteur EIGSI** | **Étudiant** |
| CHOKRI Soumia | M. Ayoub Amrani | Jesse MPIGA-ODOUMBA |

Année Académique : 2025 - 2026

Table des matières

[Liste des figures et des tableaux 3](#_Toc225222689)

[I. Contexte du Projet 1](#_Toc225222690)

[1\. Problématique de la Souveraineté Numérique en Afrique 1](#_Toc225222691)

[2\. Présentation de la Solution : 1](#_Toc225222692)

[3\. Architecture Modulaire et Résultats Attendus 2](#_Toc225222693)

[4\. Indicateurs de Succès du Projet 2](#_Toc225222694)

[5\. Positionnement du Projet dans le Cadre du Stage 2](#_Toc225222695)

[6\. Enjeux Stratégiques 3](#_Toc225222696)

[7\. Synthèse 3](#_Toc225222697)

[8\. État de l'art et Positionnement Technologique 4](#_Toc225222698)

[II. Analyse Fonctionnelle 6](#_Toc225222699)

[1\. Analyse Fonctionnelle Externe 6](#_Toc225222700)

[2\. Fonctions Principales et Contraintes 7](#_Toc225222701)

[3\. Analyse Fonctionnelle Interne 8](#_Toc225222702)

[III. Objectifs 9](#_Toc225222703)

[IV. Enjeux 11](#_Toc225222704)

[V. Livrables Attendus 12](#_Toc225222705)

[VI. Périmètre du Projet 13](#_Toc225222706)

[VII. Planning Prévisionnel 14](#_Toc225222707)

[1\. Work Breakdown Structure (WBS) 14](#_Toc225222708)

[2\. Resource Breakdown Structure (RBS) 15](#_Toc225222709)

[1\. Organizational Breakdown Structure (OBS) 16](#_Toc225222710)

[2\. Macro Planning 18](#_Toc225222711)

[VIII. Risques 19](#_Toc225222712)

[IX. Budget 20](#_Toc225222713)

[1\. Coûts de Développement, Ressources Humaines 20](#_Toc225222714)

[2\. Coûts Matériels & Logiciels 21](#_Toc225222715)

[3\. Récapitulatif Budgétaire 21](#_Toc225222716)

[4\. Retour sur Investissement (ROI) Estimatif 21](#_Toc225222717)

[Webographie 22](#_Toc225222718)

# Liste des figures et des tableaux

[Figure 1:Bête à Corne 4](#_Toc225219939)

[Figure 2:Diagramme Pieuvre 5](#_Toc225219940)

[Figure 3 : Diagramme FAST 6](#_Toc225219941)

[Figure 4 :Work Breakdown Structure (WBS) 11](#_Toc225219942)

[Figure 5 :Resource Breakdown Structure (RBS) 12](#_Toc225219943)

[Figure 6 : Macro Planning — Diagramme de Gantt 15](#_Toc225219944)

[Figure 7 : Matrice des Risques 17](#_Toc225219945)

[Figure 8 :Matrice des Risques 18](#_Toc225219946)

[Tableau 1:Fonctions Principales et Contraintes 5](#_Toc225219947)

[Tableau 2 : Cadrage & Conception de l'Architecture 7](#_Toc225219948)

[Tableau 3 : Développement du Module de Stockage Local 7](#_Toc225219949)

[Tableau 4:Développement du Module de Stockage Local 8](#_Toc225219950)

[Tableau 5 : Sécurité & Exposition d'APIs 8](#_Toc225219951)

[Tableau 6 :Tests, Documentation & Clôture 9](#_Toc225219952)

[Tableau 7 : Livrables Attendus 10](#_Toc225219953)

[Tableau 8 : Contraintes 11](#_Toc225219954)

[Tableau 9 : Macro-Tâche 12](#_Toc225219955)

[Tableau 10 : Ressources Humaines 13](#_Toc225219956)

[Tableau 11: Ressources Techniques & Logicielles 13](#_Toc225219957)

[Tableau 12 : RACI 14](#_Toc225219958)

[Tableau 13 : Matrice RACI 14](#_Toc225219959)

[Tableau 14 : Jalon macro planning 16](#_Toc225219960)

[Tableau 15 : Risque Identifié 17](#_Toc225219961)

[Tableau 16 : Coûts de Développement, Ressources Humaines 18](#_Toc225219962)

[Tableau 17 : Coûts Matériels & Logiciels 19](#_Toc225219963)

[Tableau 18 : Récapitulatif Budgétaire 19](#_Toc225219964)

[Tableau 19 : Retour sur Investissement (ROI) Estimatif 19](#_Toc225219965)

# I. Contexte du Projet

## Problématique de la Souveraineté Numérique en Afrique

Le secteur de la gestion des données en Afrique est aujourd’hui caractérisé par une dépendance critique aux solutions cloud centralisées étrangères. Cette dépendance technologique engendre des risques majeurs en matière de souveraineté numérique, de sécurité des données et de continuité de service, en contradiction directe avec les orientations stratégiques définies par l’AU Data Policy Framework (AUDPF), validé par l’Union Africaine en décembre 2025.

En effet, bien que les organisations africaines disposent d’un accès à des infrastructures cloud performantes (AWS, Azure, Google Cloud), elles ne maîtrisent ni la localisation physique de leurs données, ni les conditions d’accès, ni la pérennité des services utilisés.

Dans ce contexte, la problématique centrale de ce projet peut être formulée comme suit : Comment garantir la souveraineté, la sécurité et la résilience des données dans un environnement africain, tout en réduisant la dépendance aux infrastructures cloud centralisées étrangères ?

L’enjeu est donc de concevoir une alternative fiable et performante reposant sur des technologies open-source, permettant aux organisations de reprendre le contrôle de leurs données.

## Présentation de la Solution :

Une Brique Universelle Décentralisée Pour répondre à cette problématique, ce projet vise à concevoir et implémenter une architecture de type Coffre-Fort Data P2P Souveraine, pensée comme une “brique universelle” de Backend-as-a-Service (BaaS) décentralisée.

L’approche retenue repose sur trois axes principaux :

- **Autonomie Infrastructurelle**

Chaque terminal est transformé en un nœud de stockage intelligent, capable d’assurer la persistance locale des données ainsi que leur réplication automatique en pair-à-pair (P2P), grâce à des mécanismes de synchronisation distribuée (Syncthing), de découverte de services (mDNS) et de résolution de conflits (CRDTs).

- **Ingénierie par Orchestration**

Plutôt que de développer une solution complète from scratch, le projet s’appuie sur l’orchestration de technologies open-source éprouvées telles que DuckDB, SQLite, Syncthing et Docker. Ce choix permet de garantir une meilleure fiabilité, une réduction des coûts de développement, ainsi qu’une maintenabilité accrue.

- **Sécurité et Interopérabilité**

La solution intègre nativement des mécanismes de sécurité robustes, incluant le chiffrement des communications (TLS 1.3), l’authentification mutuelle par certificats x509, ainsi que le chiffrement des données au repos (AES-256). Par ailleurs, l’exposition d’API standards (REST et gRPC) permet une intégration simple avec des applications métier existantes ou futures.

## Architecture Modulaire et Résultats Attendus

L’architecture proposée repose sur une approche modulaire en cinq couches :

- Application Métier : gestion des cas d’usage spécifiques
- API (REST/gRPC) : exposition des services
- Orchestration & Logique : gestion des flux et règles
- Stockage & Synchronisation: DuckDB, SQLite, Syncthing, CRDTs
- Infrastructure & Sécurité : Docker, TLS, mDNS

Cette structuration permet d’assurer une séparation claire des responsabilités, facilitant l’évolutivité et la maintenance du système.

## Indicateurs de Succès du Projet

Le projet sera validé à travers la réalisation d’un Proof of Concept (POC) fonctionnel répondant aux critères suivants :

- Réplication validée sur un minimum de 3 terminaux
- 0 perte de données lors des synchronisations
- 0 conflit non résolu grâce aux CRDTs
- Latence inférieure à 100 ms en local
- Déploiement complet en moins de 30 minutes via Docker

## Positionnement du Projet dans le Cadre du Stage

Ce projet s’inscrit dans le cadre d’un stage de fin d’études au sein de l’entreprise AL BARAA CONSULTING, où l’objectif est de proposer une solution innovante répondant à des enjeux concrets liés à la gestion et à la souveraineté des données.

Dans ce contexte, le stagiaire intervient en tant qu’ingénieur en conception et développement, avec pour responsabilités principales :

- L’analyse des besoins techniques et stratégiques
- La conception de l’architecture du système
- L’implémentation des modules clés (stockage, synchronisation, API)
- La validation des performances et de la sécurité
- La production de la documentation technique

Cette expérience permet de mobiliser des compétences en Big Data, systèmes distribués, cybersécurité et ingénierie logicielle, tout en répondant à une problématique réelle du marché.

## Enjeux Stratégiques

Au-delà de son aspect technique, ce projet répond à plusieurs enjeux majeurs :

- Souveraineté numérique : réduction de la dépendance aux solutions étrangères
- Accessibilité : solutions déployables dans des environnements à ressources limitées
- Coût : utilisation exclusive de technologies open-source
- Résilience : fonctionnement distribué sans point unique de défaillance

## Synthèse

En synthèse, ce projet vise à démontrer qu’il est possible de concevoir une architecture de gestion de données performante, sécurisée et souveraine, adaptée au contexte africain, en s’appuyant sur des technologies open-source et des paradigmes distribués

## État de l'art et Positionnement Technologique

Afin de valider la pertinence de l'architecture "Brique Universelle", une étude comparative a été menée vis-à-vis des solutions de stockage décentralisées et distribuées existantes. Ce benchmark permet de justifier le choix d'une orchestration de briques open-source (DuckDB, SQLite, Syncthing) face aux standards du marché

| **Critères de performance** | **IPFS/Filecoin** | **Solid(Pods)** | **Brique Universelle** |
| --- | --- | --- | --- |
| Stockage de stockage | Blobs/Fichier bruts | Graphe de données (RDF) | Relationnel & Analytique |
| Capacité OLAP | Inexistante | Très limitée | Native (via DockDB) |
| Disponibilité Réseau | Dépendance aux nœuds | Requiert une connexion | Local-first(Offline-total) |
| Réplication P2P | Native (BitSwap) | Non (Client-Serveur) | Optimisée (Syncthing) |
| Résolution de Conflits | Immuabilité simple | Manuelle / Basique | Automatique (via CRDTs) |
| Souveraineté | Dépendance tiers (Pinning) | Dépendance hébergeur | Souveraineté Infrastructurelle |

Tableau 1 : État de l'art et Positionnement Technologique

- Analyse Détaillée du Positionnement :

L'analyse de cet état de l'art met en lumière trois piliers différenciateurs qui font de la Brique Universelle une solution unique pour le contexte du stage :

1.  Le passage du stockage passif à l'intelligence active (OLAP) :

Contrairement aux solutions comme IPFS ou Storj qui se concentrent exclusivement sur le stockage de fichiers statiques (images, documents), la Brique Universelle intègre une puissance de calcul analytique. En utilisant DuckDB, elle permet d'exécuter des requêtes complexes sur des millions de lignes de données directement sur le terminal utilisateur, sans solliciter de serveur central. C'est un atout majeur pour les applications de Business Intelligence (BI) en zones isolées.

1.  La résilience face au "Gap de Connectivité" :

Face à des initiatives comme Solid de Tim Berners-Lee, qui repose sur une architecture où le "Pod" de données doit souvent être accessible en ligne, notre solution garantit une continuité de service totale. Le paradigme "Offline-first" est ici poussé à son paroxysme : l'application métier continue de fonctionner à 100% de ses capacités sans internet, et la synchronisation P2P via Syncthing se déclenche de manière transparente dès qu'un canal (Réseau local) est détecté.

1.  Une Souveraineté Radicale et Infrastructurelle :

Alors que la plupart des réseaux décentralisés actuels créent une nouvelle forme de dépendance envers des "mineurs" ou des "fournisseurs de stockage tiers", la Brique Universelle redonne le contrôle total à l'organisation. L'infrastructure est le réseau des terminaux de l'entreprise. En orchestrant Docker et TLS 1.3, nous garantissons que les données ne quittent jamais le périmètre de confiance défini par l'organisation, répondant ainsi directement aux directives de l'AU Data Policy Framework.

- Synthèse de la Valeur Ajoutée :

En conclusion, ce positionnement ne vise pas à "réinventer la roue", mais à assembler les meilleures technologies open-source existantes pour combler un vide technologique : le besoin d'une base de données relationnelle, décentralisée, sécurisée et capable de fonctionner sur des infrastructures hétérogènes. Cette approche par orchestration de briques matures garantit au projet une fiabilité industrielle immédiate tout en minimisant les risques de développement "from scratch".

# II. Analyse Fonctionnelle

Afin de bien cerner le besoin réel du client et de structurer les fonctions à assurer par notre solution, nous avons mené une analyse fonctionnelle approfondie à travers trois outils complémentaires : la bête à cornes pour exprimer la finalité du projet, le diagramme pieuvre pour identifier les interactions avec l'environnement, et le diagramme FAST pour hiérarchiser les fonctions selon leur logique d'enchaînement.

## Analyse Fonctionnelle Externe

**_Diagramme Bête à Corne :_**

Figure 1:Bête à Corne

Cette analyse met en évidence que la solution vise principalement à garantir la souveraineté et la sécurité des données tout en facilitant leur partage décentralisé entre organisations partenaires. Elle confirme ainsi la pertinence du développement d’une architecture distribuée adaptée au contexte africain

**_Diagramme Pieuvre :_**

Figure 2:Diagramme Pieuvre

## Fonctions Principales et Contraintes

| **ID** | **Fonction** | **Description** |
| --- | --- | --- |
| FP1 | Stocker et synchroniser les données | Persistance locale + réplication P2P automatique |
| FP2 | Fournir API standard | Interface REST/gRPC pour applications externes |
| FC1 | Assurer la sécurité | Chiffrement TLS + au repos, authentification mutuelle |
| FC2 | Garantir les performances | Latence &lt; 100ms, débit &gt; 10 MB/s |
| FC3 | Assurer la portabilité | Support Linux, Windows, macOS via Docker |
| FC4 | Découverte automatique des nœuds | mDNS, détection < 30s |
| FC5 | Respecter la conformité réglementaire | AUDPF, GDPR si applicable |

Tableau 2:Fonctions Principales et Contraintes

Ce diagramme permet d’identifier les exigences fonctionnelles majeures ainsi que les contraintes liées à la sécurité, à la compatibilité technique et à la réglementation. Ces éléments serviront de base à la définition du cahier des charges fonctionnel.

## Analyse Fonctionnelle Interne

**_Diagramme FAST :_**

Figure 3 : Diagramme FAST

La décomposition FAST articule les fonctions autour de deux axes principaux : le stockage et la synchronisation (FP1), et l'exposition d'une API standard (FP2).

FP1 se déploie en trois sous-fonctions : stocker localement (DuckDB pour l'analytique, SQLite pour les métadonnées, système de fichiers pour les données brutes), répliquer automatiquement (Syncthing, CRDTs, versioning), et découvrir les pairs (mDNS/Avahi). FP2 couvre l'exposition d'une API REST avec endpoints CRUD et documentation OpenAPI, ainsi qu'une API gRPC optionnelle pour les cas de haute performance.

# III. Objectifs

L'objectif principal du stage est de concevoir et de mettre en œuvre une architecture Coffre-Fort Data P2P Souveraine fonctionnelle, documentée et testée. Cet objectif se décline en plusieurs objectifs spécifiques SMART :

1.  **Cadrage & Conception de l'Architecture**

|     |     |     |
| --- | --- | --- |
| **S** | **SPÉCIFIQUE** |     | Produire une architecture technique complète (diagrammes C4, spécifications API, modèles de données) validée par le tuteur entreprise |
| **M** |     | **MESURABLE** | Plan Directeur déposé sur la plateforme EIGSI avant le 19 mars 2026 |
| **A** | **ATTEIGNABLE** |     | S’appuyer sur les ressources existantes et les outils de modélisation disponibles (draw.io, Lucidchart) |
| **R** | **RÉALISTE** |     | Basé sur les besoins identifiés et les technologies maîtrisées (Python, Docker, DuckDB, SQLite) |
| **T** | **TEMPOREL** |     | Livrable validé dans les 4 premières semaines du stage |

Tableau 3 : Cadrage & Conception de l'Architecture

- **Livrable :** Plan Directeur déposé sur la plateforme EIGSI avant le 26 mars 2026

1.  **Développement du Module de Stockage Local**

|     |     |     |
| --- | --- | --- |
| **S** | **SPÉCIFIQUE** | Implémenter un module de stockage combinant DuckDB (analytique), SQLite (métadonnées) et système de fichiers avec API CRUD complète |
| **M** | **MESURABLE** | Coverage de tests unitaires > 80%, performance DuckDB < 1s sur 1M lignes |
| **A** | **ATTEIGNABLE** | Utiliser les librairies Python éprouvées (duckdb, sqlite3) et les patterns REST standards |
| **R** | **RÉALISTE** | Technologies open-source disponibles, documentation complète, environnement Docker standardisé |
| **T** | **TEMPOREL** | Module opérationnel et validé d’ici la semaine 10 |

Tableau 4 : Développement du Module de Stockage Local

- **Livrable :** Module de stockage opérationnel, performances validées (DuckDB < 1s sur 1M lignes

1.  **Développement du Module de Synchronisation P2P**

|     |     |     |
| --- | --- | --- |
| **S** | **SPÉCIFIQUE** | Intégrer Syncthing avec découverte mDNS et résolution de conflits via CRDTs entre plusieurs terminaux |
| **M** | **MESURABLE** | Réplication validée entre 3 terminaux : 0 perte de données, 0 conflit non résolu |
| **A** | **ATTEIGNABLE** | Syncthing est un outil mature avec API REST documentée ; CRDTs bien documentés en littérature scientifique |
| **R** | **RÉALISTE** | Environnement de test multi-noeuds configurable localement via Docker Compose |
| **T** | **TEMPOREL** | POC de synchronisation P2P validé d’ici la semaine 14 |

Tableau 5:Développement du Module de Stockage Local

- **Livrable :** POC de synchronisation P2P validé sur 3+ nœuds.

1.  **Sécurité & Exposition d'APIs**

|     |     |     |
| --- | --- | --- |
| **S** | **SPÉCIFIQUE** | Sécuriser toutes les communications (TLS 1.3, authentification mutuelle x509) et exposer une API REST 100% documentée (Swagger/OpenAPI) |
| **M** | **MESURABLE** | 0 vulnérabilité critique détectée (Trivy, Bandit), API documentée à 100% |
| **A** | **ATTEIGNABLE** | Utilisation de bibliothèques standards (cryptography, certifi) et outils de scan automatisés |
| **R** | **RÉALISTE** | TLS 1.3 et x509 sont des standards industriels bien supportés par les frameworks Python |
| **T** | **TEMPOREL** | Sécurisation et documentation complètes d’ici la semaine 18 |

Tableau 6 : Sécurité & Exposition d'APIs

- Livrable : API REST documentée + rapport de scan sécurité (Trivy, Bandit).

1.  **Tests, Documentation & Clôture**

|     |     |     |
| --- | --- | --- |
| **S** | **SPÉCIFIQUE** | Valider le POC complet (tests intégration, tests charge 10+ noeuds) et produire documentation technique et rapport final conformes aux exigences EIGSI |
| **M** | **MESURABLE** | 100% des tests d’intégration passants, système validé sous charge 10+ noeuds, rapport validé par les tuteurs |
| **A** | **ATTEIGNABLE** | Planning structuré avec semaine tampon, outils de test automatisés (pytest, Locust) disponibles |
| **R** | **RÉALISTE** | Capitaliser sur toutes les phases précédentes pour une clôture structurée et documentée |
| **T** | **TEMPOREL** | Soutenance et livraison finale avant le 6 août 2026 |

Tableau 7 :Tests, Documentation & Clôture

- **Livrable :** POC documenté (PDF) + support de soutenance (PPT/PDF, 15-20 slides).

# IV. Enjeux

Le développement d'une architecture P2P souveraine présente des enjeux majeurs à plusieurs niveaux :

- **Technique**
- Qualité et cohérence des données répliquées : sans résolution de conflits robuste (CRDTs), les données peuvent diverger entre nœuds.
- Performance et scalabilité : le système doit répondre en moins de 100ms pour les requêtes locales et supporter 10+ nœuds simultanément.
- Intégration transparente : la brique doit être deployable via Docker en moins de 5 minutes, sans configuration manuelle complexe.
- **Métiers**
- Adoption par les organisations africaines : la solution doit être simple à déployer et à maintenir, même sans expertise système avancée.
- Alignement avec l'AUDPF : la conformité avec le cadre de politique de données de l'Union Africaine est un critère de succès stratégique.
- **Économique**
- • ROI démontrable : le projet doit démontrer des économies concrètes vs. cloud étranger (estimées à 500 MAD/mois/organisation).
- Budget nul en logiciels : la stratégie 100% open-source élimine tout coût de licence propriétaire.
- **Réglementaire**
- • Conformité RGPD : traitement des données personnelles avec chiffrement et contrôle d'accès conformes.
- Conformité AUDPF : stockage local garanti, sans transfert vers des serveurs étrangers sans consentement explicite

# V. Livrables Attendus

Voici les livrables attendus à l'issue du projet, chacun contribuant à la mise en œuvre complète et fonctionnelle du système :

| **Livrable** | **Format** | **Taille / Durée** | **Date Limite** |
| --- | --- | --- | --- |
| Plan Directeur d’expérience professionnelle | PDF | 20-30 pages | 26/03/2026 |
| Architecture Technique | PDF + diagrammes | 15-20 pages | 14/04/2026 |
| POC Fonctionnel | Docker + Git | Image < 500 MB | 07/07/2026 |
| Documentation Technique | Web (MkDocs) + PDF | 30+ pages | 07/07/2026 |
| Rapport de Tests | PDF + logs | 10-15 pages | 07/07/2026 |
| Rapport Final de Stage | PDF | 30+ pages | 23/07/2026 |
| Évaluation des compétences par l’entreprise | PDF (signé + tamponné) | Grille EIGSI | 23/07/2026 |
| Plan de soutenance | PDF | 1-2 pages | 30/07/2026 |
| Support de soutenance | PPT ou PDF | 15-20 slides | 06/08/2026 |

Tableau 8 : Livrables Attendus

# VI. Périmètre du Projet

|     |     |
| --- | --- |
| **INCLUS DANS LE PROJET** | **HORS PÉRIMÈTRE** |
| **●** Analyse et état de l’art des technologies P2P, synchronisation et BDD embarquées<br><br>**●** Conception complète de l’architecture technique (diagrammes C4, spécifications API, modèles de données)<br><br>**●** Développement d’un POC fonctionnel (Docker + code source Git)<br><br>**●** Intégration des modules : stockage local (DuckDB + SQLite), synchronisation P2P (Syncthing), découverte (mDNS), sécurité (TLS 1.3), API REST<br><br>**●** Tests unitaires (coverage > 80%), intégration (100% passants), charge (10+ nœuds) et sécurité (0 vulnérabilité critique)<br><br>**●** Documentation technique complète (guide installation, utilisation, API, déploiement multi-nœuds)<br><br>**●** Rapport final de stage et support de soutenance | **●** Développement d’applications métier front-end ou logique métier<br><br>**●** Mise en production industrielle à grande échelle<br><br>**●** Intégration avec systèmes tiers extérieurs (ERP, CRM, comptabilité)<br><br>**●** Support utilisateur post-déploiement<br><br>**●** Développements mobiles (Android, iOS) |

Tableau 9 : Périmètre du Projet

- **Contraintes**

|     |     |
| --- | --- |
| **Contrainte** | **Description** |
| **Durée** | 24 semaines (19 février — 6 août 2026) |
| **Technologies** | 100% open-source —pas de licences propriétaires |
| **Performance** | Réplication validée entre minimum 3 terminaux, latence < 100ms |
| **Sécurité** | Chiffrement obligatoire (TLS 1.3), authentification mutuelle (x509) |
| **Périmètre** | POC fonctionnel uniquement — pas de mise en production industrielle |

Tableau 10 : Contraintes

# VII. Planning Prévisionnel

La planification prévisionnelle constitue un pilier fondamental pour la réussite du projet. Elle est structurée autour de trois axes complémentaires : la décomposition des tâches (WBS), la gestion des ressources (RBS), et la définition des rôles organisationnels (OBS).

## Work Breakdown Structure (WBS)

Afin d'avoir une vision structurée et opérationnelle du projet, j'ai défini une décomposition hiérarchique des tâches à réaliser. Cette approche permet de planifier de manière précise les livrables à chaque étape du projet :

Figure 4 :Work Breakdown Structure (WBS)

| **Macro-Tâche** | **Durée** | **Période** | **Jalon** |
| --- | --- | --- | --- |
| Cadrage & Conception | 4 semaines | S1 — S4 | J1 : Plan Directeur (26/03) |
| Mise en Place Environnement | 4 semaines | S5 — S8 | J2 : Env. opérationnel (14/04) |
| Module Stockage Local | 4 semaines | S7 — S10 | J3 : Module validé (05/05) |
| Module P2P | 4 semaines | S11 — S14 | J4 : Sync P2P validée (02/06) |
| Sécurité & API | 4 semaines | S15 — S18 | J5 : API & sécurité (30/06) |
| Tests, Validation & Documentation | 6 semaines | S19 — S24 | J6 : Livraison finale (06/08) |

Tableau 11 : Macro-Tâche

## Resource Breakdown Structure (RBS)

La réussite du projet repose sur une allocation efficace des ressources humaines, techniques et documentaires. La RBS ci-dessous définit les profils clés, les outils mobilisés ainsi que l'infrastructure technique nécessaire :

Figure 5 :Resource Breakdown Structure (RBS)

- **Ressources Humaines**

| **Ressource** | **Rôle** | **Responsabilités** | **Temps Alloué** |
| --- | --- | --- | --- |
| Jesse MPIGA-ODOUMBA | Stagiaire PFE / Développeur Principal | Conception, développement, tests, documentation | 100% (40h/sem × 24 sem) |
| Mme Soumia CHOKRI | Tuteur Entreprise | Encadrement, validation des livrables, expertise métier | 10% (4h/sem) |
| M. Ayoub Amrani | Tuteur Académique | Suivi pédagogique, évaluation, jury soutenance | 5% (3 contacts × 2h) |
| Équipe IT AL BARAA | Support Technique | Accès infrastructure, support technique ponctuel | 5% (2h/sem) |

Tableau 12 : Ressources Humaines

- **Ressources Techniques & Logicielles**

| **Catégorie** | **Outil / Technologie** | **Licence** | **Usage** |
| --- | --- | --- | --- |
| Développement | Python 3.11+, VS Code, Git/GitHub | Open Source | Langage et environnement principal |
| Conteneurisation | Docker / Podman, Docker Compose | Open Source | Déploiement multi-nœuds |
| Bases de Données | DuckDB, SQLite | Open Source | Stockage analytique et transactionnel |
| Synchronisation | Syncthing, zeroconf (mDNS) | Open Source | Réplication P2P et découverte |
| Cohérence | CRDTs (Yjs / Automerge) | Open Source | Résolution de conflits distribués |
| Sécurité | OpenSSL, TLS 1.3 | Open Source | Chiffrement et certificats x509 |
| API | FastAPI, Swagger UI | Open Source | Exposition et documentation API |
| Tests | pytest, locust, Trivy, Bandit | Open Source | Tests unitaires, charge, sécurité |
| Documentation | MkDocs, GitHub Actions | Open Source | Documentation technique et CI/CD |

Tableau 13: Ressources Techniques & Logicielles

## Organizational Breakdown Structure (OBS)

Pour assurer une coordination optimale entre les acteurs du projet, nous avons mis en place une matrice RACI qui définit clairement les responsabilités de chaque intervenant à chaque étape du cycle de vie du projet.

| **Code** | **Rôle** | **Description** |
| --- | --- | --- |
| R   | Réalisateur | Exécute la tâche. Est responsable de l'action et de sa réalisation. |
| A   | Approbateur | Responsable final. Approuve le travail. Un seul A par tâche. |
| C   | Consulté | Fournit une expertise ou des informations. Communication bidirectionnelle. |
| I   | Informé | Tenu informé de l'avancement. Communication unidirectionnelle. |

Tableau 14 : RACI

**Matrice RACI — Organisation du Projet (OBS)**

| **Activité / Livrable** | **Jesse (Stagiaire)** | **Mme CHOKRI (Tuteur Ent.)** | **M. Ayoub Amrani** | **Équipe IT AL BARAA** |
| --- | --- | --- | --- | --- |
| Plan Directeur | **R** | **C** | **C** | **I** |
| Architecture Technique | **R** | **A** | **I** | **C** |
| Dev. Module Stockage | **R** | **A** | **I** | **C** |
| Dev. Module P2P | **R** | **A** | **I** | **C** |
| Sécurité & API | **R** | **A** | **C** | **C** |
| Tests & Validation | **R** | **A** | **I** | **C** |
| Documentation Technique | **R** | **A** | **I** | **I** |
| Rapport Final | **R** | **C** | **A** | **I** |
| Soutenance | **R** | **C** | **A** | **I** |
| Décisions Techniques | **R** | **I** | **I** | **C** |

Tableau 15 : Matrice RACI

**Légende :** **R** = Réalise **A** = Responsable final **C** = Consulté **I** = Informé

La répartition des responsabilités a été définie pour garantir une agilité maximale tout en respectant les exigences académiques et professionnelles. En tant qu'élève-ingénieur, j'assure la réalisation technique (R) de l'ensemble des briques logicielles. La validation de la cohérence technique est portée par la tutrice entreprise (A), tandis que la conformité aux attendus du diplôme d'ingénieur est supervisée par le tuteur académique (A pour les livrables de fin de cycle). L'équipe IT d'Al Baraa intervient en support consultatif (C) pour assurer l'interopérabilité avec les infrastructures existantes.

## Macro Planning

Le macro planning ci-dessous intègre les différentes phases dans un diagramme de Gantt, permettant de décomposer le projet en tâches chronologiques, d'estimer la durée de chaque phase, d'identifier les dépendances et de suivre les jalons clés.

Figure 6 : Macro Planning — Diagramme de Gantt

| **Jalon** | **Date** | **Livrable** | **Critères de Validation** |
| --- | --- | --- | --- |
| J1  | 16/03/2026 | Plan Directeur validé | Document 20-30 pages, conforme guide EIGSI, validé tuteur entreprise |
| J2  | 14/04/2026 | Environnement opérationnel | Docker déployé en < 5 min, BDD accessibles, CI/CD fonctionnel |
| J3  | 05/05/2026 | Module Stockage complet | CRUD validé, coverage > 80%, DuckDB < 1s sur 1M lignes |
| J4  | **24/06/2026** | Synchronisation P2P validée | Réplication 3 nœuds, 0 perte données, 0 conflit non résolu |
| J5  | 30/06/2026 | API & Sécurité complètes | 100% comm. chiffrées, API 100% doc. Swagger, 0 vulnérabilité critique |
| J6  | 06/08/2026 | Livraison finale & Soutenance | Tous objectifs SMART atteints, rapport validé, soutenance réussie |

Tableau 16 : Jalon macro planning

# VIII. Risques

Une identification proactive des risques a été réalisée afin d'anticiper les éventuels obstacles susceptibles d'impacter le bon déroulement du projet. Cette démarche vise à réduire l'incertitude et à mettre en place des plans de mitigation adaptés.

Figure 7 : Matrice des Risques

| **Risque Identifié** | **Impact** | **Mesure Palliative** |
| --- | --- | --- |
| Incohérence des données (Conflits P2P) | Majeur | Implémentation d'algorithmes de résolution CRDT et versioning sémantique. |
| Latence réseau élevée (Contexte Afrique) | Moyen | Utilisation de la réplication asynchrone et priorisation des métadonnées légères. |
| Sécurité des nœuds (Vols/Intrusions) | Critique | Chiffrement des bases SQLite/DuckDB au repos (AES-256) et isolation par Tenant (Silo). |

Tableau 17 : Risque Identifié

| **Risque** | **Prob.** | **Impact** | **Criticité** | **Mesures Préventives** |
| --- | --- | --- | --- | --- |
| RT-04 : Conflits P2P non résolus (CRDTs) | Élevée (3) | Élevé (3) | CRITIQUE | Librairie éprouvée (Yjs), tests scénarios conflits S13, fallback last-write-wins |
| RP-01 : Retard Plan Directeur (deadline 26/03) | Moy. (2) | Élevé (3) | CRITIQUE | Rédaction dès S1, point hebdo, buffer 3 jours |
| RP-04 : Retard rédaction rapport final | Moy. (2) | Élevé (3) | CRITIQUE | Rédaction progressive dès S19, template EIGSI utilisé |
| RT-05 : Bugs critiques découverts tard | Moy. (2) | Élevé (3) | CRITIQUE | CI/CD dès S8, tests continus, buffer S23 |
| RT-09 : Vulnérabilités sécurité critiques | Moy. (2) | Élevé (3) | CRITIQUE | Scans automatisés S17 (Trivy, Bandit), dépendances à jour |
| RT-03 : Complexité Syncthing | Moy. (2) | Moy. (2) | ÉLEVÉ | Étude doc. officielle S11, tests incrémentaux, fallback rsync |
| RT-01 : Compatibilité Docker multi-plateforme | Moy. (2) | Moy. (2) | ÉLEVÉ | Tests Linux/Windows/macOS dès S5 |
| RT-02 : Performance DuckDB insuffisante | Faible (1) | Moy. (2) | MOYEN | Benchmarks dès S7, indexation optimale |
| RT-06 : Scalabilité limitée (> 10 nœuds) | Moy. (2) | Faible (1) | FAIBLE | Tests charge S20, limitation documentée si POC uniquement |

Figure 8 :Matrice des Risques

# IX. Budget

Ce budget est académique et estimatif, destiné à valoriser le projet et à démontrer les compétences en gestion budgétaire. La monnaie utilisée est le Dirham Marocain (MAD). Le projet est réalisé dans le cadre d'un stage PFE non rémunéré pour AL BARAA CONSULTING.

## Coûts de Développement, Ressources Humaines

| **Ressource** | **Durée** | **Taux Horaire Estimé** | **Coût Total** |
| --- | --- | --- | --- |
| Jesse MPIGA-ODOUMBA (Stagiaire) | 24 sem × 40h = 960h | 25 MAD/h | 24 000 MAD |
| Mme Soumia CHOKRI (Encadrement) | 24 sem × 4h = 96h | 60 MAD/h | 5 760 MAD |
| M. Ayoub Amrani (Suivi académique) | 3 contacts × 2h = 6h | 80 MAD/h | 480 MAD |
| Équipe IT AL BARAA (Support) | 24 sem × 2h = 48h | 50 MAD/h | 2 400 MAD |
| TOTAL RESSOURCES HUMAINES | 1 110 heures | —   | 32 640 MAD |

Tableau 18 : Coûts de Développement, Ressources Humaines

## Coûts Matériels & Logiciels

| **Poste** | **Coût** | **Commentaire** |
| --- | --- | --- |
| Postes de travail | 0 MAD | Fournis par AL BARAA CONSULTING |
| Serveurs de test (3 VMs) | 0 MAD | Infrastructure existante réutilisée |
| Logiciels (Docker, DuckDB, SQLite, Syncthing, Python, FastAPI…) | 0 MAD | Stratégie 100% open-source |
| Formation (autodidacte, ressources en ligne) | 0 MAD | Documentation gratuite |
| TOTAL MATÉRIEL & LOGICIELS | 0 MAD | —   |

Tableau 19 : Coûts Matériels & Logiciels

## Récapitulatif Budgétaire

| **Catégorie** | **Coût POC (24 semaines)** | **Coût Production (An 1)** |
| --- | --- | --- |
| Ressources Humaines | 32 640 MAD | Variable |
| Matériel | 0 MAD | 0 MAD (infrastructure existante) |
| Logiciels (100% open-source) | 0 MAD | 0 MAD |
| Déploiement cloud (hors périmètre POC) | —   | 5 532 MAD/an (estimatif) |
| TOTAL | 32 640 MAD | 5 532 MAD (hors RH) |

Tableau 20 : Récapitulatif Budgétaire

## Retour sur Investissement (ROI) Estimatif

Scénario : adoption par 10 PME africaines économisant 500 MAD/mois chacune en coûts cloud (vs AWS/Azure).

| **Métrique** | **Valeur** |
| --- | --- |
| Coût développement POC | 32 640 MAD |
| Économies par organisation (vs cloud) | 500 MAD/mois × 12 = 6 000 MAD/an |
| Économies 10 organisations | 10 × 6 000 MAD = 60 000 MAD/an |
| ROI année 1 | 60 000 MAD / 32 640 MAD = 184% |
| Break-even | 6,5 mois |

Tableau 21 : Retour sur Investissement (ROI) Estimatif

- **Souveraineté numérique :** données 100% contrôlées localement, aucune dépendance à un fournisseur étranger
- **Indépendance technologique :** solution pérenne basée sur des standards ouverts
- **Conformité AUDPF :** alignement avec le cadre de politique de données de l'Union Africaine
- **Coût direct pour AL BARAA CONSULTING : 0** MAD (stage non rémunéré, matériel existant, logiciels gratuits)

# Webographie

1.  AU Commences Validation of Data Governance Frameworks — African Union, déc. 2025, consulté le février 27, 2026, https://au.int/en/pressreleases/20251202/validation-data-governance-frameworks-accelerate-digital-single-market
2.  AU Data Policy Framework (AUDPF) — African Union, consulté le février 27, 2026, https://au.int/sites/default/files/documents/42078-doc-DATA-POLICY-FRAMEWORKS-2024-ENG-V2.pdf
3.  SQLite Extension — DuckDB Documentation officielle, consulté le février 27, 2026, https://duckdb.org/docs/stable/core_extensions/sqlite
4.  DuckDB : the Rise of In-Process Analytics and Data Singularity — endjin, consulté le février 27, 2026, https://endjin.com/blog/2025/04/duckdb-rise-of-in-process-analytics-understanding-data-singularity
5.  Monolithic vs Microservices Architecture — AWS, consulté le février 27, 2026, https://aws.amazon.com/compare/the-difference-between-monolithic-and-microservices-architecture/
6.  Microservices vs. Monolithic Architecture — Atlassian, consulté le février 27, 2026, https://www.atlassian.com/microservices/microservices-architecture/microservices-vs-monolith
7.  Multi-Tenancy in Software Architecture: A Comprehensive Guide — Medium, consulté le février 27, 2026, https://medium.com/@a_farag/datmulti-tenancy-in-software-architecture-a-comprehensive-guide-fd4c92e2ca00
8.  CAP Theorem Explained: Consistency, Availability & Partition Tolerance — TiDB / PingCAP, consulté le février 27, 2026, https://www.pingcap.com/article/understanding-cap-theorem-basics-in-distributed-systems/
9.  What Is CAP Theorem and Why It Matters for Storage — MinIO, consulté le février 27, 2026, https://www.min.io/learn/cap-theorem
10. Blockchain Technology: Architecture, Applications, and Challenges — ResearchGate, consulté le février 27, 2026, https://www.researchgate.net/publication/387713923_Blockchain_Technology_Architecture_Applications_and_Challenges
11. What Is Blockchain? — IBM, consulté le février 27, 2026, https://www.ibm.com/think/topics/blockchain
12. Africa’s Digital Sovereignty Trap: The Data Center Dilemma — New America, consulté le février 27, 2026, https://www.newamerica.org/planetary-politics/briefs/africas-digital-sovereignty-trap/
13. Digital Sovereignty in Africa: Moving beyond Local Data Ownership — CIGI, consulté le février 27, 2026, https://www.cigionline.org/documents/2845/PB_no.185_eRCHbMI.pdf
14. IPFS Comparisons with Other Distributed Storage Systems — IPFS Documentation, consulté le février 27, 2026, https://docs.ipfs.tech/concepts/comparisons/
15. Decentralized Storage vs Traditional Cloud: What It Means for Your Personal Data, consulté le février 27, 2026, https://www.vana.org/articles/decentralized-storage-vs-traditional-cloud
16. La Blockchain expliquée en emojis , https://www.youtube.com/watch?v=bKtFYnrDXFk
17. La blockchain expliquée simplement ! https://youtu.be/_XC8P93Dc7k
18. Manus AI Plateforme d’assistance par intelligence artificielle pour la génération et l’analyse de contenu, \[https://manus.im/\] (https://manus.im/)
19. Gemini Assistant d’intelligence artificielle développé par Google, \[https://gemini.google.com/app?hl=fr\] (https://gemini.google.com/app?hl=fr)
20. Claude AI Assistant conversationnel basé sur l’intelligence artificielle développé par Anthropic, \[https://claude.ai/\] (https://claude.ai/)
21. 21\. Trello, Outil de gestion de projet basé sur la méthode Kanban, développé par Atlassian, \[https://trello.com/\](https://trello.com/)
22. 22\. Draw.io (Diagrams.net) — Outil de conception de diagrammes et d’architectures systèmes, \[https://draw.io/\](https://draw.io/)