# Fiche d'entraînement — Questions / Réponses du Jury

> Soutenance PFE — Jesse MPIGA-ODOUMBA · أمان (Amān) — Framework SaaS Souverain
> Réponses-modèles **courtes** (à dire à l'oral, pas à réciter). Adapte avec tes mots.

---

## 1. Entreprise & contexte

**Q. Présentez votre entreprise d'accueil en une phrase.**
> AL BARAA CONSULTING est un cabinet de conseil et d'ingénierie numérique basé à Casablanca (fondé en 2017), qui réalise des solutions sur mesure pour des clients publics et privés — ministères, agences d'urbanisme, entreprises — notamment en SIG et systèmes d'information.

**Q. En quoi le projet est-il pertinent pour leurs clients ?**
> Leurs clients (Ministère de l'Intérieur, de l'Urbanisme, Agence Urbaine de Rabat, Commune de Casablanca…) manipulent des données très sensibles — foncier, urbanisme, citoyens. Ils ont une exigence forte de garder ces données sous contrôle local. Amān répond exactement à ce besoin : un logiciel en mode SaaS sans confier les données à un cloud.

**Q. Quel était votre rôle ?**
> J'étais développeur et architecte principal du projet, avec une responsabilité complète : conception, développement, déploiement et validation, en autonomie, avec des points réguliers avec ma DG, Mme Chokri.

---

## 2. Problématique & souveraineté

**Q. Quelle est la problématique en une phrase ?**
> Comment un éditeur peut-il vendre un logiciel métier en SaaS — comptes, licences, mises à jour — tout en étant *cryptographiquement incapable* de lire les données de ses clients ?

**Q. Qu'est-ce que le zero-knowledge concrètement ?**
> Ça veut dire que l'éditeur héberge des sauvegardes chiffrées qu'il ne peut pas ouvrir, parce qu'il ne possède aucune clé. Même saisi ou piraté, le relais ne contient que des octets chiffrés inexploitables.

**Q. C'est juste une promesse contractuelle ?**
> Non, c'est une propriété **cryptographique**. L'éditeur ne *peut pas* lire les données, même s'il le voulait : la clé (DEK) n'existe que sur les machines de la PME et sous son code de récupération, que l'éditeur ne connaît jamais.

**Q. C'est quoi l'AUDPF ?**
> L'AU Data Policy Framework, validé par l'Union Africaine fin 2025 : il pose que les données des organisations africaines doivent rester sous contrôle local. Amān est conforme **par conception**, pas par configuration.

---

## 3. Architecture & technique

**Q. Décrivez l'architecture en 30 secondes.**
> Trois acteurs : un **SaaS éditeur** (Django) qui gère comptes et licences ; un **relais zero-knowledge** qui stocke des blobs chiffrés opaques ; et des **clusters PME** qui exécutent le logiciel et détiennent les données en clair, sur leurs propres machines. Les données ne sortent que chiffrées.

**Q. Pourquoi Rust pour le cœur ?**
> Pour la sûreté mémoire et la performance, et surtout pour avoir **un seul cœur** réutilisable desktop et mobile (via UniFFI), sans le réécrire deux fois.

**Q. Quelle crypto, et l'avez-vous écrite vous-même ?**
> Non — règle stricte : **aucune primitive réinventée**. Tout passe par **libsodium** : XChaCha20-Poly1305 pour les données, X25519 pour l'identité des appareils, Argon2id pour le code de récupération, sealed box pour l'enrôlement.

**Q. Comment fonctionne la réplication ?**
> PostgreSQL en primaire/standby avec **streaming WAL**. J'ai ajouté un **slot de réplication** pour que le primaire ne recycle jamais le WAL dont le standby a besoin — ce qui élimine les ruptures au redémarrage.

**Q. Qu'est-ce que le fencing et pourquoi ?**
> C'est ce qui empêche un ancien primaire déchu de revenir écrire et de créer un **split-brain** (deux primaires). J'utilise le **timeline natif de PostgreSQL** comme jeton d'époque : un nœud avec un timeline inférieur à celui du cluster se clôture lui-même. Validé sur le banc.

**Q. Pourquoi pas un algorithme de consensus maison (Raft/Paxos) ?**
> Réinventer un consensus serait une faute d'ingénierie : risque élevé, valeur nulle face à des briques éprouvées. J'utilise la promotion de standby PostgreSQL, supervision type Patroni.

**Q. L'agent IA fait quoi exactement ? Est-il en production ?**
> Il supervise la **santé** des trois acteurs (réplication, époque, heartbeats) et produit un diagnostic + score de risque via Mistral. Il est **implémenté et opérationnel**, et respecte le zero-knowledge : il ne lit que des métriques d'infrastructure, jamais les données métier. La couche **prédictive** (apprendre les dérives) est la prochaine étape.

**Q. Et si le relais est saisi ou piraté ?**
> Rien d'exploitable. Il ne stocke que des blobs chiffrés et une copie de la DEK scellée sous le code de récupération du client. Sans ce code — que l'éditeur ne connaît pas — aucun déchiffrement n'est possible.

**Q. Comment une PME récupère ses données après un sinistre total ?**
> Elle contacte l'éditeur, qui lui restitue son coffre chiffré (qu'il n'a jamais pu lire). Elle l'ouvre avec **son code de récupération** et redéchiffre ses données sur une nouvelle machine.

---

## 4. Modèle économique

**Q. Si le logiciel est open source, comment gagnez-vous de l'argent ?**
> On ne vend pas le logiciel, ni l'hébergement. On facture l'**implémentation souveraine clé-en-main** : déploiement, licences par poste, relais, mises à jour, support. La PME tourne sur ses propres équipements.

**Q. Pourquoi est-ce moins cher qu'un SaaS classique ?**
> Parce que l'éditeur n'a **pas** le coût du cloud des données clients. D'où un prix par poste structurellement plus bas — à 10 postes, ~70 % d'économie vs un SaaS classique — et **aucun coût caché** : ni facturation au Go, ni frais de sortie de données.

---

## 5. Gestion de projet ⭐

**Q. Quelle méthodologie de gestion de projet avez-vous suivie ?**
> Une approche **itérative et incrémentale**, structurée en **phases**, avec une règle d'or : *« le socle d'abord, le métier ensuite »*. J'ai d'abord dérisqué le cœur technique avant d'écrire la moindre logique métier.

**Q. Comment avez-vous découpé le projet ?**
> En 4 phases : **Phase 0** — spike de dérisquage (prouver crypto + réplication + failover sur 3 machines réelles) ; **Phase 1** — SaaS éditeur (comptes, licences) ; **Phase 2** — relais zero-knowledge ; **Phase 3** — module métier. Les phases 0 et 1 pouvaient avancer en parallèle.

**Q. Pourquoi un "spike de dérisquage" en premier ?**
> Parce que si le socle (crypto, réplication, failover) change après coup, tout le métier bâti dessus est à réécrire. Le spike prouve les fondations **avant** d'investir dans le métier — c'est de la **gestion du risque** : on attaque l'incertitude maximale en premier.

**Q. Comment avez-vous priorisé ?**
> Par le risque et la dépendance : ce qui est le plus incertain et dont tout dépend (le socle crypto/réplication) en premier ; ce qui est connu et indépendant (l'UI, le confort) plus tard. J'ai aussi appliqué YAGNI — ne pas construire de fonctionnalités spéculatives.

**Q. Comment avez-vous géré les risques techniques ?**
> Trois leviers : (1) le spike Phase 0 qui valide les fondations tôt ; (2) le choix de **briques éprouvées** (PostgreSQL, libsodium, Docker) plutôt que du code maison risqué ; (3) un banc de test multi-OS réel (Windows, Ubuntu, Kali, Debian) pour découvrir les problèmes de portabilité tôt.

**Q. Comment avez-vous planifié sur 24 semaines ?**
> Stage du 19 février au 1er juillet 2026. Découpage par phases avec des jalons : socle validé, SaaS éditeur fonctionnel, relais, puis tests d'intégration. Le suivi se faisait par points réguliers avec Mme Chokri et un journal technique tenu au fil de l'eau.

**Q. Quels outils de gestion / suivi avez-vous utilisés ?**
> **Git** pour le versioning et l'historique (commits atomiques, messages conventionnels), un **journal technique** documentant chaque décision et incident, et des **guides de test** reproductibles. La documentation était traitée comme un livrable de premier ordre.

**Q. Quels étaient vos livrables ?**
> Le code (cœur Rust, SaaS Django, relais), la spécification technique, les guides et campagnes de test, le rapport, et le support de soutenance. Plus une documentation continue exploitable par l'entreprise.

**Q. Comment avez-vous mesuré le succès ?**
> Par des **critères objectifs et testés** : 6/6 scénarios métier validés, 4/4 protections de résilience prouvées sur banc réel, déploiement auto-adaptatif fonctionnel. Pas d'auto-évaluation au doigt mouillé — des tests reproductibles.

**Q. Qu'est-ce qui a pris plus de temps que prévu ?**
> Les détails d'intégration réseau multi-machines (réplication, découverte, isolation des segments) — typique des systèmes distribués où chaque couche cache la suivante. J'ai sécurisé ça avec des conventions stables (adresses internes, déploiement auto-adaptatif).

**Q. Comment avez-vous géré l'autonomie ?**
> Le cabinet est à taille humaine, donc j'étais référent technique principal. J'avançais en autonomie sur les décisions d'architecture, avec validation aux jalons par Mme Chokri. Ça demande de la rigueur : documenter, justifier chaque choix, et savoir trancher.

**Q. Avez-vous travaillé en équipe ?**
> J'ai porté la conception et le développement, avec des échanges réguliers (encadrante entreprise, tuteur EIGSI). La collaboration s'est surtout faite sur les besoins métier et les références clients.

**Q. Avez-vous suivi une méthode agile ?**
> Oui, dans l'esprit, pas en Scrum « par le livre » (je travaillais seul). J'ai appliqué les principes agiles : itérations courtes, logiciel qui marche à chaque étape, et surtout *« répondre au changement plutôt que suivre un plan »*. Mes points hebdomadaires avec Mme Chokri jouaient le rôle de **revues de sprint** (démo + réajustement), mon journal technique celui du **backlog**, et le suivi des tâches passait par un tableau **Kanban (Trello)**.

**Q. Comment articulez-vous « agile » et « planning par phases » ? N'est-ce pas contradictoire ?**
> Non, c'est complémentaire. Le **WBS et le Gantt** donnaient le cap (6 macro-tâches, 6 jalons J1→J6, une semaine tampon en fin de parcours). À l'intérieur de ce cadre, j'avançais de façon **itérative**, en validant chaque brique avant la suivante. Le plan fixe les jalons ; l'agilité gère *comment* on les atteint et *quoi* on corrige en chemin.

**Q. Comment était structuré votre planning ?**
> Un **WBS** en 6 macro-tâches sur 24 semaines : cadrage & conception (S1–S4), mise en place de l'environnement (S5–S8), socle de stockage/persistance (S7–S10), réplication & résilience (S11–S14), sécurité & API (S15–S18), tests & documentation (S19–S24). Chaque macro-tâche se terminait par un **jalon** daté avec critères de validation. Un **diagramme de Gantt** matérialisait les dépendances.

**Q. Comment avez-vous réparti les rôles (RACI) ?**
> Une matrice **RACI** : moi **Responsable (R)** de la réalisation technique de toutes les briques ; Mme Chokri **Approbateur (A)** de la cohérence technique et des livrables ; M. Amrani **Approbateur** des livrables de fin de cycle (rapport, soutenance) ; l'équipe IT d'AL BARAA **Consultée (C)** pour l'interopérabilité. Un seul A par tâche — pas de dilution de responsabilité.

**Q. Comment avez-vous géré les jalons et le risque de retard ?**
> Chaque jalon avait des **critères objectifs** (ex. « réplication validée sur le banc, 0 perte »). Pour les risques de calendrier (Plan Directeur, rapport final), j'ai prévu des **buffers** et une rédaction progressive dès le début de chaque phase, pas en fin de parcours. Les risques techniques critiques étaient attaqués **tôt**, via le spike Phase 0.

---

## 6. Difficultés & posture ingénieur

**Q. Quelle a été votre plus grande difficulté ?**
> Diagnostiquer un **split-brain** en production : le tableau de bord affichait « cluster sain » alors qu'un nœud disait l'inverse. La cause : deux sources de vérité divergentes. Le réflexe technicien (redémarrer) ne réglait rien ; j'ai corrigé la cause — faire dire la vérité au dashboard à partir d'une mesure réelle — puis ajouté le fencing.

**Q. Qu'avez-vous appris de cet incident ?**
> Qu'un tableau de bord ne doit afficher que ce qu'il a **mesuré**, jamais ce qu'on lui a déclaré. Et qu'un bon ingénieur traite la **cause racine**, pas le symptôme.

**Q. Une erreur que vous avez commise ?**
> Pendant la mise au point, un de mes serveurs de transfert temporaire a squatté un port et faisait échouer des requêtes — j'ai mis du temps à voir que le bug venait de mon propre environnement, pas du code. Leçon : toujours nettoyer ses outils temporaires et vérifier l'environnement avant d'incriminer le code.

---

## 7. Limites & perspectives

**Q. Quelles sont les limites actuelles ?**
> C'est un socle prouvé, pas un produit fini : le failover automatique par quorum (≥3 nœuds) reste à généraliser, le module métier est minimal, et l'agent IA n'a pas encore sa couche prédictive. Pas de durcissement production ni d'audit sécurité externe.

**Q. Et les perspectives ?**
> Quorum ≥3 nœuds, couche IA prédictive, module métier complet, frontend Tauri + mobile via le cœur Rust, et un job de purge des blobs des tenants désactivés.

---

## 8. Questions pièges / transverses

**Q. Pourquoi pas simplement chiffrer chez un cloud classique (AWS, etc.) ?**
> Parce que le cloud détient alors les clés ou l'infrastructure de déchiffrement, et les données transitent par des serveurs étrangers. Ici, les données restent **physiquement** chez la PME, et l'éditeur n'a **aucune** clé.

**Q. En quoi ce projet relève-t-il de votre spécialité Big Data & IA ?**
> Le système produit en continu des séries temporelles de métriques (réplication, époque, heartbeats) — du Big Data d'infrastructure. L'agent IA exploite ces données pour superviser et, à terme, prédire les pannes. C'est l'IA appliquée à la fiabilité d'un système distribué.

**Q. Si c'était à refaire, que changeriez-vous ?**
> Je figerais plus tôt les conventions réseau (adresses stables, déploiement auto-adaptatif) — j'ai perdu du temps sur des adresses qui changeaient. Et j'aurais écrit certains tests d'intégration encore plus tôt.

---

## 9. Évolution du sujet depuis le Plan Directeur ⭐ (à préparer absolument)

> ⚠️ Le jury a lu le **Plan Directeur** (déposé le 19/02/2026), qui parlait d'un *« Coffre-Fort Data P2P / Brique Universelle BaaS »* avec Syncthing, DuckDB, CRDT, mTLS, FastAPI. Le projet livré est un **SaaS Souverain** (Rust, libsodium, PostgreSQL, zero-knowledge). Il **faut** assumer et expliquer cette évolution avec aplomb.

**Q. Le sujet déposé et le projet final ne sont pas les mêmes. Le thème a-t-il été mélangé / changé en cours de route ?**
> Le **thème** n'a pas changé : la **souveraineté des données** est restée le fil rouge du premier au dernier jour. Ce qui a évolué, c'est la **solution technique**, après le spike de dérisquage. Le Plan Directeur posait une *hypothèse* — une brique de stockage P2P générique. En la confrontant au besoin réel d'AL BARAA et de ses clients, j'ai compris que la vraie valeur n'était pas la réplication de fichiers, mais un **modèle** : vendre un logiciel métier en SaaS sans pouvoir lire les données. C'est ça, le déclic.

**Q. Quel a été précisément le déclic ?**
> Deux constats pendant la Phase 0. (1) Les données métier — stock, facturation — exigent une **cohérence forte** : un CRDT « last-write-wins » peut *perdre* une écriture, ce qui est inacceptable pour un stock. PostgreSQL et sa réplication transactionnelle se sont imposés. (2) Le besoin client n'était pas « stocker des fichiers », mais « acheter un logiciel géré **sans confier ses données** ». D'où la bascule vers le **zero-knowledge cryptographique**. Le spike a donc joué pleinement son rôle : dérisquer **et réorienter** avant d'investir dans le mauvais socle.

**Q. Qu'avez-vous gardé du Plan Directeur ?**
> Beaucoup : la thèse de souveraineté et l'alignement **AUDPF**, le principe d'**orchestration de briques éprouvées** plutôt que du from-scratch, le **packaging Docker**, le **local-first**, l'**analyse fonctionnelle** (bête à cornes, pieuvre, FAST), toute la **structure de planification** (WBS, RACI, Gantt, jalons) et l'**approche pilotée par le risque**. La méthode est restée ; l'implémentation a mûri.

**Q. Qu'avez-vous abandonné, et pourquoi ?**
> Syncthing/CRDT (cohérence éventuelle inadaptée aux invariants métier → PostgreSQL streaming) ; DuckDB (l'OLAP n'était pas le cœur du besoin → priorité à l'intégrité transactionnelle) ; Python/FastAPI **pour le cœur** (→ Rust, pour un cœur unique, sûr en mémoire, réutilisable desktop/mobile) ; mTLS/x509 « fait maison » (→ libsodium + zero-knowledge, qui tient mieux la promesse). Chaque abandon est justifié par une leçon du terrain — Django (Python) reste, lui, pour le SaaS éditeur.

**Q. N'est-ce pas un aveu d'échec de la planification initiale ?**
> Au contraire : c'est **exactement** ce qu'un spike de dérisquage doit produire. On planifie pour **apprendre**, pas pour s'enfermer. « Répondre au changement plutôt que suivre un plan » est un principe agile. La planification a tenu — phases, jalons, livrables ; ce sont les **hypothèses techniques** qui ont été corrigées par la preuve. C'est de l'ingénierie, pas de l'improvisation.

**Q. Théorème CAP : où vous situez-vous, et qu'est-ce que ça change vs le sujet initial ?**
> Pour les données métier, je choisis **CP** (Cohérence + tolérance au Partitionnement) : en cas de coupure, le primaire **bloque** plutôt que de diverger — un stock faux est pire qu'un stock momentanément indisponible. C'est précisément pourquoi j'ai écarté l'approche **AP/éventuelle** (CRDT) du Plan Directeur pour les invariants forts. Le pivot, au fond, c'est un choix CAP assumé.

---

## 10. Positionnement : SaaS classique, On-Premise, BYOC ⭐

**Q. Quelle différence entre un SaaS classique, de l'On-Premise, du BYOC et votre modèle ?**
> - **SaaS classique** : l'éditeur héberge tout et détient les données **et les clés**. Pratique, mais souveraineté nulle.
> - **On-Premise** : le client héberge et gère tout lui-même. Souverain, mais il perd le service géré (mises à jour, support) et porte toute la charge d'exploitation.
> - **BYOC (Bring Your Own Cloud)** : l'éditeur déploie dans le **compte cloud du client**. Mieux pour la localisation, mais l'éditeur garde souvent un accès admin ou des clés, et ça reste du cloud.
> - **Notre modèle (SaaS Souverain)** : le client garde ses données sur **ses** machines **et** reçoit un service géré (comptes, licences, MAJ), pendant que l'éditeur est **cryptographiquement aveugle**. On combine le **contrôle de l'On-Premise** avec le **confort du SaaS** — et on va plus loin que le BYOC, car l'éditeur ne peut **jamais** lire.

**Q. Pourquoi pas simplement du BYOC, qui est très à la mode (Snowflake, Databricks…) ?**
> Le BYOC résout la **localisation** (les données restent dans le cloud du client), mais pas la **confiance** : l'éditeur opère l'instance, souvent avec des clés ou un accès admin. Notre promesse est plus forte : l'éditeur ne détient **aucune** clé. La garantie est **cryptographique**, pas contractuelle ni organisationnelle. Et notre cible — des PME africaines — n'a pas forcément de compte cloud à « apporter » : on s'appuie sur leurs propres machines.

**Q. Et face aux « sovereign cloud » des géants (AWS European Sovereign Cloud, Azure, Oracle régions souveraines) ?**
> Ces offres améliorent la localisation et la gouvernance, mais le **fournisseur reste l'opérateur** — donc potentiellement soumis à des lois extraterritoriales (ex. US CLOUD Act). Notre approche **déplace la confiance** : les données ne sont jamais lisibles par l'éditeur, où qu'il soit. C'est complémentaire pour les organisations qui veulent une garantie **technique**, pas seulement juridique.

**Q. Votre modèle est-il viable économiquement face au SaaS classique ?**
> Oui, et c'est même un avantage : l'éditeur n'a **pas** le coût du cloud des données clients (ni stockage, ni egress, ni scaling). On facture l'**implémentation souveraine** (déploiement, licences par poste, relais, support), pas l'hébergement. Le prix par poste est structurellement plus bas, **sans coût caché**.

---

## 11. Actualité & tendances (montre que tu suis le secteur)

**Q. En quoi votre sujet est-il d'actualité en 2026 ?**
> La souveraineté numérique est un sujet brûlant : l'**AUDPF** validé par l'Union Africaine fin 2025, la multiplication des **lois de localisation** des données en Afrique (Nigéria, Kenya, Afrique du Sud…), les inquiétudes persistantes sur l'**extraterritorialité** (CLOUD Act, Schrems II), et la vague des offres « **sovereign cloud** ». Mon projet répond à cette demande avec une garantie **technique**, pas seulement réglementaire.

**Q. Le zero-knowledge / chiffrement de bout en bout, est-ce une vraie tendance ou un effet de mode ?**
> Une tendance de fond : **confidential computing**, chiffrement de bout en bout généralisé (messageries, sauvegardes), « **bring your own key** ». Les fournisseurs cherchent tous à **prouver** qu'ils ne peuvent pas voir les données de leurs clients. Mon projet applique ce principe à un **logiciel métier complet**, pas juste à du stockage ou de la messagerie.

**Q. Quel est le lien avec l'IA, au cœur de l'actualité ?**
> Double lien. (1) À l'ère de l'IA, la **gouvernance des données** devient critique — *où entraîne-t-on, avec quelles données, qui y accède ?* La souveraineté est un **prérequis**. (2) J'utilise un **agent IA (Mistral)** pour superviser l'infrastructure : ça démontre qu'on peut faire de l'IA **utile** sans jamais aspirer les données métier. La souveraineté et l'IA ne s'opposent pas.

**Q. Pourquoi Mistral et pas OpenAI / un modèle américain ?**
> Cohérence avec la thèse : **Mistral est un modèle européen**, et l'agent ne traite **que des métriques d'infrastructure**, jamais des données métier. Utiliser un modèle souverain pour parler de souveraineté, c'est aligné. Cela dit, l'architecture reste **agnostique au fournisseur** : on pourrait basculer vers un modèle local auto-hébergé sans rien changer au principe.

**Q. Votre solution pourrait-elle s'appliquer hors d'Afrique ?**
> Oui — la problématique est universelle (santé, finance, secteur public en Europe comme ailleurs). J'ai centré le discours sur l'Afrique parce que c'est le contexte d'AL BARAA et l'impulsion de l'AUDPF, mais le **modèle zero-knowledge** est transposable à toute organisation soumise à des exigences fortes de confidentialité ou de localisation.

---

*Conseil : pour chaque réponse, donne d'abord la phrase-clé, puis 1 exemple concret. Reste sous 45 s par réponse. Si tu ne sais pas, dis « je ne l'ai pas traité, mais voici comment je m'y prendrais ».*
