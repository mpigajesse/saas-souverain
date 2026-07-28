ACT — AFRICA CENTRED TECHNOLOGY 

## **SOVEREIGN SAAS FRAMEWORK** 

Note de cadrage du projet R&D Version 2.0 — Mars 2026 

_Document destiné à l’équipe projet — Lecture intégrale requise avant toute contribution_ 

Sovereign SaaS Framework — Note de cadrage v2.0 

## **Table des matières** 

2 

Sovereign SaaS Framework — Note de cadrage v2.0 

## **0. Objet et mode d’emploi de ce document** 

Ce document est la note de cadrage du projet R&D « Sovereign SaaS Framework » porté par ACT. Il constitue le référentiel unique du projet : toute personne qui rejoint l’équipe doit le lire intégralement avant de commencer à travailler. Toute décision structurante doit pouvoir être tracée vers ce document ou vers une mise à jour documentée de ce document. 

Le document est organisé en 12 sections. Les sections 1 à 3 expliquent le problème. Les sections 4 à 5 décrivent ce qui existe déjà. Les sections 6 à 8 définissent notre approche. Les sections 9 à 12 organisent l’exécution. 

## **0.1 Nature du projet** 

Projet R&D à double vocation : recherche (produire de la connaissance publiable) et produit (construire un framework technologique exploitable). Les deux volets avancent en parallèle en boucle de rétroaction : la recherche valide ou invalide des hypothèses, le produit les matérialise et révèle de nouvelles questions. 

## **0.2 Stratégie d’exploitation** 

Phase 1 : ACT intègre le framework dans ses propres solutions SaaS (premier client = soimême). Phase 2 : ouverture en open source ou licence commerciale à d’autres éditeurs. Phase 3 (optionnelle) : offre directe aux entreprises clientes de SaaS non souverains. 

## **0.3 Conventions du document** 

Les encadrés en retrait avec bordure gauche sont des exemples, analogies ou études de cas destinés à illustrer un concept abstrait. Les termes techniques sont définis lors de leur première utilisation et repris dans le glossaire (section 12). Chaque hypothèse est formulée avec sa méthode de validation et ses critères d’invalidation. Chaque phase du projet est décrite avec ses livrables, ses critères de succès et sa durée estimée. 

3 

Sovereign SaaS Framework — Note de cadrage v2.0 

## **1. Le problème en profondeur** 

Cette  section  décompose  le  problème  en  ses  composantes  élémentaires.  Elle  est volontairement  longue  et  pédagogique  parce  que  la  profondeur  de  compréhension  du problème détermine directement la qualité de la solution. 

## **1.1 Le modèle SaaS : fonctionnement et implications** 

Un logiciel SaaS (Software as a Service) est un logiciel que le client n’installe pas sur ses machines. Il y accède via Internet, généralement à travers un navigateur web, contre un abonnement mensuel ou annuel. L’éditeur héberge l’application et les données sur ses propres serveurs (ou ceux d’un fournisseur de cloud comme AWS, Azure ou GCP). Le client n’a aucun contrôle sur l’infrastructure. 

Ce modèle a des avantages considérables : pas de coût d’infrastructure initiale, mises à jour automatiques, accessibilité universelle, scalabilité transparente. Il a permis la démocratisation de logiciels puissants auparavant réservés aux grandes entreprises. 

Mais ce modèle crée une asymétrie fondamentale. Le client délègue simultanément quatre fonctions à l’éditeur : le traitement (le calcul qui transforme les données), le stockage (où les données persistent), l’accès (qui peut voir quoi), et la continuité (que se passe-t-il si le service s’arrête). Cette délégation est totale et indivisible dans le modèle SaaS classique. C’est précisément cette indivisibilité que le projet cherche à briser. 

_Analogie du notaire : Imaginez que vous confiez tous vos documents personnels (titre de propriété, testament, contrats) à un notaire. Le notaire garde les originaux, vous donne accès à des copies numériques via son site web. Vous payez un abonnement mensuel. Si vous arrêtez de payer, vous perdez l’accès. Si le notaire fait faillite, vos documents sont en péril. Si le gouvernement du pays où le notaire est installé exige de voir vos documents, le notaire est légalement obligé de les fournir — même si vous êtes citoyen d’un autre pays. C’est exactement la situation d’un client SaaS._ 

## **1.2 Les quatre dimensions de la souveraineté des données** 

La souveraineté des données n’est pas un concept monolithique. Elle a quatre dimensions distinctes, chacune posant des problèmes différents. 

## _**1.2.1 La résidence (où sont les données ?)**_ 

La résidence des données désigne l’emplacement physique des serveurs qui hébergent les données. Elle répond à la question : dans quel pays, dans quel datacenter, mes données sontelles stockées ? La résidence est le niveau le plus basique de souveraineté. Beaucoup d’éditeurs proposent des « régions » cloud (Europe, Asie, Amérique) pour répondre à cette exigence. Mais la résidence seule est insuffisante. 

_Analogie : savoir que votre coffre-fort est dans une banque à Paris ne vous dit rien sur qui a la clé._ 

## _**1.2.2 La juridiction (sous quelle loi ?)**_ 

La juridiction détermine quel cadre légal s’applique aux données. Ce n’est pas seulement le pays où les données sont stockées qui compte, mais aussi le pays de l’entité qui les contrôle. Le CLOUD Act américain (Clarifying Lawful Overseas Use of Data Act, 2018) est l’illustration la plus frappante : il permet aux autorités américaines d’exiger des données détenues par 

4 

Sovereign SaaS Framework — Note de cadrage v2.0 

toute entreprise américaine, peu importe où ces données sont physiquement stockées dans le monde.  Cela  signifie  qu’un  hôpital  européen  utilisant  un  SaaS  américain  hébergé  en Allemagne peut voir les données médicales de ses patients réclamées par le FBI. 

L’arrêt Schrems II de la Cour de Justice de l’UE (16 juillet 2020, affaire C-311/18) a invalidé le Privacy Shield, le mécanisme qui légitimait le transfert de données personnelles entre l’UE et les États-Unis. La Cour a jugé que les lois américaines de surveillance ne garantissent pas un niveau de protection équivalent au RGPD. Le EU-US Data Privacy Framework adopté en 2023 tente de combler ce vide, mais de nombreux juristes estiment qu’il pourrait être invalidé à son tour (un « Schrems III »). 

## _**1.2.3 Le contrôle technique (qui peut accéder ?)**_ 

Même avec des garanties juridiques, la souveraineté est fragile si l’éditeur a un accès technique aux données en clair. Dans un SaaS classique, les données sont protégées en transit (HTTPS/TLS) et au repos (chiffrement du disque). Mais elles sont nécessairement déchiffrées pour être traitées par l’application. C’est le problème des « données en cours d’utilisation » (data in use), le maillon faible de la chaîne de sécurité. 

Les employés de l’éditeur (développeurs, DBA, équipes support) ont généralement un accès administrateur aux bases de données. Les sous-traitants (hébergeur cloud, prestataires de monitoring) peuvent aussi avoir des accès. Tout attaquant qui compromet l’infrastructure de l’éditeur accède potentiellement aux données de tous les clients simultanément. 

## _**1.2.4 La portabilité (puis-je partir ?)**_ 

La portabilité désigne la capacité du client à récupérer ses données dans un format exploitable et à les transférer vers un autre système. Sans portabilité, la souveraineté est illusoire : on est propriétaire de données qu’on ne peut pas libérer. En pratique, beaucoup d’éditeurs SaaS limitent volontairement les options d’export (formats propriétaires, export partiel, absence d’API d’extraction). Certains suppriment les données rapidement après résiliation (parfois 30 jours seulement). Le EU Data Act (applicable depuis septembre 2025) impose des obligations de portabilité, mais l’implémentation reste hétérogène. 

