# Accès aux interfaces — AMANE V3

Toutes les interfaces du projet, côté **éditeur** comme côté **PME**, avec leur URL exacte, qui a le droit d'y entrer et d'où elles viennent dans le code.

Les URL sont données pour un poste de développement (`localhost`). Sur le banc de test, remplacer par l'IP de la machine.

> La console React est le **seul frontend** de l'éditeur. Django ne sert que l'**API REST** (`/api/…`) et son **administration** (`/admin/`) : la console HTML Django a été retirée en V3.

---

## 1. Démarrage

### Poste de développement

```bash
docker compose up -d                       # etcd + Patroni + backend + frontend + monitoring
```

ou, hors conteneur :

```powershell
.\env\Scripts\python.exe manage.py runserver 0.0.0.0:8000
cd frontend ; npm run dev                  # console React sur :5173
```

Un `.env` est **obligatoire** à la racine (voir `.env.example`), en UTF-8 **sans BOM**. Il porte aussi `FRONTEND_ORIGINS` : les origines autorisées à appeler l'API avec les cookies de session.

### Machine PME

Rien à copier à la main : la PME télécharge depuis son espace (`/bienvenue`) l'installeur de chaque machine, **serveur** ou **témoin**, pour Linux (`.sh`) ou Windows (`.bat`). L'installeur vérifie Docker, détecte l'adresse de la machine et le chiffrement du disque, écrit le `.env` et lance la pile générée par le portail (`templates/tenants/docker-compose.yml.tmpl`, `docker-compose-temoin.yml.tmpl`).

Les serveurs après le premier démarrent **sans DEK** : un administrateur les enrôle (section 5.3).

---

## 2. Vue d'ensemble des ports

### Portail éditeur

| Port | Service | Qui l'utilise |
| :--- | :--- | :--- |
| **5173** | **Console React — le frontend de l'éditeur** | Éditeur et PME |
| **8000** | Django — API REST + `/admin/` | Console React, nœuds PME |
| **5000** | Registre d'images de l'éditeur | Machines PME (téléchargement de la pile et des applications) |
| **8080** | Relais zero-knowledge (`ss-relay`) — API seule | Nœuds PME, console éditeur |
| **3000** | Grafana | Éditeur |
| **9090** | Prometheus | Éditeur |

### Machine PME (pile générée)

| Port | Service | Qui l'utilise |
| :--- | :--- | :--- |
| **9001** | `ss-node` : application hébergée à la racine, console AMANE sous `/amane/` (`WEB_PORT`) | Utilisateurs et administrateurs de la PME |
| **5050** | pgAdmin | Administrateur PME |
| **5432** | PostgreSQL (Patroni) | Réplication entre serveurs, outils DBA |
| **8008** | API REST Patroni | Orchestrateur, nœud, exploitation |
| **2379 / 2380** | etcd (client / pairs) — aussi sur le témoin | Orchestrateurs, Patroni |

HAProxy (`amane-haproxy:5000`) n'est pas publié : il n'est joint que depuis la pile.

---

## 3. Console React — `http://localhost:5173`

Point d'entrée unique. L'authentification est celle de Django (session + cookie CSRF) : les comptes sont les mêmes que ceux de `/admin/`.

Après connexion, la destination dépend du rôle : un compte **éditeur** (staff) arrive sur `/`, un compte **PME** (rattaché à un tenant) sur `/mon-espace`. Une route de l'autre rôle renvoie vers sa propre page d'accueil.

### 3.1 Pages publiques

| Page | URL |
| :--- | :--- |
| Connexion | `/login` |
| Inscription d'une PME | `/register` |

### 3.2 Espace éditeur (rôle `editor`)

