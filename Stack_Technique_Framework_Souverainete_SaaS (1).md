# Stack technique — Framework de souveraineté des données pour logiciels métier en SaaS

**Architecture : cluster actif/passif sur les machines du client + relais éditeur aveugle**

Document de développement de la solution. Il décrit la pile technologique retenue pour réaliser l'architecture décrite dans le document de cadrage *Phase 0 — Spike de dérisquage*.

Le principe directeur de toute la stack : **la lecture est disponible hors-ligne ; l'écriture passe toujours par le nœud actif**, qui voit le clair (machine du client) et sérialise toutes les écritures. Le relais éditeur ne reçoit **que du chiffré**.

> 🚨 **RÈGLE D'OR — à garder en tête à chaque décision.**
> **Le socle d'abord, le métier ensuite.** Le métier repose sur la crypto, la sérialisation et le failover : tant que ces fondations ne sont pas **prouvées** (Phase 0), aucune ligne de métier réel n'est écrite — sinon tout serait à réécrire. La règle impose un **ordre**, pas l'immobilisme. Formulation complète en fin de document (§12).

---

## 0. Décisions actées (Phase 0 §7.1)

Les quatre choix structurants à figer **avant de coder** sont tranchés. Ils forment un ensemble cohérent.

| # | Décision | Choix acté | Conséquence |
| :--- | :--- | :--- | :--- |
| 1 | **Langage du cœur** | **Rust** | Sûreté mémoire pour la crypto, export mobile via UniFFI, intégration native à Tauri. |
| 2 | **Moteur de base — nœud actif** | **PostgreSQL** | La réplication primaire/standby et la bascule sont prêtes à l'emploi (effort hors équipe). SQLite reste la réplique locale lecture seule des passifs. |
| 3 | **Consensus / failover** | **Promotion de standby PostgreSQL** | Découle du choix 2 : pas de Raft séparé à ajouter ; supervision type Patroni pour la bascule. |
| 4 | **Format du journal d'opérations** | **CBOR, schéma minimal versionné** | Compact, sûr en Rust (`ciborium`), adapté au chiffrement. Forme minimale pour le spike, pas définitive. |

### 0.1 Schéma minimal d'une opération (spike)

Chaque opération du journal porte, en CBOR puis chiffrée avec la DEK avant écriture disque :

| Champ | Rôle |
| :--- | :--- |
| `seq` | Numéro de séquence imposé par l'actif — l'ordre unique, pièce maîtresse anti-survente. |
| `type` | Nature de l'opération : `SALE` ou `STOCK_ADJUST` pour le spike. |
| `op_id` | Identifiant unique de l'opération (idempotence, déduplication). |
| `ts` | Horodatage. |
| `payload` | Contenu métier minimal (ex. quantité). |
| `schema_version` | Version du format → fait évoluer le journal sans casser le rejeu. |

> Le rejeu ordonné de ces opérations reconstruit l'état (stock + numérotation). Le moteur SQL applique l'atomicité et les contraintes `UNIQUE`.

---

## 1. Vue d'ensemble en couches

La solution s'organise en cinq couches, du cœur partagé jusqu'au relais aveugle.

| Couche | Rôle | Technologies retenues |
| :--- | :--- | :--- |
| **Cœur partagé** | Logique métier + crypto + logique client/serveur, écrite **une seule fois** | **Rust**, compilé/lié pour poste de travail et mobile |
| **Persistance & journal** | Base locale + journal d'opérations append-only | **PostgreSQL** (nœud actif) + SQLite en réplique locale lecture seule |
| **Réplication & consensus** | Réplication synchrone du journal, élection, fencing | **Réplication primaire/standby PostgreSQL + promotion de standby** (supervision type Patroni) |
| **Cryptographie** | Chiffrement données/journal, enrôlement, récupération | libsodium (XChaCha20-Poly1305, X25519, Argon2id) |
| **Relais éditeur** | Sauvegarde chiffrée hors-site, rendez-vous multi-sites, plan de contrôle | Service stateless minimal stockant des blobs opaques |
| **Packaging & déploiement** | Exécuter le nœud nativement sur chaque OS du parc | **Binaire natif** par OS (issu du cœur Rust), sans conteneur |

---

## 2. Le cœur partagé (un seul cœur, jamais deux)