## **1.3 Le problème spécifique du multi-tenant** 

Le multi-tenancy (multi-location) est le modèle architectural dominant du SaaS. Plusieurs clients (tenants) partagent la même infrastructure, parfois la même base de données, leurs données étant séparées logiquement (par un identifiant tenant_id dans chaque table). Ce modèle réduit les coûts mais aggrave le risque de souveraineté : une faille d’isolation (un oubli de clause WHERE, un bug d’autorisation) peut exposer les données d’un client à un autre. Les attaques de type side-channel (extraction d’information via des mesures indirectes comme le timing ou la consommation mémoire) sont des vecteurs documentés dans les environnements partagés. 

## **1.4 Le problème géopolitique** 

La souveraineté numérique est devenue un enjeu géopolitique de premier ordre. Quelques chiffres clés illustrent l’ampleur du problème : 

- 97% de l’infrastructure cloud européenne est détenue par des acteurs non-européens (principalement américains). 

5 

Sovereign SaaS Framework — Note de cadrage v2.0 

- 67% de l’infrastructure cloud néerlandaise est fournie par Google, Amazon et Microsoft (Cour des comptes des Pays-Bas). 

- 78% des entreprises allemandes s’estiment trop dépendantes des fournisseurs cloud américains (enquête Bitkom). 

- 45% des organisations et 56% des digital natives citent la souveraineté des données comme préoccupation principale pour 2026 (IDC, juin 2025). 

- Près de 9 organisations digital-native sur 10 prévoient d’augmenter leur budget de protection des données SaaS (IDC, 2025). 

Dans  un  contexte  de  tensions  géopolitiques  (guerre  en  Ukraine,  tensions  USA-Chine, sanctions contre l’Iran et la Russie), la dépendance à un éditeur SaaS étranger devient un risque stratégique. Un gouvernement peut du jour au lendemain sanctionner un pays, couper l’accès à un service cloud, ou exiger des données sous contrainte légale. 

## **1.5 La tension fondamentale : les deux secrets** 

Le projet tente de résoudre ce qui ressemble à un paradoxe. Pour garantir la souveraineté des données, il faut que les données restent chez le client et que le code vienne vers elles. Pour protéger la propriété intellectuelle de l’éditeur, il faut que le code reste chez l’éditeur et que les données viennent vers lui. Ces deux exigences sont en contradiction directe. Toute l’ambition du projet se résume à cette question : comment faire collaborer deux secrets sans qu’aucun ne soit révélé à l’autre partie ? 

Cette question n’est pas nouvelle en cryptographie. Elle a été formalisée sous différentes formes :  le problème des millionnaires de Yao (1982), le calcul multi-parties sécurisé (Goldreich, Micali, Wigderson, 1987), le chiffrement homomorphe (Gentry, 2009). Ce qui est nouveau, c’est de l’appliquer à l’échelle d’une application SaaS complète en temps réel. 

## **1.6 Études de cas détaillées** 

## _**Étude de cas 1 — L’hôpital et le SaaS de gestion hospitalière**_ 

Un éditeur français de logiciel de gestion hospitalière répond à un appel d’offres d’un hôpital universitaire en Allemagne. Le logiciel gère les dossiers patients, les prescriptions, les plannings du personnel soignant. L’hôpital exige que les données des patients ne quittent jamais l’Allemagne et que personne en dehors de l’Allemagne n’y ait accès, conformément au RGPD  et  à  la  loi  fédérale  allemande  sur  la  protection  des  données  de  santé (Bundesdatenschutzgesetz). L’éditeur propose son SaaS classique hébergé en France. L’hôpital refuse : les administrateurs de l’éditeur en France auraient un accès technique aux données. L’éditeur propose une version on-premise. L’hôpital refuse : il n’a ni l’équipe ni le budget pour opérer le logiciel en interne. Le deal est bloqué. Montant du contrat : 2,4M€ sur 5 ans. Un framework souverain aurait débloqué cette situation. 

## _**Étude de cas 2 — Sanctions géopolitiques et rupture de service**_ 

Un éditeur américain de CRM (gestion de la relation client) compte 3 000 entreprises clientes en Turquie. Suite à une crise diplomatique, le gouvernement américain impose des restrictions commerciales. L’éditeur, soumis aux lois américaines, est contraint de suspendre le service pour ses clients turcs avec un préavis de 60 jours. Les entreprises turques doivent migrer en urgence vers une alternative, tout en essayant d’exporter leurs données clients accumulées sur des années. L’export est partiel : les pièces jointes, les historiques de communication, et les workflows automatisés ne sont pas inclus dans l’export standard. Certaines entreprises 

6 

Sovereign SaaS Framework — Note de cadrage v2.0 

perdent des années de données commerciales. Avec un framework souverain, les données seraient restées chez les entreprises turques et auraient survécu à la coupure du service. 

## _**Étude de cas 3 — Faillite de l’éditeur**_ 

Un éditeur SaaS de comptabilité pour PME levèe 15M€ de financement, accumule 12 000 clients, mais ne trouve pas la rentabilité et fait faillite. Les serveurs seront éteints dans 90 jours. Les clients reçoivent un email les invitant à exporter leurs données. Le format d’export est un CSV contenant les écritures comptables, mais pas les pièces justificatives (factures scannées), ni l’historique des modifications, ni les paramètres de rapprochement bancaire. 60% des PME n’exportent pas dans les 90 jours (par manque de temps, de compétences, ou parce qu’elles n’ont pas lu l’email). Elles perdent l’intégralité de leur comptabilité. 

## _**Étude de cas 4 — L’audit de conformité qui bloque**_ 

Une banque européenne utilise un SaaS américain de gestion des risques. Lors d’un audit de conformité DORA (Digital Operational Resilience Act, en vigueur depuis janvier 2025), les auditeurs demandent où sont stockées les données, qui y a accès, et quelles sont les mesures en cas de défaillance du prestataire. L’éditeur ne peut pas garantir qu’aucun employé américain n’accède aux données. L’audit identifie un risque de non-conformité. La banque est mise en demeure par le régulateur de migrer vers une solution conforme sous 12 mois, sous peine de sanctions pouvant atteindre 2% de son chiffre d’affaires mondial. 

## _**Étude de cas 5 — Le vol de propriété intellectuelle**_ 

Un éditeur d’ERP spécialisé dans la supply chain pharmaceutique décide de livrer son logiciel en on-premise pour répondre aux exigences de souveraineté d’un laboratoire. Six mois plus tard,  un  concurrent  lance  un  produit  étonnamment  similaire.  L’enquête  révèle  qu’un administrateur  système  du  laboratoire  a  décompilé  le  code,  extrait  les  algorithmes  de planification propriétaires, et les a revendus. L’éditeur entame une procédure judiciaire, mais la preuve du vol est difficile à établir et la procédure coûte plus cher que le préjudice. Cet exemple illustre pourquoi la souveraineté du client ne doit pas se faire au détriment de la protection de l’éditeur. 

7 

Sovereign SaaS Framework — Note de cadrage v2.0 

## **2. Le paysage réglementaire** 

Cette section détaille les réglementations qui créent la demande de souveraineté. Chaque réglementation  est  décrite  avec  son  périmètre,  ses  obligations,  ses  sanctions,  et  ses implications spécifiques pour notre projet. 

