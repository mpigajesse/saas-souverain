## **Plan directeur** 

Stage de Fin d'Études Spécialité : Big Data & IA 

19/02/2026 

**AL BARAA CONSULTING** 

# **Conception et Implémentation d'une Architecture Coffre-Fort Data P2P Souveraine** 

|**Tuteur Entreprise**|**Tuteur EIGSI**|**Étudiant**|
|---|---|---|
|CHOKRI Soumia|M. Ayoub Amrani|Jesse MPIGA-ODOUMBA|



Année Académique : 2025 - 2026 

## Table des matières 

|Liste des figures et des tableaux................................................................................................ 3|
|---|
|I. Contexte du Projet................................................................................................................ 1|
|1.<br>Problématique de la Souveraineté Numérique en Afrique.................................................. 1|
|2.<br>Présentation de la Solution :........................................................................................... 1|
|3.<br>Architecture Modulaire et Résultats Attendus................................................................... 2|
|4.<br>Indicateurs de Succès du Projet....................................................................................... 2|
|5.<br>Positionnement du Projet dans le Cadre du Stage.............................................................. 2|
|6.<br>Enjeux Stratégiques....................................................................................................... 3|
|7.<br>Synthèse....................................................................................................................... 3|
|8.<br>État de l'art et Positionnement Technologique.................................................................. 4|
|II. Analyse Fonctionnelle......................................................................................................... 6|
|1.<br>Analyse Fonctionnelle Externe....................................................................................... 6|
|2.<br>Fonctions Principales et Contraintes................................................................................ 7|
|3.<br>Analyse Fonctionnelle Interne........................................................................................ 8|
|III. Objectifs........................................................................................................................... 9|
|IV. Enjeux............................................................................................................................ 11|
|V. Livrables Attendus............................................................................................................ 12|
|VI. Périmètre du Projet.......................................................................................................... 13|
|VII. Planning Prévisionnel..................................................................................................... 14|
|1.<br>Work Breakdown Structure (WBS)............................................................................... 14|
|2.<br>Resource Breakdown Structure (RBS)........................................................................... 15|
|1.<br>Organizational Breakdown Structure (OBS)................................................................... 16|
|2.<br>Macro Planning........................................................................................................... 18|
|VIII. Risques........................................................................................................................ 19|
|IX. Budget............................................................................................................................ 20|
|1.<br>Coûts de Développement, Ressources Humaines............................................................ 20|
|2.<br>Coûts Matériels & Logiciels......................................................................................... 21|
|3.<br>Récapitulatif Budgétaire............................................................................................... 21|
|4.<br>Retour sur Investissement (ROI) Estimatif..................................................................... 21|
|Webographie........................................................................................................................ 22|



## **Liste des figures et des tableaux** 

Figure 1:Bête à Corne ............................................................................................................................ 6 Figure 2:Diagramme Pieuvre ................................................................................................................. 7 Figure 3 : Diagramme FAST ................................................................................................................. 8 Figure 4 :Work Breakdown Structure (WBS) ..................................................................................... 14 Figure 5 :Resource Breakdown Structure (RBS) ................................................................................. 15 Figure 6 : Macro Planning — Diagramme de Gantt ............................................................................ 18 Figure 7 : Matrice des Risques............................................................................................................. 19 Figure 8  :Matrice des Risques............................................................................................................. 20 

Tableau 1:Fonctions Principales et Contraintes ..................................................................................... 7 Tableau 2 : Cadrage & Conception de l'Architecture ............................................................................ 9 Tableau 3 : Développement du Module de Stockage Local .................................................................. 9 Tableau 4:Développement du Module de Stockage Local .................................................................. 10 Tableau 5 : Sécurité & Exposition d'APIs ........................................................................................... 10 Tableau 6 :Tests, Documentation & Clôture ....................................................................................... 11 Tableau 7 : Livrables Attendus ............................................................................................................ 12 Tableau 8 : Contraintes ........................................................................................................................ 13 Tableau 9 : Macro-Tâche ..................................................................................................................... 14 Tableau 10 : Ressources Humaines ..................................................................................................... 15 Tableau 11:  Ressources Techniques & Logicielles ............................................................................ 16 Tableau 12 : RACI ............................................................................................................................... 17 Tableau 13 : Matrice RACI.................................................................................................................. 17 Tableau 14 : Jalon macro planning ...................................................................................................... 18 Tableau 15 : Risque Identifié ............................................................................................................... 19 Tableau 16 : Coûts de Développement, Ressources Humaines ........................................................... 21 Tableau 17 : Coûts Matériels & Logiciels ........................................................................................... 21 Tableau 18 : Récapitulatif Budgétaire ................................................................................................. 21 Tableau 19 : Retour sur Investissement (ROI) Estimatif ..................................................................... 22 

## **I. Contexte du Projet** 

## **1. Problématique de la Souveraineté Numérique en Afrique** 

Le secteur de la gestion des données en Afrique est aujourd’hui caractérisé par une dépendance critique aux solutions cloud centralisées étrangères. Cette dépendance technologique engendre des risques majeurs en matière de souveraineté numérique, de sécurité des données et de continuité de service, en contradiction directe avec les orientations stratégiques définies par l’AU Data Policy Framework (AUDPF), validé par l’Union Africaine en décembre 2025. 

En effet, bien que les organisations africaines disposent d’un accès à des infrastructures cloud performantes (AWS, Azure, Google Cloud), elles ne maîtrisent ni la localisation physique de leurs données, ni les conditions d’accès, ni la pérennité des services utilisés. 

