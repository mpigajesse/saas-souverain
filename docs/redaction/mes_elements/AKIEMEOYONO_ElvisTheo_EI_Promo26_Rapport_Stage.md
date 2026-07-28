282 Route de l'Oasis 20410 Casablanca 

Résidence Al Amane GH31 Imm. 253 appt 1, Ain Sebaa, Casablanca 

## RAPPORT 

## STAGE ELEVE-INGENIEUR 

Date de début : 12/01/2026 Date de fin : 03/04/2026 

## SUJET : 

## MISE EN PLACE D'UNE SOLUTION WEBMAPPING POUR LE SUIVI DE LA GESTION ET DE LA MAINTENANCE DES ESPACES VERTS DE LA VILLE DE BENGUERIR 

|Tuteur Entreprise|Tuteur EIGSI|Etudiant|
|---|---|---|
|Soumia CHOKRI|Badr-eddineBEN EL MOSTAFA|Elvis-TheoAKIEMEOYONO|
|Directeur Général|Enseignant IABD|Elève-ingénieur|
||+212662 777507|+212 779 635 809|
|chokri.soumaya90@gmail.com|be.benelmostafa@eigsica.ma|et.akieme.26@eigsica.ma|



Année académique : 2025-2026 

## TABLE DES MATIÈRES 

|TABLE DES ILLUSTRATIONS ..................................................................................................... iii|
|---|
|LISTE DES ABREVIATIONS ........................................................................................................ iv|
|RESUME ...................................................................................................................................... I|
|REMERCIEMENTS ...................................................................................................................... II|
|INTRODUCTION GENERALE ....................................................................................................... 1|
|Partie I – Présentation de l’Entreprise et Contexte du stage ................................................. 3|
|I.1.<br>AL BARAA CONSULTING : Présentation et positionnement ..................................... 3|
|I.2.<br>Cadre de travail et organisation de la mission ......................................................... 4|
|I.3.<br>Contexte et analyse du projet .................................................................................... 4|
|I.3.1.<br>Diagnostic de l’existant ....................................................................................... 4|
|I.3.2.<br>Analyse stratégique – SWOT............................................................................... 5|
|I.3.3.<br>Analyse fonctionnelle – bête à cornes ............................................................... 6|
|I.3.4.<br>Analyse fonctionnelle externe – diagramme pieuvre ....................................... 7|
|I.3.5.<br>Analyse FAST et SADT ......................................................................................... 8|
|I.4.<br>Définition de la mission et enjeux ............................................................................. 9|
|I.5.<br>Objectifs SMART de la mission .................................................................................. 9|
|Partie II – Bilan Technique ..................................................................................................... 10|
|II.1.<br>Méthodologie de travail ........................................................................................... 10|
|II.2.<br>Work Breakdown Structure actualisé ..................................................................... 11|
|II.3.<br>Architecture technique et choix technologiques ................................................... 12|
|II.3.1.<br>Architecture globale ......................................................................................... 12|
|II.3.2. Justification des choix technologiques ............................................................... 13|
|II.3.3.<br>Modèle de données.......................................................................................... 14|
|II.4.<br>Développement Backend ........................................................................................ 16|
|II.4.1.<br>Mise en place de la base de données PostGIS .............................................. 16|
|II.4.2.<br>Architecture de l’API REST ............................................................................... 17|
|II.4.3.<br>Tâches asynchrones et automatisations ........................................................ 21|
|II.5.<br>Développement Frontend et applications clientes ............................................... 22|
|II.5.1.<br>Interface Webmapping – Back-office .............................................................. 23|
|II.5.2.<br>Interface opérateur terrain – Web responsive ............................................... 26|
|II.5.3.<br>Portail client ...................................................................................................... 27|
|II.5.4.<br>Défis techniques transversaux ........................................................................ 27|
|II.6.<br>Tests, validation et recette ...................................................................................... 28|
|II.6.1.<br>Tests automatisés backend............................................................................. 28|



i - 

|II.6.2.<br>Recette client et suivi des anomalies ............................................................. 30|
|---|
|II.6.3.<br>Bilan qualité ...................................................................................................... 31|
|II.7.<br>Déploiement ............................................................................................................. 32|
|II.8.<br>Conclusion technique .............................................................................................. 33|
|Partie III : Bilan de l’expérience ............................................................................................. 35|
|CONCLUSION GÉNÉRALE ....................................................................................................... 37|



ii - 

## TABLE DES ILLUSTRATIONS 

|Figure 1: Diagramme bête à cornes ........................................................................................ 6|Figure 1: Diagramme bête à cornes ........................................................................................ 6|
|---|---|
|Figure 2: Diagramme pieuvre ................................................................................................... 7|Figure 2: Diagramme pieuvre ................................................................................................... 7|
|Figure 3: Diagramme FAST ....................................................................................................... 8|Figure 3: Diagramme FAST ....................................................................................................... 8|
|Figure 4: Actigramme SADT niveau A-0 ................................................................................... 8|Figure 4: Actigramme SADT niveau A-0 ................................................................................... 8|
|Figure 5: Architecture technique de|Figure 5: Architecture technique deGreenSIG..................................................................... 12|
|Figure 6: Architecture de l'API REST GreenSIG .................................................................... 17|Figure 6: Architecture de l'API REST GreenSIG .................................................................... 17|
|Figure 7: Cycle de vie d'une tâche GreenSIG ....................................................................... 19|Figure 7: Cycle de vie d'une tâche GreenSIG ....................................................................... 19|
|Figure 8: Formule de calcul de la charge estimée d'une intervention ............................... 19|Figure 8: Formule de calcul de la charge estimée d'une intervention ............................... 19|
|Figure 9: Workflow des réclamations .................................................................................... 20|Figure 9: Workflow des réclamations .................................................................................... 20|
|Figure 10: Timeline auto-clôture des réclamations ............................................................. 22|Figure 10: Timeline auto-clôture des réclamations ............................................................. 22|
|Figure 11: Vue cartographique satellite de GreenSIG ......................................................... 24|Figure 11: Vue cartographique satellite de GreenSIG ......................................................... 24|
|Figure 12: Calendrier de planification mensuel de GreenSIG ............................................ 25|Figure 12: Calendrier de planification mensuel de GreenSIG ............................................ 25|
|Figure 13: Tableau de bord des indicateurs de performance (KPI) ................................... 25|Figure 13: Tableau de bord des indicateurs de performance (KPI) ................................... 25|
|Figure 14: Interface de suivi des tâches sur mobile ............................................................ 26|Figure 14: Interface de suivi des tâches sur mobile ............................................................ 26|
|Figure 15: Fiche de suivi d'une réclamation ........................................................................ 27|Figure 15: Fiche de suivi d'une réclamation ........................................................................ 27|
|Figure 16: Tableau synthétique de la couverture des tests ................................................ 30|Figure 16: Tableau synthétique de la couverture des tests ................................................ 30|



iii - 

## LISTE DES ABREVIATIONS 

|Abréviation|Signification|
|---|---|
|API|Application Programming Interface|
|ASGI|Asynchronous ServerGatewayInterface|
|CI/CD|Continuous Integration / Continuous Deployment|
|CSRF|Cross-Site Request Forgery|
|CSS|Cascading Style Sheets|
|DOM|Document Object Model|
|EIGSI|École d'Ingénieurs enGénie des SystèmesIndustriels|
|FAST|Function Analysis System Technique|
|GANTT|Diagramme de planificationtemporelle (dunomdeHenry Gantt)|
|HTTPS|HyperText Transfer Protocol Secure|
|IABD|IntelligenceArtificielle etBigData|
|JSON|JavaScript Object Notation|
|JWT|JSON WebToken|
|KML|Keyhole Markup Language|
|KPI|Key Performance Indicator|
|MAD|Dirham marocain|
|ORM|Object-Relational Mapping|
|OS|Operating System|
|PDF|Portable Document Format|
|PostGIS|Extensiongéospatiale dePostgreSQL|
|PWA|Progressive Web Application|
|RACI|Responsible,Accountable, Consulted,Informed|
|RBAC|Role-Based Access Control|
|RBS|Risk BreakdownStructure|
|REST|Representational State Transfer|
|RH|Ressources Humaines|
|SADT|Structured Analysis and Design Technique|
|SARL|Société à Responsabilité Limitée|
|SIG|Système d'InformationGéographique|
|SMART|Specific, Measurable, Achievable, Realistic, Time-bound|
|SQL|Structured QueryLanguage|
|SRID|Spatial Reference Identifier|
|SWOT|Strengths,Weaknesses, Opportunities,Threats|
|UX|User Experience|
|WBS|Work BreakdownStructure|
|WGS|World Geodetic System|
|WSL|Windows Subsystem for Linux|
|XSS|Cross-Site Scripting|



iv . 

## RESUME 

Ce rapport présente les travaux réalisés lors d'un stage élève-ingénieur de trois mois au sein d'AL BARAA CONSULTING, portant sur la mise en place de GreenSIG , un Système d'Information Géographique dédié à la gestion des espaces verts de la ville de Benguerir . Face à une gestion jusqu'alors empirique, la mission consistait à concevoir, développer et déployer une plateforme Webmapping complète intégrant la visualisation cartographique du patrimoine végétal et hydraulique, la planification des interventions de maintenance et le traitement des réclamations. La solution livrée repose sur une architecture conteneurisée (Docker) articulant un backend Django/GeoDjango, un frontend React/OpenLayers, une base de données PostgreSQL/PostGIS et des services asynchrones Celery/Redis. Elle gère vingt-huit entités de données, expose plus de cent endpoints API REST et a été validée par cent douze tests automatisés et quatre-vingt-dixsept remarques de recette traitées avec le client. Le rapport détaille les choix architecturaux, les défis techniques rencontrés et propose une réflexion sur la notion de qualité dans le développement logiciel. 