|**Réglem**<br>**entatio**<br>**n**|**Périmètre**|**Entrée**<br>**en**<br>**vigueur**|**Sanctions**|**Implication pour le projet**|
|---|---|---|---|---|
|RGPD|Données<br>personnelles, UE +<br>entités  traitant  des<br>données<br>d’Européens|Mai<br>2018|Jusqu’à 4%<br>CA mondial<br>ou 20M€|Exige  des  mesures  techniques  de<br>protection, limite les transferts hors<br>UE, impose la portabilité|
|CLOUD<br>Act|Données  détenues<br>par des entreprises<br>US, partout dans le<br>monde|Mars<br>2018|Sanctions<br>pénales<br>et<br>civiles|Rend tout SaaS américain<br>potentiellement<br>non-souverain,<br>même hébergé en UE|
|EU Data<br>Act|Données<br>non-<br>personnelles, objets<br>connectés, services<br>cloud|Septem<br>bre<br>2025|Sanctions<br>similaires au<br>RGPD|Impose la portabilité des données et<br>l’interopérabilité entre services cloud|
|NIS2|Sécurité<br>des<br>infrastructures<br>essentielles<br>et<br>importantes|Octobre<br>2024|Jusqu’à 10M€<br>ou 2% CA|Exige la maîtrise de la chaîne<br>d’approvisionnement<br>numérique,<br>incluant les SaaS|
|DORA|Résilience<br>opérationnelle du<br>secteur financier|Janvier<br>2025|Sanctions<br>prudentielles|Exige des plans de sortie de<br>prestataires cloud, des tests de<br>résilience|
|Schrem<br>s II|Transferts UE-US<br>de<br>données<br>personnelles|Juillet<br>2020|Invalidation<br>des<br>mécanismes<br>de transfert|Crée une insécurité juridique<br>permanente pour tout SaaS<br>américain|
|HIPAA|Données de santé,<br>USA|1996<br>(renforc<br>é<br>en<br>continu)|Jusqu’à<br>1,5M$ par<br>violation|Exige des contrôles stricts sur qui<br>accède aux données de santé|
|PIPL<br>(Chine)|Données<br>personnelles<br>traitées par des<br>entités chinoises ou<br>étrangères|Novemb<br>re 2021|Jusqu’à 50M¥<br>ou 5% CA|Exige la localisation des données en<br>Chine pour certaines catégories|
|DPDP<br>(Inde)|Données<br>personnelles des<br>citoyens indiens|2023|Jusqu’à 250<br>crore INR|Restrictions sur les transferts<br>transfrontaliers|



Le cumul de ces réglementations crée un mille-feuille juridique que les éditeurs SaaS doivent naviguer.  Un  éditeur  qui  vend  son  logiciel  à  des  clients  dans  20  pays  doit  respecter 

8 

Sovereign SaaS Framework — Note de cadrage v2.0 

potentiellement 20 régimes réglementaires différents. Un framework souverain qui élimine par conception le transit des données vers l’éditeur simplifie drastiquement cette conformité. 

9 

Sovereign SaaS Framework — Note de cadrage v2.0 

## **3. Cartographie des contraintes** 

La conception du framework doit naviguer dans un espace de contraintes multidimensionnel. Ces contraintes sont souvent en tension : optimiser l’une dégrade souvent l’autre. Chaque décision technique est un compromis explicite. 

|**Contraint**<br>**e**|**Description**|**Tension**<br>**avec**|**Indicateur de mesure**|
|---|---|---|---|
|Performan<br>ce|L’expérience<br>utilisateur ne doit pas<br>être dégradée au-<br>delà d’un seuil<br>perceptible|Souverain<br>eté,<br>sécurité|Latence  p95,  throughput,  temps  de  réponse<br>page|
|Coût|Surcoût<br>max<br>acceptable : 30-50%<br>vs SaaS classique|Souverain<br>eté,<br>résilience|TCO comparé, coût par utilisateur par mois|
|Simplicité<br>d’intégrati<br>on|Un éditeur doit<br>intégrer le framework<br>en semaines, pas en<br>mois|Puissance<br>du<br>framework|Temps d’intégration, lignes de code spécifiques|
|Souverain<br>eté des<br>données|Les données ne<br>doivent  jamais  être<br>accessibles en clair à<br>l’éditeur|Performan<br>ce,<br>fonctionnal<br>ité|Surface d’exposition mesurée par audit|
|Protection<br>PI|Le code de l’éditeur<br>ne doit pas être<br>extractible|Souverain<br>eté<br>(déploiem<br>ent local)|Résistance au reverse engineering mesurée|
|Scalabilité|De 5 à 5 000<br>utilisateurs, même<br>architecture|Uniformité,<br>coût|Performance linéaire vs exponentielle|
|Résilience|RPO et RTO définis<br>par client|Simplicité,<br>coût|RPO/RTO mesurés en exercice|
|Conformit<br>é|Multi-réglementation<br>(RGPD, CLOUD Act,<br>NIS2, DORA...)|Universalit<br>é|Nombre de réglementations couvertes|
|Opérabilité|Mises à jour, support,<br>debugging restent<br>possibles|Isolation|Temps de résolution de bug moyen|
|Portabilité|Le client peut migrer<br>ses<br>données<br>librement|Lock-in<br>commerci<br>al|Format d’export, temps de migration|



10 

Sovereign SaaS Framework — Note de cadrage v2.0 

## **4. État de l’art : paradigmes existants** 

Sept approches tentent de répondre à la problématique. Cette section les analyse en détail avec des exemples réels, et identifie précisément ce que chacune résout et ce qu’elle laisse ouvert. 

## **4.1 SaaS avec résidence locale** 

**Principe :** l’éditeur déploie ses serveurs dans le pays du client (régions cloud locales). 

**Exemples réels :** Microsoft Azure France, AWS Europe (Paris, Francfort), Google Cloud Europe. 

**Ce que ça résout :** la conformité aux exigences de résidence des données. 

**Ce que ça ne résout pas :** le CLOUD Act s’applique toujours. L’éditeur et ses employés ont un accès technique complet. L’arrêt Schrems II a jugé cette approche insuffisante. 

## **4.2 Cloud souverain** 

**Principe :** des acteurs sous juridiction locale opèrent l’infrastructure. 

**Exemples  réels  :** OVHcloud  (France),  Scaleway  (France),  NumSpot  (joint-venture Docaposte/Dassault/Bouygues), S3NS (partenariat Thales/Google avec SecNumCloud), Bleu (partenariat Orange/Capgemini/Microsoft). 

**Ce que ça résout :** la souveraineté de l’infrastructure (matériel, opérations). 

**Ce que ça ne résout pas :** si le logiciel SaaS est d’un éditeur étranger, le problème de la juridiction logicielle persiste. C’est une souveraineté de l’infrastructure sans souveraineté du logiciel. 

## **4.3 On-premise / Self-hosted** 

**Principe :** le client installe et opère le logiciel sur sa propre infrastructure. 

**Exemples réels :** SAP On-Premise, Microsoft Exchange Server (avant la migration vers Exchange Online), GitLab Self-Managed. 

**Ce que ça résout :** souveraineté totale des données. 

**Ce que ça ne résout pas :** coût élevé, retard sur les mises à jour, charge ops, variabilité des environnements. Et la PI de l’éditeur est exposée (le code est chez le client). 

## **4.4 BYOC (Bring Your Own Cloud)** 

**Principe :** séparation control plane (chez l’éditeur) / data plane (dans le VPC du client). 

