## **SOUVERAINETÉ DES DONNÉES** 

## **DANS LES SAAS** 

Problématique, état de l’art, cartographie des contraintes et pistes de réflexion pour la conception d’une solution 

## **Document de cadrage — Mars 2026** 

_Ce document n’a pas vocation à proposer une solution mais à guider et structurer la réflexion._ 

Souveraineté des données dans les SaaS 

## **Table des matières** 

2 

Souveraineté des données dans les SaaS 

## **1. La problématique en profondeur** 

## **1.1 Définitions fondamentales** 

**Résidence des données (Data Residency) :** désigne l’emplacement physique où les données sont stockées. C’est une question géographique : dans quel pays, dans quel datacenter. La résidence répond à la question « où est le coffre-fort ? » 

**Souveraineté des données (Data Sovereignty) :** va au-delà de la résidence. Elle englobe le contrôle juridique, opérationnel et technique sur les données. La souveraineté répond à la question « qui peut ouvrir le coffre-fort, et sous quelle loi ? » Même si les données sont physiquement dans le bon pays, elles ne sont pas souveraines si une entité étrangère peut y accéder par voie légale ou technique. 

**Portabilité des données (Data Portability) :** la capacité pour le propriétaire de récupérer ses données dans un format exploitable et de les transférer vers un autre système. Sans portabilité, la souveraineté est illusoire : on possède des données qu’on ne peut pas libérer. 

**Propriété des données (Data Ownership) :** la question juridique et philosophique de savoir qui « possède » les données. Dans un SaaS, le contrat peut stipuler que le client est propriétaire, mais si l’éditeur détient les seuls moyens d’accès, la propriété est de facto vidée de sa substance. 

## **1.2 Anatomie du problème dans le SaaS** 

Le modèle SaaS crée une asymétrie fondamentale : le client délègue simultanément le traitement et le stockage de ses données à un tiers. Cette délégation engendre plusieurs couches de vulnérabilité qui s’empilent. 

## _**1.2.1 La couche juridictionnelle**_ 

Les données hébergées chez un éditeur sont soumises aux lois du pays où il opère. Le CLOUD Act américain permet aux autorités d’exiger l’accès aux données détenues par des entreprises américaines, même si ces données sont physiquement stockées hors des ÉtatsUnis. Cette extraterritorialité crée un conflit direct avec le RGPD européen et les lois de protection des données nationales. L’arrêt Schrems II de 2020 a invalidé le Privacy Shield précisément pour cette raison, créant une insécurité juridique qui persiste malgré le EU-US Data Privacy Framework. 

## _**1.2.2 La couche technique**_ 

Même avec des garanties contractuelles, l’éditeur SaaS a techniquement accès aux données en clair. Les données sont chiffrées au repos et en transit, mais elles sont nécessairement déchiffrées pour être traitées. C’est cette troisième phase — les données en cours d’utilisation — qui constitue le maillon faible. L’éditeur, ses employés, ses sous-traitants, ou tout attaquant ayant compromis l’infrastructure, peuvent accéder aux données à ce moment. 

## _**1.2.3 La couche multi-tenant**_ 

Le modèle multi-tenant aggrave le risque : les données de multiples clients coexistent sur la même infrastructure, parfois dans la même base de données. Une vulnérabilité dans l’isolation (une clause WHERE manquante, un bug d’autorisation) peut exposer les données d’un client à un autre. Les attaques de type noisy neighbor ou side-channel dans les environnements partagés sont des vecteurs documentés. 

3 

Souveraineté des données dans les SaaS 

## _**1.2.4 La couche de dépendance**_ 

Le SaaS crée une dépendance totale et multi-dimensionnelle : dépendance à la disponibilité (si l’éditeur tombe, le client est paralysé), dépendance à la pérennité (si l’éditeur fait faillite ou est racheté, les données sont en péril), dépendance à la politique tarifaire (l’éditeur peut augmenter ses prix sachant que le coût de migration est prohibitif), et dépendance à l’export (beaucoup d’éditeurs limitent les formats d’export ou suppriment les données rapidement après résiliation). 