Mots-clés : SIG, Webmapping, espaces verts, Django, PostGIS, OpenLayers, React, gestion de maintenance, ville verte, Benguerir. 

I - 

## REMERCIEMENTS 

Je tiens à adresser mes sincères remerciements à toutes les personnes qui ont contribué, directement ou indirectement, à la réussite de ce stage et à l'aboutissement de ce rapport. 

Je remercie tout d'abord Mme Soumia CHOKRI , Directrice Générale d'AL BARAA CONSULTING et tutrice entreprise, pour la confiance qu'elle m'a accordée en me confiant la responsabilité complète du projet GreenSIG . Son encadrement, ses orientations stratégiques et sa disponibilité tout au long de la mission ont été déterminants dans la conduite du projet. 

Je remercie M Badr-eddine BEN EL MOSTAFA , mon tuteur pédagogique à l'EIGSI , pour son suivi régulier, ses conseils méthodologiques et sa capacité à maintenir l'alignement entre l'expérience terrain et les exigences de la formation. 

Je tiens à exprimer ma reconnaissance particulière à Mlle Safae BOUZEKRAOUI , contact cliente du projet GreenSIG . Son implication rigoureuse dans la phase de recette, testant l'application de fond en comble, identifiant et documentant méthodiquement chaque anomalie et chaque piste d'amélioration, a été un levier essentiel de la qualité du produit livré. Sa rigueur et son exigence, loin d'être des obstacles, ont constitué une contribution précieuse à la réussite du projet. 

Enfin, je remercie l'ensemble du corps enseignant de l'EIGSI Casablanca pour la formation solide qui m'a permis d'aborder ce stage avec les compétences nécessaires à sa réalisation. 

II - 

## INTRODUCTION GENERALE 

Lancée en 2012, la Ville Verte Mohamed VI de Benguerir incarne l’une des ambitions urbaines les plus emblématiques du Maroc contemporain : bâtir, sur plus de 1 000 hectares, un écosystème urbain durable autour d’une coulée verte de 80 hectares, de plus de 50 000 arbres et d’un ratio cible de 20 m[2] d’espaces verts par habitant. Ces chiffres, aussi prometteurs soient-ils, soulèvent une question rarement posée : comment gère-t-on, au quotidien, un patrimoine végétal et hydraulique de cette envergure ? Car derrière la vitrine du projet, il y a la réalité opérationnelle – celle des équipes de terrain, des plannings d’intervention, des inventaires d’actifs et des arbitrages budgétaires. 

Cette question s’inscrit dans un mouvement plus large. Avec 63 % de sa population résidant en zone urbaine en 2025 et une projection de 81 % à l’horizon 2050, le Maroc traverse une transformation urbaine profonde qui met sous pression ses infrastructures, ses ressources en eau et ses espaces naturels. Face à ces enjeux, les collectivités et les opérateurs privés du secteur sont confrontés à un impératif : passer d’une gestion empirique, souvent fondée sur le papier et les tableurs, à une gestion numérique capable d’assurer un suivi fiable des actifs, une planification efficace des interventions et un reporting structuré pour les parties prenantes. La transformation numérique de la gestion urbaine n’est plus un luxe technologique ; elle devient une condition de viabilité opérationnelle. 

C’est précisément dans ce contexte que s’inscrit la mission qui m’a été confiée par AL BARAA CONSULTING : concevoir et développer de A à Z GreenSIG, un Système d’Information Géographique ( SIG ) dédié à la gestion numérique des infrastructures végétales et hydrauliques de la ville de Benguerir . Ce stage, qui s’étend du 12 janvier au 03 avril 2026, m’a été confié avec une responsabilité totale sur l’ensemble du cycle de développement – de l’analyse des besoins au déploiement en production – et constitue ma première expérience professionnelle significative dans le cadre de ma formation d’ingénieur généraliste option Intelligence Artificielle et Big Data à l’EIGSI . 

Ce projet soulève une problématique centrale : dans quelle mesure un Système d’Information Géographique peut-il transformer la gestion des espaces verts urbains en passant d’un suivi manuel à un pilotage numérique intégré ? Cette interrogation en appelle d’autres, plus spécifiques : quels choix architecturaux et technologiques permettent de répondre aux contraintes d’un déploiement réel ? Comment garantir la fiabilité et la qualité 

1 - 

d’un système développé dans un cadre temporel contraint ? Et quels enseignements méthodologiques peut-on tirer d’un premier cycle complet de développement logiciel en situation professionnelle ? 

Pour répondre à ces questions, le présent rapport s’articule en trois temps. La première partie pose le cadre de la mission en présentant l’entreprise d’accueil, le contexte sectoriel et les enjeux auxquels GreenSIG entend répondre. La deuxième partie, la plus substantielle, propose un bilan technique critique du projet : architecture, choix technologiques, défis rencontrés et solutions mises en œuvre. La troisième partie, enfin, prend du recul pour interroger la notion de qualité dans le développement logiciel, thème transversal directement ancré dans le vécu de ce stage. 

2 - 

## Partie I – Présentation de l’Entreprise et Contexte du stage 

Avant d’aborder les dimensions techniques du projet, il convient de poser le cadre dans lequel cette mission s’est déroulée. Cette première partie présente l’entreprise d’accueil et son positionnement, les conditions d’organisation du stage, puis le diagnostic de l’existant et l’analyse fonctionnelle qui ont fondé les choix de conception de GreenSIG . 

## I.1. AL BARAA CONSULTING : Présentation et positionnement 

AL BARAA CONSULTING est un cabinet de conseil et d’ingénierie numérique fondé en mars 2017, constitué sous la forme juridique de Société à Responsabilité Limitée à Associé Unique (SARL AU) avec un capital de 100 000 MAD. Basé à Casablanca (Résidence Al Amane GH31, Ain Sebaa), le cabinet est dirigé par Soumia CHOKRI, Directrice Générale, et intervient sur des problématiques complexes alliant développement logiciel, architecture de systèmes d’information et transformation numérique pour une clientèle publique et privée. 

La structure à taille humaine du cabinet constitue à la fois un atout et un défi. Elle favorise la réactivité, la communication directe et une forte responsabilisation des collaborateurs, mais implique également une dépendance marquée aux compétences individuelles mobilisées sur chaque projet. 

Le positionnement d’AL BARAA CONSULTING se distingue par une approche intégrée : chaque mission démarre par une analyse du contexte client avant de produire une solution sur mesure. Cette démarche est particulièrement pertinente pour les projets SIG , où les spécificités métier et géographiques rendent les solutions génériques inadaptées. Le projet GreenSIG illustre parfaitement ce positionnement : une plateforme Webmapping entièrement conçue et développée pour répondre aux besoins précis d’un acteur de la gestion des espaces verts de Benguerir . 

Pour le cabinet, ce projet revêt un enjeu stratégique majeur. Il constitue une première référence concrète dans le domaine de la géomatique appliquée à la gestion urbaine – un segment en forte croissance au Maroc , porté par la dynamique de digitalisation des collectivités territoriales. La réussite de GreenSIG offre au cabinet un produit potentiellement duplicable à d’autres villes vertes et à d’autres opérateurs du secteur, enrichissant ainsi son portefeuille de solutions et renforçant son positionnement concurrentiel. 

3 - 

## I.2. Cadre de travail et organisation de la mission 

Dès le premier jour, j’ai été positionné comme développeur unique sur le projet GreenSIG , sous la supervision directe de Soumia CHOKRI . Cette configuration, exigeante, s’est révélée très formatrice : elle m’a confronté à la réalité d’un projet de production avec toutes ses contraintes de responsabilité, de prise de décision et de livraison tout en développant des compétences en communication et en gestion autonome du temps. 

Mon encadrement pédagogique a été assuré par Badr-eddine BEN EL MOSTAFA ( EIGSI ), avec qui des points de suivi ont permis de maintenir l’alignement entre l’expérience terrain et les objectifs de formation. Côté entreprise, la gestion de projet était pilotée via Jira, qui a servi de référentiel pour le suivi des tâches, la traçabilité des décisions et le respect des jalons. 

Sur le plan des ressources techniques, j’ai disposé d’un poste de travail Windows 10 avec Visual Studio Code comme environnement de développement principal. L’environnement serveur reposait sur PostgreSQL/PostGIS pour le stockage des données géospatiales, Redis pour la gestion du cache et des files d’attente, et Docker sous WSL pour la conteneurisation des services. Le déploiement en production a été réalisé sur un serveur on-premise du client, avec l’acquisition d’un nom de domaine et mise en place d’un tunnel Cloudflare pour l’accès sécurisé. Les données cartographiques initiales ont été fournies par le client sous forme de fichiers GeoJSON. 

## I.3. Contexte et analyse du projet 

- I.3.1. Diagnostic de l’existant 

Avant GreenSIG , le client gérait ses infrastructures vertes avec des outils non spécialisés : tableurs Excel pour le suivi des interventions, documents papier pour les réclamations, aucun outil cartographique centralisé. Trois dysfonctionnements critiques ont été identifiés : 

- Perte d’information : l’absence d’outil centralisé entraine une déperdition de l’historique des interventions (taille, arrosage, traitement) ; 

- Manque de visibilité : l’absence de remontée d’information en temps réel nuit à la réactivité et à la transparence vis-à-vis des citoyens ; 

4 - 

- Inefficacité opérationnelle : la dispersion de l’information (papiers volants, connaissances orales) empêche une prise de décision fondée sur des données fiables. 

Ces lacunes avaient des conséquences directes sur l’efficacité opérationnelle, la qualité de service et la capacité du client à valoriser son action auprès des autorités municipales. L’enjeu du projet était donc double : d’une part, numériser et centraliser l’information relative aux infrastructures vertes, et d’autre part, outiller les équipes pour planifier, suivre et analyser leur activité de manière rigoureuse. 