**Exemples réels :** Databricks (pionnier depuis 2013), Redpanda (Data Plane Atomicity), ClickHouse (zero-trust via Tailscale), Confluent/WarpStream, Aiven (custom clouds). 

**Concept clé — Data Plane Atomicity (Redpanda) :** le data plane n’a aucune dépendance externe pour fonctionner. Même si le control plane de l’éditeur tombe, le système du client continue de fonctionner. C’est la propriété la plus avancée du BYOC. 

11 

Sovereign SaaS Framework — Note de cadrage v2.0 

**Ce que ça résout :** souveraineté forte, mises à jour gérées par l’éditeur, coexistence des deux mondes. 

**Ce que ça ne résout pas :** le control plane peut avoir un accès résiduel. Le modèle est prouvé pour l’infrastructure data mais pas pour les SaaS métier. Le client doit avoir un compte cloud. 

## **4.5 Chiffrement côté client** 

**Principe :** le client chiffre toutes ses données avant envoi. L’éditeur ne voit que du bruit. 

**Exemples réels :** Proton Mail (email), Tresorit (stockage), SpiderOak (backup), Signal (messagerie). 

**Ce que ça résout :** souveraineté totale. L’éditeur est aveugle. 

**Ce que ça ne résout pas :** l’éditeur ne peut plus traiter les données. Impossible de faire une recherche, un tri, une agrégation, un dashboard. Fonctionne pour le stockage pur, pas pour les applications métier. 

## **4.6 Code-to-Data** 

**Principe :** l’éditeur envoie le code (image conteneur) chez le client. Tout tourne localement. 

**Exemples réels :** Docsie (documentation, déploiement on-premise en 25 minutes), certains modules Palantir. 

**Ce que ça résout :** souveraineté totale des données, fonctionnement hors-ligne possible. 

**Ce que ça ne résout pas :** la PI de l’éditeur est exposée. Le client doit avoir du compute. Le support à distance est complexe. 

## **4.7 Décentralisation (Solid, Web3)** 

**Principe :** les données sont dans des pods contrôlés par l’utilisateur. Les applications demandent la permission d’y accéder. 

**Exemples réels :** Solid/Inrupt (Tim Berners-Lee), IPFS/Filecoin (stockage décentralisé), Storj, Arweave. 

**Ce que ça résout :** souveraineté totale, portabilité native, pas de vendor lock-in. 

**Ce que ça ne résout pas :** maturité insuffisante, performance, adoption quasi nulle en entreprise, complexité d’intégration. 

12 

Sovereign SaaS Framework — Note de cadrage v2.0 

## **5. Technologies habilitantes : analyse détaillée** 

Cette section explique en profondeur chaque technologie qui pourrait contribuer à la solution. Pour chaque technologie : comment elle fonctionne (expliqué simplement), à quel aspect du problème elle répond, son niveau de maturité, ses limites, et les acteurs clés. 

## **5.1 Chiffrement homomorphe (FHE)** 

Le chiffrement classique protège les données au repos (stockées) et en transit (envoyées). Mais pour utiliser les données, il faut les déchiffrer. Le chiffrement homomorphe résout ce problème : il permet de faire des calculs directement sur les données chiffrées. Le résultat du calcul, une fois déchiffré, est identique à ce qu’on aurait obtenu en clair. 

_Comment ça marche (simplifié) : Imaginez un cadenas spécial qu’on peut mettre autour de deux nombres. Quelqu’un peut additionner les deux nombres « à travers » le cadenas, sans le déverrouiller. Quand vous déverrouillez, vous trouvez le bon résultat. C’est exactement ce que fait le FHE, mais avec n’importe quelle opération mathématique._ 

**Niveau de maturité :** recherche avancée, début de commercialisation. Les performances restent 10⁴ à 10⁶ fois plus lentes que le calcul en clair. Des accélérateurs matériels (FPGA, ASIC) sont en développement. Viabilité commerciale partielle estimée d’ici 2-3 ans pour des opérations simples. 

**Acteurs clés :** Zama (startup française, spécialisée FHE, levée 73M$), Microsoft SEAL (bibliothèque open-source), IBM HElib, Intel HEXL, PALISADE/OpenFHE. 

**Pertinence pour le projet :** si le FHE atteint des performances acceptables, il résout définitivement la tension fondamentale : l’éditeur traite les données sans jamais les voir. Le code reste chez l’éditeur, les données restent chiffrées partout. Mais on ne peut pas construire le MVP sur le FHE à l’état actuel. 

**Sous-types à connaître :** PHE (Partially Homomorphic, ne supporte qu’un type d’opération, beaucoup  plus  rapide),  SHE  (Somewhat  Homomorphic,  supporte  un  nombre  limité d’opérations), FHE (Fully Homomorphic, supporte toute opération, beaucoup plus lent). Le PHE  pourrait  être  suffisant  pour  certaines  opérations  métier  spécifiques  (agrégations, comptages). 

## **5.2 Confidential Computing (TEE)** 

Un TEE (Trusted Execution Environment) est une zone matérielle isolée dans un processeur où le code et les données sont protégés même contre l’administrateur de la machine. Personne, pas même celui qui possède le serveur, ne peut lire ce qui se passe dans l’enclave. 

_Comment ça marche (simplifié) : Le processeur crée une « pièce fermée à clé » dans sa propre mémoire. Le code et les données sont chargés dans cette pièce, déchiffrés à l’intérieur,  traités,  puis  les  résultats  sont  rechiffrés  avant  de  sortir.  Le  système d’exploitation lui-même ne peut pas regarder à l’intérieur. Un mécanisme d’attestation permet à un tiers de vérifier cryptographiquement que le code dans l’enclave est bien celui attendu et n’a pas été modifié._ 

**Implémentations :** Intel SGX (Software Guard Extensions, enclaves applicatives), Intel TDX (Trust Domain Extensions, VMs entières), AMD SEV-SNP (Secure Encrypted Virtualization, 

13 

Sovereign SaaS Framework — Note de cadrage v2.0 

VMs avec intégrité), ARM CCA (Confidential Compute Architecture), AWS Nitro Enclaves, IBM Secure Execution. 

**Niveau de maturité :** disponible en production. Les principaux cloud providers proposent des VMs confidentielles (Azure Confidential VMs, GCP Confidential VMs, AWS Nitro). NVIDIA a publié en mars 2026 une architecture de référence pour des AI factories zero-trust. 

**Limites connues :** vulnérabilités aux attaques par canaux auxiliaires (side-channel attacks) documentées  dans  la  littérature  (Foreshadow,  Plundervolt,  SGAxe).  Certaines  ont  été corrigées, d’autres restent des risques théoriques. La taille de l’enclave peut limiter les applications complexes. 

**Pertinence pour le projet :** les TEE sont la technologie la plus prometteuse à court terme. Ils résolvent potentiellement les deux côtés de la tension : le code de l’éditeur tourne dans l’enclave (le client ne peut pas le lire), les données du client sont traitées dans l’enclave (l’éditeur ne peut pas les lire). L’attestation permet à chaque partie de vérifier que l’autre respecte les règles. 

## **5.3 Calcul multi-parties sécurisé (MPC)** 

Le MPC permet à plusieurs parties de calculer conjointement un résultat à partir de leurs entrées respectives, sans qu’aucune partie ne voie les entrées des autres. C’est le seul paradigme qui résout mathématiquement le problème de la défiance mutuelle. 