Dans ce contexte, la problématique centrale de ce projet peut être formulée comme suit : Comment garantir la souveraineté, la sécurité et la résilience des données dans un environnement africain, tout en réduisant la dépendance aux infrastructures cloud centralisées étrangères ? 

L’enjeu est donc de concevoir une alternative fiable et performante reposant sur des technologies opensource, permettant aux organisations de reprendre le contrôle de leurs données. 

## **2. Présentation de la Solution :** 

Une Brique Universelle Décentralisée Pour répondre à cette problématique, ce projet vise à concevoir et implémenter une architecture de type Coffre-Fort Data P2P Souveraine, pensée comme une “brique universelle” de Backend-as-a-Service (BaaS) décentralisée. 

L’approche retenue repose sur trois axes principaux : 

## ➢ **Autonomie Infrastructurelle** 

Chaque terminal est transformé en un nœud de stockage intelligent, capable d’assurer la persistance locale des données ainsi que leur réplication automatique en pair-à-pair (P2P), grâce à des mécanismes de synchronisation distribuée (Syncthing), de découverte de services (mDNS) et de résolution de conflits (CRDTs). 

## ➢ **Ingénierie par Orchestration** 

Plutôt que de développer une solution complète from scratch, le projet s’appuie sur l’orchestration de technologies open-source éprouvées telles que DuckDB, SQLite, Syncthing et Docker. Ce choix permet de garantir une meilleure fiabilité, une réduction des coûts de développement, ainsi qu’une maintenabilité accrue. 

1 

## ➢ **Sécurité et Interopérabilité** 

La solution intègre nativement des mécanismes de sécurité robustes, incluant le chiffrement des communications (TLS 1.3), l’authentification mutuelle par certificats x509, ainsi que le chiffrement des données au repos (AES-256). Par ailleurs, l’exposition d’API standards (REST et gRPC) permet une intégration simple avec des applications métier existantes ou futures. 

## **3. Architecture Modulaire et Résultats Attendus** 

L’architecture proposée repose sur une approche modulaire en cinq couches : 

- ➢ Application Métier : gestion des cas d’usage spécifiques 

- ➢ API (REST/gRPC) : exposition des services 

- ➢ Orchestration & Logique : gestion des flux et règles 

- ➢ Stockage & Synchronisation: DuckDB, SQLite, Syncthing, CRDTs 

- ➢ Infrastructure & Sécurité : Docker, TLS, mDNS 

Cette structuration permet d’assurer une séparation claire des responsabilités, facilitant l’évolutivité et la maintenance du système. 

## **4. Indicateurs de Succès du Projet** 

Le projet sera validé à travers la réalisation d’un Proof of Concept (POC) fonctionnel répondant aux critères suivants : 

- ➢ Réplication validée sur un minimum de 3 terminaux 

- ➢ 0 perte de données lors des synchronisations 

- ➢ 0 conflit non résolu grâce aux CRDTs 

- ➢ Latence inférieure à 100 ms en local 

- ➢ Déploiement complet en moins de 30 minutes via Docker 

## **5. Positionnement du Projet dans le Cadre du Stage** 

Ce projet s’inscrit dans le cadre d’un stage de fin d’études au sein de l’entreprise AL BARAA CONSULTING, où l’objectif est de proposer une solution innovante répondant à des enjeux concrets liés à la gestion et à la souveraineté des données. 

2 

Dans ce contexte, le stagiaire intervient en tant qu’ingénieur en conception et développement, avec pour responsabilités principales : 

- ➢ L’analyse des besoins techniques et stratégiques 

- ➢ La conception de l’architecture du système 

- ➢ L’implémentation des modules clés (stockage, synchronisation, API) 

- ➢ La validation des performances et de la sécurité 

- ➢ La production de la documentation technique 

Cette expérience permet de mobiliser des compétences en Big Data, systèmes distribués, cybersécurité et ingénierie logicielle, tout en répondant à une problématique réelle du marché. 

## **6. Enjeux Stratégiques** 

Au-delà de son aspect technique, ce projet répond à plusieurs enjeux majeurs : 

- ➢ Souveraineté numérique : réduction de la dépendance aux solutions étrangères 

- ➢ Accessibilité : solutions déployables dans des environnements à ressources limitées 

- ➢ Coût : utilisation exclusive de technologies open-source 

- ➢ Résilience : fonctionnement distribué sans point unique de défaillance 

## **7. Synthèse** 

En synthèse, ce projet vise à démontrer qu’il est possible de concevoir une architecture de gestion de données performante, sécurisée et souveraine, adaptée au contexte africain, en s’appuyant sur des technologies open-source et des paradigmes distribués 

3 

## **8. État de l'art et Positionnement Technologique** 

Afin de valider la pertinence de l'architecture "Brique Universelle", une étude comparative a été menée vis-à-vis des solutions de stockage décentralisées et distribuées existantes. Ce benchmark permet de justifier le choix d'une orchestration de briques open-source (DuckDB, SQLite, Syncthing) face aux standards du marché 

|||||
|---|---|---|---|
|**Critères de**<br>**performance**|**IPFS/Filecoin**|**Solid(Pods)**|**Brique Universelle**|
|||||
|Stockage de<br>stockage|Blobs/Fichier<br>bruts|Graphe de données (RDF)|Relationnel &<br>Analytique|
|Capacité OLAP|Inexistante|Très limitée|Native (via DockDB)|
|Disponibilité Réseau|Dépendance aux<br>nœuds|Requiert une connexion|Local-first(Offline-<br>total)|
|Réplication P2P|Native (BitSwap)|Non (Client-Serveur)|Optimisée (Syncthing)|
|Résolution de<br>Conflits|Immuabilité<br>simple|Manuelle / Basique|Automatique (via<br>CRDTs)|
|Souveraineté|Dépendance tiers<br>(Pinning)|Dépendance hébergeur|Souveraineté<br>Infrastructurelle|