## I.3.2. Analyse stratégique – SWOT 

Avant la phase de conception, une analyse SWOT a été conduite pour évaluer la pertinence de la solution envisagée et anticiper les facteurs de risque. 

|Forces|Faiblesses|
|---|---|
|▪ Maitrise interne des technologies de<br>développement Web et SIG ;<br>▪ Approche modulaire permettant des<br>livraisons incrémentales ;<br>▪ Expertise full-stack centralisée sur le<br>projet ;<br>▪ Pile technologique entièrement<br>open source, réduisant les coûts de<br>licence.|▪ Délais de réalisation contraints (3<br>mois) ;<br>▪ Dépendance à la qualité des données<br>cartographiques fournies par le client ;|
|Opportunités|Menaces|
|▪ Modèle duplicable pour d’autres<br>villes vertes ;<br>▪ Forte demande nationale pour la<br>digitalisation des services<br>publics ;<br>▪ Valorisation du savoir-faire<br>technique d’AL BARAA<br>CONSULTING.|▪ Résistance potentielle au<br>changement des équipes terrain<br>(fracture numérique) ;<br>▪ Instabilité de la couverture réseau<br>mobile sur certaines zones isolées ;<br>▪ Risque de dette technique lié à un<br>développement accéléré sous<br>contrainte temporelle.|



_Tableau 1: Analyse SWOT du projet_ 

5 

- I.3.3. Analyse fonctionnelle – bête à cornes 

L’analyse « bête à cornes » a permis de formaliser la finalité du produit : 

- A qui le produit rend-il service ? – la ville de Benguerir et ses acteurs de la gestion des espaces verts ; 

- Sur quoi agit-il ? – les espaces verts : patrimoine végétal et infrastructures hydrauliques ; 

- Dans quel but ? – permettre la gestion centralisée et le pilotage de la maintenance 

   - des espaces verts via une solution webmapping. 

_Figure 1: Diagramme bête à cornes_ 

Cette analyse a confirmé que GreenSIG est un outil de pilotage opérationnel au service d’une démarche de « ville verte », et non un simple outil de cartographie. 

6 

## I.3.4. Analyse fonctionnelle externe – diagramme pieuvre 

_Figure 2: Diagramme pieuvre_ 

Le diagramme pieuvre a permis d’identifier les fonctions de service liant GreenSIG à son environnement. Le tableau de caractérisation ci-dessous en synthétise les éléments essentiels : 

|F.S|Enoncé|Critères d’appréciation|Niveau et Flexibilité|
|---|---|---|---|
|FP|Permettre la<br>visualisation<br>cartographique des<br>espaces verts.|▪ Fluidité d’affichage ;<br>▪ Précision<br>géographique ;<br>▪ Fraîcheur des<br>données ;|▪ Temps de<br>chargement < 2 sec ;<br>▪ Zoom min : niv. 18<br>(échelle rue) ;<br>▪ Mise à jour : temps<br>réel|
|FC1|Exploiter des fonds<br>de carte<br>géographiques|▪ Source des données<br>▪ Coût d’utilisation<br>▪ Types de vue<br>disponibles|▪ OpenStreetMap ;<br>▪ Open source ;<br>▪ Vue « Plan » et<br>«Satellite»|
|FC2|S’adapter aux<br>terminaux mobiles<br>des opérateurs|▪ Compatibilité OS ;<br>▪ Affichage<br>(Responsive)|▪ Android (Tablette) ;<br>▪ Lisible sur écran > 8<br>pouces|
|FC3|Fonctionner via<br>internet|▪ Disponibilité|▪ Accessibilité 24/7|
|FC4|Garantir la sécurité<br>des données et des<br>accès|▪ Authentification ;<br>▪ Cloisonnement|▪ Protocole JWT ;<br>▪ 3 rôles distincts<br>(Administrateur,<br>Superviseur, Client)|



_Tableau 2: Tableau de caractérisation des fonctions_ 

7 

## I.3.5. Analyse FAST et SADT 

Une analyse FAST (Function Analysis System Technique) a été conduite pour décomposer les fonctions de service en fonctions techniques élémentaires, permettant de 

valider la cohérence entre les besoins fonctionnels et les choix d’implémentation. 

_Figure 3: Diagramme FAST_ 

Et l’analyse SADT au niveau A-0 a permis de modéliser GreenSIG comme une boite noire recevant en entrée les données terrain (géométries, interventions, réclamations) et les accès utilisateurs, et produisant en sortie les visualisations cartographiques, les rapports et les indicateurs de performance KPI. 

_Figure 4: Actigramme SADT niveau A-0_ 

8 

## I.4. Définition de la mission et enjeux 

La mission confiée est de concevoir, développer et déployer une solution Webmapping complète articulée autour de trois volets fonctionnels définis dans le cahier des charges : 

- Back-office : planification intelligente et pilotage par la donnée (module administrateur/superviseur) ; 

- Interface web terrain : accès aux tâches et saisie des rapports d’intervention, accessible sur tablette via navigateur web ; 

- Portail client : suivi des prestations, émission et suivi des réclamations. 

Les enjeux finaux sont de trois ordres : un enjeu de pilotage (rendre la prise de décision fondée sur des données tangibles), un enjeu terrain (adapter la solution aux conditions réelles de travail des opérateurs), et un enjeu de précision géomatique (garantir une localisation fiable des interventions). 

## I.5. Objectifs SMART de la mission 

- Spécifique : Mettre en place une application SIG métier complète intégrant la visualisation cartographique et le suivi des interventions, et la gestion administrative avec reporting ; 

- Mesurable : La solution est conforme si l’ensemble des fonctionnalités du cahier des charges fonctionnel est implémenté, si aucun défaut bloquant ou critique n’est constaté lors de la phase de test, et si la plateforme est validée par le client final ; 

- Atteignable : L’objectif est atteignable grâce à l’utilisation de technologies SIG et Web open source matures, à un périmètre fonctionnel clairement défini, et à un encadrement assuré par l’entreprise ; 

- Réaliste : Le projet est réaliste au regard de la durée du stage, des ressources disponibles et de l’adoption d’une méthodologie de développement itérative permettant des ajustements progressifs ; 

- Temporel : La solution devra être entièrement fonctionnelle, documentée et déployée en environnement de production avant la date de fin du stage au 03 avril 2026. 

9 - 

## Partie II – Bilan Technique 

## II.1. Méthodologie de travail 

La conduite du projet GreenSIG a reposé sur une adaptation pragmatique de la méthodologie Agile , ajustée aux contraintes d’une mission en solo avec un interlocuteur unique côté client. Il convient d’être transparent sur cette adaptation : en l’absence d’équipe pluridisciplinaire, plusieurs pratiques de l’Agilité – daily standups, rétrospectives collectives, revues de sprint en équipe – n’avaient pas lieu d’être. L’esprit Agile a néanmoins été préservé à travers trois principes directeurs : la livraison incrémentale de fonctionnalités testables, l’accueil du changement en cours de projet, et la validation continue par le client. 

Concrètement, le projet a été structuré selon le découpage défini dans le WBS du plan directeur, organisé en huit lots de travail allant de pilotage à la clôture du projet. Ce découpage a servi de colonne vertébrale au planning GANTT , qui a défini quatre jalons structurants : la validation du cadrage le 15 janvier, la validation du cahier des charges fonctionnel le 28 janvier, le début du développement frontend le 10 février, et le lancement de la phase de tests et recette le 10 mars, avec une livraison finale au 31 mars. La matrice RACI, quant à elle, a formalisé la répartition des responsabilités entre les parties prenantes. 

Chaque cycle hebdomadaire s’organisait en trois (3) temps : analyse fine de la tâche à partir des spécifications du WBS, implémentation avec écriture simultanée des tests, puis validation lors des points bihebdomadaires avec la tutrice entreprise. En l’absence de rétrospective d’équipe, j’ai compensé par une pratique individuelle de journal de bord technique, consignant les décisions d’architecture, les difficultés rencontrées et les arbitrages réalisés – un outil qui s’est révélé précieux pour la traçabilité des choix et la rédaction du présent rapport. 

L’analyse des risques ( RBS ) conduite en amont dans le plan directeur, a identifié dix-huit (18) risques répartis en six (6) catégories. Parmi eux, le risque R2 (évolution du périmètre fonctionnel) s’est effectivement matérialisé en cours de projet avec l’ajustement majeur décrit à la section II.8. L’approche itérative a permis de traiter cet ajustement avec sérénité, en repriorisant les tâches sans remettre en cause l’architecture globale. De 

10 - 

même, le risque R10 (surcharge de travail) a nécessité une gestion active du planning, confirmant la pertinence des marges prévisionnelles intégrées au GANTT initial. 

Le recours systématique aux tests automatisés a constitué un filet de sécurité essentiel, permettant les refactorisations sans risque de régression non détectée. Ce choix méthodologique, s’il a initialement ralenti le rythme de développement, a prouvé sa valeur lors des phases d’intégration et de recette, où le taux de défauts bloquants est resté très faible. 

## II.2. Work Breakdown Structure actualisé 

Le WBS prévisionnel, élaboré dans le plan directeur, structurait le projet en huit (8) lots de travail : pilotage et gestion de projet (1.1), analyse des besoins et conception fonctionnelle (1.2), conception de l’architecture technique (1.3), développement backend (1.4), développement frontend et application cliente (1.5), test, validation et recette (1.6), déploiement (1.7), et documentation, formation et clôture (1.8). Ce découpage, fondé sur une approche par livrables, a constitué le référentiel de planification tout au long du stage. 