_Comment ça marche (simplifié, problème des millionnaires de Yao) : Deux millionnaires veulent savoir qui est le plus riche sans révéler leur fortune. Ils utilisent un protocole où chacun divise son montant en fragments aléatoires, échange certains fragments de manière contrôlée, et à la fin du protocole, ils obtiennent la réponse (« A est plus riche que B ») sans qu’aucun n’ait appris le montant de l’autre._ 

**Pertinence pour le projet :** dans notre contexte, l’éditeur fournit la logique de calcul (sans voir les données) et le client fournit les données (sans voir la logique). Le résultat émerge entre les deux. C’est théoriquement la solution parfaite à notre problème de « deux secrets ». 

**Limites :** les protocoles MPC sont lents (facteur 1000x à 100 000x par rapport au calcul en clair). Ils nécessitent une communication intense entre les parties. Inadapté pour un usage SaaS interactif en temps réel aujourd’hui. Mais adapté pour des opérations batch ou des vérifications ponctuelles. 

## **5.4 Preuves à connaissance nulle (ZKP)** 

Les ZKP permettent de prouver qu’une affirmation est vraie sans révéler pourquoi elle est vraie. 

_Exemple d’Ali Baba : Imaginez un tunnel circulaire avec une porte au milieu qui ne s’ouvre qu’avec un mot de passe. Alice entre par un côté aléatoire. Bob lui demande de ressortir par le côté qu’il choisit. Si Alice connaît le mot de passe, elle réussit à chaque fois. Si elle ne le connaît pas, elle échoue une fois sur deux. Après 20 répétitions réussies, Bob est convaincu qu’Alice connaît le mot de passe, sans qu’elle ne le lui ait révélé._ 

**Variantes à connaître :** zk-SNARKs (Succinct Non-interactive Arguments of Knowledge, petites  preuves,  setup  de  confiance  nécessaire),  zk-STARKs  (Scalable  Transparent Arguments of Knowledge, pas de setup de confiance, preuves plus grandes), Bulletproofs (sans setup, taille logarithmique). 

14 

Sovereign SaaS Framework — Note de cadrage v2.0 

**Pertinence pour le projet :** les ZKP pourraient permettre au client de prouver à l’éditeur que ses données respectent des règles métier sans révéler les données. Ou à l’éditeur de prouver au client que le bon code a été exécuté. Utile pour l’audit, la facturation, et la vérification d’intégrité. 

## **5.5 Autres technologies pertinentes** 

**Differential Privacy :** ajout de bruit statistique aux données agrégées pour garantir la non-réidentification. Utilisée par Apple (iOS) et Google (Chrome). Pertinent pour la télémétrie sans fuite. 

**SSI / DID (Self-Sovereign Identity) :** identité contrôlée par l’utilisateur, sans fournisseur central. Standard W3C. Pertinent pour découpler l’authentification de l’éditeur. 

**CRDTs (Conflict-free Replicated Data Types) :** structures de données fusionnables sans conflit. Utilisé par Figma, Linear. Pertinent pour le paradigme local-first où l’application tourne d’abord localement. 

**Conteneurs chiffrés (OCI Encrypted Images) :** images conteneur chiffrées, déchiffrées uniquement au runtime par une clé éphémère. Pertinent pour le modèle code-to-data avec protection PI. 

**WebAssembly (WASM) :** format d’exécution portable, isolé, performant. Peut tourner dans un navigateur ou en serveur. Redpanda l’utilise comme co-processeur dans son BYOC. 

## **5.6 Matrice de couverture technologies × problèmes** 

|**Technologie**|**Souveraineté**<br>**données**|**Protection PI**|**Performance**|**Maturité**<br>**production**|
|---|---|---|---|---|
|FHE|(parfaite)<br>★★★★★|(code<br>★★★★★<br>chez l’éditeur)|(très lent)<br>★|(labo/niche)<br>★★|
|TEE|(forte)<br>★★★★|(code dans<br>★★★★<br>l’enclave)|(proche<br>★★★★<br>du natif)|★★★★<br>(production)|
|MPC|(parfaite)<br>★★★★★|(code<br>★★★★★<br>chez l’éditeur)|(lent)<br>★★|(cas<br>★★<br>spécifiques)|
|ZKP|(vérification)<br>★★★|(vérification)<br>★★★|(variable)<br>★★★|★★★<br>(blockchain)|
|CSE|(parfaite)<br>★★★★★|(code<br>★★★★★<br>chez l’éditeur)|(pas<br>★★★★★<br>de surcoût)|(mais<br>★★★★★<br>ne traite pas)|
|CRDTs/Local-<br>first|(données<br>★★★★<br>locales)|(code<br>★★<br>distribué)|(local)<br>★★★★★|(Figma,<br>★★★<br>Linear)|



15 

Sovereign SaaS Framework — Note de cadrage v2.0 

## **6. Hypothèses fondatrices du projet** 

Le projet repose sur cinq hypothèses. Chaque hypothèse est formulée précisément, avec sa méthode de validation, ses critères de succès quantifiés, et les signaux qui indiqueraient qu’elle est fausse. 

## **H1 — La demande existe et est solvable** 

**Formulation :** il existe une demande réelle, mesurable en revenus perdus, de la part d’éditeurs SaaS qui perdent des deals dans les secteurs réglementés (santé, finance, défense, administration) à cause de l’incapacité à garantir la souveraineté des données. 

**Protocole de validation :** entretiens semi-directifs avec 15 à 20 éditeurs SaaS (B2B, 10 à 500 employés, marché européen). Guide d’entretien structuré autour de trois thèmes : fréquence des questions de souveraineté dans les cycles de vente, deals perdus à cause de cette question (montant, secteur, pays), et solutions de contournement déjà tentées. Compléter par une analyse de 50 appels d’offres publics dans les secteurs santé, finance et administration en France, Allemagne et Benelux. 

**Critère de succès :** au moins 60% des éditeurs interrogés rapportent avoir perdu au moins un deal significatif (>50K€) pour cette raison dans les 12 derniers mois. 

**Signe d’invalidation :** la souveraineté est un critère secondaire, facilement compensé par d’autres avantages (prix, fonctionnalités). Les éditeurs ne perdent pas de deals pour cette raison. Le marché adressable est trop petit (<50M€). 

## **H2 — La coexistence code souverain / données souveraines est possible** 

**Formulation :** il est techniquement possible de faire coexister un logiciel propriétaire et des données souveraines sur la même infrastructure sans que l’un compromette l’autre, en utilisant les TEE comme mécanisme d’isolation. 

**Protocole de validation :** Proof of Concept en 3 étapes. Étape 1 : déployer une application web simple (CRUD + recherche) dans une enclave Intel TDX ou AMD SEV-SNP. Étape 2 : mesurer si un administrateur root de la machine hôte peut extraire le code ou les données de l’enclave (test d’intrusion). Étape 3 : vérifier l’attestation de bout en bout (le client vérifie que c’est bien le code de l’éditeur, l’éditeur vérifie que les données ne sortent pas de l’enclave). 

**Critère de succès :** aucune extraction de code ou de données lors du test d’intrusion. Attestation fonctionnelle de bout en bout. 

**Signe d’invalidation :** les TEE sont compromises par une attaque side-channel reproductible dans notre configuration. Ou le modèle d’attestation est trop complexe pour être déployé en pratique. 

## **H3 — Les technologies sont assez mûres** 

**Formulation :** les TEE sont suffisamment matures pour héberger une application SaaS métier complète (pas seulement une démo) avec des performances acceptables. 