## _**1.2.5 La couche géopolitique**_ 

La souveraineté numérique est devenue un enjeu géopolitique de premier ordre. L’Europe constate que 97% de son infrastructure cloud est détenue par des acteurs non-européens. Des initiatives comme Gaia-X, le European Cloud Code of Conduct, le EU Data Act (applicable depuis septembre 2025), NIS2 et DORA témoignent d’une volonté politique croissante de reprendre le contrôle. Selon une enquête IDC de juin 2025, 45% des organisations et 56% des digital natives citent la souveraineté des données comme leur préoccupation principale pour 2026. 

4 

Souveraineté des données dans les SaaS 

## **2. Cartographie des contraintes** 

Toute solution au problème de souveraineté dans les SaaS doit naviguer dans un espace de contraintes multidimensionnel. Ces contraintes sont souvent en tension les unes avec les autres, ce qui explique qu’aucune solution triviale n’existe. 

|**Dimension**|**Contrainte**|**Tension avec**|
|---|---|---|
|Juridique|Les données doivent rester sous la<br>juridiction choisie par le client|Performance, coût (data centers locaux<br>rares)|
|Technique|Les données ne doivent pas être<br>accessibles en clair par l’éditeur|Fonctionnalité (traitement nécessaire)|
|Sécurité|Résilience aux pannes, aux attaques,<br>aux pertes|Simplicité (réplication et chiffrement =<br>complexité)|
|Économique|Coût inférieur ou égal au SaaS<br>classique|Souveraineté (infra dédiée coûte plus<br>cher)|
|Expérience|L’utilisateur ne doit percevoir aucune<br>différence|Complexité architecturale sous-jacente|
|Propriété<br>intellectuelle|L’éditeur doit protéger son code source|Déploiement local (code chez le client)|
|Opérationnel|Mises à jour, support, debugging<br>simples|Accès restreint aux données et à<br>l’environnement|
|Scalabilité|De 5 à 5 000 utilisateurs|Uniformité architecturale|
|Conformité|Multi-réglementation (RGPD, HIPAA,<br>etc.)|Universalité de la solution|
|Interopérabilit<br>é|Le client doit pouvoir changer d’éditeur|Lock-in nécessaire à la viabilité de<br>l’éditeur|



La difficulté fondamentale est que ces contraintes forment un système non-linéaire : optimiser une dimension dégrade souvent une autre. La conception d’une solution viable exige de définir explicitement les compromis acceptés. 

5 

Souveraineté des données dans les SaaS 

## **3. État de l’art : paradigmes existants** 

Plusieurs approches tentent de répondre à la problématique, chacune avec des compromis différents. Aucune ne résout le problème complètement. 

## **3.1 Le SaaS avec résidence locale** 

L’éditeur déploie ses serveurs dans le pays du client (régions cloud locales). C’est l’approche de Microsoft, Google, AWS avec leurs régions européennes et souveraines. 

_Limites : résout la résidence mais pas la souveraineté. L’éditeur et sa juridiction d’origine conservent l’accès technique et légal aux données. Le CLOUD Act s’applique toujours._ 

## **3.2 Le cloud souverain** 

Des acteurs locaux (OVHcloud, Scaleway, NumSpot, S3NS en Europe) proposent des infrastructures opérées sous juridiction locale, parfois avec qualification de sécurité nationale (SecNumCloud en France). 

_Limites : l’infrastructure est souveraine mais le logiciel SaaS qui tourne dessus ne l’est pas forcément. Si le SaaS est un produit américain déployé sur un cloud souverain, la question de l’accès au code et aux mises à jour reste entière._ 

## **3.3 Le on-premise / self-hosted** 

