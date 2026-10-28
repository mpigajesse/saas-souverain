# Accès aux interfaces — AMANE SaaS Souverain

Toutes les interfaces du projet, côté **éditeur** comme côté **PME**, avec leur URL exacte, qui a le droit d'y entrer et d'où elles viennent dans le code.

Les URL sont données pour un poste de développement (`localhost`). Sur le banc de test, remplacer par l'IP de la machine (ex. `192.168.200.128`).

> **Depuis la migration « 100 % React »** : la console React est le **seul frontend** de l'éditeur. Django ne sert plus que l'**API REST** (`/api/…`) et son **administration** (`/admin/`). Les anciennes pages HTML Django restent en place et fonctionnelles, mais ne sont plus la référence — elles sont listées en section 6 pour mémoire.

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

Un `.env` est **obligatoire** à la racine (voir `.env.example`), en UTF-8 **sans BOM**. Il porte désormais aussi `FRONTEND_ORIGINS` : les origines autorisées à appeler l'API avec les cookies de session.

### Machine PME

```bash
cd pme-deploy
./setup-primary.sh                         # VM primaire
./setup-standby.sh <IP_PRIMAIRE>           # VM standby
```

---

## 2. Vue d'ensemble des ports

| Port | Service | Qui l'utilise |
| :--- | :--- | :--- |
| **5173** | **Console React — le frontend de l'éditeur** | Éditeur et PME |
| **8000** | Django — API REST + `/admin/` | Console React, nœuds PME |
| **9001** | Interface métier du nœud PME (`ss-node`) | Employés et admin de la PME |
| **8080** | Relais zero-knowledge (`ss-relay`) — API seule | Nœuds PME, console éditeur |
| **5050** | pgAdmin (stack `pme-deploy`) | Admin PME |
| **3000** | Grafana | Éditeur |
| **9090** | Prometheus | Éditeur |
| **8008 / 8009** | API REST Patroni (primaire / réplica) | Exploitation |
| **5432 / 5434 / 5433** | PostgreSQL — primaire (5434 = port de secours) / réplica | Outils DBA |
| **2379** | etcd | Orchestrateur Go |

---

## 3. Console React — `http://localhost:5173`

Point d'entrée unique. L'authentification est celle de Django (session + cookie CSRF) : les comptes sont les mêmes que ceux de `/admin/`.

Après connexion, la destination dépend du rôle : un compte **éditeur** (staff) arrive sur `/`, un compte **PME** (rattaché à un tenant) sur `/mon-espace`. Une route de l'autre rôle renvoie automatiquement vers sa propre page d'accueil.

### 3.1 Pages publiques

| Page | URL |
| :--- | :--- |
| Connexion | `/login` |
| Inscription d'une PME | `/register` |

### 3.2 Espace éditeur (rôle `editor`)

| Page | URL | Remplace |
| :--- | :--- | :--- |
| **Console** (tableau de bord) | `/` | `templates/dashboard/index.html` |
| Tenants | `/tenants` | `templates/tenants/list.html` |
| Créer un tenant | `/tenants/nouveau` | `templates/tenants/form.html` |
| Fiche d'un tenant | `/tenants/:id` | `templates/tenants/detail.html` |
| Modifier un tenant | `/tenants/:id/modifier` | `templates/tenants/form.html` |
| Licences | `/licenses` | `templates/licenses/list.html` + `form.html` |
| Appareils | `/devices` | `templates/devices/list.html` |
| Clusters | `/clusters` | `templates/devices/clusters.html` |
| Relais | `/relay` | `templates/dashboard/relay.html` |
| Architecture | `/architecture` | `templates/dashboard/architecture.html` |

### 3.3 Espace PME (rôle `tenant`)

| Page | URL | Remplace |
| :--- | :--- | :--- |
| **Mon espace** | `/mon-espace` | `templates/tenants/mon-espace.html` |
| Installation / onboarding | `/bienvenue` | `templates/tenants/bienvenue.html` |

Les trois téléchargements (docker-compose, installeur Linux, installeur Windows) se font depuis `/bienvenue` par de vrais liens de navigation, pour que le cookie de session accompagne la requête.

Les liens directs et le rafraîchissement (F5) fonctionnent sur toutes ces routes, y compris dans le conteneur : `frontend/nginx.conf` renvoie `index.html` en dernier recours.

---

## 4. API Django — `http://localhost:8000/api`

Authentification par **session + CSRF**. Toute écriture exige l'en-tête `X-CSRFToken` ; le cookie s'obtient via `GET /api/auth/csrf/`.

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
| POST | `/api/devices/<uuid>/toggle/` |
| CRUD | `/api/tenants/` · `/api/licenses/` · `/api/devices/` |

### 4.3 Espace PME (compte rattaché à un tenant — `403` sinon)

| Méthode | URL |
| :--- | :--- |
| GET | `/api/tenants/me/` |
| GET | `/api/tenants/me/onboarding/` |
| GET | `/api/tenants/me/download/compose/` |
| GET | `/api/tenants/me/download/installer/linux\|windows/` |

### 4.4 Endpoints publics des nœuds PME