**Protocole de validation :** benchmark comparatif sur un scénario réaliste. Scénario de référence : application web avec 100 utilisateurs simultanés, 50% lectures / 30% écritures / 

16 

Sovereign SaaS Framework — Note de cadrage v2.0 

20% recherches, base de données de 10 Go. Mesurer : latence p50 et p95 par opération, throughput (requêtes/seconde), surcoût mémoire et CPU, en comparant enclave TEE vs déploiement natif. 

**Critère de succès :** surcoût de latence inférieur à 2x sur le p95. Throughput supérieur à 50% du natif. Surcoût mémoire inférieur à 30%. 

**Signe d’invalidation :** surcoût supérieur à 5x, rendant l’application inutilisable pour un usage interactif.  Ou  le  TEE  impose  des  contraintes  architecturales  incompatibles  avec  une application métier standard. 

## **H4 — Le surcoût est acceptable** 

**Formulation :** le coût total de possession (TCO) d’une architecture souveraine ne dépasse pas 150% du TCO d’un SaaS classique équivalent. 

**Protocole de validation :** modélisation économique complète sur 3 profils clients (10 utilisateurs, 100 utilisateurs, 1000 utilisateurs). Inclure le coût d’infrastructure, le surcoût de compute lié aux TEE, le coût d’intégration du framework, et soustraire les coûts évités (pas de frais d’egress data, conformité intégrée, pas de coût de migration en cas de changement d’éditeur). 

**Critère de succès :** TCO inférieur à 150% pour les 3 profils. De préférence inférieur à 130%. 

## **H5 — Un framework générique est possible** 

**Formulation :** un framework unique peut couvrir au moins 3 types de SaaS métier différents sans que le code spécifique à chaque type dépasse 20% du code total. 

**Protocole de validation :** intégrer le framework dans 3 applications de types différents (ex: gestion de projet, CRM, e-learning). Mesurer le ratio code framework générique / code spécifique pour chaque intégration. 

**Critère de succès :** code spécifique inférieur à 20% pour au moins 2 applications sur 3. 

17 

Sovereign SaaS Framework — Note de cadrage v2.0 

## **7. Méthodologie de recherche détaillée** 

## **7.1 Systematic Literature Review — Protocole PRISMA** 

La SLR suit le protocole PRISMA 2020 (Preferred Reporting Items for Systematic Reviews and Meta-Analyses), standard de référence publié dans le BMJ (Page et al., 2021). Le protocole est décrit ci-dessous dans son intégralité. 

## _**7.1.1 Questions de recherche**_ 

- RQ1 : Quels modèles architecturaux ont été proposés dans la littérature pour concilier souveraineté des données et distribution de logiciels en mode service ? 

- RQ2  :  Quelles  Privacy-Enhancing  Technologies  (PETs)  ont  été  appliquées  au problème de la défiance mutuelle (protection simultanée des données du client et du code de l’éditeur) ? 

- RQ3 : Quels sont les benchmarks de performance documentés pour les TEE, FHE et MPC dans des scénarios applicatifs réalistes ? 

- RQ4 : Quels modèles économiques ont été proposés pour des SaaS souverains, et quels sont les surcoûts documentés ? 

## _**7.1.2 Bases de données**_ 

|**Base**|**Spécialité**|**Couverture**|**Accès**|
|---|---|---|---|
|IEEE Xplore|Informatique, ingénierie|4,5M+ documents|Abonnement<br>institutionnel|
|ACM Digital Library|Informatique|3M+ documents|Abonnement<br>institutionnel|
|Springer Link|Multidisciplinaire|13M+ documents|Abonnement<br>institutionnel|
|ScienceDirect<br>(Elsevier)|Multidisciplinaire|18M+ documents|Abonnement<br>institutionnel|
|arXiv|Préprints|2,3M+ documents|Libre accès|
|Google Scholar|Multidisciplinaire|Agrégateur|Libre accès|
|DBLP|Informatique<br>(bibliographique)|6M+ références|Libre accès|



## _**7.1.3 Chaînes de recherche**_ 

Les chaînes de recherche sont construites en combinant quatre blocs sémantiques avec l’opérateur AND. Chaque bloc utilise l’opérateur OR pour couvrir les synonymes. 

**Bloc A (Problème) :** ("data sovereignty" OR "data residency" OR "data localization" OR "digital sovereignty") 

**Bloc B (Contexte) :** ("SaaS" OR "Software as a Service" OR "cloud computing" OR "multitenant") 

**Bloc  C  (Solution)  :** ("architecture"  OR  "framework"  OR  "model"  OR  "design"  OR "deployment") 

18 

Sovereign SaaS Framework — Note de cadrage v2.0 

**Bloc D (Technologies) :** ("confidential computing" OR "trusted execution" OR "homomorphic encryption" OR "secure multi-party computation" OR "zero-knowledge") 

Combinaisons à exécuter : A AND B AND C (pour RQ1), A AND D (pour RQ2), D AND "benchmark" AND "performance" (pour RQ3), A AND B AND ("cost" OR "pricing" OR "economic") (pour RQ4). Plus des combinaisons incluant "software protection" OR "intellectual property" OR "code confidentiality" pour couvrir le deuxième côté de la tension. 

## _**7.1.4 Critères d’inclusion et d’exclusion**_ 

|**Critère**|**Inclusion**|**Exclusion**|
|---|---|---|
|Période|2018 – 2026|Avant 2018 (sauf travaux fondateurs :<br>Gentry 2009, Yao 1982)|
|Langue|Anglais, français|Autres langues|
|Type<br>de<br>source|Articles peer-reviewed, conférences<br>ACM/IEEE, rapports techniques|Blog posts, articles de presse, opinions,<br>brevets|
|Pertinence|Traite  explicitement  de  souveraineté<br>des données ET d’architecture logicielle|Traite de souveraineté au sens politique<br>uniquement|
|Niveau<br>de<br>contribution|Propose ou évalue une solution<br>technique concrète|Reste purement descriptif sans<br>contribution originale|
|Contexte<br>applicatif|Applications métier, SaaS, cloud|Applications militaires classifiées,<br>théorie pure sans application|



## _**7.1.5 Processus de sélection (flow PRISMA)**_ 

Le processus suit les 4 phases du flow diagram PRISMA 2020 : 

**Phase  1  —  Identification  :** exécution  des  chaînes  de  recherche  dans  les  7  bases. Consolidation dans un gestionnaire de références (Zotero ou Mendeley). Déduplication automatique. 

**Phase 2 — Screening (titre + abstract) :** deux reviewers indépendants évaluent chaque article sur la base du titre et du résumé. Critères : pertinence par rapport aux RQ, respect des critères d’inclusion. Les désaccords sont résolus par discussion ou par un troisième reviewer. 

**Phase 3 — Éligibilité (full-text) :** lecture intégrale des articles retenus. Évaluation de la qualité méthodologique. Extraction des données dans une grille structurée. 

**Phase 4 — Inclusion :** articles finaux retenus pour l’analyse. Classement par catégorie (paradigme architectural, technologie, benchmark, modèle économique). 

## _**7.1.6 Grille d’extraction des données**_ 

Pour  chaque  article  retenu,  les  informations  suivantes  sont  extraites  dans  un  tableur structuré : 

|**Champ**|**Description**|
|---|---|
|ID|Identifiant unique de l’article|
|Auteurs, année, venue|Référence bibliographique complète|
|RQ adressée|Quelle(s) question(s) de recherche l’article adresse|



19 

Sovereign SaaS Framework — Note de cadrage v2.0 

