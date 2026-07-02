# Glossaire & Acronymes — Framework SaaS Souverain (أمان / Amān)

> Document d'accompagnement de la soutenance — définitions des termes techniques.
> Jesse MPIGA-ODOUMBA · EIGSI Casablanca · Promotion 2026 · AL BARAA CONSULTING

Les définitions sont volontairement **accessibles** : elles visent la compréhension par le jury, pas l'exhaustivité académique.

---

## 1. Concepts clés du projet

| Terme | Définition simple |
|-------|-------------------|
| **أمان (Amān)** | Nom du logiciel métier. Mot arabe signifiant « sécurité, confiance ». Logiciel **open source**, installé chez le client. |
| **Souveraineté des données** | Principe selon lequel les données d'une organisation restent sous son contrôle, sur ses propres machines, sans transiter par un cloud étranger. |
| **Zero-knowledge** (« connaissance nulle ») | Propriété garantissant qu'un acteur (ici l'éditeur/le relais) **ne peut pas lire** les données qu'il héberge, même s'il le voulait — parce qu'elles sont chiffrées et qu'il n'a aucune clé. |
| **SaaS** (*Software as a Service*) | Logiciel vendu en service/abonnement plutôt qu'acheté une fois. Ici : un SaaS **souverain** (comptes et licences gérés, mais données non hébergées par l'éditeur). |
| **Tenant** | Une entreprise cliente (PME) dans un système multi-clients. Chaque tenant est cloisonné des autres (ex. MPJ, Yasmine Argan). |
| **Multi-tenant** | Architecture où une même plateforme sert plusieurs tenants isolés les uns des autres. |
| **Les 3 acteurs** | (1) **SaaS éditeur** : comptes/licences. (2) **Relais zero-knowledge** : stockage chiffré. (3) **Cluster PME** : exécute le logiciel et détient les données en clair. |
| **Cluster PME** | L'ensemble des machines de la PME exécutant Amān (ex. 2 nœuds PostgreSQL répliqués). |
| **Relais** | Serveur de l'éditeur qui stocke des sauvegardes **chiffrées opaques** (blobs) sans pouvoir les lire. |
| **Blob** (*Binary Large OBject*) | Un paquet de données binaires opaques. Ici : un coffre **chiffré** que le relais stocke sans en connaître le contenu. |
| **Coffre de récupération** | Le blob contenant la clé de chiffrement (DEK) emballée sous le code de récupération du client, stocké sur le relais. |

---

## 2. Cryptographie