**Décision structurante (Phase 0 §7.1).** La logique métier, la cryptographie et la logique client/serveur sont écrites **une seule fois** dans un cœur unique, partagé entre la variante poste de travail et la variante mobile. Deux implémentations cryptographiques séparées finiraient par diverger et multiplier les bugs de sécurité.

| Élément | Choix | Justification |
| :--- | :--- | :--- |
| **Langage du cœur** | **Rust** (acté) | Sûreté mémoire, bindings natifs vers libsodium, compilation vers desktop **et** mobile (FFI iOS/Android), pas de runtime lourd. |
| **Liaison libsodium** | Crate `libsodium-sys` / `sodiumoxide` (Rust) | Expose directement les primitives sûres sans réimplémentation. |
| **Variante desktop** | Cœur Rust + UI native ou Tauri | Le même cœur sert le serveur local et les clients. |
| **Variante mobile** | Cœur Rust via FFI (UniFFI) | Évite de réécrire la crypto en Swift/Kotlin. |

> Règle absolue : **aucune primitive cryptographique réimplémentée à la main.** Tout passe par libsodium via le cœur unique.

### 2.1 Frontend métier (recommandation — Phase 3, à valider après le spike)

> ⏳ **Hors Phase 0.** Le frontend métier relève de la **Phase 3** : il ne se décide ni ne se code avant que le socle soit prouvé (règle d'or). La ligne ci-dessous est une **recommandation d'orientation**, pas une décision actée.

**Ligne retenue : Tauri (React + TypeScript) sur desktop, SwiftUI / Compose sur mobile, le tout sur le cœur Rust via UniFFI.**

| Cible | UI | Lien au cœur |
| :--- | :--- | :--- |
| **Desktop** (Kali, Ubuntu, Windows 11) | **Tauri** + React/TypeScript | Tauri est en Rust : le cœur s'y intègre nativement, binaire léger, cross-OS. |
| **Mobile** | **SwiftUI** (iOS) / **Jetpack Compose** (Android) | Cœur Rust exposé via **UniFFI** ; UI native idéale pour le QR d'enrôlement. |

> Invariant UI : le frontend **n'écrit jamais directement** dans la réplique locale (§3.2). Coupé du nœud actif, il bascule en **lecture seule visible** et le signale à l'opérateur — il ne fait jamais semblant d'écrire.

---

## 3. Persistance et journal d'opérations

### 3.1 Le journal append-only

Le nœud actif n'enregistre pas « l'état final » mais la **suite ordonnée des opérations** (« vente n°46 », « vente n°47 »…). C'est un journal **append-only** : on n'ajoute qu'à la fin, jamais de réécriture du passé. Chaque machine rejoue le journal pour reconstruire l'état. Cet ordre unique, imposé par un seul serveur à la fois, garantit l'absence de survente et la continuité des numéros.

| Élément | Choix | Rôle |
| :--- | :--- | :--- |
| **Représentation d'une opération** | **CBOR** (schéma minimal versionné, cf. §0.1) | Ordonnée, rejouable, vérifiable, compacte et chiffrable. |
| **Stockage du journal** | Chiffré avec la DEK (XChaCha20-Poly1305) avant écriture disque | Le journal au repos est toujours chiffré. |
| **État reconstruit** | Tables SQL dérivées du rejeu | Le moteur SQL applique l'atomicité et les contraintes `UNIQUE`. |

### 3.2 Moteur de base — décision structurante

Le choix du moteur dépend directement de la stratégie de réplication actif/passif.

| Moteur | Réplication primaire/standby native | Verdict |
| :--- | :--- | :--- |
| **PostgreSQL** (recommandé) | **Oui** : réplication synchrone + promotion de standby prêtes à l'emploi | La brique failover est largement disponible. Effort déplacé hors de l'équipe. |
| **SQLite** | **Non** : à bâtir au-dessus du journal d'opérations | Excellent en base embarquée locale, mais la réplication reste à construire. |

**Choix acté : PostgreSQL** pour la réplication synchrone primaire/standby native et la promotion de standby. SQLite reste pertinent pour la réplique locale embarquée des clients passifs en lecture seule.

> Règle métier à acter : **pour les opérations à invariant fort, réplication synchrone vers au moins un passif** — une telle écriture n'est jamais confirmée à l'opérateur tant qu'elle n'est pas répliquée. Les opérations tolérantes à la perte peuvent rester en asynchrone (voir §4.1).

---

## 4. Réplication, consensus, failover et fencing

C'est le **vrai risque du projet** — opérationnel, pas cryptographique.

### 4.1 Réplication : deux modes, choisis selon l'opération

Les deux modes sont **conservés** et appliqués **par type de tâche**. Le bon mode dépend du besoin de l'opération : tolère-t-elle, ou non, la perte de la dernière écriture si l'actif tombe juste après la confirmation ?

| Mode | Comportement | Coût | À utiliser pour |
| :--- | :--- | :--- | :--- |
| **Synchrone** | Confirme **après** accusé d'au moins un passif → l'écriture existe sur ≥ 2 machines | Plus lent (aller-retour réseau par écriture) | Opérations à **invariant fort** : numérotation de factures, décrément de stock, écritures comptables. Aucune perte tolérée. |
| **Asynchrone** | Confirme **immédiatement**, réplique « quand il peut » | Rapide | Opérations **tolérantes à la perte de la dernière écriture** : brouillons, journaux d'audit non critiques, préférences, télémétrie locale. |

Le système expose donc un **niveau de durabilité par opération**, et non un réglage global figé. C'est l'arbitrage métier du document : on paie la lenteur du synchrone **uniquement** là où une transaction confirmée ne doit jamais être perdue.

> Garde-fou : si tous les passifs sont injoignables, une opération marquée **synchrone** doit **se bloquer** (refuser d'écrire) plutôt que dégrader silencieusement en asynchrone. Le choix de dégrader, s'il est offert, reste explicite et tracé.

### 4.2 Consensus et élection

> Règle absolue : **on n'écrit pas un algorithme de consensus à la main.**

| Besoin | Technologie | Rôle |
| :--- | :--- | :--- |
| Élection de l'actif, réplication du journal | **Promotion de standby PostgreSQL** (supervision type Patroni) | Sérialise les écritures, élit le nœud actif — consensus éprouvé, non réimplémenté. |
| Quorum anti-split-brain | Majorité parmi ≥ 3 machines participantes | Empêche deux serveurs simultanés. |

> Contrainte à connaître : une bascule **automatique sûre** suppose **au moins 3 machines**. Avec 2 machines, seule la bascule **manuelle** est sûre.

### 4.3 Fencing — le retour piégeux de l'ancien actif

Le danger n'est pas la panne mais le **retour après panne** : un ancien actif qui revient et se croit toujours serveur crée un split-brain traître.

| Mécanisme | Technologie | Rôle |
| :--- | :--- | :--- |
| **Jeton d'époque** (epoch token) | Compteur monotone incrémenté à chaque élection | Périme l'autorité de l'ancien actif. |
| **Fencing** | Isolation du nœud déchu jusqu'à resynchronisation | Empêche toute écriture sans réautorisation. |

### 4.4 Les deux modes de bascule (testés tous les deux en Phase 0)

| Mode | Machines | Risque split-brain | Coût |
| :--- | :--- | :--- | :--- |
| **Manuelle** | 2 suffisent | Nul (humain décide) | Courte interruption d'écriture. |
| **Automatique** | ≥ 3 (quorum) | Maîtrisé par quorum | Complexité, gestion du consensus. |

---

## 5. Cryptographie

Tout repose sur **libsodium**.

| Besoin | Primitive | Rôle |
| :--- | :--- | :--- |
| Chiffrer données & journal | **XChaCha20-Poly1305** (AEAD) | Confidentialité + intégrité, nonce long. |
| Identité d'appareil | **X25519** | Paire clé publique/privée par appareil. |
| Dériver une clé depuis une phrase secrète | **Argon2id** | Lente et gourmande en mémoire → anti-bruteforce. |
| Emballer un secret pour un appareil | **Sealed box** | Donne la DEK à un appareil sans l'exposer au relais. |

### 5.1 Hiérarchie de clés

| Niveau | Clé | Portée |
| :--- | :--- | :--- |
| **1** | **DEK** (Data Encryption Key), symétrique, **unique par entreprise** | Chiffre tout le contenu métier (données + journal). |
| **2** | Paire de clés d'appareil (X25519) | Une par appareil autorisé. |

Le lien : la **DEK est emballée (sealed box) vers la clé publique de chaque appareil autorisé**. La DEK en clair ne transite jamais par le relais. Autoriser un appareil = emballer la DEK pour sa clé publique.

### 5.2 Enrôlement (inspiré de Signal/WhatsApp)

1. Le nouvel appareil génère sa paire de clés et présente sa **clé publique** (QR code).
2. Un appareil déjà autorisé lit ce QR sur un canal authentifié.
3. L'appareil autorisé **emballe la DEK** (sealed box) pour la clé publique du nouvel appareil.
4. Le nouvel appareil ouvre le blob avec sa clé privée et récupère la DEK.

Le relais ne voit jamais la DEK ni aucune clé privée.

### 5.3 Récupération de clé — sujet de Phase 0, pas de confort

En zero-knowledge, **qui perd sa clé perd ses données, définitivement**. Pour de la comptabilité légale, c'est existentiel. Trois couches combinées :

| Mécanisme | Rôle |
| :--- | :--- |
| **Redondance multi-appareils** | Tant qu'un appareil survit, la DEK survit. |
| **Code de récupération** | DEK emballée sous une clé à haute entropie, imprimée une fois (« au coffre »). |
| **Partage de secret de Shamir** (optionnel) | M parts sur N pour reconstituer la clé, sans point unique de défaillance. |

---

## 6. Le relais éditeur (aveugle)

Situé **hors** du périmètre de confiance du client. Volontairement réduit et **ne voit jamais que du chiffré**. Jamais dans le chemin d'une écriture.

| Fonction | Technologie | Garantie |
| :--- | :--- | :--- |
| **Sauvegarde chiffrée hors-site** | Stockage objet de blobs opaques (S3-compatible / MinIO) | Aucune clé, aucun clair. |
| **Rendez-vous multi-sites** | Annuaire de rendez-vous routant du chiffré | Au pire des identifiants opaques de machines. |
| **Plan de contrôle** | API licence / mises à jour | Ne touche pas aux données métier. |
| **Implémentation** | Service stateless léger (Rust/Go) | Mutualisable comme un SaaS classique. |

> Même entièrement compromis, saisi ou malveillant, le relais ne peut rien révéler du clair.

---

## 7. Constitution et gestion du parc — « zéro administrateur réseau »

L'éditeur ne fournit pas le mapping mais le **mécanisme** qui permet aux machines de se trouver. **On ne demande jamais à une PME d'éditer un fichier de configuration avec des adresses IP.**

| Besoin | Technologie | Principe |
| :--- | :--- | :--- |
| **Découverte locale** | **mDNS / zeroconf** | Les machines se découvrent sans configuration (comme une imprimante réseau). |
| **Appartenance = enrôlement** | QR code (cf. §5.2) | Appartenir au cluster = posséder la DEK = avoir été enrôlé. Pas de second système de mapping. |
| **Rendez-vous distant** | Relais aveugle | Pour le multi-sites, sans rien voir. |

### 7.1 Le point dur : le dé-enrôlement

Ajouter une machine est facile ; la **retirer** proprement est le vrai sujet.

| Conséquence du retrait | Action technique |
| :--- | :--- |
| **Sécurité** | **Rotation de la DEK** : nouvelle clé re-emballée pour les **seules machines restantes**. |
| **Consensus** | **Recalcul du quorum** + alerte si le failover automatique n'est plus sûr (ex. 3 → 2 machines). |

---

## 8. Récapitulatif de la pile

| Domaine | Choix retenu |
| :--- | :--- |
| **Cœur partagé** | Rust (cœur unique desktop + mobile via UniFFI) |
| **Frontend métier** *(Phase 3, à valider)* | Tauri + React/TypeScript (desktop) · SwiftUI / Compose (mobile) |
| **Crypto** | libsodium — XChaCha20-Poly1305, X25519, Argon2id, sealed box |
| **Base — nœud actif** | PostgreSQL (réplication primaire/standby native, mode synchrone **ou** asynchrone selon l'opération) |
| **Réplique locale clients** | SQLite embarqué, lecture seule |
| **Journal** | Append-only, opérations CBOR, chiffré DEK |
| **Consensus / failover** | Promotion de standby PostgreSQL (supervision type Patroni) + quorum |
| **Fencing** | Jeton d'époque + isolation du nœud déchu |
| **Découverte parc** | mDNS / zeroconf + enrôlement QR |
| **Packaging / déploiement** | Binaire natif par OS (Kali, Ubuntu, Windows 11), sans conteneur |
| **Relais éditeur** | Service stateless + stockage objet de blobs chiffrés (MinIO/S3) |
| **Récupération** | Multi-appareils + code de récupération + Shamir (optionnel) |

---

## 9. Packaging et déploiement — exécution native, sans Docker

Le nœud client est distribué et exécuté **nativement** sur chaque machine du parc, **sans conteneur**. Ce choix découle directement des contraintes du document.

| Cible | Forme livrée |
| :--- | :--- |
| **Nœud desktop** (Kali, Ubuntu, Windows 11) | **Binaire natif** par OS, issu du cœur Rust (cœur applicatif + base locale + journal). |
| **Variante mobile** | Cœur Rust via UniFFI, intégré à l'appli — sans rapport avec un conteneur. |
| **Relais éditeur** | Service serveur classique côté éditeur (déploiement à la discrétion de l'éditeur). |

### 9.1 Pourquoi pas de Docker

- **Le produit final ne tourne pas en conteneur.** Il s'exécute sur les postes que la PME possède déjà, dont Windows 11, chez des utilisateurs sans informaticien (§3.1, §6). Un binaire natif s'installe et se lance directement ; demander d'installer Docker reviendrait à réintroduire la barrière qui a tué le on-premise.
- **mDNS sans friction.** La découverte locale « zéro administrateur réseau » (§7) fonctionne directement sur le réseau de l'OS hôte, sans bridge ni NAT Docker à contourner pour le multicast.
- **Cible mobile.** Le cœur doit aussi tourner sur téléphone (§7.1), hors de portée de Docker.
- **Banc d'essai plus fidèle.** Les pannes sont éprouvées sur les **vraies** machines (Kali, Ubuntu, Windows 11) : couper le réseau, arrêter le service, poser une règle de pare-feu temporaire. C'est plus probant qu'un cluster conteneurisé pour démontrer failover, quorum et fencing en conditions réelles, et cela révèle tôt les écarts cross-OS que la Phase 0 doit lever.

---

## 10. Stratégie de déploiement — « zéro administrateur réseau »

Le déploiement n'est pas une installation à configurer : c'est une **suite d'enrôlements**, un geste que le gérant comprend. Conformément au document (§6.1), *appartenir au cluster = posséder la clé = avoir été enrôlé*. Le mapping du parc n'est rien d'autre que la liste des machines enrôlées — **aucun fichier d'adresses IP à éditer, jamais**.

### 10.1 Le déroulé

| Étape | Geste | Résultat |
| :--- | :--- | :--- |
| **1. Premier poste — Windows 11** | Installer le binaire natif sur le poste **Windows 11** (poste du gérant). | Il s'auto-désigne **nœud actif** : cluster d'une seule machine, service déjà opérationnel en local. |
| **2. Enrôler le 2ᵉ poste** | Installer le binaire ; le poste affiche son **QR code** (clé publique). Le poste Windows 11 le scanne et **emballe la DEK** (sealed box) pour lui. | Le 2ᵉ poste récupère la DEK, rejoint comme **passif** en lecture. |
| **3. Enrôler le 3ᵉ poste** | Même geste de scan QR depuis un poste déjà autorisé. | 3 machines enrôlées → **quorum atteint** pour le failover automatique. |
| **4. Découverte automatique** | Rien à faire. | Les nœuds se repèrent via **mDNS/zeroconf** sur le réseau local, sans saisir d'adresse. |
| **5. Vie du cluster** | Rien à faire. | Actif/passif, réplication et bascule gérés par la promotion de standby PostgreSQL + quorum. |

> Sur le banc d'essai Phase 0, l'ordre est donc : **Windows 11 d'abord** (actif initial), puis enrôlement d'Ubuntu et de Kali par QR.

### 10.2 Qui fait quoi

| Acteur | Rôle |
| :--- | :--- |
| **Éditeur** | Fournit le **mécanisme** : binaire par OS, découverte mDNS, enrôlement QR, relais de rendez-vous. Jamais le mapping. |
| **Client (gérant)** | Fait les **gestes** : installer le binaire, scanner les QR pour enrôler chaque poste. |
| **Cluster** | Gère **le reste automatiquement** : qui est actif, qui est passif, réplication, failover. |
| **Relais éditeur** | Déjà en service côté éditeur (sauvegarde chiffrée, rendez-vous multi-sites). Le client s'y connecte ; il ne voit que du chiffré. |

### 10.3 Le point dur : retirer une machine

Ajouter est trivial ; **retirer** proprement est le vrai sujet (§6.2), surtout subi (poste volé, départ d'un employé). Deux gestes obligatoires :

- **Rotation de la DEK** — la machine retirée détient encore la clé : on génère une nouvelle DEK re-emballée pour les **seules machines restantes**.
- **Recalcul du quorum + alerte** — si le parc retombe de 3 à 2 machines, le failover automatique n'est plus sûr, et le produit **prévient** le client.

---

## 11. Banc d'essai — le parc de trois machines

Le spike est validé sur le parc réel de l'équipe : **trois machines**, ce qui est précisément le minimum requis pour démontrer le failover automatique par quorum (§4.5) et donc couvrir **les deux modes de bascule** (§4.6).

| Machine | OS | Rôle dans le scénario |
| :--- | :--- | :--- |
| **Machine 1** | Windows 11 | **Nœud actif initial** (premier poste installé) — valide aussi le **cœur unique multi-OS**. |
| **Machine 2** | Ubuntu | Nœud du cluster, enrôlé par QR (candidat actif/passif). |
| **Machine 3** | Kali Linux | Nœud du cluster, enrôlé par QR + poste d'inspection (vérifie que le relais ne reçoit que du chiffré). |

### 11.1 Ce que ce parc permet de prouver

| Capacité testée | Configuration utilisée |
| :--- | :--- |
| Écritures concurrentes sans violation d'invariant | Plusieurs opérateurs répartis sur les 3 machines. |
| Lecture hors-ligne / écriture refusée hors-ligne | Une machine coupée du nœud actif. |
| Confirmation après réplication synchrone | Actif + ≥ 1 passif accusant réception. |
| **Bascule manuelle** | 2 machines (la 3ᵉ éteinte). |
| **Bascule automatique par quorum** | Les 3 machines participantes. |
| Anti-split-brain | Coupure réseau provoquée entre les nœuds. |
| Fencing (retour de l'ancien actif) | Ancien actif réintroduit après promotion. |
| Enrôlement / dé-enrôlement + rotation DEK | Retrait d'une machine → quorum recalculé 3 → 2 + alerte. |
| Cœur unique cross-OS | Même cœur Rust exécuté sur Linux et Windows 11. |

### 11.2 Points d'attention propres à ce parc

- **Cross-OS dès la Phase 0.** Avoir Windows 11 dans le cluster impose de valider tôt la portabilité du cœur (libsodium, chemins de fichiers, services réseau) — un risque qu'il vaut mieux lever maintenant que plus tard.
- **mDNS sur réseau hétérogène.** La découverte locale (§7) doit fonctionner entre Linux et Windows ; à vérifier que le pare-feu Windows ne bloque pas le multicast.
- **Quorum à exactement 3.** Le retrait d'**une seule** machine fait repasser le cluster sous le seuil du failover automatique sûr : c'est précisément le cas que le produit doit détecter et signaler (§7.1).

---

## 12. Ce qui est hors périmètre de la Phase 0

Pour rester concentré, le spike **exclut délibérément** : la logique métier réelle (facturation, paie, comptabilité), une interface soignée, les permissions fines entre opérateurs, la découverte automatique complète et le rendez-vous multi-sites, le durcissement de production et l'audit de sécurité, le modèle d'abonnement et le mode dégradé.

> ⚠️ **Piège à éviter.** Ne pas commencer par l'application métier.

---

> # 🚨 Règle d'or du projet
>
> ## Le socle d'abord, le métier ensuite.
>
> **Pourquoi.** Le métier (facturation, paie, stock…) repose sur trois fondations : la **cryptographie**, la **sérialisation des écritures** et le **failover**. Si l'une de ces fondations change après coup, tout le métier bâti dessus est à réécrire. On ne construit pas sur un sol qui n'a pas encore pris.
>
> **⛔ Ce qui est interdit.** Écrire la moindre ligne de logique métier réelle **tant que le socle n'a pas été prouvé** par le spike de la Phase 0.
>
> **✅ Ce qui est autorisé — dès maintenant.** Tout le reste : lancer le spike, préparer la spec, arbitrer les choix techniques. La règle n'interdit pas d'avancer ; elle impose un **ordre** et un **verrou de validation** entre le socle et le métier.
>
> *La tentation de commencer par le métier sera forte (c'est gratifiant et ça se montre). C'est précisément l'erreur à ne pas commettre.*
