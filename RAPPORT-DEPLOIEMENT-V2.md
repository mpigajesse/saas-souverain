# Rapport — Image V2 publiable et ouverture du PC éditeur au réseau local

**Date :** 31 août 2026
**Branche :** `main2`
**Périmètre demandé :** recompiler l'image Docker V2 du nœud PME, la rendre publiable et téléchargeable depuis l'interface éditeur, et faire du PC de l'éditeur un serveur joignable par les autres machines du réseau local.

> **Tout est en place.** Le serveur est joignable sur **`http://10.128.177.37:5173/`** (réseau Wi-Fi « YaPasGiga 3 »).
>
> *Mise à jour du 1er septembre 2026 : le réseau a changé depuis la première rédaction — l'ancienne adresse `192.168.7.7` (Wi-Fi « Nothing is free 2 ») n'existe plus. Aucune reconfiguration n'a été nécessaire, l'adresse étant dérivée dynamiquement partout. Voir §10 : vérification de l'image publiée et correctif d'un défaut nginx découvert à cette occasion.*

---

## 1. Ce qui a été livré

| Chantier | État |
| :--- | :--- |
| Image `ss-node:v2` reconstruite depuis le workspace Rust `spike/` | Fait, binaire vérifié |
| Publication au registre Docker de l'éditeur | Fait, `ss-node:v2` servi sur le port 5000 |
| Archive téléchargeable (`docker save` + gzip + SHA256) | Fait, 35,7 Mo |
| Page « Déploiement » dans la console éditeur | Fait, route `/deploiement` |
| Téléchargement de l'image depuis l'interface | Fait, deux voies (éditeur et PME) |
| Console + API accessibles depuis le réseau par l'IP du PC | Fait, vérifié sur `10.128.177.37` |
| Installeurs PME pointant le bon registre | Fait, adresse dérivée de la requête |
| Ouverture du pare-feu Windows | Fait, 5 règles posées et vérifiées de l'extérieur |

---

## 2. L'image V2

Construite depuis `spike/` avec `-f node/Dockerfile` (contexte = racine du workspace Cargo, obligatoire).

```
ss-node:v2                   a1d84c1f0159   142 MB
ss-node:latest               a1d84c1f0159   142 MB
localhost:5000/ss-node:v2    a1d84c1f0159   142 MB
```

Les trois tags pointent le même digest `sha256:a1d84c1f0159b79f4e63b2a772a3913c9da1eb928c0ef4551879ce77db3fa578`.

**Binaire vérifié** — `docker run --rm --entrypoint ss-node ss-node:v2 --help` retourne bien les sous-commandes attendues : `init`, `run`, `status`, `failover`, `delist`, `adduser`.

**Publié au registre** (service compose `registry`, nouveau) :

```
GET /v2/_catalog            → {"repositories":["ss-node"]}
GET /v2/ss-node/tags/list   → {"name":"ss-node","tags":["v2"]}
```

**Archive de repli** — `media/node-images/` :

| | |
| :--- | :--- |
| Fichier | `ss-node-v2.tar.gz` |
| Taille | 35 734 279 octets |
| SHA256 | `bcc2272195c540134d9132b60a599ac323ae4d91970107b9164aaabbde8685e5` |
| Construite le | 2026-08-31T20:03:02Z |

`media/` est déjà dans `.gitignore` : l'archive de 35 Mo ne partira pas dans le dépôt.

**`spike/build-node.ps1` réécrit** : il enchaîne désormais vérification Docker → build → tags → contrôle du binaire → push → `docker save` + gzip + SHA256 → écriture de `manifeste.json`. Paramètres `-NoCache`, `-SkipPush`, `-Version` (défaut `v2`), `-Registry` (défaut `localhost:5000`). L'adresse `192.168.200.1:5000` codée en dur de l'ancien banc a disparu. Les chemins dérivent de `$PSScriptRoot`, le script tourne depuis n'importe quel répertoire.

Pour republier après une modification du code Rust :

```powershell
powershell -ExecutionPolicy Bypass -File spike\build-node.ps1
```

---

## 3. Le point qui aurait tout cassé

**L'URL du backend était figée dans le bundle React à la compilation** :

```ts
const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:8000/api'
```

Vite compile cette valeur *en dur* dans le JavaScript livré. Une machine du réseau ouvrant `http://10.128.177.37:5173` aurait donc vu son navigateur appeler **son propre `localhost`** — la console se serait affichée puis serait tombée en erreur réseau sur chaque appel, sans message explicite. Aucun réglage Django n'aurait corrigé cela.

**Correction — nginx devient le point d'entrée unique.** La base d'API passe en chemin **relatif** (`/api`), et nginx relaie vers Django :

```
Navigateur → http://<IP>:5173/          → nginx → bundle React
           → http://<IP>:5173/api/...   → nginx → backend:8000  (Django)
           → http://<IP>:5173/admin/    → nginx → backend:8000
```

Trois bénéfices, au-delà de la correction du bug :

1. **Aucune IP figée nulle part.** L'appel repart toujours vers l'hôte tapé dans la barre d'adresse. Changement d'IP, bascule Wi-Fi/Ethernet, autre réseau : rien à reconstruire.
2. **CORS devient sans objet** dans le chemin nominal — page et API partagent la même origine.
3. **Les cookies de session et CSRF** sont posés sur cette origine unique, donc l'authentification fonctionne à l'identique depuis n'importe quelle machine.

nginx transmet `Host`, `X-Real-IP`, `X-Forwarded-For` et `X-Forwarded-Proto`, ce dont Django a besoin pour la vérification CSRF *same-origin* et pour composer les bonnes URL dans les fichiers générés pour les PME.

---

## 4. Ouverture au réseau — ce qui a changé côté Django

`config/settings.py` **détecte les adresses IPv4 de la machine au démarrage** plutôt que de les coder en dur, par deux voies complémentaires : les adresses liées au nom d'hôte, et l'adresse de la route par défaut (socket UDP « connecté » sans émission de paquet — c'est la carte que les autres machines emprunteront). Les origines de confiance CSRF/CORS en sont dérivées pour les ports 5173, 8000, 80 et 3000.

Adresses détectées sur cette machine :

| Adresse | Interface | Usage |
| :--- | :--- | :--- |
| **10.128.177.37** | Wi-Fi 2 (« YaPasGiga 3 ») | **Le réseau de test — c'est cette adresse à donner aux autres machines** |
| 192.168.1.1 | Ethernet | Autre réseau |
| 192.168.198.1 / 192.168.246.1 | VMware VMnet1 / VMnet8 | Réseaux virtuels VMware |
| 172.19.144.1 | WSL | Réseau interne WSL |

Le 31 août, cette même machine était en `192.168.7.7` sur un autre Wi-Fi. Le passage à `10.128.177.37` n'a demandé **aucune modification de configuration** : la détection au démarrage a repris la nouvelle adresse, et tout ce qui est servi aux PME la dérive de la requête entrante. C'est la validation concrète du principe posé au §3.

**Réglage assumé et à connaître** : `ALLOWED_HOSTS=*` dans `.env`. Django accepte donc n'importe quel en-tête `Host`. C'est commode quand l'IP change (DHCP, bascule d'interface), mais c'est un **relâchement volontaire réservé à un réseau de test maîtrisé** — la protection contre l'empoisonnement d'en-tête `Host` est levée. En production, remplacer par la liste explicite des hôtes réels. Le réglage est isolé sur une seule ligne de `.env`, commentée, pour être resserré en une modification.

Les cookies restent en clair (`SESSION_COOKIE_SECURE=False`) puisque le banc est en HTTP ; à repasser à `True` dès qu'un HTTPS existe.

---

## 5. Le pare-feu Windows — fait

Le pare-feu est actif sur les trois profils et bloque les connexions entrantes par défaut. Sans règle, aucune machine du réseau n'aurait vu le serveur, quelle que soit la configuration logicielle.

Les règles ont été posées par `scripts/ouvrir-pare-feu-lan.ps1`, lancé en administrateur via une demande UAC validée par l'utilisateur (code de sortie 0) :

```
AMANE - Console React (point d'entree) (TCP 5173)           Inbound  Allow  Domain, Private, Public
AMANE - API Django (acces direct) (TCP 8000)                Inbound  Allow  Domain, Private, Public
AMANE - Registre Docker (images du noeud PME) (TCP 5000)    Inbound  Allow  Domain, Private, Public
AMANE - Relais zero-knowledge (TCP 8080)                    Inbound  Allow  Domain, Private, Public
AMANE - Grafana (TCP 3000)                                  Inbound  Allow  Domain, Private, Public
```

Toutes sont limitées au sous-réseau local (`-RemoteAddress LocalSubnet`) : rien n'est exposé au-delà du LAN, même si la machine est routée.

Le profil **Public** est inclus délibérément : le Wi-Fi de test (« YaPasGiga 3 », ex-« Nothing is free 2 ») est classé Public par Windows, et des règles posées uniquement sur « Privé » ne s'y seraient **pas** appliquées — c'est la cause n°1 d'un port qui reste injoignable malgré une règle correcte. Ce choix évite de reclasser le réseau, ce qui aurait réactivé la découverte réseau et le partage de fichiers sur ce Wi-Fi : l'impact reste ainsi limité aux cinq ports listés, et au seul sous-réseau local.

Pour tout retirer :

```powershell
powershell -ExecutionPolicy Bypass -File scripts\ouvrir-pare-feu-lan.ps1 -Supprimer
```

(en administrateur — le script refuse proprement sinon)

---

## 6. Le parcours d'une machine du réseau

### Côté éditeur — page « Déploiement »

Nouvelle page `/deploiement` dans la console (entrée dans la navigation, entre Relais et Architecture). Elle affiche l'état de l'image publiée (version, taille, SHA256, date), l'état du registre, la commande `docker pull` **déjà remplie avec la bonne IP**, un bouton de téléchargement de l'archive, et la procédure de déploiement côté PME en commandes copiables. Quand aucune image n'est publiée, un état vide renvoie vers `spike/build-node.ps1`.

Deux routes API l'alimentent (`dashboard/api_images.py`) :

| Route | Accès | Rôle |
| :--- | :--- | :--- |
| `GET /api/node-image/` | éditeur | État de l'image et du registre |
| `GET /api/node-image/download/` | éditeur | Archive en flux |
| `GET /api/node-image/telecharger/?token=…` | **jeton PME** | Archive pour les installeurs |

La troisième route existe parce que **la machine qui déploie le nœud n'a aucune session éditeur**. Elle s'authentifie par le jeton d'enregistrement du tenant, exactement comme le fait déjà `/api/devices/cluster-status/`.

### Côté PME — depuis une autre machine du réseau

1. Ouvrir `http://10.128.177.37:5173/` dans le navigateur.
2. S'inscrire (`/register`) ou se connecter. Un compte PME arrive sur `/mon-espace`.
3. Télécharger l'installeur ou le `docker-compose.yml` depuis son espace.
4. Lancer l'installeur : il détecte Docker (et l'installe au besoin), déclare le registre de l'éditeur, récupère l'image et démarre le nœud.

Ou manuellement :

```bash
# Voie 1 — depuis le registre de l'éditeur
docker pull 10.128.177.37:5000/ss-node:v2

# Voie 2 — par l'archive, si le registre est fermé ou non déclarable
curl -O http://10.128.177.37:5173/api/node-image/telecharger/?token=<JETON>
docker load -i ss-node-v2.tar.gz
```