Le client installe et opère le logiciel sur sa propre infrastructure. Souveraineté totale. 

_Limites : coût d’exploitation élevé, retard chronique sur les mises à jour, variabilité des environnements, charge de maintenance, besoin de compétences internes. C’est le modèle que le SaaS a précisément été conçu pour remplacer._ 

## **3.4 Le BYOC (Bring Your Own Cloud)** 

L’éditeur sépare son plan de contrôle (gestion, orchestration, UI) de son plan de données (compute,  stockage).  Le  plan  de  données  tourne  dans  le  VPC  du  client.  Adopté  par Databricks, Redpanda, ClickHouse, Confluent. 

_Limites : le plan de contrôle reste chez l’éditeur et peut avoir un accès résiduel aux données. Le modèle est mature pour l’infrastructure data mais quasi inexistant pour les SaaS applicatifs métier. Le client doit disposer d’un compte cloud._ 

## **3.5 Le chiffrement côté client** 

Le client chiffre ses données avant de les envoyer au SaaS. L’éditeur ne voit que du bruit. C’est le modèle de Proton Mail, Tresorit, SpiderOak. 

_Limites : l’éditeur ne peut plus traiter les données (pas de recherche, pas de tri, pas d’agrégation). Ce modèle fonctionne pour le stockage et la messagerie mais s’effondre pour les applications métier qui nécessitent du traitement côté serveur._ 

6 

Souveraineté des données dans les SaaS 

## **3.6 Le Code-to-Data** 

Au lieu d’envoyer les données vers le code, on envoie le code vers les données. L’éditeur livre une image exécutable (conteneur) qui tourne entièrement chez le client, à côté de ses données. Les données ne sortent jamais. 

_Limites : la protection de la propriété intellectuelle de l’éditeur. Le client doit disposer d’une capacité de calcul minimale. Les mises à jour et le support deviennent plus complexes._ 

## **3.7 La décentralisation totale (Solid, Web3)** 

Le projet Solid de Tim Berners-Lee propose de découpler radicalement les données des applications. Chaque utilisateur stocke ses données dans un pod (Personal Online Data Store) qu’il contrôle. Les applications demandent l’autorisation d’accéder aux données via des protocoles standardisés. Le stockage décentralisé blockchain (IPFS, Filecoin, Arweave, Storj) explore des pistes similaires. 

_Limites  :  maturité  technologique  insuffisante  pour  les  applications  métier  complexes. Performance et latence. Adoption quasi nulle en entreprise. Complexité d’intégration._ 

7 

Souveraineté des données dans les SaaS 

## **4. Technologies habilitantes** 

Au-delà  des  paradigmes  architecturaux,  plusieurs  technologies  émergentes  peuvent contribuer à une solution. Chacune adresse un aspect spécifique du problème. 

## **4.1 Chiffrement homomorphe (FHE)** 

Le Fully Homomorphic Encryption permet d’effectuer des calculs sur des données chiffrées sans les déchiffrer. Le résultat, une fois déchiffré par le client, est identique au résultat qu’on aurait obtenu en clair. C’est la technologie qui pourrait théoriquement résoudre le problème de manière définitive : l’éditeur traite les données sans jamais les voir. 

**Maturité :** les performances restent un frein majeur. Un calcul qui prend 1 seconde en clair peut prendre plusieurs jours en FHE. Des progrès considérables sont en cours (Zama, Microsoft SEAL, IBM HElib, accélérateurs hardware FPGA). Viabilité commerciale partielle estimée d’ici 2 à 3 ans. 

## **4.2 Confidential Computing (TEE)** 

Les Trusted Execution Environments (Intel SGX/TDX, AMD SEV-SNP, ARM CCA, AWS Nitro Enclaves) créent des zones mémoire isolées où même l’administrateur root ou l’hébergeur ne peut pas lire les données en cours de traitement. Un mécanisme d’attestation cryptographique permet de vérifier que le code qui tourne dans l’enclave est bien celui attendu. Disponible en production  aujourd’hui,  vulnérable  à  certaines  attaques  par  canaux  auxiliaires  mais  la technologie progresse. 