| Terme | Définition simple |
|-------|-------------------|
| **Chiffrement** | Transformer des données lisibles en données illisibles sans la clé. |
| **DEK** (*Data Encryption Key*) | Clé de chiffrement des données. **Unique par entreprise** : elle chiffre toutes les données métier et le journal. |
| **Clé symétrique** | Une seule clé sert à chiffrer **et** déchiffrer (ex. la DEK). |
| **Clé asymétrique** | Une paire de clés : une **publique** (partageable) et une **privée** (secrète). Sert à l'enrôlement des appareils. |
| **libsodium** | Bibliothèque cryptographique reconnue et auditée. **Aucune crypto réinventée à la main** dans le projet — tout passe par elle. |
| **XChaCha20-Poly1305** | Algorithme de chiffrement symétrique (avec contrôle d'intégrité). Utilisé pour les données et le journal. |
| **X25519** | Algorithme à clés publiques/privées. Sert à l'**identité de chaque appareil** et au partage sécurisé de la DEK. |
| **Argon2id** | Fonction qui transforme un mot de passe (le code de récupération) en clé robuste, **résistante au craquage**. |
| **Sealed box** (« boîte scellée ») | Mécanisme pour emballer un secret (la DEK) à destination d'une clé publique précise — seul le détenteur de la clé privée peut l'ouvrir. |
| **AEAD** (*Authenticated Encryption with Associated Data*) | Chiffrement qui garantit à la fois la **confidentialité** et l'**intégrité** (détecte toute altération). |
| **Nonce** | Valeur aléatoire utilisée une seule fois pour chaque chiffrement, afin que deux messages identiques ne donnent jamais le même résultat. |
| **Sel** (*salt*) | Donnée aléatoire ajoutée avant de dériver une clé d'un mot de passe, pour empêcher les attaques par tables précalculées. |
| **Code de récupération** | Code secret haute entropie détenu **uniquement par la PME**. Seule clé permettant de récupérer les données en cas de sinistre. L'éditeur ne le connaît jamais. |
| **Enrôlement** | Procédure d'ajout d'un nouvel appareil au cluster : on lui transmet la DEK de façon chiffrée (sealed box). |
| **Hiérarchie de clés** | Organisation des clés : la DEK chiffre les données ; elle est elle-même emballée pour chaque appareil et sous le code de récupération. |

---

## 3. Base de données, réplication & haute disponibilité

| Terme | Définition simple |
|-------|-------------------|
| **PostgreSQL** | Système de base de données relationnelle open source, robuste et éprouvé. |
| **HA** (*High Availability*) | Haute disponibilité : capacité du système à continuer de fonctionner malgré la panne d'une machine. |
| **Réplication** | Copie automatique des données d'un serveur vers un autre, pour ne pas tout perdre si l'un tombe. |
| **Nœud primaire** | Le serveur qui accepte les écritures (la « source de vérité »). |
| **Nœud standby** | Un serveur en **lecture seule** qui reçoit une copie du primaire et peut prendre le relais. |
| **Streaming WAL** | Réplication en continu du **journal des transactions** (WAL) du primaire vers le standby, en quasi temps réel. |
| **WAL** (*Write-Ahead Log*) | Journal où PostgreSQL écrit chaque modification **avant** de l'appliquer — base de la réplication et de la récupération. |
| **Slot de réplication** | Mécanisme qui force le primaire à **conserver** le WAL tant que le standby ne l'a pas reçu (évite les ruptures de réplication). |
| **pg_basebackup** | Outil PostgreSQL qui crée une copie complète du primaire pour initialiser un standby (« cloner »). |
| **Failover** (bascule) | Promotion automatique ou manuelle du standby en primaire quand le primaire tombe. |
| **Switchover** | Bascule **planifiée** des rôles primaire/standby (sans panne). |
| **Quorum** | Règle de majorité : il faut ≥ 3 nœuds pour décider automatiquement et sûrement d'un failover. À 2 nœuds, bascule **manuelle** uniquement. |
| **Split-brain** | Situation dangereuse où **deux nœuds se croient primaires** en même temps → données divergentes. À éviter absolument. |
| **Fencing** (« clôturage ») | Mécanisme qui **isole** un ancien primaire déchu pour l'empêcher d'écrire et de provoquer un split-brain. |
| **Époque / Timeline** | Compteur **monotone** (qui ne fait qu'augmenter) identifiant la « génération » du cluster. PostgreSQL incrémente son *timeline* à chaque promotion → sert de jeton de fencing. |
| **Réplication synchrone / asynchrone** | **Synchrone** : le primaire attend la confirmation du standby (sécurité maximale). **Asynchrone** : il n'attend pas (plus rapide, risque de perte minime). |
| **ACID** | 4 garanties d'une transaction fiable : **A**tomicité, **C**ohérence, **I**solation, **D**urabilité. |
| **MVCC** (*Multi-Version Concurrency Control*) | Technique permettant à plusieurs utilisateurs d'écrire en même temps sans se corrompre les données. |
| **CBOR** (*Concise Binary Object Representation*) | Format binaire compact pour sérialiser des données structurées. Utilisé pour le journal des écritures. |
| **Journal append-only** | Journal où l'on ne fait qu'**ajouter** (jamais modifier/supprimer) — traçabilité et intégrité. |

---

## 4. Infrastructure & déploiement

| Terme | Définition simple |
|-------|-------------------|
| **Docker** | Outil qui empaquette un logiciel et tout son environnement dans un **conteneur** portable, qui tourne à l'identique partout. |
| **Conteneur** | Une « boîte » isolée exécutant un logiciel avec ses dépendances. |
| **Image (Docker)** | Le modèle figé à partir duquel on lance un conteneur. Distribuée à la PME. |
| **Registre (Docker)** | Serveur qui stocke et distribue les images Docker (ici : registre privé de l'éditeur). |
| **docker-compose** | Fichier décrivant plusieurs conteneurs et comment les lancer ensemble (base, logiciel, interface). |
| **Watchtower** | Outil qui met à jour automatiquement les conteneurs quand une nouvelle image est disponible. |
| **Volume** | Espace de stockage persistant d'un conteneur (les données survivent au redémarrage). |
| **Multi-arch** | Image compatible plusieurs architectures (ex. linux/amd64, windows/amd64). |

---

## 5. Réseau & développement

| Terme | Définition simple |
|-------|-------------------|
| **Rust** | Langage de programmation rapide et **sûr en mémoire**. Le cœur d'Amān est en Rust (un seul code pour desktop et mobile). |
| **UniFFI** | Outil qui expose un cœur Rust unique à plusieurs plateformes (desktop, mobile) sans le réécrire. |
| **Django** | Framework web Python. Utilisé pour le **SaaS éditeur** (portail, comptes, licences). |
| **API REST** | Interface standardisée permettant à deux logiciels de communiquer via HTTP (ex. un nœud s'enregistre auprès du SaaS). |
| **UUID** (*Universally Unique IDentifier*) | Identifiant unique (ex. `0fadfd3d-…`). Sert d'identité d'installation et de tenant — **pas l'adresse MAC** (falsifiable). |
| **LAN / WAN** | **LAN** : réseau local (les machines de la PME). **WAN** : réseau étendu (vers l'éditeur). |
| **DHCP / IP statique** | **DHCP** : adresse IP attribuée automatiquement. **IP statique** : adresse fixée manuellement (ex. le relais). |
| **Loopback** | L'adresse `127.0.0.1`, qui désigne **la machine elle-même** (ne sert jamais à joindre une autre machine). |
| **Egress (frais de sortie)** | Frais facturés par les clouds classiques pour **extraire** ses propres données. Avec Amān : **aucun** (données déjà chez la PME). |
| **Hostname** | Nom d'une machine sur le réseau (ex. `relay`). |
| **netplan / VMnet** | **netplan** : configuration réseau sous Ubuntu/Debian. **VMnet** : réseau virtuel VMware reliant les VMs. |

---

## 6. Intelligence artificielle & Big Data

| Terme | Définition simple |
|-------|-------------------|
| **Agent IA de supervision** | Module qui surveille la **santé** des 3 acteurs et produit un diagnostic + un score de risque, sans jamais accéder aux données métier. |
| **Big Data** | Exploitation de grands volumes de données — ici, l'historique des métriques techniques du cluster (séries temporelles). |
| **Série temporelle** | Suite de mesures horodatées (ex. l'état de réplication toutes les 60 s) — matière première de l'IA. |
| **Détection d'anomalie** | Méthode IA qui apprend le comportement « normal » et alerte sur les écarts (ex. avant une panne). |
| **Isolation Forest / z-score** | Techniques de détection d'anomalie non supervisée (perspective d'évolution de l'agent). |
| **Mistral AI** | Modèle d'IA (LLM) utilisé pour générer le diagnostic en langage naturel. |
| **Heartbeat** (« battement de cœur ») | Signal périodique qu'un nœud envoie pour signaler qu'il est vivant (ici toutes les ~60 s). |
| **Fail-safe** | Repli sûr : si l'IA externe est indisponible, une analyse locale prend le relais — le système ne casse jamais. |

---

## 7. Conformité & divers

| Terme | Définition simple |
|-------|-------------------|
| **AUDPF** | Cadre de l'Union Africaine (déc. 2025) exigeant que les données des organisations africaines restent sous contrôle local. Amān y est **conforme par design**. |
| **Open source** | Logiciel dont le code est ouvert. Amān est open source ; l'éditeur facture l'**implémentation souveraine**, pas le logiciel. |
| **On-premise** | Logiciel installé sur les serveurs du client (par opposition au cloud). |
| **Spike (Phase 0)** | Prototype de **dérisquage** : prouver que le socle technique (crypto, réplication, failover) fonctionne avant d'écrire la logique métier. |
| **PME** | Petite et Moyenne Entreprise — le client cible d'Amān. |

---

*Astuce soutenance : si le jury bute sur un terme, renvoyer à ce glossaire. Les 5 mots à maîtriser absolument : **zero-knowledge, DEK, réplication primaire/standby, fencing, tenant**.*