Pour la voie 1, Docker exige que le registre en clair soit déclaré non sécurisé — sinon il réclame du TLS et le pull échoue (`server gave HTTP response to HTTPS client`). Les installeurs le font désormais **sur les deux systèmes** :

- **Linux** : `/etc/docker/daemon.json` (existait déjà, port désormais issu des réglages au lieu d'être figé).
- **Windows** : c'était un **trou** — l'installeur ne configurait rien, le pull aurait échoué sur toute machine Windows. Il édite maintenant `%PROGRAMDATA%\Docker\config\daemon.json`, redémarre Docker, attend qu'il réponde, et **bascule sur l'archive + `docker load` si le pull échoue quand même**.

### L'adresse n'est jamais codée en dur

`SS_NODE_IMAGE` vaut `{registry}/ss-node:v2`. Le marqueur est remplacé à la volée par « l'IP par laquelle la PME vient de joindre l'éditeur » + `REGISTRY_PORT`. Une PME qui arrive par `10.128.177.37` reçoit `10.128.177.37:5000/ss-node:v2` ; une autre qui arriverait par `10.0.0.42` recevrait `10.0.0.42:5000/...`. Quatre tests verrouillent ce comportement, dont un qui vérifie explicitement qu'aucune adresse étrangère ne fuit dans le fichier généré.

---

## 7. Vérifications réellement effectuées

Tout ce qui suit a été exécuté, pas supposé.

**Tests Django — 40 tests, tous passants** (`env/Scripts/python.exe manage.py test api`) :
- 23 tests de frontière des rôles et d'authentification (préexistants, intacts) ;
- 7 sur l'état et le téléchargement éditeur de l'image ;
- 6 sur la voie PME par jeton (jeton absent → 400, mal formé → 400, inconnu → 403, tenant inactif → 403, image absente → 404, cas nominal servi sans session) ;
- 4 sur les fichiers générés pour la PME (compose, installeurs Linux et Windows, suivi de l'hôte de la requête).

**Frontend** : `npx tsc -b` et `npm run lint` sortent tous deux propres, sans avertissement.

**Django** : `manage.py check` — aucun problème.

**Accès réseau réel, via l'IP LAN `10.128.177.37`** :

| Requête | Résultat |
| :--- | :--- |
| `GET /` | 200 (console React) |
| `GET /deploiement` | 200 |
| `GET /api/auth/csrf/` | 200 (API relayée, même origine) |
| `GET /api/auth/me/` | 401 (pas de session — attendu) |
| `GET /api/tenants/` | 403 (route éditeur protégée) |
| `GET /admin/` | 200 |
| `GET /api/node-image/` | 403 sans session éditeur |
| `GET /api/node-image/telecharger/` | 400 sans jeton, 403 sur jeton inconnu |
| `GET :5000/v2/ss-node/tags/list` | `{"name":"ss-node","tags":["v2"]}` |

**Chaîne complète du manifeste** — l'endpoint lu avec une session éditeur réelle et un `Host` de `10.128.177.37:5173` renvoie l'archive réelle (35 734 279 octets, SHA256 correct) et la commande `docker pull 10.128.177.37:5000/ss-node:v2`, le registre étant signalé joignable avec le tag `v2`.

**Montage des médias** — le conteneur backend voit bien `/app/media/node-images/ss-node-v2.tar.gz` (montage lié `./media`, ajouté : le service n'avait aucun volume, l'archive produite sur l'hôte lui était donc invisible).

**Accès entrant réel, à travers le pare-feu** — un `curl` lancé **depuis un conteneur Docker** vers l'IP LAN de l'hôte. Le trafic sort de la machine virtuelle Docker et rentre par la carte réseau : il traverse donc réellement le pare-feu Windows, contrairement à un appel en boucle locale qui l'aurait contourné et n'aurait rien prouvé.

| Depuis un conteneur | Résultat |
| :--- | :--- |
| `http://10.128.177.37:5173/` | 200 |
| `http://10.128.177.37:5173/api/auth/csrf/` | 200 |
| `http://10.128.177.37:5000/v2/ss-node/tags/list` | 200 |

Les cinq règles ont par ailleurs été relues après coup (`Get-NetFirewallRule`) : toutes présentes, activées, entrantes, en autorisation, sur les trois profils.

### Ce qui n'a pas pu être vérifié

- **L'accès depuis une seconde machine physique** : je n'en ai pas. Le test par conteneur ci-dessus est le plus proche équivalent — il valide le chemin entrant complet (pare-feu, publication de port Docker, nginx, Django) — mais il ne traverse pas le Wi-Fi lui-même.
- **Le déploiement effectif d'un nœud sur une machine PME** (installeur joué de bout en bout) : nécessite une seconde machine, et un tenant — la base ne contient aujourd'hui aucun tenant actif, seulement le compte éditeur `admin`. Le premier test consistera donc à créer une PME via `/register`.
- **L'installeur Windows n'a pas été exécuté.** Sa logique de déclaration du registre et de repli sur l'archive est vérifiée par test au niveau du contenu généré, pas par une exécution réelle sur un poste Windows vierge.

---

## 8. Fichiers touchés

**Créés**
- `dashboard/api_images.py` — état et téléchargement de l'image
- `frontend/src/pages/DeploiementPage.tsx` — page « Déploiement »
- `scripts/ouvrir-pare-feu-lan.ps1` — ouverture du pare-feu
- `media/node-images/` — archive et manifeste (non versionnés)
- `RAPPORT-DEPLOIEMENT-V2.md` — ce document

**Modifiés**
- `frontend/nginx.conf` — proxy `/api`, `/admin`, `/static`, `/media`
- `frontend/src/api/client.ts` — base d'API relative
- `config/settings.py` — détection des IP locales, origines de confiance, `REGISTRY_PORT`
- `docker-compose.yml` — service `registry`, montage `./media` sur le backend
- `.env` / `.env.example` — `ALLOWED_HOSTS`, `REGISTRY_PORT`, `SS_NODE_IMAGE`
- `tenants/views_web.py` — résolution du marqueur `{registry}`
- `templates/tenants/install-linux.sh.tmpl`, `install-windows.bat.tmpl` — registre et repli archive
- `spike/build-node.ps1` — pipeline de publication complet
- `config/urls.py`, `frontend/src/App.tsx`, `Layout.tsx`, `types/index.ts` — câblage
- `api/tests.py` — 23 → 40 tests
- `CONTRAT-API.md` — nouvelles routes documentées

Aucun commit n'a été fait : tout est dans l'arbre de travail, à votre main.

---

## 9. Points de vigilance

1. **`ALLOWED_HOSTS=*`** — relâchement assumé pour le banc de test, à resserrer avant toute exposition réelle.
2. **Registre Docker en HTTP sans authentification** — quiconque atteint le port 5000 peut tirer *et pousser* des images. Acceptable sur un LAN de test fermé ; à passer en TLS + authentification au-delà.
3. **Le réseau Wi-Fi est classé « Public »** — traité en incluant ce profil dans les règles (§5). À garder en tête si vous changez de réseau : une carte reclassée peut faire retomber les ports dans un profil non couvert. `scripts\ouvrir-pare-feu-lan.ps1` est idempotent, le relancer suffit.
4. **Watchtower est actif** dans le `docker-compose.yml` livré aux PME (vérification horaire). Republier `ss-node:v2` avec le même tag met donc automatiquement à jour les nœuds déployés — pratique en test, à surveiller.
5. **Le conteneur backend ne monte pas le code source.** Après toute modification Python, `docker compose build backend && docker compose up -d backend`, sinon le conteneur sert l'ancienne image. Pour les tests, utiliser l'interpréteur local `env/Scripts/python.exe`.

---

## 10. Reprise du 1er septembre 2026 — changement de réseau et vérification de l'image

### 10.1 Nouvelle adresse

La machine a changé de réseau Wi-Fi entre les deux séances :

| | 31 août | 1er septembre |
| :--- | :--- | :--- |
| Réseau | Nothing is free 2 | **YaPasGiga 3** |
| Adresse | 192.168.7.7 | **10.128.177.37** |
| Passerelle | — | 10.128.177.69 |
| Classement Windows | Public | Public |

**Aucun fichier de configuration n'a eu à être modifié.** C'est la validation du parti pris du §3 : rien n'est figé, tout se dérive à l'exécution.

- Django a détecté `10.128.177.37` au démarrage et en a dérivé les origines de confiance (`http://10.128.177.37:5173`, `:8000`, `:80`, `:3000`).
- La commande de récupération d'image servie par l'API est devenue `docker pull 10.128.177.37:5000/ss-node:v2` d'elle-même.
- Les règles de pare-feu ont tenu : elles portent sur des **profils** (dont Public) et une portée `LocalSubnet` que Windows réévalue sur le nouveau sous-réseau `10.128.177.0/24`. Vérifié après coup, les cinq règles sont toujours actives.

### 10.2 Vérification de l'image publiée

Le contrôle demandé avant que des machines tirent l'image. Cinq points, tous vérifiés par commande :

| Contrôle | Résultat |
| :--- | :--- |
| Sources `spike/` postérieures à l'image ? | Non — dernier fichier source le 31/08 à 14:55, image construite à 16:11. Rien n'a bougé depuis. |
| Les trois tags pointent-ils la même image ? | Oui — `ss-node:v2`, `ss-node:latest` et `localhost:5000/ss-node:v2` = `sha256:a1d84c1f0159…` |
| Le registre sert-il bien cette image ? | Oui — `Docker-Content-Digest: sha256:a1d84c1f0159…`, plateforme `linux/amd64` |
| L'archive contient-elle la même image ? | Oui — index OCI embarqué : `sha256:a1d84c1f0159…`, tag `ss-node:v2` |
| Le binaire fonctionne-t-il ? | Oui — exécuté **depuis l'image du registre** : `ss-node 0.1.0`, six sous-commandes attendues |

L'archive a par ailleurs été recontrôlée : SHA256 recalculé identique au manifeste (`bcc22721…`), taille identique (35 734 279 octets), intégrité gzip validée.

**Conclusion : image locale, image du registre et archive téléchargeable sont un seul et même artefact**, construit à partir des sources actuelles. Ce que les autres machines vont tirer est bien la bonne image.

### 10.3 Un défaut de la configuration nginx, corrigé

Le changement de réseau a fait remonter un défaut qui n'avait pas de rapport avec lui, mais qui aurait fini par mordre.

**Symptôme** : après un redémarrage du backend, toute l'API répondait `502` derrière nginx, alors que le backend interrogé en direct sur `:8000` répondait `200`.

**Cause** : dans un bloc `upstream` — comme dans un `proxy_pass` littéral — **nginx ne résout le nom d'hôte qu'une seule fois, au démarrage**, et conserve l'adresse en cache pour toute la durée de vie du processus. Or l'IP d'un conteneur change à chaque recréation. nginx visait encore `172.18.0.4` quand le backend était reparti sur `172.18.0.7`. Le DNS du conteneur, lui, répondait correctement — ce qui rend le diagnostic trompeur.

**Correction** : passer par le résolveur DNS interne de Docker et une variable, ce qui force nginx à redemander le nom à chaque requête.

```nginx
resolver 127.0.0.11 valid=10s ipv6=off;
set $django backend:8000;
...
proxy_pass http://$django;
```

**Preuve du correctif** : le backend a été forcé de changer d'adresse (`172.18.0.7` → `172.18.0.10`, en faisant occuper l'ancienne IP par un conteneur jetable). L'API est restée en `200` à travers nginx **sans redémarrage du frontend**. Avec l'ancienne configuration, c'était un `502`.

Sans ce correctif, la panne serait survenue au premier `docker compose up -d backend` — c'est-à-dire à la première correction de code Python — en pleine session de test avec les autres machines.

### 10.4 Contrôles refaits après ces changements

- **40 tests Django** : OK.
- **`tsc -b` et `eslint`** : propres.
- **Accès via `10.128.177.37`** depuis l'hôte : `/` 200, `/deploiement` 200, `/api/auth/csrf/` 200, `/api/auth/me/` 401, `/api/tenants/` 403, `/api/node-image/` 403, `/admin/` 200, registre 200.
- **Accès depuis un conteneur Docker** (trafic traversant réellement le pare-feu Windows) : console 200, API 200, registre 200.
- **Chaîne du manifeste** relue avec une session éditeur : `disponible: true`, registre joignable, tag `v2`, commande `docker pull 10.128.177.37:5000/ss-node:v2`.

### 10.5 Un point à surveiller

`spike/build-node.ps1` se trouve **à l'intérieur du contexte de build Docker** (`spike/`), et `spike/.dockerignore` ne l'exclut pas. Toute modification du script invalide donc la couche `COPY . .` et déclenche une recompilation Rust complète, alors que le binaire produit est identique. Sans conséquence sur la validité de l'image, mais coûteux en temps et source de digests qui changent sans raison. À corriger en ajoutant `*.ps1` à `spike/.dockerignore` lors d'une prochaine passe.

---

## 11. Compte de test PME et adresses à communiquer

### 11.1 Identifiants

Compte créé le 1er septembre via le vrai parcours réseau (`POST /api/auth/register/` depuis l'IP LAN, réponse `201`), pas par un raccourci en base.

| | |
| :--- | :--- |
| **Adresse** | **`http://10.128.177.37:5173/`** |
| **Utilisateur** | `pme` |
| **Mot de passe** | `amane2026` |
| Entreprise | PME Test |
| Licence | starter, **3 sièges** — donc jusqu'à 3 nœuds |
| Jeton d'enregistrement | `f49b6360-e87e-40c8-a9ab-5c4411f93744` |

Un **seul** compte pour les deux machines : elles sont deux nœuds du même cluster PME, pas deux clients. Chacune génère son propre `installation_id` et apparaît comme un appareil distinct sous ce tenant.

Mot de passe volontairement simple pour le banc de test — à ne pas reconduire ailleurs.

### 11.2 Le bon port, et ce que renvoient les autres

Les deux symptômes rencontrés depuis les autres machines s'expliquent entièrement :

- **« Identifiants invalides »** — la base ne contenait *aucun* compte PME, seulement `admin`. Aucun identifiant ne pouvait fonctionner. Corrigé par la création ci-dessus.
- **« Page blanche »** — le port **5000** répond `200` avec un corps **vide de 0 octet** (c'est l'API du registre Docker, pas une page web). Un navigateur affiche donc une page parfaitement blanche, sans erreur. C'est le piège.

| Port | Ce qu'il sert | À utiliser dans un navigateur ? |
| :--- | :--- | :--- |
| **5173** | **Console React — le point d'entrée** | **Oui, uniquement celui-ci** |
| 8000 | Django : API + `/admin/` + anciennes pages HTML | Non — mène à l'ancienne interface |
| 5000 | Registre Docker (API seule) | **Non — page blanche garantie** |
| 3000 | Grafana | Seulement pour les tableaux de bord |

### 11.3 Vérifié

- `POST /api/auth/register/` depuis l'IP LAN → `201`, tenant et licence d'essai créés.
- `POST /api/auth/login/` avec `pme` / `amane2026` → `200`, rôle `tenant`.
- `GET /api/tenants/me/` avec la session → `200` : PME Test, licence starter 3 sièges, 0 appareil.
- Bundle React servi depuis l'extérieur : `index.html` 471 o, JS 425 359 o, CSS 846 o — tous en `200`. Le serveur n'était pas en cause dans la page blanche.

### 11.4 Détail cosmétique

L'onglet du navigateur affiche « frontend » (titre Vite par défaut, `frontend/index.html`). Sans effet fonctionnel, mais peu engageant pour les personnes qui se connectent. À remplacer par « AMANE — Console » lors d'une prochaine reconstruction du frontend.

---

## 12. Procédure — ajouter une machine comme nœud de la PME Test

À suivre **sur chacune des deux machines**, sous Windows avec Docker Desktop. Compter 10 minutes par machine.

### 12.0 Ce qui bloquait

```
failed to resolve reference "10.128.177.37:5000/ss-node:v2":
Head "https://10.128.177.37:5000/v2/ss-node/manifests/v2":
http: server gave HTTP response to HTTPS client
```

Docker **exige du TLS sur tout registre**, sauf ceux explicitement déclarés comme non sécurisés. Le registre de l'éditeur est en HTTP simple (banc de test), et Docker a donc tenté du HTTPS puis refusé la réponse en clair.

C'est l'**installeur** qui règle cela automatiquement, pas le `docker-compose.yml` seul. En téléchargeant uniquement le compose, on saute cette étape. L'étape 1 ci-dessous la rejoue à la main.

### 12.1 Déclarer le registre de l'éditeur (la cause de l'erreur)

Dans **Docker Desktop** → icône ⚙️ **Settings** → **Docker Engine**. Cet écran édite le fichier `%USERPROFILE%\.docker\daemon.json`.

Contenu **par défaut** (relevé sur le PC serveur) :

```json
{
  "builder": {
    "gc": {
      "defaultKeepStorage": "20GB",
      "enabled": true
    }
  },
  "experimental": false
}
```

Contenu **après modification** — la ligne à ajouter est la dernière :

```json
{
  "builder": {
    "gc": {
      "defaultKeepStorage": "20GB",
      "enabled": true
    }
  },
  "experimental": false,
  "insecure-registries": ["10.128.177.37:5000"]
}
```

`insecure-registries` est une clé **de premier niveau**, au même rang que `builder` et `experimental` — surtout pas imbriquée dans `builder`. Deux erreurs qui font échouer l'application des réglages :

- **la virgule après `false`** est obligatoire, sans elle le JSON est invalide ;
- **pas de `http://`** dans la valeur : on écrit `"10.128.177.37:5000"`, jamais `"http://10.128.177.37:5000"`.

Si le JSON diffère légèrement (versions de Docker Desktop), la règle ne change pas : ajouter la clé au premier niveau, à côté de `experimental`.

Cliquer sur **Apply & Restart** et attendre que Docker redémarre complètement. Contrôler ensuite :

```powershell
docker info | Select-String -Pattern "Insecure" -Context 0,5
```

`10.128.177.37:5000` doit figurer dans la liste affichée.

> **Pourquoi le problème n'apparaît que sur les autres machines.** Le démon déclare d'office `127.0.0.0/8` et `::1/128` comme non sécurisés : sur le PC serveur, `localhost:5000` passe donc sans aucun réglage. Depuis une autre machine, `10.128.177.37:5000` est une adresse ordinaire, et Docker y exige du TLS. Rien n'est cassé côté serveur — c'est le premier accès réellement distant au registre qui révèle la contrainte.

Vérifier que c'est pris en compte :

```powershell
docker pull 10.128.177.37:5000/ss-node:v2
```

La commande doit se terminer par `Status: Downloaded newer image`. Si l'erreur HTTPS persiste, Docker n'a pas fini de redémarrer, ou le JSON est mal formé (une virgule manquante suffit).

### 12.2 Récupérer le docker-compose.yml

1. Ouvrir **`http://10.128.177.37:5173/`** — *ce port et aucun autre* (le 5000 renvoie une page blanche, c'est l'API du registre).
2. Se connecter : **`pme`** / **`amane2026`**.
3. Depuis `/mon-espace`, télécharger le `docker-compose.yml`.

Le fichier est déjà pré-rempli avec le jeton de la PME, l'adresse du serveur éditeur et la référence de l'image.

### 12.3 Renseigner l'adresse de la machine — à ne pas oublier

Le fichier téléchargé contient un emplacement à compléter :

```yaml
NODE_ADDR: "<IP_VM>:9001"
```

C'est l'adresse que le nœud **annonce aux autres** : laissée telle quelle, la machine s'enregistre avec une adresse invalide et le cluster ne pourra pas la joindre.

Relever l'IP de la machine :

```powershell
ipconfig
```

Prendre l'**adresse IPv4** de la carte connectée au réseau (`10.128.177.x`), puis remplacer :

```yaml
NODE_ADDR: "10.128.177.52:9001"    # exemple — mettre l'IP réelle de CETTE machine
```

### 12.4 Démarrer le nœud

Dans le dossier contenant le `docker-compose.yml` :

```powershell
docker compose up -d
docker compose logs -f node1
```

Ouvrir le port inter-nœuds dans le pare-feu de la machine (PowerShell **administrateur**) :

```powershell
New-NetFirewallRule -DisplayName "AMANE - Noeud PME (TCP 9001)" `
  -Direction Inbound -Action Allow -Protocol TCP -LocalPort 9001 `
  -Profile Domain,Private,Public -RemoteAddress LocalSubnet
```

### 12.5 Vérifier côté éditeur

Sur le PC serveur, dans la console :

- **`/devices`** — la machine apparaît, avec son `installation_id`, son nom d'hôte et son adresse ;
- **`/clusters`** — elle est rattachée à « PME Test », en rôle `primary` ou `standby`.

Le premier nœud qui s'annonce devient primaire ; le second passe automatiquement en standby, et le SaaS rétrograde tout primaire concurrent (garde-fou anti-split-brain).

### 12.6 Solution de rechange : sans toucher à Docker Desktop

Si l'étape 12.1 est impossible (poste verrouillé, pas d'accès aux réglages Docker), l'image se récupère en archive — aucune configuration de registre n'est alors nécessaire :

```powershell
curl.exe -L -o ss-node-v2.tar.gz "http://10.128.177.37:5173/api/node-image/telecharger/?token=f49b6360-e87e-40c8-a9ab-5c4411f93744"
docker load -i ss-node-v2.tar.gz
docker tag ss-node:v2 10.128.177.37:5000/ss-node:v2
docker compose up -d
```

Le `docker tag` est indispensable : il fait correspondre l'image chargée localement à la référence écrite dans le `docker-compose.yml`, ce qui évite à Docker d'aller la chercher sur le réseau.

Cette route est authentifiée par le jeton de la PME, pas par une session : elle fonctionne depuis une machine où personne n'est connecté. Débit mesuré sur le banc : ~8,7 Mo/s, soit environ 4 secondes pour les 35 Mo.

### 12.7 Points à connaître

- **Une seule PME pour les deux machines.** Elles sont deux nœuds du même cluster, pas deux clients. La licence porte **3 sièges** : une troisième machine est possible, une quatrième serait refusée (`licence_exceeded`).
- **Watchtower a besoin d'Internet.** Le service `updater` du compose tire `containrrr/watchtower` depuis le Docker Hub. Sans accès Internet il échoue — sans bloquer le nœud lui-même. En environnement fermé, supprimer ce service du fichier.
- **Failover manuel uniquement à deux nœuds.** Le basculement automatique par quorum exige **3 nœuds** minimum.
- **Le relais affiché « hors ligne » est normal** : aucun conteneur de relais ne tourne dans cette stack, il se déploie séparément depuis `relay-deploy/`.

---

## 13. Cas Linux / Fedora — pas de Docker Desktop

Sur Fedora il n'y a pas d'écran « Docker Engine » : la configuration se fait dans un fichier. Et un piège précède tout le reste — **Fedora livre `podman` par défaut**, pas Docker. Le paquet `podman-docker` fournit même une commande `docker` qui est en réalité podman. Dans ce cas, `/etc/docker/daemon.json` n'a **aucun effet** : podman ne le lit pas.

### 13.0 Identifier le moteur — à faire en premier

```bash
docker --version          # "Docker version 27.x"  -> voie A
                          # "podman version 5.x"   -> voie B
readlink -f "$(command -v docker)"   # s'il pointe sur .../podman -> voie B
```

### 13.A Voie A — Docker Engine (moby-engine ou docker-ce)

Le fichier n'existe pas sur une installation neuve, il faut le créer :

```bash
sudo mkdir -p /etc/docker
sudo tee /etc/docker/daemon.json > /dev/null <<'EOF'
{
  "insecure-registries": ["10.128.177.37:5000"]
}
EOF

sudo systemctl restart docker
```

**Si le fichier existait déjà**, ne pas l'écraser : ouvrir avec `sudo nano /etc/docker/daemon.json` et ajouter la clé `insecure-registries` **au premier niveau**, comme sous Windows (voir §12.1 pour la syntaxe et les deux erreurs classiques).

Vérifier :

```bash
docker info | grep -A3 "Insecure Registries"
docker pull 10.128.177.37:5000/ss-node:v2
```

### 13.B Voie B — Podman

Podman n'a pas de démon à redémarrer et ignore `/etc/docker/daemon.json`. Sa configuration de registres vit dans `/etc/containers/registries.conf`. Plutôt que de modifier ce fichier système, on dépose un fichier additionnel — plus propre, et supprimable d'un `rm` :

```bash
sudo mkdir -p /etc/containers/registries.conf.d
sudo tee /etc/containers/registries.conf.d/amane.conf > /dev/null <<'EOF'
[[registry]]
location = "10.128.177.37:5000"
insecure = true
EOF
```

Aucun redémarrage nécessaire. Vérifier :

```bash
podman pull 10.128.177.37:5000/ss-node:v2
```

> **Attention à `docker compose` sous podman** : la commande n'existe pas telle quelle. Utiliser `podman compose` (podman 4.7+) ou installer `podman-compose` (`sudo dnf install podman-compose`). Le `docker-compose.yml` téléchargé reste valable.

### 13.C La voie recommandée sur Linux : l'installeur

Sous Linux, l'installeur fait **tout** ce qui précède et bien plus, sans configuration manuelle. C'est le chemin à privilégier, contrairement à Windows où le compose seul suffit souvent.

```bash
# Depuis /mon-espace sur http://10.128.177.37:5173/, télécharger l'installeur Linux
chmod +x install-pme-test.sh
sudo ./install-pme-test.sh
```

Il enchaîne : détection de l'IP de la machine, installation de Docker s'il manque, déclaration du registre non sécurisé (fusion idempotente dans `daemon.json`), interrogation du SaaS pour savoir si ce nœud sera **primaire ou standby**, mise en place de PostgreSQL avec réplication, pgAdmin, ouverture des ports dans **firewalld** (5432, 5050, 9001), et enregistrement d'un service `systemd` pour un démarrage automatique.

Il exige les droits root et il est **idempotent** : le relancer ne casse rien.

### 13.D Spécificités Fedora à connaître

- **firewalld** est actif par défaut. L'installeur ouvre les ports ; en partant du compose seul, le faire à la main :
  ```bash
  sudo firewall-cmd --permanent --add-port=9001/tcp
  sudo firewall-cmd --reload
  ```
- **SELinux** est en mode *enforcing*. Sans effet ici : le compose n'utilise que des volumes nommés, pas des montages liés d'un dossier de l'hôte, donc pas besoin du suffixe `:z`.
- **`NODE_ADDR`** doit être renseigné comme sous Windows (§12.3), avec l'IP relevée par `ip -4 addr show` au lieu de `ipconfig`.

---

## 14. Deux nœuds annoncés « passif », et l'affichage en ligne / hors ligne

Constaté sur le banc avec deux machines installées depuis le `docker-compose.yml` téléchargé.

### 14.1 Pourquoi les deux nœuds s'annonçaient « passif »

Le nœud décide du rôle qu'il déclare au portail à partir d'une **variable d'environnement** :

```rust
// spike/node/src/cmd/run.rs
let role = match std::env::var("NODE_MODE").as_deref() {
    Ok("active") => "primary",
    _            => "standby",   // tout le reste, variable absente comprise
};
```

Or le `docker-compose.yml` généré passait le mode en **argument de ligne de commande** (`command: ["run", "--mode", "active"]`) sans jamais définir `NODE_MODE`. La variable étant absente, les deux nœuds tombaient dans la branche par défaut et s'annonçaient `standby`, quel que soit l'argument.

L'installeur Linux, lui, définissait bien `NODE_MODE` (il interroge `/api/devices/cluster-status/` pour savoir s'il existe déjà un primaire) — d'où un comportement correct par cette voie, et faux par le compose seul. L'installeur Windows ne la définissait pas non plus.

**Conséquence moins visible mais plus sérieuse** : l'entrypoint de l'image lit la même variable pour choisir entre `ss-node init --first`, qui **génère la DEK**, et `ss-node init` simple. Sans `NODE_MODE`, aucun nœud n'exécutait jamais `--first`.

**Correctif** (`templates/tenants/docker-compose.yml.tmpl`) : la variable est désormais déclarée, avec interpolation docker compose et `active` par défaut.

```yaml
NODE_MODE: "${NODE_MODE:-active}"
command: ["run", "--mode", "${NODE_MODE:-active}"]
```

Le premier nœud n'a rien à faire. Les suivants surchargent la variable :

```bash
NODE_MODE=passive docker compose up -d          # Linux
```
```powershell
$env:NODE_MODE="passive"; docker compose up -d  # Windows
```

Verrouillé par le test `test_compose_declare_node_mode`.

> **Attention aux volumes existants.** L'entrypoint ne lance `init` que si `config.toml` est absent. Une machine déjà démarrée sans `NODE_MODE` garde son état : pour qu'elle s'initialise réellement en primaire, il faut repartir d'un volume vide (`docker compose down -v`).

### 14.2 Pourquoi un nœud apparaît « hors ligne » alors qu'il tourne

**Il n'existe aucun battement de cœur vers le SaaS.** Le nœud n'appelle `/api/devices/register/` qu'à deux occasions : au démarrage, et lors d'un changement de rôle détecté.

Or `Device.last_seen` est en `auto_now=True` : il n'est mis à jour qu'à cet appel. Et l'état « en ligne » se calcule comme `last_seen` de moins de **2 minutes**.

**Tout nœud en bonne santé bascule donc en « hors ligne » deux minutes après son démarrage.** Relevé sur le banc : dernières annonces à 725 s et 333 s — les deux appareils étaient hors ligne, le « en ligne » observé étant simplement le nœud le plus récemment redémarré.

Ce n'est pas un défaut de configuration mais une **fonctionnalité manquante** dans le nœud : il faudrait une tâche périodique ré-annonçant le nœud (toutes les 60 s par exemple), ce qui rafraîchirait `last_seen` et rendrait l'indicateur fiable. Correctif non appliqué à ce stade : il touche le code Rust et impose de reconstruire puis republier l'image, donc de faire retirer la nouvelle image aux machines déjà déployées.

### 14.3 Troisième anomalie relevée dans les données

Le premier appareil enregistré portait `node_addr = "<IP_VM>:9001"` : l'emplacement du §12.3 n'avait pas été remplacé. Le nœud s'était donc annoncé avec une adresse littéralement invalide, injoignable par les autres nœuds. Il était par ailleurs `is_active = False`, à réactiver depuis `/devices`.

---

## 15. L'installeur Windows se fermait sans message

Symptôme signalé : double-clic sur l'installeur, une fenêtre `cmd` s'ouvre et se referme aussitôt, sans succès ni message exploitable.

### 15.1 Cause — des caractères non-ASCII que j'avais introduits

`cmd` lit un fichier `.bat` **par décalage d'octets**, et non ligne à ligne en mémoire. Un caractère multi-octets lui fait perdre sa position après chaque `goto` : il reprend la lecture au milieu d'une ligne et exécute des fragments tronqués.

En ajoutant la gestion du registre non sécurisé (§12), j'avais glissé des guillemets français `« »` et un tiret cadratin dans des commentaires. Le reste du gabarit était en **ASCII pur** — l'auteur d'origine l'avait manifestement fait exprès, aucun accent nulle part.

Reproduction, sur deux copies identiques du fichier à ces caractères près :

```
===== A : avec les guillemets français =====
'OCKER_READY' is not recognized as an internal or external command,
'ER_DL_ERROR' is not recognized as an internal or external command,
'RREUR' is not recognized as an internal or external command,

===== B : tout en ASCII =====
 Installation terminee
```

`OCKER_READY`, `ER_DL_ERROR`, `RREUR` sont des **fragments tronqués** de labels et de lignes : la signature d'une lecture à un mauvais décalage.

**Correctif** : gabarit ramené en ASCII pur, et le nom du tenant — seule donnée variable pouvant réintroduire des accents — est désormais replié en ASCII (`unicodedata.normalize('NFKD', …)`) avant injection.

### 15.2 Deuxième défaut trouvé en chemin : un label dans un bloc parenthésé

J'avais écrit :

```bat
if /I "!REG_STATE!"=="MODIFIE" (
    ...
:WAIT_DOCKER_REG          <-- label DANS le bloc
    ...
    goto WAIT_DOCKER_REG
) else (
    echo Registre deja autorise.
)
```

Contrairement à ce que j'ai d'abord supposé, cmd ne refuse pas ce fichier — c'est plus sournois : **le flux est corrompu**. Vérifié en exécutant les deux structures côte à côte, l'ancienne version exécute la boucle *puis enchaîne sur la branche `else`*, comme si le bloc n'avait plus de structure.

Réécrit en enchaînement par `goto`, sans aucun label à l'intérieur de parenthèses.

### 15.3 Troisième défaut : `NODE_MODE` absent du compose écrit par l'installeur Windows

Même bug qu'au §14, à un autre endroit : le `docker-compose.yml` que l'installeur Windows *génère* ne déclarait pas `NODE_MODE`. Tout nœud installé par cette voie s'annonçait donc `standby` et n'exécutait jamais `ss-node init --first`. La variable est maintenant écrite, avec la valeur décidée par le portail selon l'état du cluster.

### 15.4 Tests de non-régression

Quatre tests ajoutés (`InstalleurWindowsTests`), sur l'installeur réellement servi par l'API :

| Test | Ce qu'il verrouille |
| :--- | :--- |
| `test_installeur_est_en_ascii_pur` | Aucun octet > 127 dans le `.bat` |
| `test_nom_du_tenant_est_replie_en_ascii` | Un tenant nommé « Boulangerie Éclair & Cie — Crémerie » ressort en ASCII |
| `test_aucun_label_dans_un_bloc_parenthese` | Compte les parenthèses ouvrantes/fermantes et refuse tout label à profondeur > 0 |
| `test_compose_genere_porte_node_mode` | `NODE_MODE` présent dans le compose écrit |

Le premier a été vérifié en conditions d'échec : en réintroduisant un seul `«` dans le gabarit, il tombe en `FAILED`.

**52 tests au total, tous passants.** L'installeur régénéré déroule désormais ses étapes et, en cas de problème, affiche une erreur explicite au lieu de disparaître.

### 15.5 Règle à retenir

**Un fichier `.bat` de ce dépôt reste en ASCII pur.** Pas d'accents, pas de guillemets typographiques, pas de tirets cadratins — y compris dans les commentaires. La même prudence vaut pour les `.ps1` lus par Windows PowerShell 5.1, qui interprète un fichier UTF-8 sans BOM selon la page de codes ANSI (rencontré sur `scripts/ouvrir-pare-feu-lan.ps1`, §5).

---

## 16. Faut-il réinstaller les machines après le correctif du heartbeat ?

Question posée avant d'engager le correctif du §14.2 : une fois le battement de cœur ajouté au nœud, faut-il supprimer et retélécharger l'installeur sur la machine 1 (Windows) et la machine 2 (Fedora) ?

### 16.1 La réponse

**Non, pas pour recevoir le correctif.** Le heartbeat est du code Rust *dans l'image*. Les installeurs, eux, ne produisent que des fichiers de configuration — `docker-compose.yml`, `.env`, service systemd, tâche planifiée — qu'aucune ligne du correctif ne modifie. Retélécharger l'installeur ne changerait donc strictement rien.

Le point décisif est l'identité de l'installation. `installation_id` est `config.node_id`, écrit dans `/data/config.toml`, **à l'intérieur du volume du nœud** (`spike/node/src/cmd/run.rs`, champ `installation_id` de la charge envoyée à `/api/devices/register/`). Tant que le volume survit :

- la machine se ré-annonce avec le **même** `installation_id` ;
- `Device.objects.get_or_create(installation_id=…)` retrouve la **même ligne** côté SaaS ;
- aucun siège de licence supplémentaire n'est consommé, aucun nettoyage éditeur n'est requis.

**Oui, en revanche, si l'objectif est de retester les deux installeurs** depuis une machine vierge — ce qui est le cas ici. Voir la voie B.

### 16.2 Voie A — mettre à jour l'image sans rien désinstaller (recommandée en production)

| Machine | Comportement |
| :--- | :--- |
| **1 — Windows** | Le compose écrit par `install-windows.bat` embarque `containrrr/watchtower` (`WATCHTOWER_POLL_INTERVAL: 3600`) : la nouvelle image est tirée et le conteneur recréé **automatiquement sous une heure**. |
| **2 — Fedora** | Le compose écrit par `install-linux.sh` contient postgres + ss-node + pgadmin, **sans watchtower** : la mise à jour est **manuelle**. |

Forcer la mise à jour, machine 1 (PowerShell administrateur) :

```powershell
cd "$env:PROGRAMDATA\ElBaraaConsult\PME"
docker compose pull
docker compose up -d
```

Machine 2 (Fedora, root) :

```bash
cd /opt/elbaraa-pme
docker compose pull ss-node
docker compose up -d ss-node
```

**Deux pièges :**

1. **Le tag reste `v2`.** `docker compose up -d` seul ne retire rien : Docker voit le tag déjà présent localement et ne contacte pas le registre. Le `pull` explicite est obligatoire. C'est aussi ce qui fait fonctionner watchtower : il compare les digests d'un **même tag**. Republier sous un tag neuf (`v2.1`) casserait au contraire la mise à jour automatique, puisque les compose déjà déployés référencent `v2`.
2. **Ne jamais faire `docker compose down -v` pour une mise à jour.** Le `-v` détruit le volume, donc `config.toml` **et la DEK**. Sur un nœud primaire en production, les données chiffrées existantes deviennent illisibles. `pull` + `up -d` suffit et préserve tout.

Contrôle après mise à jour :

```bash
docker logs --tail 20 ss-node        # doit afficher « appareil enregistré »
docker image inspect --format '{{.Id}}' 10.128.177.37:5000/ss-node:v2
```

Puis, côté éditeur, la colonne « dernière annonce » de `/devices` doit se rafraîchir toute seule — c'est précisément ce que le correctif apporte.

### 16.3 Voie B — suppression complète, pour retester les deux installeurs

> **Réservé au banc de test.** Cette voie détruit le volume du nœud, donc la DEK. Sur une installation réelle, elle équivaut à une perte des données métier chiffrées.

**L'ordre compte.** Arrêter les nœuds PME **avant** de purger le SaaS : un nœud encore en marche se ré-annonce et recrée sa ligne dans la seconde qui suit.

#### B.1 — Machine 1 (Windows), PowerShell en administrateur

```powershell
schtasks /delete /tn "ElBaraaConsultPME" /f
cd "$env:PROGRAMDATA\ElBaraaConsult\PME"
docker compose down -v                       # -v : supprime le volume node1-data
cd $env:USERPROFILE
Remove-Item -Recurse -Force "$env:PROGRAMDATA\ElBaraaConsult"
netsh advfirewall firewall delete rule name="ElBaraaConsult-PME-Node"
docker volume ls                             # ne doit plus lister pme_node1-data
docker image rm 10.128.177.37:5000/ss-node:v2 ss-node:v2   # facultatif : force un pull neuf
```

Laisser en place l'entrée `insecure-registries` de `%PROGRAMDATA%\Docker\config\daemon.json` : elle sera nécessaire à la réinstallation, et l'installeur la détecte comme « déjà autorisée » (§12.1).

#### B.2 — Machine 2 (Fedora), en root

```bash
systemctl disable --now elbaraa-pme.service
rm -f /etc/systemd/system/elbaraa-pme.service && systemctl daemon-reload
cd /opt/elbaraa-pme && docker compose down -v
rm -rf /opt/elbaraa-pme
```

**Le piège Fedora — `down -v` ne suffit pas.** Les volumes `pg-data` et `node-data` sont déclarés en *bind* :

```yaml
  node-data:
    driver: local
    driver_opts: {type: none, o: bind, device: /opt/pme-node/node-data}
```

`down -v` supprime l'objet volume Docker, **pas le contenu du répertoire hôte**. `config.toml` et les données PostgreSQL survivent, et la réinstallation repart sur l'ancien `installation_id` — ce qui donne l'illusion que la purge a échoué. Il faut effacer les répertoires à la main :

```bash
rm -rf /opt/pme-node/node-data /opt/pme-node/pg-data
ls /opt/pme-node 2>/dev/null      # doit être vide ou absent
```

`pgadmin-data` est un volume nommé ordinaire : `down -v` l'emporte bien.

Conserver `/etc/docker/daemon.json` (insecure-registries) et les règles firewalld : l'installeur les repose de façon idempotente.

#### B.3 — Côté SaaS éditeur

Une fois les deux nœuds arrêtés, les lignes `Device` doivent disparaître **ou être désactivées** — sinon elles occupent les sièges de licence et `api_register.py` refuse la réinstallation avec `licence_exceeded`.

`Device.is_active = False` suffit pour les deux effets recherchés : le compte de sièges (`tenant.devices.filter(is_active=True).count()`) et la détection de rôle de l'installeur (`/api/devices/cluster-status/` ne renvoie que les appareils actifs) ignorent l'un comme l'autre les appareils inactifs.

| Voie | Comment | Effet |
| :--- | :--- | :--- |
| **Console React** — `/devices` | Bouton d'alimentation sur chaque ligne (`POST /api/devices/<uuid>/toggle/`) | Désactive : siège libéré, ligne conservée pour l'historique |
| **Admin Django** — `/admin/devices/device/` | Sélectionner les lignes → action « Supprimer » | Suppression définitive |
| **API** | `DELETE /api/devices/<uuid>/` (compte éditeur, `IsStaffUser`) | Suppression définitive |

Pour un banc de test remis à zéro, la suppression est plus lisible : la table `/devices` doit être **vide** pour le tenant avant de relancer les installeurs. Vérification :

```bash
curl -s "http://10.128.177.37:5173/api/devices/cluster-status/?token=<REGISTRATION_TOKEN>"
# attendu : {"tenant_name":"…","node_count":0,"active_nodes":[]}
```

Tant que `node_count` n'est pas à `0`, la première machine réinstallée ne se déclarera **pas** primaire (§14.1) : l'installeur lit ce compteur pour choisir entre `active` et `passive`.

#### B.4 — Réinstallation

L'ordre est imposé par la détection de rôle : le **premier** installeur lancé voit `node_count = 0` et devient primaire ; le second voit le primaire et demande son IP.

1. Machine 1 (Windows) — depuis `/mon-espace` sur `http://10.128.177.37:5173/`, télécharger `installer-pme.bat`, clic droit → *Exécuter en tant qu'administrateur*. Elle devient **primaire** (`NODE_MODE=active`, `ss-node init --first`).
2. Vérifier côté éditeur que la machine 1 est bien listée en `primary` **avant** de passer à la suivante.
3. Machine 2 (Fedora) — télécharger `installer-pme.sh`, puis `sudo bash installer-pme.sh`. Elle détecte le primaire, propose son IP et s'installe en **standby**.
4. Contrôler `/devices` : deux lignes, un `primary`, un `standby`, deux `node_addr` en IP réelles du LAN (pas `<IP_VM>`, pas `172.x` — §14.3).

**Ne pas télécharger les deux installeurs à l'avance.** `node_mode` est figé dans le fichier au moment du téléchargement (§14.1) : télécharger celui de la machine 2 *après* l'installation de la machine 1, sinon les deux fichiers portent `active` et l'on obtient deux primaires.

### 16.4 Ce qui survit à quoi

| Élément | `pull` + `up -d` (voie A) | `down` sans `-v` | `down -v` (Windows) | `down -v` (Fedora) |
| :--- | :---: | :---: | :---: | :---: |
| Code du nœud (image) | **remplacé** | conservé | conservé | conservé |
| `config.toml` / `installation_id` | conservé | conservé | **détruit** | conservé — bind, à effacer à la main |
| DEK | conservée | conservée | **détruite** | conservée — idem |
| Données PostgreSQL | conservées | conservées | s.o. (pas de PG sur cette voie) | conservées — bind, à effacer à la main |
| Ligne `Device` côté SaaS | conservée | conservée | orpheline, à purger | orpheline, à purger |
| Siège de licence | inchangé | inchangé | **occupé pour rien** | **occupé pour rien** |

### 16.5 À retenir

- Un correctif du **code du nœud** se diffuse par l'image, jamais par l'installeur.
- Un correctif des **installeurs ou du compose** (comme le §15) exige, lui, un retéléchargement — mais pas forcément une purge des volumes : réécrire les fichiers et relancer `up -d` suffit tant que `installation_id` doit être conservé.
- Toute purge de volume côté PME impose une purge correspondante côté SaaS, **sinon les sièges de licence restent consommés par des machines qui n'existent plus**.
- Sur Fedora, un volume déclaré en *bind* ne se vide pas avec `down -v` : c'est la cause la plus probable d'une « réinstallation » qui repart avec l'ancienne identité.

---

## 17. Battement de cœur du nœud et harmonisation des deux installeurs

Travail mené en deux lots parallèles sur des jeux de fichiers disjoints : **A** — le nœud Rust (`spike/node/src/cmd/run.rs`) ; **B** — la chaîne de déploiement Django (gabarits, vues, tests). Aucun fichier partagé, donc aucun conflit.

### 17.1 Le vrai diagnostic du §14.2

Le §14.2 concluait à une « fonctionnalité manquante ». C'est inexact, et la nuance change le correctif : **le battement de cœur existait**, mais il était enfermé dans la boucle de supervision PostgreSQL (`run_supervision_loop`, ré-enregistrement tous les `tick % 12`, soit 60 s). Or `run()` comporte trois chemins d'exécution :

| Chemin | Condition | Ré-annonce au SaaS |
| :--- | :--- | :---: |
| 1 | `PG_URL` défini **et** connexion réussie | oui, toutes les 60 s |
| 2 | `PG_URL` défini, connexion **échouée** | non — `return Ok(())`, le nœud **s'arrêtait** |
| 3 | `PG_URL` **absent** | non — boucle d'annonce au **relais** seulement, jamais au SaaS |

Les nœuds installés par l'installeur Windows n'avaient pas de PostgreSQL du tout : ils empruntaient le chemin 3. Le chemin 2, lui, faisait sortir le processus, donc redémarrer le conteneur en boucle (`restart: unless-stopped`) — un second mécanisme qui interdisait toute présence stable.

**Les deux sujets de cette section n'en font donc qu'un** : le nœud paraissait hors ligne *parce que* la pile Windows était amputée.

### 17.2 Lot A — le battement de cœur

`spike/node/src/cmd/run.rs`, +352/−82.

- **Tâche dédiée** lancée par `lancer_heartbeat_saas()` juste après l'enregistrement initial, **avant tout embranchement** : elle couvre donc les trois chemins ci-dessus.
- **Période** : 60 s par défaut, réglable par `SAAS_HEARTBEAT_SECS`, bornée à [10 s, 300 s] ; `0` désactive. La fonction `periode_heartbeat` est pure, donc testable sans toucher à l'environnement du processus. Le seuil « en ligne » de la console étant de 2 minutes (`ONLINE_THRESHOLD_MS`), un test garde-fou vérifie que le défaut reste en dessous.
- **Rôle réellement courant** : un canal `tokio::sync::watch` porte le rôle (`primary`/`standby`), initialisé depuis `NODE_MODE` puis mis à jour partout où le rôle change — correction par `pg_is_in_recovery()` au démarrage, promotion, failover automatique. Le heartbeat lit ce canal à chaque itération, il n'annonce donc jamais un rôle périmé.
- **Déduplication** : les quatre ré-enregistrements `tick % 12` de la boucle de supervision sont retirés. Ne subsistent que les annonces **immédiates sur bascule** (`majr_role` ne paie un aller-retour HTTP que si le rôle a réellement changé). Le régime nominal est entretenu par le seul heartbeat.
- **Journalisation** : silence en régime nominal. Seuls sont tracés le premier succès, chaque échec, et la reprise après échec — sinon une ligne toutes les 60 s rendrait `docker logs` illisible.
- **PostgreSQL injoignable** : `connecter_postgres_avec_reprise()` réessaie indéfiniment toutes les 10 s (une ligne de log par minute) au lieu de sortir de `run()`. Le heartbeat continue de tourner pendant ce temps, donc le nœud reste visible « en ligne » même avec sa base à terre — ce qui est précisément l'information utile pour l'exploitant.
- **Aucun clone de `NodeConfig`** : seul `node_id` (un `Uuid`, `Copy`) part dans la tâche. `NodeConfig` porte `secret_key_hex` et `sealed_dek_hex` ; dupliquer du matériel de clé sans nécessité contreviendrait à la règle « socle crypto » du dépôt. Tout le reste (`SAAS_URL`, `REGISTRATION_TOKEN`, `HOSTNAME`, `NODE_ADDR`, `WEB_PORT`) est relu dans l'environnement à chaque appel.

**Un piège `tokio` évité, qui vaut d'être retenu.** `watch::Sender::send` renvoie une erreur **et n'écrit pas** quand plus aucun récepteur n'existe. Avec `SAAS_HEARTBEAT_SECS=0`, le récepteur est détruit : un `send` classique aurait laissé le rôle figé, et chaque tick de supervision aurait re-détecté un « changement », donc déclenché une annonce HTTP toutes les 5 secondes. D'où l'usage de `send_replace`, verrouillé par le test `suivi_role_courant_tient_sans_recepteur`.

### 17.3 Lot B — une définition unique de la pile

Le problème de fond n'était pas « Windows est incomplet », mais **trois définitions divergentes de la même pile** : l'installeur Linux écrivait la sienne, l'installeur Windows la sienne, et `templates/tenants/docker-compose.yml.tmpl` en servait une troisième pour le téléchargement manuel. Rien ne les tenait en cohérence.

`docker-compose.yml.tmpl` devient donc **la seule description de la pile**, et les deux installeurs la **téléchargent** au lieu de la régénérer.

| Service | Image | Rôle | Ports |
| :--- | :--- | :--- | :--- |
| `postgres` | `postgres:16-bookworm` | Données métier, réplication physique primaire/standby | 5432 |
| `ss-node` | image du registre éditeur | Interface web métier, supervision, bascule | 9001 |
| `pgadmin` | `dpage/pgadmin4:8` | Consultation des données par la PME | 5050 |
| `updater` | `containrrr/watchtower` | Mise à jour automatique de l'image (1 h) | — |

Volumes nommés `pg-data`, `node-data`, `pgadmin-data` — **plus de volumes *bind*** vers `/opt/pme-node/` : ce montage n'existe pas sous Windows, et c'était l'obstacle direct à un compose unique. Effet de bord bienvenu : la purge par `docker compose down -v` fonctionne enfin réellement des deux côtés (voir §16.3, dont le piège Fedora disparaît pour les nouvelles installations).

**Aucun fichier annexe.** L'installeur Linux écrivait `postgres/entrypoint.sh`, `postgres/primary-init.sh` et `pgadmin/servers.json`, montés en volume. Reproduire l'écriture d'un script bash depuis un `.bat` aurait été un nid à bugs d'échappement (`>`, `|`, `%`) — la leçon du §15. La logique part donc **en ligne dans l'entrypoint** :

```yaml
entrypoint:
  - "/bin/bash"
  - "-c"
  - |
    ... bascule standby / pg_basebackup / écriture de 10-replication.sh ...
    exec /usr/local/bin/docker-entrypoint.sh "$$@"
  - "--"
```

Trois subtilités à connaître avant d'y toucher :
- les `$` sont écrits `$$` pour survivre à l'interpolation de Compose ;
- le `--` final sert de `$0` à `bash -c`, si bien que les éléments de `command:` (`postgres -c wal_level=replica …`) arrivent en `$@` et sont repassés intacts à l'entrypoint officiel de l'image ;
- côté primaire, l'entrypoint **écrit lui-même** `/docker-entrypoint-initdb.d/10-replication.sh`, que l'image exécutera au seul premier init : c'est le script d'origine mot pour mot, seule sa provenance change.

Le comportement PostgreSQL est identique à l'ancien script Linux : attente du primaire puis `pg_basebackup -Fp -Xs -P -R --checkpoint=fast` côté standby, création de `replicator` et lignes `scram-sha-256` dans `pg_hba.conf` côté primaire, options serveur `wal_level=replica`, `max_wal_senders=3`, `wal_keep_size=64`, `listen_addresses=*`, `synchronous_commit=remote_write`.

**Port web : 9001 partout.** Trois conventions coexistaient (le binaire prend 3000 par défaut, l'ancien gabarit publiait 9001 *et* 3000, l'installeur Linux imposait 9001). 9001 l'emporte : c'est déjà le port annoncé au portail dans `NODE_ADDR` et le seul ouvert par le pare-feu Linux.

**Le `.env` est le seul paramétrage**, 17 clés identiques des deux côtés : `NODE_ROLE`, `NODE_MODE`, `PG_USER`, `PG_PASSWORD`, `PG_DB`, `REPLICATION_PASSWORD`, `PG_REPLICATION_SUBNET`, `PRIMARY_IP`, `WEB_PORT`, `NODE_ADDR`, `NODE_HOSTNAME`, `RELAY_URL`, `SAAS_URL`, `REGISTRATION_TOKEN`, `TENANT_ID`, `PGADMIN_EMAIL`, `PGADMIN_PASSWORD`. Toutes ont une valeur par défaut dans le gabarit, donc le compose reste utilisable seul après un téléchargement manuel depuis la console.

### 17.4 Nouveau point d'entrée : la pile servie par jeton

```
GET /api/tenants/pme/compose/?token=<registration_token>
```

Calqué sur `node_image_telecharger_pme` (§2) : `AllowAny`, authentifié par le **jeton d'enregistrement** et non par session — l'installeur tourne sur une machine sans compte éditeur. Codes : **400** jeton absent, **400** UUID mal formé, **403** jeton inconnu, **403** tenant inactif, **200** sinon. Il réutilise `build_compose_response`, donc la voie par session et la voie par jeton servent rigoureusement le même contenu (verrouillé par test). La route est déclarée **avant** le routeur DRF, sans quoi `pme` serait capturé comme un `<pk>`.

### 17.5 Les deux installeurs, dix étapes identiques

1. Détection de l'IP de la machine → 2. Docker (installation si absent) → 3. Déclaration du registre non sécurisé → 4. Récupération de l'image (`docker pull`, **repli sur l'archive** puis `docker load` + `docker tag`) → 5. Interrogation de `/api/devices/cluster-status/` → 6. Dérivation des mots de passe → 7. Écriture du `.env` → 8. **Téléchargement du compose depuis le portail** → 9. Pare-feu (5432, 5050, 9001) et `docker compose up -d` → 10. Démarrage automatique au boot, puis récapitulatif.

Ce que chacun a gagné :

| Élément | Linux avant | Windows avant | Après (les deux) |
| :--- | :---: | :---: | :---: |
| PostgreSQL + réplication | oui | **non** | oui |
| pgAdmin | oui | **non** | oui |
| `ss-node` avec `PG_URL` (supervision, failover) | oui | **non** | oui |
| watchtower | **non** | oui | oui |
| Rôle décidé par `cluster-status` | oui | **non** | oui |
| Saisie de l'IP du primaire | oui | **non** | oui |
| Mots de passe dérivés du jeton | oui | **non** | oui |
| Fichier `.env` | oui | **non** | oui |
| `docker pull` + repli archive | **non** | oui | oui |
| Pare-feu 5432 / 5050 / 9001 | oui | **9001 seul** | oui |
| Compose téléchargé du portail | non | non | oui |

Différences résiduelles — elles portent sur *comment* l'OS fait, jamais sur *quoi* :

| Point | Linux | Windows |
| :--- | :--- | :--- |
| Répertoire | `/opt/elbaraa-pme` | `%PROGRAMDATA%\ElBaraaConsult\PME` |
| Élévation | `$EUID -ne 0` | `net session` |
| Installation Docker | `get.docker.com` + systemd | Docker Desktop `.exe` |
| Registre | `/etc/docker/daemon.json` + `systemctl restart docker` | `%ProgramData%\Docker\config\daemon.json` + `Restart-Service com.docker.service` |
| IP locale | `ip -4 addr` | `Get-NetIPAddress` |
| Lecture JSON | `python3 -c` | `Invoke-RestMethod` (python3 n'est pas garanti sous Windows) |
| Pare-feu | `ufw` / `firewall-cmd` | `netsh advfirewall` |
| Démarrage au boot | unité systemd | `schtasks /sc onstart /ru SYSTEM` |
| Saisie clavier | `read -rp` | `set /p` |

### 17.6 Le bug qui explique vos ennuis de registre du §12

**`cmd` n'applique pas l'échappement par caret à l'intérieur des guillemets d'un `for /f "usebackq"`.** Le `^` arrive tel quel à PowerShell, qui refuse la ligne — et la variable reste **vide, sans le moindre message**.

Vérifié sur cette machine, deux `for /f` identiques à un caret près :

```
1,2,3 ^| Measure-Object -Sum ^| Select-Object -ExpandProperty Sum
      ~
Unexpected token '^' in expression or statement.

AVEC_CARET=[]
SANS_CARET=[6]
```

L'installeur Windows du banc comptait **11 occurrences** de `^|`. Conséquences, qui recoupent exactement ce qui a été observé aux §12 et §14.3 :

- la détection d'IP retombait **toujours** sur `127.0.0.1`, d'où le `NODE_ADDR=127.0.0.1:9001` annoncé au portail ;
- `daemon.json` n'était **jamais** écrit, d'où les `docker pull` échouant en « server gave HTTP response to HTTPS client » alors que l'étape affichait « Registre deja autorise ».

Les 11 occurrences sont corrigées, la règle est documentée en tête du `.bat` et verrouillée par le test `test_aucun_tube_echappe_dans_un_for_f`. **Elle rejoint les deux règles du §15.5 : dans un `.bat` de ce dépôt, ASCII pur, aucun label dans un bloc parenthésé, et aucun tube échappé dans un `for /f`.**

### 17.7 Les mots de passe coïncident entre Windows et Fedora

C'est la condition sine qua non de votre test : la réplication ne démarre que si `REPLICATION_PASSWORD` est identique des deux côtés. Linux dérive `sha256sum` de `<jeton>-repl` tronqué à 24 caractères ; Windows fait `[Text.Encoding]::ASCII.GetBytes()` → `SHA256::ComputeHash()` → `ToLower().Substring(0,24)`.

Vérifié **à l'exécution** — le `.bat` réellement rendu par Django a été exécuté par `cmd.exe` et son `.env` comparé à `sha256sum`, sur deux jetons distincts :

```
Jeton a548e8a0-097a-4c2b-887b-80e4674486d6
  PG_PASSWORD   .bat 64a063dc6fdd219d0f1692e1   bash 64a063dc6fdd219d0f1692e1
  REPLICATION   .bat b25224932fc5bbe0f20223ab   bash b25224932fc5bbe0f20223ab
  PGADMIN       .bat f707ce4ebd9cfcd33f379db8   bash f707ce4ebd9cfcd33f379db8

Jeton a395d79c-3c98-43f5-b1d0-108638817d5e
  PG_PASSWORD   .bat 5d0ac2a42e43dc54bc3b5f67   bash 5d0ac2a42e43dc54bc3b5f67
  REPLICATION   .bat 23a8d1f9d20a65f327618a0c   bash 23a8d1f9d20a65f327618a0c
  PGADMIN       .bat 0847333bb7ec46375f81c761   bash 0847333bb7ec46375f81c761
```

Ces valeurs coïncident aussi avec `tenants/views_espace._derive_credentials`, qui alimente déjà l'affichage du portail.

### 17.8 Les deux machines sont enfin distinguables

Le gabarit figeait `HOSTNAME: node1`. Les deux installeurs servant désormais le même compose, **les deux machines du parc se seraient annoncées sous le même nom** : colonne « Hostname » inutilisable dans `/devices`, et impossible de savoir laquelle est primaire — exactement ce qu'il faut voir pendant un test de failover.

Le nom est désormais lu dans le `.env` (`HOSTNAME: "${NODE_HOSTNAME:-node1}"`), rempli par `hostname` côté Linux et `%COMPUTERNAME%` côté Windows. Verrouillé par `test_le_nom_de_machine_nest_pas_fige`.

### 17.9 Tests

**75 tests Django** (52 avant) et **9 tests Rust**, tous passants, rejoués indépendamment des agents qui les ont écrits.

Côté Django, quatre familles :

| Classe | Nb | Ce qu'elle verrouille |
| :--- | :---: | :--- |
| `InstalleurWindowsTests` (étendue) | 5 | ASCII pur, aucun label en bloc parenthésé, **aucun tube échappé**, `NODE_MODE` écrit dans le `.env` et lu par le compose |
| `ComposeParJetonTests` | 6 | Les cinq codes de retour du nouveau point d'entrée, et l'égalité des deux voies (session / jeton) |
| `PilePmeTests` | 5 | `PG_URL` et `service_healthy`, les 5 options de réplication, l'init sans fichier annexe, YAML valide, nom de machine non figé |
| `PariteInstalleursTests` | 11 | Chaque élément fonctionnel présent dans **les deux** installeurs (~70 assertions via `subTest`) |

La classe de parité est le vrai filet : elle tombe dès que l'un des deux installeurs régresse par rapport à l'autre.

**Rejouer les tests.** Django n'a pas besoin de la stack : un module de réglages SQLite suffit (Docker éteint, le PostgreSQL de dev sur 5434 est injoignable) —

```bash
PYTHONPATH=<répertoire du module> env/Scripts/python.exe manage.py test --settings=settings_test_sqlite
```

Rappel du §16 et de la mémoire projet : utiliser l'interpréteur du venv local, **jamais** `docker compose exec backend`, ce conteneur ne monte pas le code.

Côté Rust, **aucun toolchain n'est installé sur cette machine** : la compilation passe par l'image du Dockerfile, ce qui laisse le `target/` de l'hôte intact —

```bash
docker run --rm -v "C:/Users/Siradiou/saas-souverain/spike:/workspace" -w /workspace \
  -v amane_cargo_registry:/usr/local/cargo/registry -v amane_node_target:/workspace/target \
  rust:1.88-bookworm cargo test -p node
```

Les 9 tests portent sur la période du heartbeat (défaut, désactivation, valeurs invalides, bornes, garde-fou des 120 s), la traduction de `NODE_MODE`, et le suivi du rôle courant.

### 17.10 Conséquence directe sur les deux machines du banc

**La réponse du §16.1 ne s'applique plus à cette livraison.** Elle valait pour le heartbeat seul ; ici la pile elle-même change, et la machine Windows passe d'un nœud isolé à une pile complète avec PostgreSQL. Il faut donc appliquer la **voie B du §16.3** :

1. Purger les deux machines (§16.3 B.1 et B.2). Sur la Fedora, ne pas oublier les répertoires *bind* `/opt/pme-node/node-data` et `/opt/pme-node/pg-data` : l'ancienne pile les utilisait, la nouvelle non — s'ils subsistent, ils ne seront plus lus mais occuperont l'espace disque.
2. Purger les lignes `Device` côté éditeur (§16.3 B.3), sans quoi les sièges de licence restent consommés et la réinstallation est refusée.
3. **Republier l'image du nœud** — le correctif du heartbeat est du code Rust : `powershell -ExecutionPolicy Bypass -File spike\build-node.ps1`. Sans cette étape, les machines réinstalleraient l'ancien binaire.
4. Retélécharger les deux installeurs **dans l'ordre** : Windows d'abord (elle devient primaire), vérifier qu'elle apparaît en `primary` dans `/devices`, puis Fedora, qui détectera le primaire et s'installera en standby.

Ce qu'il devient alors possible de tester, et qui ne l'était pas : réplication PostgreSQL entre les deux OS, failover automatique du standby Fedora si la primaire Windows tombe, remontée du rôle au portail dans la seconde, et présence « en ligne » stable des deux machines.

### 17.11 Points ouverts, à trancher plus tard

- **`SAAS_HEARTBEAT_SECS` n'est déclaré dans aucun compose** : le défaut de 60 s s'applique partout. À exposer dans le `.env` si vous voulez le régler par machine.
- **`devices/api_register.py` complète `web_addr` avec le port 3000** quand le nœud n'annonce rien, alors que la pile publie 9001. Sans effet tant que `WEB_PORT` est renseigné, mais incohérent avec le port harmonisé.
- **pgAdmin ne pré-enregistre plus le serveur** : `servers.json` était un fichier annexe, il a disparu avec eux. La voie propre serait un bloc `configs:` en ligne dans le compose.
- **PyYAML n'est pas dans `requirements.txt`** : le test `test_le_compose_est_un_yaml_valide` se met en `skip` s'il est absent. Il passe en local, il sera silencieusement sauté en CI tant que la dépendance n'est pas déclarée.
- **Conflit possible sur le port 5432** si une machine Windows héberge déjà un PostgreSQL natif ; l'installeur ne le détecte pas.
- **Vocabulaire double** : le relais parle `active`/`passive`, le SaaS `primary`/`standby`. Aucune conséquence fonctionnelle, mais la lecture des logs y perd.
- **`POST /api/devices/register/` est lourd pour un simple battement de cœur** — il réécrit `hostname`, `os`, `node_addr`, `web_addr`, `node_role` chaque minute. Un point d'entrée dédié ne touchant que `last_seen` serait plus économe.

### 17.12 Publication effectuée — 2 septembre 2026

Livraison réellement déployée sur le banc, et non seulement écrite. État trouvé au départ : Docker venait d'être redémarré en cours de session, `etcd`, `postgres-primary` et `postgres-replica` étaient **exited**, et `backend` tournait en **crash-loop** (`could not translate host name "postgres-primary" to address`).

**1 — Socle remonté.** `docker compose up -d etcd postgres-primary postgres-replica`, puis attente de la santé Patroni sur `:8008`.

**2 — Backend reconstruit.** `docker compose up -d --build backend`. Le rebuild n'est pas facultatif : ce service **ne monte pas le code source** (voir §16 et la mémoire projet), il servait donc encore les gabarits de l'ancienne image. Contrôle par empreinte SHA-256, gabarit par gabarit :

| Gabarit | Conteneur vs dépôt |
| :--- | :--- |
| `install-windows.bat.tmpl` | identique |
| `install-linux.sh.tmpl` | identique |
| `docker-compose.yml.tmpl` | identique |

**3 — Image du nœud recompilée et poussée.** `powershell -ExecutionPolicy Bypass -File spike\build-node.ps1`.

| | Avant | Après |
| :--- | :--- | :--- |
| Image | `a1d84c1f0159` | **`89aace6eb1b1`** |
| Digest servi par le registre pour `v2` | — | `sha256:89aace6eb1b1ef9f6c32d1624c640b5d558e949b835ad514824f6aabd18a8da2` |
| Archive `ss-node-v2.tar.gz` | 35 734 279 o | **35 773 368 o** |
| SHA256 de l'archive | `bcc22721…` | **`d8d660b25fd10c12c41343c41ffac2540cf805a2b27a1f40e216da9d80cdb22f`** |
| Construite le | 31/08 20:03 UTC | **01/09 23:25 UTC** |

**Un tag local était resté en arrière.** `10.128.177.37:5000/ss-node:v2` pointait encore `a1d84c1f0159` : `build-node.ps1` n'étiquette que `ss-node:v2`, `ss-node:latest` et `localhost:5000/ss-node:v2`, alors que c'est la référence dérivée de l'IP LAN que les installeurs inscrivent dans le compose. Sans conséquence pour les PME — elles tirent du registre, qui sert bien le nouveau digest — mais trompeur sur cette machine. Réaligné à la main par `docker tag`. **À ajouter à `build-node.ps1`** pour que le cas ne se represente pas.

**4 — Contrôles effectués après publication.**

| Contrôle | Résultat |
| :--- | :--- |
| Le binaire porte le correctif | `grep -a` dans `/usr/local/bin/ss-node` : 2 occurrences de « battement de », 1 de `SAAS_HEARTBEAT_SECS` |
| Le binaire démarre | `ss-node --help` liste `init`, `run`, `status`, `failover`, `delist`, `adduser` |
| Registre | `{"name":"ss-node","tags":["v2"]}`, digest = image locale |
| Archive servie par l'API | `HTTP 200`, **35 773 368 octets** — exactement la nouvelle archive (`media/` est monté en volume, donc vu à chaud) |
| `GET /api/tenants/pme/compose/` | 400 sans jeton ; avec jeton valide, sert `postgres`, `ss-node`, `pgadmin`, `updater`, `PG_URL` et `HOSTNAME: "${NODE_HOSTNAME:-node1}"` |
| Installeur Windows rendu | HTTP 200, **10/10 étapes**, **ASCII pur**, cluster-status, pgAdmin, `NODE_HOSTNAME`, téléchargement du compose |
| Installeur Linux rendu | HTTP 200, **10/10 étapes**, mêmes éléments |

À ne pas mal lire : **aucun des deux installeurs ne mentionne watchtower dans son texte**, et c'est voulu — il vient du compose partagé que les deux téléchargent. Aucun installeur ne redéclare de service, c'est ce que verrouille le test de parité.

**5 — Effet immédiat sur les machines déjà déployées.** Le tag `v2` ayant été republié avec un nouveau digest, **watchtower tire la nouvelle image sur la machine Windows dans l'heure**. Sur son ancienne pile sans PostgreSQL, le nouveau binaire ne fait qu'ajouter le battement de cœur : elle cessera d'apparaître « hors ligne » sans aucune intervention. La Fedora, dont la pile actuelle n'a pas de watchtower, ne bougera pas : `docker compose pull ss-node && docker compose up -d ss-node` dans `/opt/elbaraa-pme`.

Cela ne dispense pas de la réinstallation complète du §17.10 : c'est elle, et elle seule, qui apporte PostgreSQL et pgAdmin à la machine Windows, donc la réplication et le failover entre les deux OS.

---

## 18. Inventaire des changements

Les sections 16 et 17 expliquent *pourquoi*. Celle-ci dit *quoi*, fichier par fichier, pour qui doit relire le diff, reprendre le chantier ou auditer ce qui a été livré.

### 18.1 Fichiers modifiés

| Fichier | Nature | Ce qui change |
| :--- | :--- | :--- |
| `spike/node/src/cmd/run.rs` | Rust — nœud PME | **+352 / −82.** Battement de cœur SaaS en tâche dédiée, canal `watch` du rôle courant, reconnexion PostgreSQL au lieu de la sortie, 9 tests unitaires. Seul fichier Rust touché. |
| `templates/tenants/docker-compose.yml.tmpl` | Gabarit Django | **Réécrit.** Passe de 2 services (`node1`, `updater`) à la pile complète : `postgres` + `ss-node` + `pgadmin` + `updater`. Initialisation PostgreSQL en ligne dans l'entrypoint. Devient la **définition unique de la pile**. |
| `templates/tenants/install-linux.sh.tmpl` | Gabarit Django | **Réécrit** en 10 étapes numérotées. Gagne `docker pull` + repli sur l'archive, `WEB_PORT`, `PG_REPLICATION_SUBNET`, `NODE_HOSTNAME`, `{% autoescape off %}`. Perd toute génération de fichiers de pile. |
| `templates/tenants/install-windows.bat.tmpl` | Gabarit Django | **Réécrit** en 10 étapes identiques. Gagne cluster-status, saisie de l'IP du primaire, dérivation des mots de passe, `.env`, téléchargement du compose, pare-feu 5432/5050/9001, `ss-node init --first`. **11 `^|` corrigés.** |
| `tenants/views_web.py` | Vue Django | + `compose_telecharger_pme` (ligne 338), + contexte `generated_at_ascii`, durcissement du repli ASCII du nom de tenant (retrait de `&<>|^%!()"`). |
| `tenants/urls_api.py` | Routage | + `path('pme/compose/', …)` (ligne 21), déclaré **avant** le routeur DRF. |
| `api/tests.py` | Tests | 1 test adapté, 1 ajouté à `InstalleurWindowsTests`, 3 classes créées. De 52 à **75 tests**. |

Le diff cumulé de ces fichiers par rapport au dernier commit affiche 1141 insertions et 457 suppressions, mais **ce chiffre est trompeur** : plusieurs de ces fichiers portaient déjà du travail non commité antérieur à ce chantier. Seul `run.rs` est intégralement issu de celui-ci (434 lignes changées = 352 + 82).

### 18.2 Ce qui a disparu

À connaître avant de relire une procédure ancienne : ces artefacts n'existent plus, et les chapitres qui les mentionnent sont caducs sur ce point.

| Disparu | Remplacé par |
| :--- | :--- |
| `postgres/entrypoint.sh` écrit par l'installeur Linux | Entrypoint en ligne dans le compose |
| `postgres/primary-init.sh` écrit par l'installeur Linux | Écrit par l'entrypoint lui-même dans `/docker-entrypoint-initdb.d/` |
| `pgadmin/servers.json` | Rien — pgAdmin ne pré-enregistre plus le serveur (§17.11) |
| Volumes *bind* `/opt/pme-node/pg-data` et `/opt/pme-node/node-data` | Volumes Docker nommés `pg-data`, `node-data` |
| Génération du compose par chaque installeur | Téléchargement depuis `/api/tenants/pme/compose/` |
| Ré-enregistrements `tick % 12` dans `run_supervision_loop` | Tâche de battement de cœur dédiée |
| Fonctions Rust `register_with_saas` et `register_with_saas_role` | `annoncer_au_saas`, `majr_role`, `lancer_heartbeat_saas` |

La disparition des volumes *bind* a un effet secondaire heureux : le piège Fedora du §16.3 — `down -v` qui ne vide rien — **ne concerne plus les installations faites avec le nouvel installeur**. Il reste valable pour purger les machines installées avec l'ancien.

### 18.3 Nouvelles interfaces

**Route HTTP** — une seule ajoutée :

```
GET /api/tenants/pme/compose/?token=<registration_token>
```

`AllowAny`, authentifiée par jeton d'enregistrement. 400 (jeton absent ou mal formé), 403 (jeton inconnu, tenant inactif), 200 sinon. Sert le même contenu que la voie par session `/api/tenants/me/download/compose/`.

**Variables d'environnement du nœud** :

| Variable | Rôle | Défaut |
| :--- | :--- | :--- |
| `SAAS_HEARTBEAT_SECS` | Période du battement de cœur ; `0` désactive ; borné à [10, 300] | 60 |
| `NODE_HOSTNAME` | Nom de la machine affiché dans la console éditeur | `node1` |

`SAAS_HEARTBEAT_SECS` n'est déclaré dans aucun compose : le défaut s'applique partout (§17.11).

**Clés du `.env`** : 17, identiques sur les deux OS — `NODE_ROLE`, `NODE_MODE`, `PG_USER`, `PG_PASSWORD`, `PG_DB`, `REPLICATION_PASSWORD`, `PG_REPLICATION_SUBNET`, `PRIMARY_IP`, `WEB_PORT`, `NODE_ADDR`, `NODE_HOSTNAME`, `RELAY_URL`, `SAAS_URL`, `REGISTRATION_TOKEN`, `TENANT_ID`, `PGADMIN_EMAIL`, `PGADMIN_PASSWORD`.

**Port web unifié : 9001.** Les trois conventions antérieures (3000 par défaut dans le binaire, 9001 + 3000 publiés par l'ancien gabarit, 9001 seul côté Linux) sont ramenées à une seule, alignée sur le port déjà annoncé dans `NODE_ADDR`.

### 18.4 Inventaire des tests

**75 tests Django** (52 avant ce chantier) et **9 tests Rust**.

| Classe | Tests | Créée ici |
| :--- | :---: | :---: |
| `RoleBoundaryTests` | 10 | |
| `NodeImageEndpointTests` | 7 | |
| `AuthEndpointTests` | 6 | |
| `NodeImagePmeDownloadTests` | 6 | |
| `FichiersGeneresPourLaPmeTests` | 6 | |
| `DetectionAdresseNoeudTests` | 6 | |
| `RegisterTests` | 4 | |
| `PublicNodeEndpointTests` | 3 | |
| `InstalleurWindowsTests` | 5 | étendue |
| **`ComposeParJetonTests`** | 6 | oui |
| **`PilePmeTests`** | 5 | oui |
| **`PariteInstalleursTests`** | 11 | oui |

Les 9 tests Rust vivent dans `spike/node/src/cmd/run.rs`, module `#[cfg(test)] mod tests` : période du battement de cœur (défaut, désactivation, valeurs invalides, bornes, garde-fou des 120 s), traduction de `NODE_MODE`, suivi du rôle courant.

Les trois règles du `.bat` sont désormais toutes verrouillées par un test :

| Règle | Test |
| :--- | :--- |
| ASCII pur | `test_installeur_est_en_ascii_pur` |
| Aucun label dans un bloc parenthésé | `test_aucun_label_dans_un_bloc_parenthese` |
| **Aucun tube échappé dans un `for /f`** | `test_aucun_tube_echappe_dans_un_for_f` |

### 18.5 Chapitres antérieurs devenus partiellement caducs

| Chapitre | Ce qui ne vaut plus |
| :--- | :--- |
| §12.3 « Renseigner l'adresse de la machine » | L'installeur détecte et écrit `NODE_ADDR` lui-même — et le faisait déjà mal à cause du bug `^|` (§17.6) |
| §14.1 « Pourquoi les deux nœuds s'annonçaient passif » | Corrigé à la source : le rôle vient de `cluster-status`, pas d'une valeur figée au téléchargement |
| §14.2 « Aucun battement de cœur » | Diagnostic réécrit au §17.1 — le heartbeat existait mais était enfermé dans la boucle PostgreSQL |
| §16.1 « Pas besoin de retélécharger » | Vrai pour le heartbeat seul ; faux pour cette livraison, qui change la pile (§17.10) |
| §16.3 B.2 piège Fedora des volumes *bind* | Ne concerne plus les nouvelles installations, seulement la purge des anciennes |

### 18.6 Rejouer les vérifications

```bash
# Tests Django — pas besoin de la stack, SQLite suffit
PYTHONPATH=<répertoire du module> env/Scripts/python.exe manage.py test --settings=settings_test_sqlite

# Tests Rust — aucun toolchain cargo sur cette machine, on passe par l'image
docker run --rm -v "C:/Users/Siradiou/saas-souverain/spike:/workspace" -w /workspace \
  -v amane_cargo_registry:/usr/local/cargo/registry -v amane_node_target:/workspace/target \
  rust:1.88-bookworm cargo test -p node

# Le conteneur sert-il bien les gabarits du dépôt ?
docker compose exec -T backend sha256sum /app/templates/tenants/install-windows.bat.tmpl

# Le binaire publié porte-t-il le correctif ?
docker run --rm --entrypoint sh ss-node:v2 -c "grep -ac SAAS_HEARTBEAT_SECS /usr/local/bin/ss-node"
```

Rappel : après toute modification de code Python, `docker compose up -d --build backend` — ce service ne monte pas le code source, et sans rebuild il continue de servir l'ancienne image.