## **4.3 Calcul multi-parties sécurisé (MPC)** 

Le Secure Multi-Party Computation permet à plusieurs parties de calculer conjointement une fonction sur leurs entrées respectives sans qu’aucune partie ne voie les entrées des autres. Potentiel  remarquable  pour  les  SaaS  :  un  éditeur  et  un  client  pourraient  exécuter conjointement un traitement où l’éditeur fournit la logique sans voir les données, et le client fournit les données sans voir la logique. 

## **4.4 Preuves à connaissance nulle (ZKP)** 

Les Zero-Knowledge Proofs permettent de prouver la véracité d’une affirmation sans révéler l’information sous-jacente. Potentiel pour les SaaS : valider des règles métier, vérifier des droits, auditer des traitements, sans exposer les données. Pertinent aussi pour la vérification d’intégrité du code dans un scénario code-to-data. 

## **4.5 Confidentialité différentielle** 

Technique statistique qui permet d’extraire des informations utiles d’un ensemble de données tout en garantissant mathématiquement qu’aucune donnée individuelle ne peut être réidentifiée. Utilisée par Apple et Google pour la télémétrie. Pourrait permettre à l’éditeur de collecter des analytics sans compromettre la souveraineté. 

## **4.6 Identité auto-souveraine (SSI / DID)** 

Le Self-Sovereign Identity et les Decentralized Identifiers (W3C DID) permettent aux individus et  organisations  de  contrôler  leur  identité  numérique  sans  dépendre  d’un  fournisseur 

8 

Souveraineté des données dans les SaaS 

centralisé. Les Verifiable Credentials permettent de partager des attestations vérifiables sans révéler les données sources. Pertinent pour découpler l’authentification du système de l’éditeur. 

## **4.7 Approche Local-First** 

Le mouvement local-first software prône des applications qui fonctionnent principalement localement, avec synchronisation optionnelle. Les CRDTs (Conflict-free Replicated Data Types) permettent la collaboration temps réel sans serveur central. Utilisé par Figma et Linear. Le SaaS devient un service de synchronisation et de mise à jour, pas un hébergeur de données. 

9 

Souveraineté des données dans les SaaS 

## **5. Axes de réflexion** 

Cette section explore systématiquement toutes les pistes identifiées, y compris les moins évidentes. L’objectif n’est pas de juger leur faisabilité immédiate mais d’ouvrir le champ des possibles. 

## **5.1 Axes architecturaux** 

## _**5.1.1 La séparation des plans**_ 

Jusqu’où peut-on découper un SaaS en plans indépendants ? Le BYOC distingue control plane et data plane. Peut-on aller plus loin avec un plan logique (règles métier), un plan de calcul (exécution), un plan de stockage (données au repos), un plan de présentation (UI), chacun pouvant être hébergé différemment ? Quels plans le client doit-il impérativement contrôler ? Quels plans peut-il déléguer sans risque ? 

## _**5.1.2 Le code éphémère**_ 

Et si le logiciel n’existait que temporairement chez le client ? L’éditeur envoie une image signée, chiffrée, avec une durée de vie limitée (bail). L’image s’auto-détruit après expiration. Le client ne peut ni la lire, ni la copier, ni la modifier. Questions ouvertes : quelle durée de bail ? Quel mécanisme d’auto-destruction ? Comment gérer les interruptions réseau pendant le bail ? 

## _**5.1.3 Le SaaS comme protocole**_ 

Et si le SaaS n’était pas une application hébergée mais un protocole ? L’éditeur publie un protocole ouvert (API, schéma de données, règles métier), fournit une implémentation de référence, et monétise des services périphériques (support, modules premium, certification). Le client choisit où et comment exécuter le protocole. C’est le modèle de Matrix/Element pour la messagerie. 