A l’issue du projet, le WBS a connu un écart significatif par rapport à sa version initiale, concentré sur le lot 1.5 (développement frontend). Le plan directeur prévoyait trois (3) volets clients : l’interface Webmapping back-office (1.5.1), une application opérateur terrain native mobile (1.5.2) et un portail client (1.5.3). En cours de projet, une décision stratégique, motivée par des contraintes budgétaires du client, a conduit à abandonner le développement de l’application mobile native au profit d’une interface web responsive accessible sur tablette via navigateur. Cette décision a modifié la structure du lot 1.5.2 : les tâches « Interface mobile responsive » et « Consultation des tâches » ont été remplacées par le développement d’une application web responsive optimisée pour les écrans tactiles supérieurs à 8 pouces. La charge associée a été réorientée vers le renforcement du backend, notamment sur l’optimisation des requêtes spatiales et le système de duplication de tâches récurrentes. 

En dehors de cet ajustement, les autres lots ont été exécutés conformément au planning prévisionnel, avec un respect global des jalons définis dans le GANTT : validation du cadrage (15 janvier), validation du cahier des charges fonctionnel (28 janvier), début du développement frontend (10 février), début de la phase de tests (10 mars) et livraison finale (31 mars). Le GANTT actualisé et l’analyse de risques complète sont consultables en annexe. 

11 - 

## II.3. Architecture technique et choix technologiques 

- II.3.1. Architecture globale ~~— —~~ == _Figure 5: Architecture technique de GreenSIG_ 

L’architecture de GreenSIG repose sur un modèle client-serveur classique en trois tiers, conteneurisé via Docker et orchestré par Docker Compose. Six (6) services constituent l’infrastructure de production : 

- Le premier tier, côté client, est une application React servie par Nginx. Le serveur Nginx joue un double rôle : il sert les fichiers statiques du frontend et agit comme 

12 

reverse proxy, redirigeant les requêtes API vers le backend Django et les connexions WebSocket vers le serveur ASGI. Les tuiles cartographiques sont consommées directement depuis les serveurs OpenStreetMap par la librairie Openlayers intégrée au frontend. 

- Le second tier, côté serveur, est un backend Django REST Framework exposant une API RESTful structurée en cinq (5) domaines fonctionnels : gestion des objets géospatiaux, authentification et gestion des utilisateurs, planification des interventions, suivi des réclamations et reporting. Le backend s’appuie sur deux (2) services complémentaires : un worker Celery pour l’exécution des tâches asynchrones (auto-clôture des réclamations, génération de rapports) et un ordonnanceur Celery Beat pour la planification des tâches périodiques, tous deux utilisant Redis comme broker de messages. 

- Le troisième tier, côté donnée, est une base de données PostgreSQL avec l’extension PostGIS, permettant le stockage et l’interrogation de données géospatiales (point, polygones, lignes) en projection WGS 84 (SRID 4326). 

Le déploiement en production a été réalisé sur un serveur on-premise du client, avec un accès sécurisé via un tunnel Cloudflare connecté à l’environnement WSL hébergeant les conteneurs Docker. 

II.3.2. Justification des choix technologiques 

Les choix technologiques ont été guidés par trois critères déterminants : la contrainte budgétaire (pile intégralement open source), la cohérence fonctionnelle avec les exigences SIG du projet, et la maturité des outils retenus. Il convient d’être transparent : il n’y a pas eu d’étude benchmark formelle comparant plusieurs options. Les choix se sont fondés sur une analyse des prédispositions fonctionnelles de chaque technologie, validée par la tutrice entreprise. 

Le choix de Django et Django REST Framework comme socle backend se justifie par plusieurs atouts structurants pour ce type de projet. L’ORM Django, combiné à l’extension GeoDjango, offre une abstraction native des types géométriques PostGIS (Point, Polygon, LineString) et des opérations spatiales (intersection, distance, contenance), ce qui a considérablement réduit la complexité des requêtes géospatiales. Par ailleurs, Django intègre en natif des mécanismes de protection contre les vulnérabilités courantes (injections SQL, attaques XSS, falsification de requêtes CSRF), un avantage 

13 - 

significatif pour un développeur junior déployant un projet en production. Le système d’authentification JWT, implémenté via la bibliothèque Simple JWT, a permis de sécuriser les échanges API tout en gérant le cloisonnement des trois rôles applicatifs (Administrateur, Superviseur, Client). 

Le frontend a été développé en React avec TypeScript, construit avec Vite. La cartographie repose sur Openlayers, une bibliothèque SIG open source complète, offrant des fonctionnalités avancées de manipulation géométrique, de gestion des projections et de rendu vectoriel. Ce choix imposé par la direction technique, s’est avéré pertinent au regard des besoins du projet : les opérations de filtrage spatial, d’affichage multi-couches et d’interaction avec les géométries complexes bénéficient directement de la richesse fonctionnelle d’OpenLayers. 

PostgreSQL avec PostGIS s’est imposé comme le seul choix réaliste pour une base de données spatiale open source de qualité industrielle. L’indexation spatiale GiST, native dans PostGIS, a été déterminante pour garantir les performances de requêtes sur les couches géographiques avec des temps de chargement inférieurs à deux secondes, conformément aux exigences du cahier des charges fonctionnel. 

Redis et Celery ont été retenus pour la gestion des traitements asynchrones. Celery Beat, couplé au scheduler de base de données Django Celery Beat, a permis d’implémenter les tâches périodiques telles que l’auto-clôture des réclamations résolues après 48 heures de silence du client et l’envoi de rappels à 24 heures. 

Enfin, Docker a été adopté comme outil de conteneurisation pour garantir la reproductibilité de l’environnement entre le poste de développement et le serveur de production, un choix dont la pertinence s’est confirmée lors de la phase de déploiement, comme le détaillera la section II.7. 

II.3.3. Modèle de données 

Le modèle de données de GreenSIG s’organise autour de cinq (5) domaines fonctionnels, totalisant plus de vingt-cinq (25) entités. 

Le domaine géospatial constitue le cœur du système. Il s’articule autour de trois entités principales : Site (représentant un périmètre géographique global avec une emprise polygonale), Sous-site (subdivision d’un site en zone opérationnelles, également 

14 - 

polygonale) et Objet (représentant un élément de patrimoine végétal ou hydraulique avec une géométrie variable : Point pour les arbres, Polygon pour les pelouses, LineString pour les canalisations). La hiérarchie Site → Sous-site → Objet permet une navigation spatiale progressive de la macro au micro. 

Le domaine utilisateurs et ressources humaines présente une architecture atypique, issue d’une refactorisation menée en cours de développement. Le modèle distingue explicitement les utilisateurs (entités pouvant se connecter à l’application : Administrateur, Superviseur, Client) des opérateurs terrain (données RH gérées dans l’application sans compte de connexion). Ce choix architectural, motivé par la réalité métier simplifie la gestion des accès tout en permettant un suivi RH complet (absences, compétences, historique d’affectation). Les équipes sont rattachées à un site principal et peuvent intervenir sur des sites secondaires, reflétant la logique de proximité géographique du terrain. 

Le domaine de la planification gère les tâches d’intervention via un modèle Tâche associé à des distributions de charges (répartition de la charge sur plusieurs jours). Le calcul de la durée estimée d’intervention repose sur une formule intégrant la productivité théorique par type de tâche et par type d’objet, rapportée à l’effectif de l’équipe assignée. Par exemple, pour une opération de nettoyage d’arbres avec une productivité théorique d’un arbre par heure, une intervention portant sur 15 arbres avec une équipe de 3 membres aboutit à une durée estimée de cinq (5) heures. Un système de duplication de tâches avec une récurrence permet de générer automatiquement les plannings périodiques. 

Le domaine des réclamations implémente un workflow à sept (7) statuts (Nouvelle, En cours, En attente de validation de clôture, Intervention rejetée, Clôturée, Rejetée) avec un historique complet des transitions, un mécanisme d’auto-clôture après 48 heures sans réponse du client, et un système de satisfaction intégré. La localisation géographique des réclamations permet une détection automatique de la zone et du site concernés par une intersection spatiale. 

Le domaine du suivi des interventions quant à lui assure la traçabilité opérationnelle : consommation de produits phytosanitaires (avec matières actives et doses recommandées), fertilisants, identification de ravageurs et maladies, et photographies avant/après intervention géolocalisées. 

15 - 

## II.4. Développement Backend 

Le développement backend constitue le lot de travail le plus conséquent du projet, couvrant la mise en place de la base de données géospatiale, la construction de l’API REST et l’implémentation de la logique métier. Cette section détaille les réalisations techniques en s’appuyant sur les choix architecturaux présentés en section II.3. 

II.4.1. Mise en place de la base de données PostGIS 

La première étape du développement a consisté à concevoir et implémenter le schéma de base de données. Le modèle de données s’organise autour de cinq (5) domaines fonctionnels totalisant vingt-cinq (25) entités. 

Le domaine géospatial repose sur une hiérarchie à trois (3) niveaux : site (emprise polygonale d’un périmètre géographique), sous-site (subdivision opérationnelle, également polygonale) et objet (élément unitaire du patrimoine). Quinze types d’objets sont gérés, répartis en deux familles : sept (7) de type végétal (arbres, gazons, palmiers, vivaces, cactus, arbustes, graminées) et huit de type hydraulique (puits, pompes, vannes, clapets, canalisations, aspersions, goutte-à-goutte, ballons). Chaque type dispose d’un type géométrique adapté à sa nature : Point pour les arbres et les équipements ponctuels, Polygon pour les gazons et les emprises, Ligne pour les canalisations. Cette hiérarchie Site → Sous-site → Objet permet une navigation spatiale progressive, de la macro au micro, et exploite pleinement les capacités de PostGIS en projection WGS 84 (SRID 4326). L’indexation spatiale GiST a été activée sur l’ensemble des champs géométriques pour garantir des temps réponse inférieurs à deux (2) secondes sur les requêtes de filtrage par emprise. 