| Page | URL |
| :--- | :--- |
| **Console** (tableau de bord) | `/` |
| Tenants | `/tenants` |
| Créer un tenant | `/tenants/nouveau` |
| Fiche d'un tenant | `/tenants/:id` |
| Modifier un tenant | `/tenants/:id/modifier` |
| Licences (dont l'application hébergée) | `/licenses` |
| **Applications hébergées** (catalogue, validation, retrait) | `/applications` |
| Appareils | `/devices` |
| Clusters | `/clusters` |
| Relais | `/relay` |
| Déploiement (image du nœud) | `/deploiement` |
| Architecture | `/architecture` |

### 3.3 Espace PME (rôle `tenant`)

| Page | URL |
| :--- | :--- |
| **Mon espace** (machines, alertes, application hébergée) | `/mon-espace` |
| Installation | `/bienvenue` |

Les téléchargements (pile, installeurs serveur et témoin) se font depuis `/bienvenue` par de vrais liens de navigation, pour que le cookie de session accompagne la requête.

Les liens directs et le rafraîchissement (F5) fonctionnent sur toutes ces routes, y compris dans le conteneur : `frontend/nginx.conf` renvoie `index.html` en dernier recours.

---

## 4. API Django — `http://localhost:8000/api`

Authentification par **session + CSRF**. Toute écriture exige l'en-tête `X-CSRFToken` ; le cookie s'obtient via `GET /api/auth/csrf/`. Le détail des formes est dans [`CONTRAT-API.md`](CONTRAT-API.md).

### 4.1 Authentification

| Méthode | URL | Accès |
| :--- | :--- | :--- |
| GET | `/api/auth/csrf/` | public — pose le cookie `csrftoken` |
| POST | `/api/auth/login/` | public — `{username, password}` |
| POST | `/api/auth/logout/` | connecté |
| GET | `/api/auth/me/` | public — `401` si anonyme |
| POST | `/api/auth/register/` | public — crée compte + tenant + licence d'essai |

### 4.2 Espace éditeur (staff — `403` sinon)

| Méthode | URL |
| :--- | :--- |
| GET | `/api/dashboard/stats/` |
| GET | `/api/clusters/` |
| GET | `/api/relay/status/` |
| GET | `/api/node-image/` · `/api/node-image/download/` |
| POST | `/api/devices/<uuid>/toggle/` |
| POST | `/api/applications/<uuid>/valider/` · `/api/applications/<uuid>/retirer/` |
| CRUD | `/api/tenants/` · `/api/licenses/` · `/api/devices/` · `/api/applications/` |

### 4.3 Espace PME (compte rattaché à un tenant — `403` sinon)

| Méthode | URL |
| :--- | :--- |
| GET | `/api/tenants/me/` |
| GET | `/api/tenants/me/onboarding/` |
| GET | `/api/tenants/me/download/compose/?type=serveur\|temoin` |
| GET | `/api/tenants/me/download/installer/linux\|windows/?type=serveur\|temoin` |

### 4.4 Endpoints publics des machines PME

**Ils doivent rester accessibles sans session** — une machine qui s'installe ou démarre n'en a pas. L'authentification est le jeton d'enregistrement du tenant :

| Méthode | URL | Rôle |
| :--- | :--- | :--- |
| POST | `/api/devices/register/` | Annonce d'un nœud ou d'un témoin (`tenant_token` + `installation_id`, états V3) |
| GET | `/api/devices/cluster-status/?token=<REGISTRATION_TOKEN>` | Pairs du cluster ; `400` sans `token` |
| GET | `/api/tenants/pme/compose/?token=…&type=serveur\|temoin` | Pile de la machine, appelée par l'installeur |
| GET | `/api/node-image/telecharger/?token=…` | Image du nœud pour une installation hors ligne |

### 4.5 Administration Django

`http://localhost:8000/admin/` — superuser. Seule interface HTML servie par Django.

---

## 5. Machine PME — `ss-node` (`http://<IP_PME>:9001`)

Servi par le binaire Rust `ss-node` (`spike/node/src/web.rs`), sur **chaque serveur** : l'application hébergée joint toujours le primaire par HAProxy, quel que soit le serveur par lequel on entre.

### 5.1 Application hébergée — `/`

La racine relaie l'application du SaaS externe (`amane-application`), avec ses propres comptes et son propre login. Sans application, la racine renvoie vers `/amane`.

### 5.2 Console AMANE — `/amane/`

| Page | URL | Accès |
| :--- | :--- | :--- |
| Connexion | `/amane/login` | public |
| Déconnexion | `/amane/logout` (POST) | connecté |
| Accueil (état de l'application hébergée) | `/amane/` | connecté |
| Administration, état du cluster (Patroni) | `/amane/admin` | administrateur |
| **Enrôlement** d'un serveur | `/amane/admin/enrolement` | administrateur |
| Utilisateurs | `/amane/admin/utilisateurs` (+ `nouveau`, `<id>/toggle`, `<id>/mdp`) | administrateur |
| API d'enrôlement | `/amane/api/enrolement` (POST) | authentifiée par le seul code à usage unique |

Le mot de passe `admin` de la console est affiché **une seule fois**, au premier démarrage du nœud :

```bash
docker logs ss-node | grep -A5 "PREMIER DÉMARRAGE"
```

### 5.3 Commandes d'administration du nœud

| Commande | Effet |
| :--- | :--- |
| `docker exec ss-node ss-node enroler --depuis http://<serveur enrôlé>:9001 --code <code>` | Reçoit la DEK, scellée pour la clé de cette machine (code généré dans `/amane/admin/enrolement`), puis `docker restart ss-node` |
| `docker exec ss-node ss-node sauvegarder` | Sauvegarde immédiate des bases, chiffrée, sur le relais |
| `docker exec ss-node ss-node restaurer --base amane\|application --jour 1..7` | Restauration depuis le relais (1 = lundi) |
| `docker exec ss-node ss-node adduser --username … --name … --role admin\|employee --password …` | Compte de la console |

### 5.4 pgAdmin — `http://<IP_PME>:5050`

Identifiant affiché dans l'espace PME ; le mot de passe est généré sur le réseau de la PME et affiché en fin d'installation — le portail ne le connaît pas. Les données y apparaissent en clair, **à l'intérieur du périmètre souverain**.

---

## 6. Relais zero-knowledge (`http://localhost:8080`)

Aucune page web — API seulement. Le relais ne voit jamais de clair.

| Endpoint | Méthode |
| :--- | :--- |
| `/health` | GET |
| `/api/nodes/announce` | POST |
| `/api/nodes` | GET |
| `/api/blobs/<tenant_id>/<clé>` | GET · PUT · DELETE (jeton `x-relay-token` si `RELAY_AUTH_TOKEN` est défini) |

À consulter depuis la page **`/relay`** de la console React.

---

## 7. Exploitation et supervision

| Interface | URL | Identifiants |
| :--- | :--- | :--- |
| Grafana | http://localhost:3000 | `admin` / `admin` (défaut du compose de développement) |
| Prometheus | http://localhost:9090 | aucun |
| Patroni (développement) | http://localhost:8008/patroni · http://localhost:8009/patroni | aucun |
| Patroni (machine PME) | `http://<IP_PME>:8008/cluster` | aucun |

---

## 8. Si une interface ne répond pas

- **`SECRET_KEY not found`** au lancement de Django : `.env` absent, ou écrit **avec un BOM** — `decouple` ne voit alors pas la première clé.
- **`password authentication failed` sur `localhost:5432`** : un PostgreSQL natif Windows occupe le port et répond à la place du conteneur. Utiliser `DB_PORT=5434`.
- **Console React : erreur CORS ou déconnexion immédiate** : l'origine du navigateur n'est pas dans `FRONTEND_ORIGINS`. `CORS_ALLOW_ALL_ORIGINS` est incompatible avec les cookies de session.
- **Écriture rejetée en 403 depuis React** : cookie CSRF absent. `GET /api/auth/csrf/` doit précéder la première écriture.
- **Pile non téléchargeable (503)** : les images de la pile ne sont pas publiées — `bash scripts/publier-pile.sh`.
- **Validation d'une application refusée (409)** : son image n'est pas dans le registre, sous ce digest.
- **Machines Windows injoignables entre elles** : pare-feu — `scripts/ouvrir-pare-feu-lan.ps1`.
- **« Sauvegardes : DEK absente »** dans les journaux du nœud : le serveur n'est pas enrôlé (section 5.3).
