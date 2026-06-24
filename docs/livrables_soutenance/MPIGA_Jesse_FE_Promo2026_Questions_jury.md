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

*Conseil : pour chaque réponse, donne d'abord la phrase-clé, puis 1 exemple concret. Reste sous 45 s par réponse. Si tu ne sais pas, dis « je ne l'ai pas traité, mais voici comment je m'y prendrais ».*