_Tableau 1 : État de l'art et Positionnement Technologique_ 

## ➢ Analyse Détaillée du Positionnement : 

L'analyse de cet état de l'art met en lumière trois piliers différenciateurs qui font de la Brique Universelle une solution unique pour le contexte du stage : 

## 1. Le passage du stockage passif à l'intelligence active (OLAP) : 

Contrairement aux solutions comme IPFS ou Storj qui se concentrent exclusivement sur le stockage de fichiers statiques (images, documents), la Brique Universelle intègre une puissance de calcul analytique. En utilisant DuckDB, elle permet d'exécuter des requêtes complexes sur des millions de lignes de données directement sur le terminal utilisateur, sans solliciter de serveur central. C'est un atout majeur pour les applications de Business Intelligence (BI) en zones isolées. 

## 2. La résilience face au "Gap de Connectivité" : 

Face à des initiatives comme Solid de Tim Berners-Lee, qui repose sur une architecture où le "Pod" de données doit souvent être accessible en ligne, notre solution garantit une continuité de service totale. Le paradigme "Offline-first" est ici poussé à son paroxysme : l'application métier continue de fonctionner à 100% de ses capacités sans internet, et la synchronisation P2P via Syncthing se déclenche de manière transparente dès qu'un canal (Réseau local) est détecté. 

## 3. Une Souveraineté Radicale et Infrastructurelle : 

Alors que la plupart des réseaux décentralisés actuels créent une nouvelle forme de dépendance envers des "mineurs" ou des "fournisseurs de stockage tiers", la Brique Universelle redonne le contrôle total à l'organisation. L'infrastructure est le réseau des terminaux de l'entreprise. En orchestrant Docker et TLS 1.3, nous garantissons que les données ne quittent jamais le périmètre de confiance défini par l'organisation, répondant ainsi directement aux directives de l'AU Data Policy Framework. 

## ➢ Synthèse de la Valeur Ajoutée : 

En conclusion, ce positionnement ne vise pas à "réinventer la roue", mais à assembler les meilleures technologies open-source existantes pour combler un vide technologique : le besoin d'une base de données relationnelle, décentralisée, sécurisée et capable de fonctionner sur des infrastructures hétérogènes. Cette approche par orchestration de briques matures garantit au projet une fiabilité industrielle immédiate tout en minimisant les risques de développement "from scratch". 

5 

## **II. Analyse Fonctionnelle** 

Afin de bien cerner le besoin réel du client et de structurer les fonctions à assurer par notre solution, nous avons mené une analyse fonctionnelle approfondie à travers trois outils complémentaires : la bête à cornes pour exprimer la finalité du projet, le diagramme pieuvre pour identifier les interactions avec l'environnement, et le diagramme FAST pour hiérarchiser les fonctions selon leur logique d'enchaînement. 

## **1. Analyse Fonctionnelle Externe** 

## _**Diagramme Bête à Corne :**_ 

_Figure 1:Bête à Corne_ 

Cette analyse met en évidence que la solution vise principalement à garantir la souveraineté et la sécurité des données tout en facilitant leur partage décentralisé entre organisations partenaires. Elle confirme ainsi la pertinence du développement d’une architecture distribuée adaptée au contexte africain 

6 

**==> picture [107 x 11] intentionally omitted <==**

**----- Start of picture text -----**<br>
Diagramme Pieuvre :<br>**----- End of picture text -----**<br>


_Figure 2:Diagramme Pieuvre_ 

**2. Fonctions Principales et Contraintes** 

|**ID**|**Fonction**|**Description**|
|---|---|---|
|FP1|Stocker et synchroniser les<br>données|Persistance locale + réplication P2P automatique|
|FP2|Fournir API standard|Interface REST/gRPCpour applications externes|
|FC1|Assurer la sécurité|Chiffrement TLS + au repos, authentification mutuelle|
|FC2|Garantir lesperformances|Latence < 100ms, débit > 10 MB/s|
|FC3|Assurer la portabilité|Support Linux, Windows, macOS via Docker|
|FC4|Découverte automatique des<br>nœuds|mDNS, détection < 30s|
|FC5|Respecter la conformité<br>réglementaire|AUDPF, GDPR si applicable|



7 

Ce diagramme permet d’identifier les exigences fonctionnelles majeures ainsi que les contraintes liées à la sécurité, à la compatibilité technique et à la réglementation. Ces éléments serviront de base à la définition du cahier des charges fonctionnel. 

## **3. Analyse Fonctionnelle Interne** 

## _**Diagramme FAST :**_ 

_Figure 3 : Diagramme FAST_ 

La décomposition FAST articule les fonctions autour de deux axes principaux : le stockage et la synchronisation (FP1), et l'exposition d'une API standard (FP2). 