**Ils doivent rester accessibles sans authentification** — un nœud qui démarre n'a pas de session :

| Méthode | URL | Rôle |
| :--- | :--- | :--- |
| POST | `/api/devices/register/` | Annonce d'un nœud (`tenant_token` + `installation_id`) |
| GET | `/api/devices/cluster-status/?token=<REGISTRATION_TOKEN>` | Statut du cluster ; `400` sans le paramètre `token` |

### 4.5 Administration Django

`http://localhost:8000/admin/` — superuser. Seule interface HTML servie par Django qui reste au programme.

---

## 5. Interface métier PME — `ss-node` (`http://<IP_PME>:9001`)

Servie par le binaire Rust `ss-node` (`spike/node/src/web.rs`). Port réglé par `WEB_PORT` (`9001` dans `pme-deploy`, `3000` par défaut hors conteneur). Sur le banc `spike/docker-compose.yml` : node1 = `:9001`, node2 = `:9002`, node3 = `:9003`.

Seul le **nœud primaire** sert l'application ; un standby affiche une page de redirection vers le primaire.

### 5.1 Accès employé

| Page | URL |
| :--- | :--- |
| Connexion | `/login` |
| Déconnexion | `/logout` (POST) |
| Tableau de bord stock | `/` |
| Liste des articles | `/articles` |
| Créer un article | `/articles/nouveau` |
| Supprimer un article | `/articles/<id>/supprimer` (POST) |
| Mouvements de stock | `/mouvements` |

### 5.2 Accès administrateur PME

| Page | URL |
| :--- | :--- |
| Administration / statut du cluster | `/admin` |
| Liste des utilisateurs | `/admin/utilisateurs` |
| Créer un utilisateur | `/admin/utilisateurs/nouveau` |
| Activer / désactiver un compte | `/admin/utilisateurs/<id>/toggle` (POST) |
| Changer un mot de passe | `/admin/utilisateurs/<id>/mdp` |

### 5.3 Comptes PME

Le mot de passe `admin` est affiché **une seule fois**, au premier démarrage du nœud :

```bash
docker compose logs ss-node | grep -A5 "PREMIER DÉMARRAGE"
```

Créer un compte en ligne de commande : `ss-node adduser`. Les comptes du banc de test sont dans `pme-deploy/COMPTES-ET-TESTS.md`.

### 5.4 pgAdmin — données métier en clair

`http://<IP_PME>:5050` — stack `pme-deploy`, identifiants dans le `.env` de la PME. Seul endroit où les données métier apparaissent en clair, et il est **à l'intérieur du périmètre souverain**.

---

## 6. Pages HTML Django (héritage, plus la référence)

Toujours servies, mais remplacées par la console React. Aucune n'a été supprimée.

`/` · `/relay/` · `/architecture/` · `/schema/` · `/tenants/` (+ `nouveau`, `<uuid>`, `modifier`, `supprimer`, `inscription`, `bienvenue`, `mon-espace`, `telecharger/…`) · `/licenses/` (+ `nouvelle`, `modifier`, `supprimer`) · `/devices/` (+ `clusters/`, `<uuid>/toggle/`) · `/login/` · `/logout/` (POST uniquement).

Les gabarits `templates/tenants/*.tmpl` (docker-compose et installeurs) ne sont **pas** de l'héritage : ils génèrent les fichiers téléchargés et restent indispensables.

---

## 7. Relais zero-knowledge (`http://localhost:8080`)

Aucune page web — API JSON seulement. Le relais ne voit jamais de clair.

| Endpoint | Méthode |
| :--- | :--- |
| `/health` | GET |
| `/api/nodes/announce` | POST |
| `/api/nodes` | GET |
| `/api/blobs/<tenant_id>/<key>` | GET · PUT · DELETE |

À consulter confortablement depuis la page **`/relay`** de la console React.

---

## 8. Exploitation et supervision

| Interface | URL | Identifiants |
| :--- | :--- | :--- |
| Grafana | http://localhost:3000 | `admin` / `admin` (défaut du compose) |
| Prometheus | http://localhost:9090 | aucun |
| Patroni — primaire | http://localhost:8008/patroni | aucun |
| Patroni — réplica | http://localhost:8009/patroni | aucun |

---

## 9. Si une interface ne répond pas

- **`SECRET_KEY not found`** au lancement de Django : `.env` absent, ou écrit **avec un BOM** — `decouple` ne voit alors pas la première clé.
- **`password authentication failed` sur `localhost:5432`** : un PostgreSQL natif Windows occupe le port et répond à la place du conteneur. Utiliser `DB_PORT=5434`.
- **Console React : erreur CORS ou déconnexion immédiate** : l'origine du navigateur n'est pas dans `FRONTEND_ORIGINS` du `.env`. `CORS_ALLOW_ALL_ORIGINS` ne peut pas être utilisé — il est incompatible avec les cookies de session.
- **Écriture rejetée en 403 depuis React** : cookie CSRF absent. `GET /api/auth/csrf/` doit précéder la première écriture.
- **Page PME vide ou redirection permanente** : le nœud est en standby ; l'application n'est servie que par le primaire.