L’import des données cartographiques initiales, fournies par le client au format GeoJSON, a nécessité un travail de nettoyage et de validation. Certaines géométries présentaient des incohérences topologiques (polygones non fermés, coordonnées hors emprise) qui ont dû être corrigées avant injection via les utilitaires GeoDjango. Ce travail, bien que non prévu dans l’estimation initiale, a confirmé la pertinence du risque R7 (données cartographiques incomplètes) identifié dans l’analyse des risques du plan directeur. Pour pérenniser ce processus, un module d’import géographique complet a été développé, supportant trois formats (GeoJSON, KML, Shapefile) avec un workflow en trois étapes : prévisualisation des données, validation des géométries, puis exécution de 

16 - 

l’import. Ce module permet au client d’enrichir son patrimoine de manière autonome après la livraison. 

## II.4.2. Architecture de l’API REST 

L’API REST a été structurée en cinq applications Django indépendantes, chacune correspondant à un domaine fonctionnel. Le schéma d’architecture ci-dessous en présente la vue d’ensemble. 

**==> picture [410 x 201] intentionally omitted <==**

**----- Start of picture text -----**<br>
a<br>eee) (ee eee<br>eee) (ee eee<br>eee) (eee ee<br>**----- End of picture text -----**<br>


_Figure 6: Architecture de l'API REST GreenSIG_ 

17 

## _II.4.2.1.  Application de gestion des objets géospatiaux, inventaire et reporting_ 

Elle constitue le socle du système. Elle expose les endpoints de gestion de la hiérarchie spatiale ainsi qu’un inventaire unifié regroupant les quinze (15) types d’objets avec filtrages dynamique, pagination et suppression en lot. Un endpoint cartographique optimisé permet au frontend de requêter les objets visibles dans la zone d’affichage courante via un filtrage par bounding box , évitant ainsi le chargement de l’intégralité du patrimoine à chaque interaction avec la carte. L’application intègre également une boîte à outils géométrique exposant six (6) opérations spatiales (simplification, découpage, fusion, validation, calcul de surfaces/périmètre, zone tampon) ainsi qu’un moteur de recherche transversal permettant de retrouver n’importe quel objet par nom, référence ou site. Enfin, le volet reporting expose les indicateurs de performance KPI et la génération de rapport mensuels. Un système de notifications temps réel interne, implémenté via Websocket et Django Channels, complète cette application en permettant la diffusion instantanée d’alertes aux utilisateurs connectés. 

## _II.4.2.2.  Application de gestion des utilisateurs et ressources humaines_ 

Elle gère l’authentification et l’ensemble du volet organisationnel. L’authentification repose sur le protocole JWT (JSON Web Token) via la bibliothèque Simple JWT , avec des endpoints dédiés à l‘obtention et au rafraîchissement des jetons de connexion. Le cloisonnement des accès est assuré par trois (3) rôles applicatifs : Administrateur (Accès complet), Superviseur (gestion opérationnelle des équipes, des sites et du planning) et Client (consultation et émission de réclamations). Ce cloisonnement est implémenté via des classes de permission personnalisées Django REST Framework , appliquées systématiquement à chaque endpoint. 

Un choix architectural notable concerne la séparation entre utilisateurs et opérateurs terrain. Les opérateurs (jardiniers) ne sont pas des utilisateurs de l’application, ils n’ont pas de compte de connexion. Ce sont des données RH gérées par les superviseurs et les administrateurs. Ce choix, qui peut sembler contre-intuitif, découle d’une réalité métier ; les équipes terrain n’ont pas de terminaux individuels ni de comptes personnels. Il simplifie considérablement la gestion des accès tout en permettant un suivi RH complet (compétences avec niveaux de maitrise, absences avec validation, historique des affectations aux équipes). 

18 - 

Le modèle d’équipe intègre une logique multisite : chaque équipe rattachée à un site principal (affectation contractuelle) et peut intervenir sur des sites secondaires (proximité géographique). Le superviseur d’une équipe est automatiquement déduit du site principal, une propriété calculée qui évite toute désynchronisation manuelle. 

## _II.4.2.3.  Application de gestion des tâches d’intervention_ 

Elle gère le cycle de vie complet des tâches, de la création à la clôture. Le diagramme d’états ci-dessous illustre les trois statuts actifs d’une tâche conformément à la réalité métier du client. 

**==> picture [334 x 41] intentionally omitted <==**

**----- Start of picture text -----**<br>
mm hl ll<br>**----- End of picture text -----**<br>


_Figure 7: Cycle de vie d'une tâche GreenSIG_ 

Le calcul automatique de la durée estimée d’une intervention repose sur une formule intégrant la productivité théorique par type de tâche, le nombre d’objets concernés et l’effectif de l’équipe assignée. La figure ci-après en illustre le mécanisme avec un exemple concret. 

**==> picture [347 x 221] intentionally omitted <==**

**----- Start of picture text -----**<br>
es<br>leae<br>a Ga<br>Co)<br>**----- End of picture text -----**<br>


_Figure 8: Formule de calcul de la charge estimée d'une intervention_ 

19 

La productivité théorique est gérée comme une entité à part entière, configurable par l’administrateur pour chaque combinaison type de tâche / type d’objet. Ce choix de modélisation rend le système adaptable sans modification de code : si les cadences évoluent sur le terrain, l’administrateur peut ajuster les ratios directement depuis l’interface. 

Le module de planification intègre un système de duplication de tâches avec récurrence, permettant de générer automatiquement les plannings périodiques selon trois (3) modes : par jours de semaines, par jours du mois ou par dates spécifiques. Ce mécanisme repose sur une fonction utilitaire optimisée qui duplique la tâche source avec ses distributions de charge en utilisant des opérations SQL en lot. Les tests de performance ont validé que la duplication de cinquante occurrences avec trois distributions chacune s’exécute en moins de quinze requêtes SQL grâce à cette approche. La répartition de la charge dans le temps est gérée via des distributions de charge, qui ventilent les heures estimées sur des jours spécifiques et permettent l’affichage dans le calendrier de planification. 

_II.4.2.4.  Application de traitement des réclamations_ 

Cette application implémente le processus le plus complexe du système. Le diagramme d’états ci-dessous en présente le workflow complet à sept (7) statuts. 

**==> picture [241 x 262] intentionally omitted <==**

**----- Start of picture text -----**<br>
[=] Figure 9: Workflow des réclamations<br>**----- End of picture text -----**<br>


20 

Le workflow gère trois scénarios de fin de cycle : la clôture validée par le client, la clôture automatique après quarante-huit (48) heures de silence (détaillée en section II.4.3), et le rejet par l’administrateur. Il gère également le scénario de refus d’intervention par le client, qui provoque un retour au statut « En cours » avec conservation du motif de refus et incrémentation d’un compteur de refus. Chaque transition de statut est historisée dans une table dédiée avec horodatage et auteur, permettant une traçabilité complète. La localisation géographique des réclamations active une détection automatique par intersection spatiale : lorsqu’un client positionne une réclamation sur la carte, le système détermine automatiquement le site concerné sans intervention manuelle, via la fonction PostGIS ST_Intersects. Un système de satisfaction client, intégré à la clôture, collecte une note de 1 à 5 accompagnée d’un commentaire optionnel. L’application expose également un export Excel des réclamations pour les besoins de reporting externe. 

_II.4.2.5.  Application de suivi des interventions et traçabilité_ 

Cette application assure la traçabilité opérationnelle des interventions. Elle gère les produits phytosanitaires avec leurs matières actives et doses recommandées, les fertilisants, l’identification des ravageurs et maladies avec association aux produits de traitement recommandés, la consommation réelle de produits par intervention, et les photographies avant/après intervention avec géolocalisation optionnelle. Les photos peuvent être rattachées indifféremment à une tâche, à une réclamation ou à un objet de l’inventaire, offrant une traçabilité visuelle transversale à l’ensemble du système. 

II.4.3. Tâches asynchrones et automatisations 

Plusieurs processus métier nécessitent une exécution différée ou périodique, incompatible avec le cycle de requête-réponse synchrone de l’API. Ces traitements ont été délégués à Celery, avec Redis comme broker de messages et Celery Beat comme ordonnanceur (utilisant le scheduler de base de données Django Celery Beat pour une configuration sans redéploiement). 

Le mécanisme le plus élaboré est l’auto-clôture des réclamations. Lorsqu’une réclamation passe au statut « En attente de validation de clôture », un compte à rebours de quarante-huit (48) heures démarre. A vingt-quatre (24) heures, si le client n’a pas réagi et qu’aucun rappel n’a encore été envoyé, une notification de rappel lui est adressée. A quarante-huit (48) heures, si le silence persiste, la réclamation est automatiquement clôturée et une évaluation de satisfaction par défaut est générée. Ce mécanisme est conçu 

21 - 

pour être idempotent : une réclamation déjà clôturée n’est jamais retraitée, et les rappels ne sont jamais envoyés en doublon. Ces propriétés ont été rigoureusement validées par des tests unitaires utilisant la bibliothèque freeze_time pour simuler le passage du temps avec une précision à l’heure près. Les cas de test couvrent notamment les frontières temporelles critiques (47h vs 49h, 23h vs 25h) et les scénarios d’idempotence. Ci-dessous une illustration de la timeline. 

**==> picture [360 x 41] intentionally omitted <==**

**----- Start of picture text -----**<br>
SS Ole<br>**----- End of picture text -----**<br>


_Figure 10: Timeline auto-clôture des réclamations_ 