FP1 se déploie en trois sous-fonctions : stocker localement (DuckDB pour l'analytique, SQLite pour les métadonnées, système de fichiers pour les données brutes), répliquer automatiquement (Syncthing, CRDTs, versioning), et découvrir les pairs (mDNS/Avahi). FP2 couvre l'exposition d'une API REST avec endpoints CRUD et documentation OpenAPI, ainsi qu'une API gRPC optionnelle pour les cas de haute performance. 

8 

## **III. Objectifs** 

L'objectif principal du stage est de concevoir et de mettre en œuvre une architecture Coffre-Fort Data P2P Souveraine fonctionnelle, documentée et testée. Cet objectif se décline en plusieurs objectifs spécifiques SMART : 

1. **Cadrage & Conception de l'Architecture** 

Produire une architecture technique complète (diagrammes C4, **S SPÉCIFIQUE** spécifications API, modèles de données) validée par le tuteur entreprise **M MESURABLE** Plan Directeur déposé sur la plateforme EIGSI avant le 19 mars 2026 S’appuyer sur les ressources existantes et les outils de modélisation **A ATTEIGNABLE** disponibles (draw.io, Lucidchart) Basé sur les besoins identifiés et les technologies maîtrisées (Python, **R RÉALISTE** Docker, DuckDB, SQLite) **T TEMPOREL** Livrable validé dans les 4 premières semaines du stage _Tableau 3 : Cadrage & Conception de l'Architecture_ 

- **Livrable :** Plan Directeur déposé sur la plateforme EIGSI avant le 26 mars 2026 

**2. Développement du Module de Stockage Local** 

Implémenter un module de stockage combinant DuckDB **S SPÉCIFIQUE** (analytique), SQLite (métadonnées) et système de fichiers avec API CRUD complète Coverage de tests unitaires > 80%, performance DuckDB < 1s sur **M MESURABLE** 1M lignes Utiliser les librairies Python éprouvées (duckdb, sqlite3) et les **A ATTEIGNABLE** patterns REST standards Technologies open-source disponibles, documentation complète, **R RÉALISTE** environnement Docker standardisé **T TEMPOREL** Module opérationnel et validé d’ici la semaine 10 _Tableau 4 : Développement du Module de Stockage Local_ 

- **Livrable :** Module de stockage opérationnel, performances validées (DuckDB < 1s sur 1M lignes 

9 

**3. Développement du Module de Synchronisation P2P** 

Intégrer Syncthing avec découverte mDNS et résolution de conflits via **S SPÉCIFIQUE** CRDTs entre plusieurs terminaux Réplication validée entre 3 terminaux : 0 perte de données, 0 conflit **M MESURABLE** non résolu Syncthing est un outil mature avec API REST documentée ; CRDTs **A ATTEIGNABLE** bien documentés en littérature scientifique Environnement de test multi-noeuds configurable localement via **R RÉALISTE** Docker Compose **T TEMPOREL** POC de synchronisation P2P validé d’ici la semaine 14 

_Tableau 5:Développement du Module de Stockage Local_ 

- **Livrable :** POC de synchronisation P2P validé sur 3+ nœuds. 

**4. Sécurité & Exposition d'APIs** 

Sécuriser toutes les communications (TLS 1.3, authentification **S SPÉCIFIQUE** mutuelle x509) et exposer une API REST 100% documentée (Swagger/OpenAPI) 0 vulnérabilité critique détectée (Trivy, Bandit), API documentée à **M MESURABLE** 100% Utilisation de bibliothèques standards (cryptography, certifi) et outils **A ATTEIGNABLE** de scan automatisés TLS 1.3 et x509 sont des standards industriels bien supportés par les **R RÉALISTE** frameworks Python **T TEMPOREL** Sécurisation et documentation complètes d’ici la semaine 18 

_Tableau 6 : Sécurité & Exposition d'APIs_ 

- Livrable : API REST documentée + rapport de scan sécurité (Trivy, Bandit). 

**5. Tests, Documentation & Clôture** 

Valider le POC complet (tests intégration, tests charge 10+ noeuds) et **S SPÉCIFIQUE** produire documentation technique et rapport final conformes aux exigences EIGSI 100% des tests d’intégration passants, système validé sous charge 10+ **M MESURABLE** noeuds, rapport validé par les tuteurs Planning structuré avec semaine tampon, outils de test automatisés **A ATTEIGNABLE** (pytest, Locust) disponibles Capitaliser sur toutes les phases précédentes pour une clôture **R RÉALISTE** structurée et documentée **T TEMPOREL** Soutenance et livraison finale avant le 6 août 2026 _Tableau 7 :Tests, Documentation & Clôture_ 

- **Livrable :** POC documenté (PDF) + support de soutenance (PPT/PDF, 15-20 slides). 

## **IV. Enjeux** 

Le développement d'une architecture P2P souveraine présente des enjeux majeurs à plusieurs niveaux : 

## ➢ **Technique** 

- Qualité et cohérence des données répliquées : sans résolution de conflits robuste (CRDTs), les données peuvent diverger entre nœuds. 

- Performance et scalabilité : le système doit répondre en moins de 100ms pour les requêtes locales et supporter 10+ nœuds simultanément. 

- Intégration transparente : la brique doit être deployable via Docker en moins de 5 minutes, sans configuration manuelle complexe. 

## ➢ **Métiers** 

   - Adoption par les organisations africaines : la solution doit être simple à déployer et à maintenir, même sans expertise système avancée. 

   - Alignement avec l'AUDPF : la conformité avec le cadre de politique de données de l'Union Africaine est un critère de succès stratégique. 

- ➢ **Économique** 

   - • ROI démontrable : le projet doit démontrer des économies concrètes vs. cloud étranger (estimées à 500 MAD/mois/organisation). 

11 

   - Budget nul en logiciels : la stratégie 100% open-source élimine tout coût de licence propriétaire. 

- ➢ **Réglementaire** 

   - • Conformité RGPD : traitement des données personnelles avec chiffrement et contrôle d'accès conformes. 

   - Conformité AUDPF : stockage local garanti, sans transfert vers des serveurs étrangers sans consentement explicite 

## **V. Livrables Attendus** 

Voici les livrables attendus à l'issue du projet, chacun contribuant à la mise en œuvre complète et fonctionnelle du système : 

|||||
|---|---|---|---|
|**Livrable**|**Format**|**Taille / Durée**|**Date Limite**|
|||||
|Plan Directeur d’expérience<br>professionnelle|PDF|20-30 pages|26/03/2026|
|Architecture Technique|PDF + diagrammes|15-20 pages|14/04/2026|
|POC Fonctionnel|Docker + Git|Image < 500 MB|07/07/2026|
|Documentation Technique|Web (MkDocs) + PDF|30+ pages|07/07/2026|
|Rapport de Tests|PDF + logs|10-15pages|07/07/2026|
|Rapport Final de Stage|PDF|30+ pages|23/07/2026|
|Évaluation des compétences<br>par l’entreprise|PDF (signé + tamponné)|Grille EIGSI|23/07/2026|
|Plan de soutenance|PDF|1-2 pages|30/07/2026|
|Support de soutenance|PPT ou PDF|15-20 slides|06/08/2026|



_Tableau 8 : Livrables Attendus_ 

12 

## **VI. Périmètre du Projet** 

## **INCLUS DANS LE PROJET HORS PÉRIMÈTRE** 

- Analyse et état de l’art des technologies P2P, synchronisation et BDD embarquées 

   - Conception complète de l’architecture 

   - technique (diagrammes C4, spécifications API, modèles de données) 

   - Développement d’un POC fonctionnel (Docker + code source Git) 

   - **●** Intégration des modules : stockage local (DuckDB + SQLite), synchronisation P2P (Syncthing), découverte (mDNS), sécurité (TLS 1.3), API REST 

- Tests unitaires (coverage > 80%), intégration (100% passants), charge (10+ nœuds) et sécurité (0 vulnérabilité critique) 

      - Développement d’applications métier frontend ou logique métier 

      - Mise en production industrielle à grande échelle 

      - Intégration avec systèmes tiers extérieurs (ERP, CRM, comptabilité) 

      - Support utilisateur post-déploiement 

      - Développements mobiles (Android, iOS) 

- Documentation technique complète (guide installation, utilisation, API, déploiement multi-nœuds) 

   - Rapport final de stage et support de soutenance 

_Tableau 9 : Périmètre du Projet_ 

➢ **Contraintes** 

**Contrainte Description Durée** 24 semaines (19 février — 6 août 2026) **Technologies** 100% open-source —pas de licences propriétaires **Performance** Réplication validée entre minimum 3 terminaux, latence < 100ms **Sécurité** Chiffrement obligatoire (TLS 1.3), authentification mutuelle (x509) POC fonctionnel uniquement — pas de mise en production **Périmètre** industrielle _Tableau 10 : Contraintes_ 

## **VII. Planning Prévisionnel** 

La planification prévisionnelle constitue un pilier fondamental pour la réussite du projet. Elle est structurée autour de trois axes complémentaires : la décomposition des tâches (WBS), la gestion des ressources (RBS), et la définition des rôles organisationnels (OBS). 

**1. Work Breakdown Structure (WBS)** Afin d'avoir une vision structurée et opérationnelle du projet, j'ai défini une décomposition hiérarchique des tâches à réaliser. Cette approche permet de planifier de manière précise les livrables à chaque étape du projet : 

_Figure 4 :Work Breakdown Structure (WBS)_ 

|**Macro-Tâche**|**Durée**|**Période**|**Jalon**|
|---|---|---|---|
|Cadrage & Conception|4 semaines|S1 — S4|J1 : Plan Directeur (26/03)|
|Mise en Place Environnement|4 semaines|S5 — S8|J2 : Env. opérationnel (14/04)|
|Module Stockage Local|4 semaines|S7 — S10|J3 : Module validé (05/05)|
|Module P2P|4 semaines|S11 — S14|J4 : Sync P2P validée (02/06)|
|Sécurité & API|4 semaines|S15 — S18|J5 : API & sécurité (30/06)|
|Tests, Validation &<br>Documentation|6 semaines|S19 — S24|J6 : Livraison finale (06/08)|



14 

## **2. Resource Breakdown Structure (RBS)** 

La réussite du projet repose sur une allocation efficace des ressources humaines, techniques et documentaires. La RBS ci-dessous définit les profils clés, les outils mobilisés ainsi que l'infrastructure technique nécessaire : 

_Figure 5 :Resource Breakdown Structure (RBS)_ 

## ➢ **Ressources Humaines** 

|**Ressource**|**Rôle**|**Responsabilités**|**Temps Alloué**|
|---|---|---|---|
|Jesse MPIGA-<br>ODOUMBA|Stagiaire PFE /<br>Développeur<br>Principal|Conception, développement,<br>tests, documentation|100% (40h/sem × 24<br>sem)|
|Mme Soumia<br>CHOKRI|Tuteur Entreprise|Encadrement, validation des<br>livrables, expertise métier|10% (4h/sem)|
|M. Ayoub Amrani|Tuteur<br>Académique|Suivi pédagogique, évaluation,<br>jury soutenance|5% (3 contacts × 2h)|
|Équipe IT AL<br>BARAA|Support<br>Technique|Accès infrastructure, support<br>technique ponctuel|5% (2h/sem)|



15 

## ➢ **Ressources Techniques & Logicielles** 

|||||
|---|---|---|---|
|**Catégorie**|**Outil / Technologie**|**Licence**|**Usage**|
|||||
|Développement|Python 3.11+, VS Code,<br>Git/GitHub|Open<br>Source|Langage et environnement principal|
|Conteneurisation|Docker / Podman, Docker<br>Compose|Open<br>Source|Déploiement multi-nœuds|
|Bases de Données|DuckDB, SQLite|Open<br>Source|Stockage analytique et transactionnel|
|Synchronisation|Syncthing, zeroconf<br>(mDNS)|Open<br>Source|Réplication P2P et découverte|
|Cohérence|CRDTs (Yjs / Automerge)|Open<br>Source|Résolution de conflits distribués|
|Sécurité|OpenSSL, TLS 1.3|Open<br>Source|Chiffrement et certificats x509|
|API|FastAPI, Swagger UI|Open<br>Source|Exposition et documentation API|
|Tests|pytest, locust, Trivy, Bandit|Open<br>Source|Tests unitaires, charge, sécurité|
|Documentation|MkDocs, GitHub Actions|Open<br>Source|Documentation technique et CI/CD|



_Tableau 13:  Ressources Techniques & Logicielles_ 

## **1. Organizational Breakdown Structure (OBS)** 

Pour assurer une coordination optimale entre les acteurs du projet, nous avons mis en place une matrice RACI qui définit clairement les responsabilités de chaque intervenant à chaque étape du cycle de vie du projet. 

||||
|---|---|---|
|**Code**|**Rôle**|**Description**|
||||
|R|Réalisateur|Exécute la tâche. Est responsable de l'action et de sa réalisation.|
|A|Approbateur|Responsable final. Approuve le travail. Un seul A par tâche.|
|C|Consulté|Fournit une expertise ou des informations. Communication bidirectionnelle.|
|I|Informé|Tenu informé de l'avancement. Communication unidirectionnelle.|
|||_Tableau 14 : RACI_|



## **Matrice RACI — Organisation du Projet (OBS)** 

||||||
|---|---|---|---|---|
|**Activité / Livrable**|**Jesse**<br>**(Stagiaire)**|**Mme**<br>**CHOKRI**<br>**(Tuteur Ent.)**|**M. Ayoub**<br>**Amrani**|**Équipe IT**<br>**AL BARAA**|
||||||
||||||
|Plan Directeur|**R**|**C**|**C**|**I**|
||||||
||||||
|Architecture Technique|**R**|**A**|**I**|**C**|
||||||
||||||
|Dev. Module Stockage|**R**|**A**|**I**|**C**|
||||||
||||||
|Dev. Module P2P|**R**|**A**|**I**|**C**|
||||||
||||||
|Sécurité & API|**R**|**A**|**C**|**C**|
||||||
||||||
|Tests & Validation|**R**|**A**|**I**|**C**|
||||||
||||||
|Documentation<br>Technique|**R**|**A**|**I**|**I**|
||||||
||||||
|Rapport Final|**R**|**C**|**A**|**I**|
||||||
||||||
|Soutenance|**R**|**C**|**A**|**I**|
||||||
||||||
|Décisions Techniques|**R**|**I**|**I**|**C**|
||||||
||_Tableau 15 : Matrice RACI_||||



**Légende : R** = Réalise **A** = Responsable final **C** = Consulté **I** = Informé 

La répartition des responsabilités a été définie pour garantir une agilité maximale tout en respectant les exigences académiques et professionnelles. En tant qu'élève-ingénieur, j'assure la réalisation technique (R) de l'ensemble des briques logicielles. La validation de la cohérence technique est portée par la tutrice entreprise (A), tandis que la conformité aux attendus du diplôme d'ingénieur est 

17 

supervisée par le tuteur académique (A pour les livrables de fin de cycle). L'équipe IT d'Al Baraa intervient en support consultatif (C) pour assurer l'interopérabilité avec les infrastructures existantes. 

## **2. Macro Planning** 

Le macro planning ci-dessous intègre les différentes phases dans un diagramme de Gantt, permettant de décomposer le projet en tâches chronologiques, d'estimer la durée de chaque phase, d'identifier les dépendances et de suivre les jalons clés. 

_Figure 6 : Macro Planning — Diagramme de Gantt_ 

|**Jalon**|**Date**|**Livrable**|**Critères de Validation**|
|---|---|---|---|
|J1|16/03/2026|Plan Directeur validé|Document 20-30 pages, conforme guide<br>EIGSI,validé tuteur entreprise|
|J2|14/04/2026|Environnement<br>opérationnel|Docker déployé en < 5 min, BDD<br>accessibles, CI/CD fonctionnel|
|J3|05/05/2026|Module Stockage<br>complet|CRUD validé, coverage > 80%, DuckDB <<br>1s sur 1M lignes|
|J4|24/06/2026|Synchronisation P2P<br>validée|Réplication 3 nœuds, 0 perte données, 0<br>conflit non résolu|
|J5|30/06/2026|API & Sécurité<br>complètes|100% comm. chiffrées, API 100% doc.<br>Swagger,0 vulnérabilité critique|
|J6|06/08/2026|Livraison finale &<br>Soutenance|Tous objectifs SMART atteints, rapport<br>validé, soutenance réussie|



18 

## **VIII. Risques** 

Une identification proactive des risques a été réalisée afin d'anticiper les éventuels obstacles susceptibles d'impacter le bon déroulement du projet. Cette démarche vise à réduire l'incertitude et à mettre en place des plans de mitigation adaptés. 

_Figure 7 : Matrice des Risques_ 

|**Risque Identifié**|**Impact**|**Mesure Palliative**|
|---|---|---|
|Incohérence des données (Conflits<br>P2P)|Majeur|Implémentation<br>d'algorithmes de<br>résolution CRDT et<br>versioningsémantique.|
|Latence réseau élevée (Contexte<br>Afrique)|Moyen|Utilisation de la<br>réplication asynchrone et<br>priorisation des<br>métadonnées légères.|
|Sécurité des nœuds<br>(Vols/Intrusions)|Critique|Chiffrement des bases<br>SQLite/DuckDB au<br>repos (AES-256) et<br>isolation par Tenant<br>(Silo).|



19 

||||||
|---|---|---|---|---|
|**Risque**|**Prob.**|**Impact**|**Criticité**|**Mesures Préventives**|
||||||
|RT-04 : Conflits P2P<br>non résolus (CRDTs)|Élevée<br>(3)|Élevé (3)|CRITIQUE|Librairie éprouvée (Yjs), tests<br>scénarios conflits S13, fallback last-<br>write-wins|
|RP-01 : Retard Plan<br>Directeur (deadline<br>26/03)|Moy. (2)|Élevé (3)|CRITIQUE|Rédaction dès S1, point hebdo, buffer<br>3 jours|
|RP-04 : Retard<br>rédaction rapport final|Moy. (2)|Élevé (3)|CRITIQUE|Rédaction progressive dès S19,<br>template EIGSI utilisé|
|RT-05 : Bugs critiques<br>découverts tard|Moy. (2)|Élevé (3)|CRITIQUE|CI/CD dès S8, tests continus, buffer<br>S23|
|RT-09 : Vulnérabilités<br>sécurité critiques|Moy. (2)|Élevé (3)|CRITIQUE|Scans automatisés S17 (Trivy, Bandit),<br>dépendances àjour|
|RT-03 : Complexité<br>Syncthing|Moy. (2)|Moy. (2)|ÉLEVÉ|Étude doc. officielle S11, tests<br>incrémentaux, fallback rsync|
|RT-01 : Compatibilité<br>Docker multi-<br>plateforme|Moy. (2)|Moy. (2)|ÉLEVÉ|Tests Linux/Windows/macOS dès S5|
|RT-02 : Performance<br>DuckDB insuffisante|Faible<br>(1)|Moy. (2)|MOYEN|Benchmarks dès S7, indexation<br>optimale|
|RT-06 : Scalabilité<br>limitée(> 10 nœuds)|Moy. (2)|Faible<br>(1)|FAIBLE|Tests charge S20, limitation<br>documentée si POC uniquement|



_Figure 8  :Matrice des Risques_ 

## **IX. Budget** 

Ce budget est académique et estimatif, destiné à valoriser le projet et à démontrer les compétences en gestion budgétaire. La monnaie utilisée est le Dirham Marocain (MAD). Le projet est réalisé dans le cadre d'un stage PFE non rémunéré pour AL BARAA CONSULTING. 

**1. Coûts de Développement, Ressources Humaines** 

|||||
|---|---|---|---|
|**Ressource**|**Durée**|**Taux Horaire**<br>**Estimé**|**Coût Total**|
|||||
|Jesse MPIGA-<br>ODOUMBA(Stagiaire)|24 sem × 40h =<br>960h|25 MAD/h|24 000 MAD|
|Mme Soumia CHOKRI<br>(Encadrement)|24 sem × 4h = 96h|60 MAD/h|5 760 MAD|
|M. Ayoub Amrani (Suivi<br>académique)|3 contacts × 2h =<br>6h|80 MAD/h|480 MAD|
|Équipe IT AL BARAA<br>(Support)|24 sem × 2h = 48h|50 MAD/h|2 400 MAD|



20 

|||||
|---|---|---|---|
|**Ressource**|**Durée**|**Taux Horaire**<br>**Estimé**|**Coût Total**|
|||||
|TOTAL RESSOURCES<br>HUMAINES|1 110 heures|—|32 640 MAD|



Tableau 18 : Coûts de Développement, Ressources Humaines 

**2. Coûts Matériels & Logiciels** 

||||
|---|---|---|
|**Poste**|**Coût**|**Commentaire**|
||||
|Postes de travail|0 MAD|Fournis par AL BARAA<br>CONSULTING|
|Serveurs de test (3 VMs)|0 MAD|Infrastructure existante réutilisée|
|Logiciels (Docker, DuckDB,<br>SQLite, Syncthing, Python,<br>FastAPI…)|0 MAD|Stratégie 100% open-source|
|Formation (autodidacte,<br>ressources en ligne)|0 MAD|Documentation gratuite|
|TOTAL MATÉRIEL &<br>LOGICIELS|0 MAD|—|



_Tableau 19 : Coûts Matériels & Logiciels_ 

**3. Récapitulatif Budgétaire** 

||||
|---|---|---|
|**Catégorie**|**Coût POC (24**<br>**semaines)**|**Coût Production (An 1)**|
||||
|Ressources Humaines|32 640 MAD|Variable|
|Matériel|0 MAD|0 MAD (infrastructure<br>existante)|
|Logiciels(100% open-source)|0 MAD|0 MAD|
|Déploiement cloud (hors<br>périmètre POC)|—|5 532 MAD/an (estimatif)|
|TOTAL|32 640 MAD|5 532 MAD(hors RH)|



_Tableau 20 : Récapitulatif Budgétaire_ 

## **4. Retour sur Investissement (ROI) Estimatif** 

Scénario : adoption par 10 PME africaines économisant 500 MAD/mois chacune en coûts cloud (vs AWS/Azure). 

|||
|---|---|
|**Métrique**|**Valeur**|
|||
|Coût développement POC|32 640 MAD|
|Économies par organisation (vs<br>cloud)|500 MAD/mois × 12 = 6 000 MAD/an|
|Économies 10 organisations|10 × 6 000 MAD = 60 000 MAD/an|
|ROI année 1|60 000 MAD / 32 640 MAD = 184%|



21 

|||
|---|---|
|**Métrique**|**Valeur**|
|||
|Break-even|6,5 mois|



_Tableau 21 : Retour sur Investissement (ROI) Estimatif_ 

- ➢ **Souveraineté numérique :** données 100% contrôlées localement, aucune dépendance à un fournisseur étranger 

- ➢ **Indépendance technologique :** solution pérenne basée sur des standards ouverts 

- ➢ **Conformité AUDPF :** alignement avec le cadre de politique de données de l'Union Africaine 

- ➢ **Coût direct pour AL BARAA CONSULTING : 0** MAD (stage non rémunéré, matériel existant, logiciels gratuits) 

## **Webographie** 

1. AU Commences Validation of Data Governance Frameworks — African Union, déc. 2025, consulté le février 27, 2026, https://au.int/en/pressreleases/20251202/validation-datagovernance-frameworks-accelerate-digital-single-market 

2. AU Data Policy Framework (AUDPF) — African Union, consulté le février 27, 2026, https://au.int/sites/default/files/documents/42078-doc-DATA-POLICY-FRAMEWORKS2024-ENG-V2.pdf 

3. SQLite Extension — DuckDB Documentation officielle, consulté le février 27, 2026, https://duckdb.org/docs/stable/core_extensions/sqlite 

4. DuckDB : the Rise of In-Process Analytics and Data Singularity — endjin, consulté le février 27, 2026, https://endjin.com/blog/2025/04/duckdb-rise-of-in-process-analytics-understandingdata-singularity 

5. Monolithic vs Microservices Architecture — AWS, consulté le février 27, 2026, https://aws.amazon.com/compare/the-difference-between-monolithic-and-microservicesarchitecture/ 

6. Microservices vs. Monolithic Architecture — Atlassian, consulté le février 27, 2026, https://www.atlassian.com/microservices/microservices-architecture/microservices-vsmonolith 

7. Multi-Tenancy in Software Architecture: A Comprehensive Guide — Medium, consulté le février 27, 2026, https://medium.com/@a_farag/datmulti-tenancy-in-software-architecture-acomprehensive-guide-fd4c92e2ca00 

8. CAP Theorem Explained: Consistency, Availability & Partition Tolerance — TiDB / PingCAP, consulté le février 27, 2026, https://www.pingcap.com/article/understanding-captheorem-basics-in-distributed-systems/ 

9. What Is CAP Theorem and Why It Matters for Storage — MinIO, consulté le février 27, 2026, https://www.min.io/learn/cap-theorem 

10. Blockchain Technology: Architecture, Applications, and Challenges — ResearchGate, 

22 

consulté le février 27, 2026, 

https://www.researchgate.net/publication/387713923_Blockchain_Technology_Architecture_ Applications_and_Challenges 

11. What Is Blockchain? — IBM, consulté le février 27, 2026, https://www.ibm.com/think/topics/blockchain 

12. Africa’s Digital Sovereignty Trap: The Data Center Dilemma — New America, consulté le - - - 

février 27, 2026, https://www.newamerica.org/planetary politics/briefs/africas digital - 

sovereignty trap/ 

13. Digital Sovereignty in Africa: Moving beyond Local Data Ownership — CIGI, consulté le février 27, 2026, https://www.cigionline.org/documents/2845/PB_no.185_eRCHbMI.pdf 

14. IPFS Comparisons with Other Distributed Storage Systems — IPFS Documentation, consulté le février 27, 2026, https://docs.ipfs.tech/concepts/comparisons/ 

15. Decentralized Storage vs Traditional Cloud: What It Means for Your Personal Data, consulté le février 27, 2026, https://www.vana.org/articles/decentralized-storage-vs-traditional-cloud 

16. La Blockchain expliquée en emojis , https://www.youtube.com/watch?v=bKtFYnrDXFk 

17. La blockchain expliquée simplement ! https://youtu.be/_XC8P93Dc7k 

18. Manus AI Plateforme d’assistance par intelligence artificielle pour la génération et l’analyse de contenu, [https://manus.im/] (https://manus.im/) 

19. Gemini Assistant d’intelligence artificielle développé par Google, 

= [https://gemini.google.com/app?hl fr] ( https://gemini.google.com/app?hl=fr ) 

20. Claude AI Assistant conversationnel basé sur l’intelligence artificielle développé par Anthropic, [https://claude.ai/] ( https://claude.ai/ ) 

21. 21. Trello, Outil de gestion de projet basé sur la méthode Kanban, développé par Atlassian, [https://trello.com/]( https://trello.com/ ) 

22. 22. Draw.io (Diagrams.net) — Outil de conception de diagrammes et d’architectures systèmes, [https://draw.io/]( https://draw.io/ ) 

23 