|Paradigme|SaaS résidence locale / cloud souverain / BYOC / code-to-data / etc.|
|---|---|
|Technologies utilisées|TEE / FHE / MPC / ZKP / autre|
|Problème résolu|Souveraineté données / protection PI / les deux / autre|
|Méthode de validation|Prototype / benchmark / preuve formelle / simulation / aucune|
|Métriques<br>de<br>performance|Latence, throughput, surcoût, taille enclave|
|Limites reconnues|Ce que l’article reconnaît ne pas résoudre|
|Pertinence pour notre<br>projet|Évaluation subjective : faible / moyenne / forte / critique|



## **7.2 Protocole expérimental** 

Les  expérimentations  suivent  un  protocole  rigoureux.  Chaque  expérimentation  est documentée AVANT son exécution avec les éléments suivants : 

- Question : que cherche-t-on à démontrer ou mesurer ? 

- Hypothèse testée : à quelle hypothèse fondatrice (H1–H5) cette expérimentation contribue ? 

- Baseline : scénario de référence sans framework, pour comparaison. 

- Variables : ce qui change entre les scénarios testés. 

- Métriques : ce qu’on mesure, comment, avec quelle précision. 

- Environnement : matériel, logiciel, versions, configuration. 

- Critères de succès / échec : définis AVANT l’expérience. 

- Nombre de répétitions : chaque mesure est répétée au minimum 10 fois. Les résultats sont présentés avec moyenne, médiane, écart-type, et percentiles (p50, p95, p99). 

20 

Sovereign SaaS Framework — Note de cadrage v2.0 

## **8. Stratégie et phases du projet** 

## **8.1 Vue d’ensemble des phases** 

|**Phase**|**Durée**|**Focus recherche**|**Focus produit**|**Livrable clé**|
|---|---|---|---|---|
|1.<br>Exploratio<br>n|3–6<br>mois|SLR complète, état de<br>l’art, cartographie|PoC isolés (TEE,<br>FHE),<br>expérimentations|Rapport SLR + matrice<br>faisabilité|
|2.<br>Converge<br>nce|3–6<br>mois|Architecture<br>candidate,<br>comparaison formelle|Prototype<br>intégré<br>v0.1, choix du 1er<br>SaaS ACT|Architecture<br>documentée + article<br>soumis|
|3.<br>Validation|3–6<br>mois|Benchmarks<br>systématiques, tests<br>d’intrusion|Déploiement<br>réel<br>interne ACT, itérations|Résultats<br>expérimentaux<br>+<br>framework v1|
|4.<br>Ouverture|6+ mois|Publications,<br>conférences|Open source ou<br>licence,<br>documentation,<br>partenariats|Framework<br>v2<br>+<br>premiers<br>utilisateurs<br>externes|



## **8.2 Phase 1 — Exploration (détail)** 

## _**Semaines 1–4 : Immersion**_ 

- Lecture complète de la présente note de cadrage par toute l’équipe. 

- Formation sur les technologies clés (TEE, FHE, MPC) via les ressources de la section 11. 

- Installation des environnements de développement (Intel TDX/AMD SEV sur machine de test). 

- Définition du protocole SLR final (ajustement des chaînes de recherche après tests préliminaires). 

## _**Semaines 5–12 : SLR + Expérimentations isolées**_ 

- Exécution de la SLR : identification, screening, éligibilité, inclusion. 

- En parallèle : PoC TEE (déployer une app web simple dans une enclave, mesurer performances). 

- En parallèle : PoC FHE (exécuter des opérations simples avec Zama/SEAL, mesurer le facteur de ralentissement). 

- En parallèle : PoC conteneur chiffré (image OCI chiffrée, déchiffrement au runtime, test de résistance à l’extraction). 

## _**Semaines 13–16 : Synthèse**_ 

- Rédaction du rapport SLR. 

- Construction de la matrice de faisabilité (technologie × critère × niveau de maturité). 

- Décision Go/No-Go pour la phase 2 sur la base des résultats. 

- Si Go : choix de l’architecture candidate principale et de l’architecture alternative. 

21 

Sovereign SaaS Framework — Note de cadrage v2.0 

## **8.3 Critères de passage entre phases** 

Le passage d’une phase à la suivante n’est pas automatique. Il est conditionné par la validation de critères explicites : 

|**Passage**|**Critères obligatoires**|
|---|---|
|Phase 1 → Phase 2|SLR  terminée.  Au  moins  un  PoC  TEE  fonctionnel.  Aucune  hypothèse<br>fondatrice invalidée de manière rédhibitoire.|
|Phase 2 → Phase 3|Architecture documentée et revue par l’équipe. Prototype intégré fonctionnel<br>(CRUD + recherche + auth). Benchmarks préliminaires dans les seuils de H3.|
|Phase 3 → Phase 4|Déploiement réel sur au moins un produit ACT. Benchmarks de performance<br>validés. Tests d’intrusion passés. Modèle économique validé (H4).|



22 

Sovereign SaaS Framework — Note de cadrage v2.0 

## **9. Risques et mitigations** 

|**Risque**|**Pro**<br>**b.**|**Imp**<br>**act**|**Mitigation**|**Indicateur d’alerte**|
|---|---|---|---|---|
|TEE<br>compromis<br>(side-channel)|Mo<br>y.|Éle<br>vé|Architecture multi-couches ne<br>dépendant pas d’une seule tech.<br>Explorer<br>FHE/MPC<br>en<br>complément.|Publication d’une attaque<br>reproductible sur nos CPUs<br>cibles|
|FHE  trop  lent<br>pour<br>tout<br>usage interactif|Éle<br>vée|Mo<br>yen|Identifier  les  opérations  où  le<br>PHE suffit. Utiliser FHE<br>uniquement en batch.|Facteur de ralentissement > 10⁶<br>sur nos opérations cibles|
|Pas<br>de<br>demande<br>éditeurs (H1<br>fausse)|Mo<br>y.|Criti<br>que|Valider H1 en phase 1 avant tout<br>investissement lourd. Pivoter<br>vers B2C si nécessaire.|< 30% des éditeurs interrogés<br>rapportent des deals perdus|
|Framework<br>trop  spécifique<br>(H5 fausse)|Mo<br>y.|Mo<br>yen|Tester sur 3 types d’applications<br>dès la phase 2.|Code spécifique > 40% dès la 2e<br>intégration|
|Concurrent<br>avec produit<br>similaire|Fai<br>ble|Éle<br>vé|Avantage ACT = premier client.<br>Accélérer  la  validation  interne.<br>Veille concurrentielle mensuelle.|Annonce publique d’un produit<br>concurrent|
|Compétences<br>crypto<br>insuffisantes|Éle<br>vée|Éle<br>vé|Plan de formation (section 11).<br>Recruter ou collaborer avec un<br>expert universitaire.|Blocage technique de plus de 2<br>semaines sur un problème<br>crypto|
|Réglementatio<br>n<br>change<br>radicalement|Fai<br>ble|Var.|Conception modulaire. Veille<br>réglementaire continue.|Nouveau<br>règlement<br>contraignant publié|



23 

Sovereign SaaS Framework — Note de cadrage v2.0 

## **10. Organisation de l’équipe** 

## **10.1 Rôles nécessaires** 