## **5.2 Axes cryptographiques** 

## _**5.2.1 Le traitement aveugle**_ 

Comment un éditeur peut-il fournir un service utile sans jamais voir les données ? Le FHE est la  réponse  théorique  parfaite  mais  impraticable  aujourd’hui.  Existe-t-il  des  compromis intermédiaires ? Un chiffrement partiellement homomorphe qui ne supporte que l’addition, combiné avec du calcul local pour le reste ? Des opérations pré-agrégées en clair côté client, dont seuls les résultats chiffrés sont envoyés à l’éditeur ? 

## _**5.2.2 La vérification sans révélation**_ 

Les ZKP ouvrent un espace de conception où l’éditeur peut vérifier que le client respecte les règles métier sans voir ses données, et où le client peut vérifier que l’éditeur exécute le bon code sans voir le code source. Cette double vérification sans révélation est-elle atteignable en pratique ? 

## _**5.2.3 La clé comme levier de pouvoir**_ 

Qui détient la clé de chiffrement détient le pouvoir réel. Les modèles BYOK, HYOK et CMK explorent différentes distributions de ce pouvoir. Question radicale : peut-on concevoir un système où la clé est partagée entre client et éditeur (secret sharing de Shamir) de sorte qu’aucun des deux ne puisse agir seul ? 

10 

Souveraineté des données dans les SaaS 

## **5.3 Axes économiques** 

## _**5.3.1 Qui paie l’infrastructure ?**_ 

Dans le SaaS classique, le coût d’infrastructure est mutualisé et inclus dans l’abonnement. Si les données restent chez le client, qui paie le stockage, le compute, la bande passante ? Comment tarifer un service où l’éditeur ne connaît même pas le volume de données du client ? 

## _**5.3.2 La souveraineté comme argument commercial**_ 

La souveraineté peut-elle justifier un premium de prix ? Ou faut-il démontrer une réduction de coût (pas d’egress fees, pas de coût de migration) ? Quel est le willingness-to-pay des entreprises pour la souveraineté ? 

## _**5.3.3 Le modèle économique sans données**_ 

Un éditeur SaaS sans données client perd un levier majeur : les analytics produit, les benchmarks agrégés, la personnalisation algorithmique. Comment compenser cette perte ? La differential privacy suffit-elle ? 

## **5.4 Axes organisationnels** 

## _**5.4.1 Le support sans accès**_ 

Comment diagnostiquer un bug quand on ne peut pas voir les données du client ? Modèles possibles : accès temporaire autorisé par token à durée limitée, données de test synthétiques générées  localement,  logs  structurés  sans  données  métier,  session  de  débogage collaborative où le client montre son écran. 

## _**5.4.2 La confiance asymétrique**_ 

Le problème fondamental est un problème de confiance. Le client ne fait pas confiance à l’éditeur pour ses données. L’éditeur ne fait pas confiance au client pour son code. Peut-on formaliser cette défiance mutuelle en un protocole mathématique ? Le MPC et les TEE sont des tentatives dans cette direction. 

## _**5.4.3 La gouvernance des données partagées**_ 

Certaines données dans un SaaS sont intrinsèquement partagées : catalogues communs, référentiels sectoriels, modèles ML entraînés sur des données agrégées. Comment gérer la frontière entre données souveraines (propres au client) et données communes (valeur partagée) ? 

## **5.5 Axes spéculatifs et prospectifs** 

## _**5.5.1 Le SaaS post-quantique**_ 

L’informatique quantique rendra obsolètes la plupart des algorithmes de chiffrement actuels. Toute solution de souveraineté fondée sur le chiffrement doit anticiper la migration vers des algorithmes post-quantiques. Les données chiffrées aujourd’hui avec des clés classiques pourront être déchiffrées demain (harvest now, decrypt later). 

## _**5.5.2 L’IA souveraine**_ 