La génération des rapports PDF constitue un autre traitement asynchrone. Initialement prévue comme une tâche automatique mensuelle, elle a été réorientée à la demande du client vers déclenchement manuel : l’administrateur sélectionne une plage de dates et lance la génération à la demande. Ce choix illustre la flexibilité permise par l’approche itérative. La mise en page des documents générés (en-têtes, pieds de page, logos, tableaux de statistiques) a nécessité plusieurs itérations pour atteindre un rendu professionnel. Les exports sont également disponibles au format Excel pour les réclamations et l’inventaire, offrant au client la possibilité d’exploiter les données dans ses propres outils d’analyse. 

## II.5. Développement Frontend et applications clientes 

Le développement frontend a mobilisé une pile technologique moderne articulée autour de React 19 avec TypeScript, construite avec Vite pour des temps de compilation optimaux. La gestion de l’état et la synchronisation avec l’API reposent sur TanStack React Query, qui assure la mise en cache automatique des données, le rafraîchissement en 

22 

arrière-plan et la gestion des états de chargement et d’erreur, une approche particulièrement adaptée à une application consommant de nombreux endpoints REST. Le routage applicatif est géré par React Router. Cette section présente les trois volets de l’interface conformément au cahier des charges, puis les défis techniques transversaux rencontrés. 

## II.5.1. Interface Webmapping – Back-office 

L’interface back-office, destinée aux administrateurs et superviseurs, constitue le cœur de l’application. Elle s’organise autour d’une barre de navigation latérale donnant accès à l’ensemble des modules fonctionnels : carte interactive, gestion des sites et de l’inventaire, gestion des équipes et des opérateurs, planification, réclamations, suivi des interventions, statistiques et paramètres. 

## _II.5.1.1.  Module cartographique_ 