|**Rôle**|**Responsabilités**|**Profil**|
|---|---|---|
|Responsable projet|Vision,  arbitrages,  relations  extérieures,<br>go/no-go|Senior,<br>compréhension<br>technique + business|
|Chercheur principal|SLR, protocoles expérimentaux, rédaction<br>scientifique|Compétences<br>méthodologiques, rédaction<br>académique|
|Développeur<br>sécurité / crypto|PoC TEE, FHE, MPC. Intégration crypto<br>dans le framework|Crypto appliquée, systèmes<br>distribués|
|Développeur<br>backend|Framework, API, intégration avec les SaaS<br>ACT|Python/Go/Rust,  PostgreSQL,<br>conteneurs|
|Analyste<br>réglementaire<br>(partiel)|Veille<br>juridique,<br>implications<br>réglementaires|Droit  du  numérique,  RGPD,<br>CLOUD Act|



## **10.2 Rituels** 

- Stand-up quotidien (15 min) : blocages, avancées, prochaines étapes. 

- Revue hebdomadaire (1h) : démonstration des avancées, revue des métriques, décisions. 

- Revue de phase (demi-journée) : bilan complet, évaluation des critères de passage, décision Go/No-Go. 

- Séminaire mensuel de lecture (2h) : présentation et discussion d’articles clés de la SLR. 

24 

Sovereign SaaS Framework — Note de cadrage v2.0 

## **11. Compétences et plan de formation** 

L’équipe doit acquérir des compétences spécifiques. Cette section détaille les ressources d’autoformation par domaine, ordonnées du plus accessible au plus avancé. 

## **11.1 Confidential Computing** 

- Introduction : article Wikipedia « Confidential Computing » + « Trusted Execution Environment ». 

- Intermédiaire : documentation Confidential Computing Consortium (confidentialcomputing.io). 

- Avancé : « Intel TDX Demystified » (Cheng et al., ACM Computing Surveys, 2024). Documentation AMD SEV-SNP. Blog NVIDIA « Building a Zero-Trust Architecture for Confidential AI Factories » (mars 2026). 

- Pratique : tutoriels Azure Confidential VMs, GCP Confidential VMs, gramine-sgx (bibliothèque pour porter des applications dans SGX). 

## **11.2 Chiffrement homomorphe** 

- Introduction : vidéo « What is Homomorphic Encryption? » (IBM Research, YouTube). 

- Intermédiaire : tutoriels Zama (zama.ai), documentation Microsoft SEAL. 

- Avancé : cours FHE.org, article de référence Craig Gentry (2009), surveys sur les performances FHE. 

## **11.3 MPC et ZKP** 

- Introduction : article Wikipedia « Secure multi-party computation » et « Zero-knowledge proof ». 

- Intermédiaire : études de cas Partisia. Survey « A Survey on the Applications of ZeroKnowledge Proofs » (arXiv, 2024). 

- Avancé : travaux fondateurs de Yao (1982), Goldreich-Micali-Wigderson (1987). 

## **11.4 BYOC et architectures souveraines** 

- Documentation Redpanda BYOC (concept de Data Plane Atomicity). 

- Architecture ClickHouse BYOC. Documentation Confluent WarpStream. 

- Article « On the future of cloud services and BYOC » (Jack Vanlightly, 2024). 

## **11.5 Méthodologie de recherche** 

- Guide Kitchenham : « Systematic Literature Reviews in Software Engineering ». 

- PRISMA 2020 Statement (Page et al., BMJ 2021). 

- PRISMA-P pour la rédaction de protocoles (Moher et al., 2015). 

25 

Sovereign SaaS Framework — Note de cadrage v2.0 

## **12. Glossaire complet** 

Définitions de référence de tous les termes techniques utilisés dans ce document. 

- **API :** Application Programming Interface. Interface standardisée de communication entre logiciels. 

- **Attestation :** procédure cryptographique par laquelle un TEE prouve à un tiers que le code qu’il exécute est intègre et non modifié. 

- **BYOC :** Bring Your Own Cloud. Modèle de déploiement où le data plane tourne dans le VPC du client. 

- **BYOK / HYOK / CMK :** Modèles de gestion de clés où le client contrôle ses propres clés de chiffrement. 

- **CLOUD Act :** Clarifying Lawful Overseas Use of Data Act (USA, 2018). Accès extraterritorial aux données d’entreprises américaines. 

- **Conteneur Docker :** unité logicielle standardisée encapsulant une application et toutes ses dépendances. 

- **Control Plane :** partie du système qui gère l’orchestration (mises à jour, monitoring) sans toucher aux données. 

- **CRDTs :** Conflict-free Replicated Data Types. Structures de données fusionnables sans conflit. 

- **Data Plane :** partie du système où les données sont stockées et traitées. 

- **Data Plane Atomicity :** propriété d’un data plane sans dépendance externe (concept Redpanda). 

- **Differential  Privacy  :** technique  de  bruit  statistique  garantissant  la  non-réidentification individuelle. 

- **DID :** Decentralized Identifier. Standard W3C d’identifiants numériques décentralisés. 

- **DORA :** Digital Operational Resilience Act. Résilience opérationnelle du secteur financier (UE, 2025). 

- **Enclave :** zone isolée dans un processeur protégée même contre l’administrateur système. 

- **Federated Learning :** entraînement d’IA distribué où les données restent locales. 

- **FHE :** Fully Homomorphic Encryption. Calcul sur données chiffrées sans déchiffrement. 

- **Gaia-X :** initiative européenne de cloud fédéré souverain. 

- **IPFS :** InterPlanetary File System. Stockage distribué content-addressed. 

- **MPC / SMPC :** Secure Multi-Party Computation. Calcul conjoint sans révélation des entrées. 

- **mTLS :** Mutual TLS. Authentification réciproque par certificats. 

- **MVP  :** Minimum  Viable  Product.  Version  minimale  suffisante  pour  valider  une hypothèse. 

- **NIS2 :** Network and Information Security Directive 2. Sécurité infra critiques (UE, 2024). 

- **OCI :** Open Container Initiative. Standard d’images conteneur. 

26 

Sovereign SaaS Framework — Note de cadrage v2.0 

- **PHE :** Partially Homomorphic Encryption. Supporte un seul type d’opération (addition ou multiplication). 

- **PI :** Propriété Intellectuelle. 

- **POD :** Personal Online Data Store (projet Solid). 

- **PRISMA :** Preferred Reporting Items for Systematic Reviews and Meta-Analyses. Standard de référence pour les SLR. 

- **RGPD :** Règlement Général sur la Protection des Données (UE, 2018). 

- **RPO :** Recovery Point Objective. Perte de données maximale tolérée (en temps). 

- **RTO :** Recovery Time Objective. Durée d’indisponibilité maximale tolérée. 

- **SaaS :** Software as a Service. Logiciel distribué comme service via Internet. 

- **Schrems II :** arrêt CJUE (2020) invalidant le Privacy Shield UE-US. 

- **SecNumCloud :** qualification sécurité cloud de l’ANSSI (France). 

- **Side-channel attack :** attaque exploitant des informations physiques indirectes. 

- **SLR :** Systematic Literature Review. 

- **Solid :** projet de Tim Berners-Lee de découplage données/applications. 

- **SSI :** Self-Sovereign Identity. Identité contrôlée par l’individu. 

- **TCO :** Total Cost of Ownership. Coût total de possession sur la durée. 

- **TEE :** Trusted Execution Environment. Zone matérielle isolée dans un processeur. 

- **VPC :** Virtual Private Cloud. Portion isolée d’un cloud public. 

- **WASM :** WebAssembly. Format d’exécution portable et isolé. 

- **ZKP :** Zero-Knowledge Proof. Preuve de véracité sans révélation de l’information. 

_Sovereign SaaS Framework — Note de cadrage v2.0 — ACT — Mars 2026_ 

27 