Les SaaS intègrent de plus en plus d’IA. Ces modèles nécessitent l’accès aux données pour fonctionner. Comment fournir des fonctionnalités d’IA dans un SaaS souverain ? Pistes : 

11 

Souveraineté des données dans les SaaS 

modèles embarqués dans l’image locale, fine-tuning local, inference en FHE ou en TEE, federated learning. 

## _**5.5.3 La réciprocité de la transparence**_ 

Et si la solution n’était pas technique mais contractuelle et sociale ? Un éditeur qui s’engage à l’open-source intégral de son code, à des audits publics réguliers, et à un fonctionnement sous une fondation indépendante, crée une forme de souveraineté par la transparence. 

## _**5.5.4 Le marché des données intentionnel**_ 

Et si le client choisissait activement de partager certaines données avec l’éditeur, en échange d’une réduction de prix ou de fonctionnalités améliorées ? Un modèle où la souveraineté par défaut est totale, et le partage est un acte délibéré, réversible, et rémunéré. 

## _**5.5.5 La dématérialisation du logiciel**_ 

Et si le logiciel n’était plus un artefact mais un flux continu ? L’éditeur stream le logiciel en temps réel vers le client (comme un jeu en cloud gaming), le client ne reçoit que des pixels. Aucun code, aucune donnée ne transite dans un sens exploitable. Les données restent chez le client via un agent local que le stream interroge. Cette approche hybride streaming plus agent local mérite exploration. 

12 

Souveraineté des données dans les SaaS 

## **6. Glossaire des concepts mobilisables** 

Cette section liste les concepts techniques, juridiques et philosophiques qui peuvent alimenter la réflexion, regroupés par domaine. 

## **6.1 Cryptographie et sécurité** 

- **FHE  (Fully  Homomorphic  Encryption)  —** Calcul  sur  données  chiffrées  sans déchiffrement. 

- **PHE (Partially Homomorphic Encryption) —** Chiffrement supportant un seul type d’opération. 

- **TEE (Trusted Execution Environment) —** Enclave matérielle d’exécution isolée (SGX, SEV, TDX, Nitro). 

- **MPC (Secure Multi-Party Computation) —** Calcul conjoint sans révélation des entrées. 

- **ZKP (Zero-Knowledge Proof) —** Preuve de véracité sans révélation de l’information. 

- **Secret Sharing (Shamir) —** Division d’un secret en N fragments dont K sont nécessaires pour reconstituer. 

- **Attestation —** Vérification cryptographique que le code dans une enclave est celui attendu. 

- **mTLS —** Authentification mutuelle par certificats entre deux parties. 

- **CSE (Client-Side Encryption) —** Chiffrement côté client avant envoi. 

- **BYOK / HYOK / CMK —** Gestion de clés par le client. 

- **Tokenisation —** Remplacement de données sensibles par des jetons non réversibles. 

- **Post-Quantum Cryptography —** Algorithmes résistants aux ordinateurs quantiques. 

## **6.2 Architecture distribuée** 

- **Control Plane / Data Plane —** Séparation gestion et données. 

- **Data Plane Atomicity —** Le data plane fonctionne sans dépendance au control plane (Redpanda). 

- **BYOC —** Déploiement dans le VPC du client. 

- **Edge Computing —** Traitement au plus près des données. 

- **CRDTs —** Structures de données fusionnables sans conflit. 

- **Event Sourcing / CQRS —** Séparation lectures/écritures, journal d’événements. 

- **Streaming Replication —** Réplication continue via WAL (actif/passif). 

- **Federated Learning —** Entraînement de modèles sans centraliser les données. 

## **6.3 Identité et gouvernance** 

- **SSI (Self-Sovereign Identity) —** Identité contrôlée par l’individu. 

- **DID (Decentralized Identifier) —** Standard W3C d’identifiants décentralisés. 

- **Verifiable Credentials —** Attestations vérifiables sans révélation de la source. 