Le module cartographique est le point d'entrée principal de l'application. Il repose sur OpenLayers, bibliothèque SIG open source offrant des capacités avancées de gestion des projections, de rendu vectoriel et de manipulation géométrique. La carte affiche les différentes couches géospatiales (sites, sous-sites, objets végétaux et hydrauliques) sur un fond OpenStreetMap avec possibilité de basculer entre vue plan et vue satellite. Un panneau de gestion des couches, accessible depuis la barre d'outils latérale droite, permet d'activer ou de désactiver les couches individuellement. La barre de recherche intégrée permet de localiser rapidement un site, un équipement ou un objet par nom ou référence. L'interaction au clic sur un objet ouvre sa fiche détaillée avec l'ensemble de ses attributs (état sanitaire, historique d'interventions, photos associées). L'optimisation des performances repose sur le chargement par bounding box : seuls les objets situés dans la zone d'affichage courante sont requêtés, évitant le transfert de l'intégralité du patrimoine à chaque déplacement de la carte. 

Ci-dessous, vous avez la vue cartographique satellite de GreenSIG avec les sites et objets géolocalisés. 

23 - 

_Figure 11: Vue cartographique satellite de GreenSIG_ 

## _II.5.1.2.  Module planification_ 

Le calendrier de planification constitue l'un des développements frontend les plus exigeants du projet. Il repose sur la bibliothèque React Big Calendar, dont le rendu par défaut a fait l'objet de personnalisations importantes pour répondre aux spécificités métier. Le calendrier offre trois vues (mensuelle, hebdomadaire, journalière) et affiche conjointement les tâches planifiées et leurs distributions de charge. Chaque entrée du calendrier est colorée selon son statut (planifiée, terminée) et affiche le type de tâche ainsi que le nombre d'équipes assignées. Un système de filtrage par nom, référence, équipe ou site permet de restreindre l'affichage aux tâches pertinentes. La navigation temporelle et l'export PDF du planning complètent ce module. La personnalisation de React Big Calendar a nécessité un travail conséquent sur le rendu des cellules, la gestion des événements multi-jours et l'intégration visuelle avec le reste de l'interface — un écart significatif entre la bibliothèque par défaut et le résultat attendu qui a consommé davantage de temps que l'estimation initiale. 

Ci-dessous, vous avez la vue du calendrier de planification mensuel de GreenSIG avec les tâches et les distributions de charge. 

24 

_Figure 12: Calendrier de planification mensuel de GreenSIG_ 

## _II.5.1.3.  Tableau de bord et indicateurs de performance_ 

Le module de statistiques présente quatre indicateurs clés calculés mensuellement : le taux de respect du planning (pourcentage de tâches terminées dans les délais), la qualité de service, le taux de réalisation des réclamations et le temps moyen de traitement. Chaque indicateur est affiché avec sa valeur courante, un seuil de référence, une barre de progression colorée selon le statut (critique, acceptable, atteint) et l'évolution par rapport au mois précédent. Les graphiques sont rendus avec Recharts. Un tableau détaillé en partie inférieure offre une vue tabulaire complète des KPI avec historique. 

_Figure 13: Tableau de bord des indicateurs de performance (KPI)_ 

25 

II.5.2. Interface opérateur terrain – Web responsive 

L'interface terrain, initialement prévue comme une application mobile native, a été réorientée vers une application web responsive accessible sur tablette via navigateur comme évoqué précédemment. Elle est conçue pour les superviseurs de terrain et donne accès à la consultation des tâches assignées, à la saisie des rapports d'intervention avec ajout de photographies (avant/après) et à la consultation de la carte. 

L'adaptation responsive a constitué l'un des défis techniques majeurs du projet. La décision d'abandonner la version mobile native étant intervenue après que le développement desktop était déjà avancé, l'ensemble de l'interface a dû être repris pour fonctionner sur des écrans de tablette (supérieurs à 8 pouces). Les problèmes rencontrés étaient concrets : impossibilité de scroller horizontalement et verticalement, contenu débordant des limites de l'écran, absence de réduction ergonomique des composants et absence d'adaptation dynamique du contenu en fonction de la résolution. La résolution de ces problèmes a nécessité une refonte des règles CSS avec l'introduction de breakpoints adaptés, le passage de certains layouts fixes à des layouts flexibles, le redimensionnement des composants interactifs (boutons, formulaires, tableaux) pour une utilisation tactile, et l'adaptation de la carte OpenLayers pour la navigation tactile (pinch-to-zoom, défilement au doigt). Ce travail, non prévu dans le planning initial, illustre l'impact concret de l'ajustement de périmètre décrit en section II.8 sur la charge de développement frontend. 

_Figure 14: Interface de suivi des tâches sur mobile_ 

26 

## II.5.3. Portail client 

Le portail client offre un accès en lecture seule aux données de suivi des prestations, avec deux fonctionnalités principales : la visualisation des interventions réalisées sur les sites du client (avec accès aux rapports et aux photographies) et le dépôt et suivi des réclamations. 

Le module de réclamations permet au client de créer une réclamation en positionnant la zone concernée directement sur la carte, de sélectionner un type et un niveau d'urgence, et de joindre des photographies. La détection automatique du site et de la zone par intersection spatiale (côté backend) simplifie la saisie. Le client peut ensuite suivre l'évolution de sa réclamation à travers les statuts du workflow, consulter l'historique des échanges, valider ou refuser la clôture proposée par l'administrateur, et déposer une évaluation de satisfaction une fois la réclamation clôturée. L'accès aux rapports PDF mensuels complète ce portail. 

_Figure 15: Fiche de suivi d'une réclamation_ 

- II.5.4. Défis techniques transversaux 

Au-delà des défis spécifiques à chaque module, plusieurs problématiques techniques transversales méritent d’être mentionnées. 

La virtualisation des listes longues a été implémentée via React Window couplé à React  Virtualized Auto Sizer et un système de chargement infini (Infinite Loader). Pour 

27 

l’inventaire, qui peut contenir plusieurs milliers d’objets répartis sur quinze (15) types, le rendu de l’intégralité des éléments dans le DOM aurait dégradé les performances de manière significative. La virtualisation ne rend que les éléments visibles à l’écran et recycle les nœuds DOM au défilement, maintenant la fluidité de l’interface indépendamment du volume des données. 

La génération de documents côté client complète les exports backend. Les bibliothèques jsPDF et ExcelJS permettent de produire des documents PDF et Excel directement dans le navigateur, sans solliciter le serveur, une fonctionnalité utile pour les exports rapides de vues filtrées ou de tableaux personnalisés. 

Enfin, le choix de TypeScript pour l’ensemble du frontend a constitué un investissement initial qui s’est révélé rentable en phase d’intégration. Le typage statique a permis de détecter en amont les incohérences entre les structures de données attendues par le frontend et celles envoyées par l’API, réduisant ainsi les allers-retours de débogage. 

## II.6. Tests, validation et recette 

La stratégie de test a été orientée vers la fiabilité de la logique métier backend, identifiée comme le maillon critique du système. Les contraintes temporelles du stage n'ont pas permis de mettre en place des tests automatisés côté frontend — un choix assumé, fondé sur le constat que les risques de régression les plus graves portaient sur les règles métier, les calculs de planification et le workflow des réclamations, et non sur le rendu des interfaces. 

## II.6.1. Tests automatisés backend 

La suite de tests, développée avec pytest et exécutée sur une base de données de test PostgreSQL/PostGIS, totalise cent douze (112) cas de test répartis en cinq fichiers couvrant cinq (5) domaines fonctionnels. 

Le premier domaine, les règles métier de planification (37 tests), valide l'ensemble des mécanismes de synchronisation entre tâches et distributions de charge. Les tests couvrent la synchronisation du statut d'une tâche après annulation de distributions (toutes annulées → tâche annulée, toutes réalisées → tâche terminée, mix réalisées/annulées → terminée), la validation des transitions de statut autorisées et interdites (un statut terminal comme REALISEE ne peut pas revenir à EN_COURS), la limite de reports dans une chaîne de distributions, la vérification de complétion des distributions, le démarrage automatique 

28 - 

d'une tâche lors de la première distribution, et la cascade d'annulation des distributions actives lorsqu'une tâche est annulée. 

Le deuxième domaine, la duplication de tâches (20 tests), valide le mécanisme de génération de plannings récurrents. Les tests vérifient la création correcte du nombre de tâches attendu, le décalage des dates, la conservation optionnelle des équipes et des objets (relations many-to-many), la duplication des distributions avec leurs dates décalées, et les cas limites (tâche sans distribution, liste de décalages vide, identifiant inexistant). Un aspect notable est le test d'efficacité SQL : la duplication de dix (10) puis cinquante (50) occurrences est vérifiée en moins de quinze requêtes SQL grâce aux opérations en lot, confirmant la scalabilité du mécanisme. Les trois fonctions appelantes (duplication par dates spécifiques, par jours de semaine, par jours du mois) font également l'objet de tests dédiés. 

Le troisième domaine, le respect du planning — KPI (16 tests), valide le calcul de l'indicateur principal du tableau de bord. La formule (tâches terminées dans les délais, avec une tolérance de sept jours, rapportées au total des tâches planifiées dans le mois) est testée exhaustivement : tâche terminée avant l'échéance, le jour même, dans la tolérance (5 jours, exactement 7 jours), hors tolérance (8 jours), tâche non terminée (planifiée, en cours), exclusion des tâches hors du mois de référence, et taux mixtes (80 %, 60 %, 100 %, 0 %). Le cas limite d'absence totale de tâches est également couvert. 

Le quatrième domaine, les permissions par rôle (29 tests), valide le cloisonnement des accès à l'API. Les tests utilisent force_authenticate() de Django REST Framework pour simuler les trois profils (Administrateur, Superviseur, Client) et vérifier les règles d'accès : l'administrateur voit toutes les tâches, réclamations et sites ; le superviseur ne voit que ceux rattachés à ses sites ; le client ne voit que ceux de sa structure. Les tests couvrent également les actions interdites : un client ne peut pas clôturer une réclamation, un client ne peut pas reporter une distribution. L'accès non authentifié est vérifié (retour 401). Les quatre classes de permission personnalisées (IsAdmin, IsSuperviseur, IsAdminOrSuperviseur, IsClient) font l'objet de tests unitaires dédiés. 

Le cinquième domaine, l'auto-clôture des réclamations (10 tests), valide le mécanisme de clôture automatique après quarante-huit heures. Les tests utilisent la bibliothèque freeze_time pour simuler le passage du temps avec précision et couvrent les frontières temporelles critiques (47 heures → pas de clôture, 49 heures → clôture), le 

29 - 

traitement de plusieurs réclamations éligibles simultanément, la création d'un historique pour chaque auto-clôture, l'idempotence (une réclamation déjà clôturée n'est pas retraitée), le mécanisme de rappel à 24 heures (envoi effectif, absence de doublon, pas de rappel avant 24 heures), et la priorité de la clôture sur le rappel (à 49 heures, la réclamation est clôturée sans rappel préalable inutile). 

Le tableau ci-dessous synthétise la couverture de test : 

|Domaine|Fichier|Nb tests|Techniques clés|
|---|---|---|---|
|Règles métier<br>planification|test_business_rule<br>s.py|37|Transitions d'état,<br>synchronisation, cascade|
|Duplication de tâches|test_dupliquer_tac<br>he.py|20|Bulk SQL, M2M,<br>récurrence, performance|
|KPI respect planning|tests_respect_plan<br>ning.py|16|Calcul de taux, tolérance<br>7j, cas limites|
|Permissions API|test_permissions_<br>api.py|29|RBAC, cloisonnement,<br>force_authenticate|
|Auto-clôture réclamations test_auto_close.py|Auto-clôture réclamations test_auto_close.py|10|freeze_time, idempotence,<br>rappels|
|||112||



_Figure 16: Tableau synthétique de la couverture des tests_ 

II.6.2. Recette client et suivi des anomalies 

La recette client s'est déroulée de manière continue tout au long du projet, conformément à l'approche itérative adoptée. Les sessions de validation prenaient la forme de prises en main de l'application par téléphone, au cours desquelles le client testait les fonctionnalités livrées et formulait ses retours en temps réel. Bien que cette approche n'ait pas donné lieu à des procès-verbaux de recette formalisés, une limite que je reconnais rétrospectivement, les remarques et anomalies ont été systématiquement consignées dans un tableur partagé servant de registre de suivi. 

30 

Ce registre a accumulé quatre-vingt-dix-sept remarques au cours du projet, couvrant un spectre large : bugs fonctionnels (problèmes de filtrage, d'affichage, de permissions par rôle), demandes d'évolution (multi-sites pour les équipes, refus d'intervention par le client, auto-clôture des réclamations, gestion des jours fériés, duplication du planning), problèmes d'affichage (débordement dans les exports PDF, doublons dans la recherche, surfaces non affichées), et corrections ergonomiques (renommage des intitulés de statuts, uniformité des symboles cartographiques, amélioration des couleurs des couches de végétation). 

L'analyse de ce registre révèle plusieurs enseignements. Premièrement, une proportion significative des remarques portait sur des fonctionnalités des profils Client et Superviseur, initialement moins aboutis que le profil Administrateur — confirmant que le développement s'était concentré sur le back-office dans un premier temps. Deuxièmement, plusieurs demandes d'évolution majeures (workflow de réclamation à sept statuts, multi-sites, jours fériés) sont venues enrichir le périmètre fonctionnel bien au-delà du cahier des charges initial, témoignant de la confiance du client dans la capacité du projet à absorber ces évolutions. Troisièmement, certaines anomalies récurrentes, notamment les problèmes de scroll et de débordement, ont confirmé la nécessité du travail de responsive détaillé en section II.5.2. 

## II.6.3. Bilan qualité 

Le bilan qualité du projet est globalement positif. Côté automatisation, les cent douze (112) tests backend couvrent les mécanismes les plus critiques du système et ont permis de détecter plusieurs régressions lors des phases de refactorisation, notamment lors de la migration de l’architecture RH (séparation utilisateurs/opérateurs) et de l’introduction du multi-sites pour les équipes. Côté recette, la grande majorité des quatrevingt-dix-sept (97) remarques ont été traitées et intégrées dans la version finale livrée. 

Les limites identifiées portent sur deux points. L’absence de tests frontend automatisés constitue un risque pour la maintenabilité à long terme : toute modification de l’interface nécessite actuellement des tests manuels pour vérifier l’absence de régression. Par ailleurs, l’absence de procès-verbaux de recette formels, bien que compensée par le registre de suivi partagé, ne constitue pas une pratique conforme aux standards de gestion de projet que j’adopterais dans un contexte professionnel futur. Ces deux (2) points sont développés dans le bilan d’expérience (Partie III). 

31 - 

## II.7. Déploiement 

Le déploiement en production a constitué l'une des phases les plus éprouvantes du projet sur le plan technique. Si l'architecture conteneurisée via Docker garantissait en théorie une reproductibilité parfaite entre l'environnement de développement et le serveur de production, la réalité du terrain a imposé des adaptations significatives. 

Le serveur mis à disposition par le client est une machine on-premise fonctionnant sous Windows. Docker Desktop, solution standard pour l'exécution de conteneurs sous Windows, s'est révélé incompatible avec la version du système d'exploitation installée. Cette contrainte a nécessité le recours à WSL (Windows Subsystem for Linux) pour héberger l'environnement Docker. Sous WSL, l'infrastructure composée des six conteneurs décrits dans le schéma d'architecture (PostgreSQL/PostGIS, Redis, Backend Django, Celery Worker, Celery Beat, Frontend Nginx) a pu être déployée, mais un second obstacle est apparu : l'impossibilité de construire les images Docker directement sur le serveur. Les restrictions réseau en place empêchaient le téléchargement des paquets et dépendances nécessaires au build des images, rendant inopérant tout pipeline d'intégration continue (CI/CD) sur site. 

La solution adoptée a consisté à préparer une « valise Docker » : les images ont été construites et testées sur mon poste de développement, exportées sous forme d'archives (docker save), puis transférées physiquement sur le serveur où elles ont été chargées (docker load) et lancées via Docker Compose. Cette approche, bien que peu orthodoxe du point de vue DevOps, a permis de contourner les restrictions réseau tout en garantissant l'intégrité des images déployées, chaque image étant strictement identique à celle validée en développement. 

L'accès sécurisé à l'application depuis l'extérieur a été assuré par la mise en place d'un tunnel Cloudflare, connecté à l'environnement WSL. Un nom de domaine a été acquis et configuré pour pointer vers le tunnel, permettant aux utilisateurs (administrateurs, superviseurs et clients) d'accéder à GreenSIG via HTTPS sans nécessiter d'ouverture de ports sur le pare-feu du serveur, un avantage significatif en termes de sécurité. 

Cette expérience de déploiement, bien que frustrante sur le moment, a constitué l'un des apprentissages les plus formateurs du stage. Elle a mis en évidence l'écart qui peut exister entre un environnement de développement maîtrisé et les contraintes d'un 

32 - 

environnement de production réel, contraintes rarement abordées dans le cadre académique mais omniprésentes en situation professionnelle. Le risque R16 (problème lors du déploiement) identifié dans le plan directeur s'est matérialisé, et l'action de maîtrise prévue (tests pré-production) n'aurait pas suffi à elle seule : c'est l'adaptabilité et la recherche de solutions alternatives qui ont permis de livrer dans les délais. 

## II.8. Conclusion technique 

Le projet GreenSIG a atteint les objectifs définis dans la formulation SMART de la mission. La solution livrée au 31 mars 2026 intègre l'ensemble des fonctionnalités prévues dans le cahier des charges fonctionnel : la visualisation cartographique des espaces verts sur fond OpenStreetMap avec gestion de quinze types d'objets végétaux et hydrauliques, la planification et le suivi des interventions avec calcul automatique des durées et système de récurrence, la gestion complète des réclamations avec un workflow à sept statuts incluant l'auto-clôture, le reporting avec quatre indicateurs de performance, et les trois interfaces clientes (back-office administrateur, interface terrain responsive, portail client). La plateforme a été déployée en production sur le serveur du client et validée lors de la phase de recette. 

Au-delà du périmètre initial, le projet a intégré des fonctionnalités nées des retours itératifs du client : le système multi-sites pour les équipes, la gestion des jours fériés avec prise en compte dans la planification, le module d'import géographique multi-format, la boîte à outils géométrique, les notifications temps réel par WebSocket, et la refactorisation de l'architecture RH séparant utilisateurs et opérateurs. Ces ajouts, rendus possibles par l'approche itérative et la modularité de l'architecture, témoignent de la capacité du système à absorber des évolutions sans remise en cause de ses fondations. 

Des limites doivent néanmoins être identifiées. L'absence de tests automatisés côté frontend constitue une fragilité pour la maintenabilité à long terme. L'impossibilité de mettre en place un pipeline CI/CD sur le serveur de production contraint les mises à jour futures au mécanisme de valise Docker, peu scalable. Enfin, vingt-deux demandes d'évolution consignées lors de la recette restent hors périmètre et constituent une feuille de route pour les développements futurs. 

En termes de perspectives, trois axes d'évolution se dessinent. Le premier est l'intégration de capteurs IoT pour l'arrosage intelligent, permettant un pilotage de la 

33 - 

consommation hydrique basé sur des données temps réel, un enjeu critique dans le contexte de stress hydrique du Maroc. Le deuxième est le développement d'un module d'analyse prédictive exploitant l'historique des interventions pour anticiper les besoins de maintenance, en cohérence avec ma spécialisation en Intelligence Artificielle et Big Data. Le troisième est la duplication du modèle GreenSIG à d'autres villes vertes marocaines, perspective identifiée dès l'analyse SWOT comme opportunité stratégique pour AL BARAA CONSULTING. 

34 - 

## Partie III : Bilan de l’expérience 

Le respect de la qualité dans le développement logiciel : enseignements d'un premier cycle complet de projet. 

Le choix de ce thème s'impose naturellement au regard de l'expérience vécue durant ce stage. Développer de A à Z un système d'information géographique en trois mois, seul, avec une mise en production réelle et un client exigeant, m'a confronté quotidiennement à une question que la formation académique aborde en théorie mais dont la portée ne se révèle pleinement qu'en situation professionnelle : qu'est-ce que la qualité logicielle, et comment la concilier avec les contraintes d'un projet réel ? 

Au démarrage du stage, ma conception de la qualité était essentiellement technique et binaire : un logiciel de qualité est un logiciel qui fonctionne. Cette vision, je le réalise rétrospectivement, était insuffisante. Pressé par les délais, j'ai commencé le développement sans avoir formalisé l'intégralité du modèle de données, sans étude comparative des technologies, en m'appuyant sur les outils que je connaissais plutôt que sur ceux qui auraient pu être optimaux. Le choix d'OpenLayers, par exemple, n'est pas issu d'un benchmark documenté mais d'une décision de la direction technique — une réalité courante dans les petites structures, mais qui m'a interrogé sur la rigueur méthodologique attendue d'un ingénieur. 

C'est paradoxalement les premières erreurs qui m'ont fait évoluer. Lorsque les données GeoJSON fournies par le client se sont révélées incohérentes, j'ai compris que la qualité ne commence pas au code, elle commence aux données. Lorsque les premiers retours de recette ont révélé des problèmes de scroll, de débordement d'écran et de permissions mal cloisonnées entre les profils, j'ai compris que la qualité perçue par le client n'est pas la qualité technique que le développeur voit dans son code. Un algorithme de planification parfaitement optimisé ne vaut rien si le superviseur ne peut pas faire défiler la page pour voir ses tâches. 

Cette prise de conscience a provoqué un tournant dans ma pratique. J'ai commencé à investir systématiquement dans les tests automatisés, cent douze (112) cas de test sur les mécanismes critiques du backend. Ce choix a eu un coût immédiat : du temps de développement consacré à écrire des tests plutôt qu'à produire des fonctionnalités visibles. Mais il a prouvé sa valeur lors de la refactorisation de l'architecture RH (séparation 

35 - 

des utilisateurs et des opérateurs) et lors de l'introduction du multi-sites pour les équipes, deux modifications structurelles qui auraient pu introduire des régressions en cascade sans filet de sécurité. J'ai choisi en revanche de ne pas investir dans les tests frontend, par manque de temps. C'est un compromis assumé, mais que je reconnais comme une dette technique : toute modification de l'interface nécessite aujourd'hui des vérifications manuelles, ce qui n'est pas viable à long terme. 

Le registre de recette, avec ses quatre-vingt-dix-sept remarques, constitue à mes yeux le document le plus révélateur de ce stage. Il montre qu'un logiciel n'est jamais terminé au sens où son développeur l'entend, il l'est lorsque son utilisateur peut s'en servir efficacement. Parmi ces remarques, certaines portaient sur des bugs que mes tests n'avaient pas couverts, d'autres sur des attentes que le cahier des charges n'avait pas formalisées, d'autres encore sur des améliorations ergonomiques que seule la confrontation avec l'usage réel pouvait révéler. La qualité, en ce sens, n'est pas un état à atteindre mais un processus itératif de convergence entre la vision du développeur et les besoins de l'utilisateur. 

En prenant du recul, je retire de cette expérience trois convictions que je porterai dans ma vie professionnelle. La première est que la qualité est un investissement, pas un coût. Chaque heure consacrée aux tests, à la validation des données ou à la documentation est une heure qui sera économisée en maintenance, en débogage et en gestion de crise. La deuxième est que la qualité est l'affaire de tous les acteurs du projet, pas du seul développeur. Le client qui teste rigoureusement et consigne ses retours, la tutrice qui challenge les choix d'architecture, le superviseur terrain qui signale un problème d'ergonomie, chacun contribue à la qualité du produit final. La troisième, enfin, est que la qualité est indissociable de la responsabilité de l'ingénieur. Un système d'information qui gère le patrimoine végétal d'une ville, qui planifie le travail d'équipes terrain, qui traite des réclamations de citoyens, n'est pas un exercice académique, c'est un outil dont la fiabilité a des conséquences concrètes sur des personnes et des organisations. Cette responsabilité, que j'ai ressentie pour la première fois lors de la mise en production de GreenSIG , constitue peut-être l'enseignement le plus durable de ce stage. 

36 - 

## CONCLUSION GÉNÉRALE 

Ce stage de trois (3) mois au sein d’AL BARAA CONSULTING m’a confié une mission aussi ambitieuse qu’exigeante : concevoir, développer et déployer en production GreenSIG , un Système d’Information Géographique dédié à la gestion des espaces verts de la ville de Benguerir . Le bilan que j’en tire est celui d’une expérience fondatrice, tant sur le plan technique que sur le plan humain et professionnel. 

Sur le plan technique, le projet a abouti à une plateforme opérationnelle intégrant l’ensemble des fonctionnalités définies dans le cahier des charges : visualisation cartographique de quinze (15) types d’objets végétaux et hydrauliques sur fond OpenStreetMap, planification des interventions avec calcul automatique des durées et des systèmes de réccurence, workflow complet de traitement des réclamations avec autoclôture, tableau de bord d’indicateurs de performance, et trois (3) interfaces clientes adaptées aux profils utilisateurs. L’architecture retenue a démontré sa robustesse et sa capacité à absorber les évolutions fonctionnelles survenues en cours de projet. Les tests automatisés et les remarques de recette traitées témoignent d’un souci de fiabilité qui a accompagné l’ensemble du cycle de développement. 

Sur le plan de la gestion de projet, ce stage m'a appris que la rigueur méthodologique n'est pas un exercice théorique mais une condition de survie en situation réelle. L'approche itérative adoptée a permis d'absorber un ajustement majeur de périmètre (le passage de l'application mobile native à une interface web responsive), d'intégrer des fonctionnalités non prévues initialement (multi-sites, jours fériés, import géographique multi-format) et de livrer dans les délais malgré les obstacles rencontrés lors du déploiement. J'ai également mesuré l'écart entre la planification et la réalité : certaines tâches ont pris plus de temps que prévu (responsive, personnalisation du calendrier, mise en page des PDF), d'autres moins, et c'est la capacité à reprioriser en continu qui a permis de maintenir le cap. 

Sur le plan personnel, cette première expérience professionnelle significative a transformé ma perception du métier d'ingénieur. J'ai découvert qu'être développeur sur un projet de production, c'est prendre des décisions sous contrainte et en assumer les conséquences, des conséquences qui ne sont pas une note dans un bulletin mais un outil dont dépendent des équipes terrain et des citoyens. La solitude du développeur unique, 

37 - 

compensée par la confiance de ma tutrice entreprise et l'accompagnement de mon tuteur EIGSI , m'a poussé à développer une autonomie et une discipline que le cadre académique ne sollicite pas avec la même intensité. 

Ce stage ouvre des perspectives à plusieurs niveaux. Pour AL BARAA CONSULTING , GreenSIG constitue une première référence opérationnelle dans la géomatique appliquée à la gestion urbaine, avec un potentiel de duplication à d'autres villes vertes marocaines. Pour le client, les vingt-deux demandes d'évolution identifiées lors de la recette dessinent une feuille de route pour les développements futurs, notamment l'intégration de capteurs IoT pour le pilotage de l'arrosage et l'exploitation de l'historique des interventions par des modèles d'analyse prédictive. Pour moi, cette expérience conforte mon orientation vers l'ingénierie logicielle appliquée aux données géospatiales et à l'intelligence artificielle, deux domaines dont la convergence me semble porteuse de solutions concrètes pour les défis urbains et environnementaux auxquels le Maroc et le continent africain sont confrontés. 

38 - 