- **Differential Privacy —** Garantie mathématique de non-ré-identification. 

13 

Souveraineté des données dans les SaaS 

- **Data Classification —** Catégorisation par niveau de sensibilité. 

- **Data Lineage —** Traçabilité de l’origine et des transformations. 

## **6.4 Décentralisation et stockage** 

- **Solid / POD —** Personal Online Data Store (Tim Berners-Lee / Inrupt). 

- **IPFS —** Système de fichiers distribué content-addressed. 

- **Tahoe-LAFS —** Stockage distribué chiffré avec erasure coding. 

- **Filecoin / Arweave / Storj —** Stockage décentralisé incitatif. 

## **6.5 Modèles de déploiement** 

- **OCI (Open Container Initiative) —** Standard d’images conteneurs. 

- **Cosign / Sigstore —** Signature et vérification d’artefacts logiciels. 

- **OCI Encrypted Images —** Images conteneur chiffrées, déchiffrées au runtime. 

- **gVisor / Kata Containers —** Runtimes conteneur durcis. 

- **WebAssembly (WASM) —** Exécution isolée et portable, potentiellement dans le navigateur. 

- **Cloud Gaming model —** Streaming de pixels, zéro code chez le client. 

## **6.6 Réglementaire** 

- **RGPD —** Règlement général sur la protection des données (UE). 

- **EU Data Act —** Droit d’accès aux données et portabilité (applicable sept. 2025). 

- **CLOUD Act —** Accès extraterritorial américain aux données. 

- **NIS2 / DORA —** Sécurité des infrastructures critiques (UE). 

- **Schrems II —** Arrêt invalidant le Privacy Shield UE-US. 

- **SecNumCloud —** Qualification de sécurité cloud française. 

- **Gaia-X —** Initiative européenne de cloud fédéré souverain. 

14 

Souveraineté des données dans les SaaS 

## **7. Questions ouvertes** 

Les questions suivantes n’ont pas de réponse unique. Elles visent à structurer le processus de conception en forçant à expliciter les choix et les compromis. 

## **Questions fondamentales** 

- Quelles données sont véritablement sensibles et lesquelles peuvent être déléguées sans risque ? Toutes les données méritent-elles le même niveau de souveraineté ? 

- La souveraineté est-elle un absolu binaire ou un spectre ? Peut-on définir des niveaux de souveraineté avec des compromis différents ? 

- Le problème est-il fondamentalement technique ou fondamentalement juridique ? La technologie peut-elle compenser un défaut de cadre légal, et inversement ? 

- Faut-il viser une solution universelle ou spécifique à un secteur (santé, finance, administration) ? 

## **Questions de conception** 

- Où placer la frontière entre ce que l’éditeur contrôle et ce que le client contrôle ? Cette frontière est-elle statique ou négociable par tenant ? 

- Comment  gérer  les  migrations  de  schéma  quand  chaque  tenant  est  une  base indépendante ? 

- Comment un éditeur peut-il améliorer son produit sans analytics sur les données d’usage réelles ? 

- Comment facturer un service quand on ne connaît pas le volume de données du client ? 

- Comment protéger le code de l’éditeur dans un modèle code-to-data ? 

## **Questions prospectives** 

- Le FHE sera-t-il suffisamment performant dans 5 ans pour rendre ce problème obsolète ? 

- L’informatique quantique peut-elle accélérer le FHE plutôt que de le menacer ? 

- Les régulations vont-elles imposer la souveraineté de fait, rendant le SaaS classique illégal pour certains usages ? 

- Le mouvement local-first peut-il rendre le concept même de SaaS centralisé obsolète ? 

- Les agents IA autonomes qui traitent les données du client vont-ils créer une nouvelle catégorie de risque de souveraineté non encore adressée ? 

_Souveraineté des données dans les SaaS — Document de réflexion — Mars 2026_ 

15 

