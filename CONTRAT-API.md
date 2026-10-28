# Contrat d'API REST — SaaS éditeur AMANE

Source de vérité de l'interface entre le **backend Django/DRF** et la **console React**. Le client HTTP (`frontend/src/api/client.ts`), les types TypeScript (`frontend/src/types/index.ts`) et les vues Python renvoient tous ici : toute divergence entre ce document et le code est un bug de l'un ou de l'autre.

Depuis la migration « 100 % React », Django ne sert plus que `/api/…` et `/admin/`. Les pages HTML historiques restent en place (voir `ACCES-INTERFACES.md` section 6) mais ne définissent plus le contrat.

Base : `http://localhost:8000/api` (surchargeable côté console par `VITE_API_URL`).

---

## 0. Conventions transverses

### 0.1 Authentification — session Django, jamais de JWT

L'API s'appuie sur la **session Django** (cookie `sessionid`) et le **cookie CSRF** (`csrftoken`). Le même compte ouvre `/admin/` et la console.

```python
# config/settings.py
REST_FRAMEWORK = {
    'DEFAULT_AUTHENTICATION_CLASSES': ['rest_framework.authentication.SessionAuthentication'],
    'DEFAULT_PERMISSION_CLASSES': ['rest_framework.permissions.IsAuthenticated'],
}
```

Le défaut est **fermé** : toute vue publique doit déclarer explicitement `AllowAny`, et toute vue de l'espace éditeur `IsStaffUser`. Se reposer sur le seul défaut `IsAuthenticated` est une faute — un compte PME, que n'importe qui crée via `POST /api/auth/register/`, est authentifié lui aussi.

Côté console, `withCredentials: true` est obligatoire sur chaque appel (déjà posé sur l'instance axios partagée).

### 0.2 CSRF

Toute écriture (`POST`, `PUT`, `PATCH`, `DELETE`) exige l'en-tête `X-CSRFToken` repris du cookie `csrftoken`. Le cookie est posé par `GET /api/auth/csrf/`, à appeler avant la première écriture d'une session. Le client le fait automatiquement : `ensureCsrf()` est branché en intercepteur de requête sur toute méthode non sûre.

### 0.3 CORS

Les cookies de session imposent une liste d'origines **explicite** — `CORS_ALLOW_ALL_ORIGINS` est incompatible avec `CORS_ALLOW_CREDENTIALS`. La liste vient de `FRONTEND_ORIGINS` dans le `.env` et alimente à la fois `CORS_ALLOWED_ORIGINS` et `CSRF_TRUSTED_ORIGINS`.

### 0.4 Les deux rôles

| Rôle | Dérivé de | Périmètre |
| :--- | :--- | :--- |
| `editor` | `user.is_staff` | Tous les tenants, licences, appareils, clusters, relais |
| `tenant` | `user.tenant` (relation inverse du `OneToOneField` `Tenant.user`) | Sa seule entreprise, via `/api/tenants/me/…` |

Les deux permissions vivent dans `api/permissions.py` : `IsStaffUser` et `IsTenantUser`. Un rôle n'accède **jamais** aux routes de l'autre — y compris un éditeur sur `/api/tenants/me/`, qui n'a pas de tenant rattaché. `api/tests.py` verrouille cette frontière route par route.

### 0.5 Codes de statut

| Code | Sens |
| :--- | :--- |
| `200` / `201` / `204` | Succès (`204` sur `logout`) |
| `400` | Validation — corps en erreurs champ par champ, ou `{"detail": "…"}` |
| `401` | **Uniquement** `GET /api/auth/me/` : aucune session |
| `403` | Session absente **ou** rôle insuffisant |
| `404` | Ressource inexistante |

Avec `SessionAuthentication` seule, DRF répond **403 et non 401** à un anonyme : cette classe ne fournit pas d'en-tête `WWW-Authenticate`. `GET /api/auth/me/` fait exception et construit son 401 elle-même, pour que la console distingue « pas encore connecté » (→ `/login`) de « connecté au mauvais rôle » (→ son espace).

### 0.6 Forme des erreurs

Deux formes seulement, toutes deux absorbées par `normalizeApiError()` :

```jsonc
{"detail": "Identifiants invalides."}                       // erreur globale
{"email": ["Une entreprise avec cet email existe déjà."]}   // erreurs par champ
```

Les erreurs par champ sont des **tableaux de chaînes** — c'est ce que l'UI attend pour les afficher sous le champ concerné. `non_field_errors` est utilisé comme message principal quand il est présent.

---

## 1. Authentification — `/api/auth/`

Implémentation : `api/auth.py`, `api/serializers.py`.

| Méthode | Route | Permission | Réponse |
| :--- | :--- | :--- | :--- |
| `GET` | `/api/auth/csrf/` | publique | `200 {"detail": "ok"}` + cookie `csrftoken` |
| `POST` | `/api/auth/login/` | publique | `200 <User>` · `400 {"detail"}` |
| `POST` | `/api/auth/logout/` | authentifié | `204` |
| `GET` | `/api/auth/me/` | publique | `200 <User>` · `401 {"detail"}` |
| `POST` | `/api/auth/register/` | publique | `201 <User>` · `400 <erreurs par champ>` |

### `<User>` — forme unique, partout

Produit par `serialize_user()`. C'est le seul objet utilisateur de l'API : `login`, `register` et `me` renvoient exactement la même forme.

```jsonc
{
  "id": 3,
  "username": "contact",
  "email": "contact@boulangerie-martin.fr",
  "is_staff": false,
  "role": "tenant",                    // "editor" si is_staff, sinon "tenant"
  "tenant": {                          // null pour un compte éditeur
    "id": "0f2c…",
    "name": "Boulangerie Martin",
    "registration_token": "8a41…"      // jeton d'installation du nœud PME
  }
}
```

`role` et `tenant` sont **dérivés**, pas des champs du modèle `User` — d'où une simple fonction plutôt qu'un `Serializer` DRF. Le sous-objet `tenant` est volontairement réduit ; il ne suit pas `TenantSerializer`.

### `POST /api/auth/login/`

```jsonc
{"username": "contact", "password": "…"}
```

Identifiants invalides → `400 {"detail": "Identifiants invalides."}` (jamais 401 : côté console c'est une erreur de formulaire, pas une redirection).

### `POST /api/auth/register/`

Inscription publique d'une PME. Passe par `InscriptionForm` puis `_create_tenant_account()` — **exactement** le chemin de la vue web `tenants.views_web.inscription`, donc mêmes validations et mêmes effets :

1. création du `User` Django (username dérivé de l'email, désambiguïsé par suffixe numérique) ;
2. création du `Tenant` lié ;
3. création d'une **licence d'essai** : plan `starter`, `seats = employee_count`, 30 jours ;
4. ouverture immédiate de la session — la console enchaîne sur `/mon-espace` sans repasser par `/login`.

```jsonc
{
  "name": "Garage Dupont",
  "email": "contact@garage-dupont.fr",
  "phone": "0102030405",              // optionnel
  "address": "1 rue des Ateliers",    // optionnel
  "employee_count": 4,
  "password": "…",
  "password_confirm": "…"
}
```

Email déjà pris ou mots de passe différents → `400` en erreurs champ par champ.

---

## 2. Espace éditeur — `IsStaffUser`

Toutes les routes de cette section refusent l'anonyme **et** le compte PME (403).

### 2.1 Tableau de bord — `GET /api/dashboard/stats/`

`dashboard/api_stats.py`. Réutilise `build_dashboard_context()`, la logique de la page Django.

```jsonc
{
  "stats": { /* compteurs — voir DashboardCounters */ },
  "recent_tenants": [ /* <Tenant>[] */ ]
}
```

### 2.2 Clusters — `GET /api/clusters/`

`devices/api_clusters.py`, via `build_cluster_overview()`. Seuil « en ligne » : **2 minutes** depuis `last_seen`.

```jsonc
{
  "alerts_total": 1,
  "total_tenants": 4,
  "clusters": [
    {
      "tenant": {"id": "…", "name": "…", "email": "…"},
      "primary": {"device": { /* <Device> */ }, "online": true},   // null si aucun primaire
      "standbys": [{"device": { /* <Device> */ }, "online": false}],
      "total": 2,
      "online_count": 1,
      "license": { /* état de conformité */ },
      "health": "…",
      "is_alert": true
    }
  ]
}
```

### 2.3 Relais — `GET /api/relay/status/`

`dashboard/api_relay.py`, via `build_relay_status()`.

**Un relais injoignable est un état normal**, pas une panne de l'API : la réponse reste `200` avec `health.reachable: false`. Ne jamais transformer cela en 500 — la console affiche l'indisponibilité, elle ne l'invente pas.

```jsonc
{
  "relay_url": "http://host.docker.internal:8080",
  "health": {"reachable": false, /* … */},
  "total_nodes": 0,
  "tenants": [{"tenant": {"id": "…", "name": "…"}, "nodes": []}],
  "editeur_subnet": "…", "relay_subnet": "…", "pme_subnet": "…"
}
```

### 2.4 Image du nœud PME — `/api/node-image/`

`dashboard/api_images.py`. L'image Docker du nœud PME est publiée par un build hors-ligne (`spike/build-node.ps1`) qui dépose `media/node-images/manifeste.json` et l'archive `.tar.gz` associée ; un registre Docker local (service `registry` de `docker-compose.yml`, port `5000`) la distribue ensuite aux machines PME du réseau.

| Méthode | Route | Réponse |
| :--- | :--- | :--- |
| `GET` | `/api/node-image/` | `<NodeImage>` |
| `GET` | `/api/node-image/download/` | fichier `.tar.gz` en streaming · `404 {"detail": "…"}` si l'archive manque |

**`disponible: false` et `registre.joignable: false` sont des états normaux, renvoyés en `200`** — jamais une 500. Même logique que le relais dans `dashboard/api_relay.py` : manifeste absent (build pas encore publié) ou registre arrêté sont des situations attendues que la console affiche calmement.

```jsonc
{
  "disponible": true,
  "version": "v2",
  "image": "ss-node:v2",
  "fichier": "ss-node-v2.tar.gz",
  "taille_octets": 123456789,
  "sha256": "…",
  "construit_le": "2026-08-31T18:30:00Z",
  "url_telechargement": "/api/node-image/download/",
  "registre": {
    "hote": "192.168.7.7:5000",          // dérivé de la requête (tenants.views_web._saas_host_port), jamais codé en dur
    "commande_pull": "docker pull 192.168.7.7:5000/ss-node:v2",
    "tags": ["v2"],                      // interrogés sur l'API du registre (GET /v2/<image>/tags/list)
    "joignable": true
  }
}
```

Quand `disponible` est `false` (manifeste ou archive absents), `version`, `image`, `fichier`, `taille_octets`, `sha256` et `construit_le` valent `null` ; `registre.hote` et `registre.commande_pull` restent calculés (ou `null` pour `commande_pull` si aucune image n'est connue).

`GET /api/node-image/download/` sert l'archive en `FileResponse` streamé (elle fait des centaines de Mo, jamais chargée en mémoire), avec `Content-Disposition: attachment; filename="ss-node-v2.tar.gz"`.

### 2.5 Tenants — `/api/tenants/`

`TenantViewSet` (`tenants/views.py`), routeur DRF. CRUD complet : `GET` liste, `POST` création, `GET`/`PATCH`/`PUT`/`DELETE` sur `/api/tenants/<uuid>/`.

`id`, `created_at` et `registration_token` sont en **lecture seule** : le jeton d'installation est généré à la création du tenant et n'est jamais modifiable par l'API.

> `/api/tenants/me/…` (section 3) est déclaré **avant** le routeur dans `tenants/urls_api.py`. Sans cela, `me` serait capturé comme `<pk>` par la route de détail du ViewSet.

### 2.6 Licences — `/api/licenses/`

`LicenseViewSet` (`licenses/views.py`). CRUD complet. `tenant_name` est aplati en lecture seule (la console affiche le nom, pas l'UUID).

Invariant validé côté serializer : **`seats` ne peut pas dépasser `employee_count` du tenant** → `400 {"seats": ["…"]}`.

### 2.7 Appareils — `/api/devices/`

`DeviceViewSet` (`devices/views.py`). CRUD complet, plus une action :

| Méthode | Route | Effet |
| :--- | :--- | :--- |
| `POST` | `/api/devices/<uuid>/toggle/` | Bascule `is_active` → `{"id": "…", "is_active": false}` |

`is_within_license` et `tenant_name` sont calculés en lecture seule.

---

## 3. Espace PME — `/api/tenants/me/` — `IsTenantUser`

`tenants/api_espace.py`. Le tenant est toujours déduit de `request.user.tenant` : **aucune route ne prend d'identifiant de tenant en paramètre**, une PME ne peut donc pas désigner l'entreprise d'une autre. La logique métier est celle de `tenants.views_espace` / `views_web` — appelée, pas réécrite.

| Méthode | Route | Réponse |
| :--- | :--- | :--- |
| `GET` | `/api/tenants/me/` | `<EspacePme>` |
| `GET` | `/api/tenants/me/onboarding/` | `<Onboarding>` |
| `GET` | `/api/tenants/me/download/compose/` | fichier `docker-compose.yml` personnalisé |
| `GET` | `/api/tenants/me/download/installer/<os_type>/` | script d'installation (`linux` \| `windows`) |

`os_type` hors de `{linux, windows}` → `400 {"detail": "os doit être 'linux' ou 'windows'."}`.

### `<EspacePme>`

```jsonc
{
  "tenant": { /* <Tenant> */ },
  "active_license": { /* <License> */ },   // null si aucune licence active
  "devices": [ /* <Device>[] */ ],
  "active_devices": 2,
  "pgadmin": {"url": "…", "email": "…", "password": "…"},
  "relay_url": "http://host.docker.internal:8080"
}
```

### `<Onboarding>`

```jsonc
{
  "tenant": { /* <Tenant> */ },
  "active_license": { /* <License> */ },
  "active_devices": 0,
  "relay_url": "…",
  "downloads": {
    "compose": "/api/tenants/me/download/compose/",
    "installer_linux": "/api/tenants/me/download/installer/linux/",
    "installer_windows": "/api/tenants/me/download/installer/windows/"
  }
}
```

Les `downloads` sont des chemins **relatifs à la racine du backend**, pas au préfixe `/api` de la console : `client.ts` les préfixe de l'origine du backend via `getApiOrigin()`.

---

## 4. Objets partagés

Reflètent `tenants/serializers.py`, `licenses/serializers.py`, `devices/serializers.py` — et, côté console, `frontend/src/types/index.ts`.

### `<Tenant>`

```jsonc
{
  "id": "uuid", "name": "…", "email": "…", "phone": "…", "address": "…",
  "employee_count": 5, "created_at": "2026-08-31T14:02:11Z", "is_active": true,
  "registration_token": "uuid"
}
```
Lecture seule : `id`, `created_at`, `registration_token`.

### `<License>`

```jsonc
{
  "id": "uuid", "tenant": "uuid", "tenant_name": "Boulangerie Martin",
  "plan": "starter",                  // starter | pro | enterprise
  "seats": 5,
  "starts_at": "2026-08-31", "expires_at": "2026-09-30",   // expires_at nullable
  "is_active": true, "created_at": "…"
}
```
Lecture seule : `id`, `tenant_name`, `created_at`.

### `<Device>`

```jsonc
{
  "id": "uuid", "tenant": "uuid", "tenant_name": "…",
  "installation_id": "uuid",          // identité d'installation authentifiée
  "mac_address": "", "hostname": "poste-caisse", "os": "Windows 11",
  "node_role": "primary",             // primary | standby | ""
  "node_addr": "192.168.200.130:9001",
  "web_addr": "192.168.200.130:3000",
  "registered_at": "…", "last_seen": "…",
  "is_active": true, "is_within_license": true
}
```
Lecture seule : `id`, `registered_at`, `last_seen`, `is_within_license`, `tenant_name`.

> La licence est liée à `installation_id`, **jamais** à `mac_address` — falsifiable et masquée par Docker. `mac_address` n'est conservé qu'à titre indicatif.

---

## 5. Endpoints publics des nœuds PME

Appelés par le binaire `ss-node`, pas par la console : **pas de session**, donc `AllowAny`. L'authentification est le `registration_token` du tenant. Ne pas les fermer par inadvertance en généralisant `IsStaffUser` sur `/api/devices/`.

### `POST /api/devices/register/`

`devices/api_register.py`. Appelé au premier lancement **et à chaque démarrage** du nœud.

```jsonc
{
  "tenant_token": "uuid",             // obligatoire
  "installation_id": "uuid",          // obligatoire — généré une fois par installation
  "hostname": "…", "os": "…", "mac_address": "…",
  "node_addr": "…", "web_addr": "…", "node_role": "primary"
}
```

| Statut | Corps | Cas |
| :--- | :--- | :--- |
| `201` | `{"status": "authorized", …}` | Nouvelle machine enregistrée |
| `200` | `{"status": "authorized", …}` | Machine déjà connue, reconnue |
| `400` | `{"status": "error", "message"}` | Champ manquant ou UUID mal formé |
| `403` | `{"status": "refused", "reason": "invalid_token"}` | Jeton inconnu ou tenant inactif |
| `403` | `{"status": "refused", "reason": "no_license"}` | Aucune licence active |
| `403` | `{"status": "refused", "reason": "device_disabled"}` | Machine désactivée par l'éditeur |
| `403` | `{"status": "refused", "reason": "licence_exceeded", "seats_used", "seats_total"}` | Quota de sièges dépassé |

Une réponse autorisée porte `device_id`, `tenant_name`, `seats_used`, `seats_total`.

Deux effets de bord importants côté serveur :

- **Failover détecté** : un nœud qui était `standby` et s'annonce `primary` incrémente `failover_count` et met à jour `last_failover_at`.
- **Un seul primaire par tenant** : quand un nœud s'annonce `primary`, les autres nœuds actifs du tenant sont rétrogradés en `standby`. C'est le garde-fou contre le split-brain après un redémarrage du SaaS.

### `GET /api/devices/cluster-status/?token=<registration_token>`

`devices/api_cluster_status.py`. Permet à l'installateur PME de savoir si le nœud qu'il déploie sera primaire ou standby.

```jsonc
{
  "tenant_name": "Boulangerie Martin",
  "node_count": 1,
  "active_nodes": [
    {"hostname": "kali", "node_addr": "192.168.200.130:9001", "node_role": "primary", "os": "linux"}
  ]
}
```

Token manquant ou mal formé → `400 {"error": "…"}` ; inconnu ou tenant inactif → `403 {"error": "…"}`.

---

## 6. Règles à tenir en modifiant l'API

1. **Le défaut est fermé.** Toute nouvelle vue déclare sa permission explicitement. Une vue de l'espace éditeur sans `IsStaffUser` est ouverte à tout compte PME — donc à tout le monde, l'inscription étant publique.
2. **Aucune route PME ne prend d'identifiant de tenant en entrée.** Le tenant se déduit de la session, jamais du client.
3. **Pas de données simulées côté console.** Un échec réseau ou serveur remonte en exception jusqu'à l'UI ; jamais de repli silencieux sur des valeurs inventées.
4. **Une seule forme de `<User>`.** Modifier `serialize_user()` change `login`, `register` et `me` d'un bloc — c'est voulu.
5. **Les endpoints de nœuds restent publics.** Ils sont authentifiés par jeton, pas par session.
6. **Toute modification de ce contrat se répercute** dans `frontend/src/types/index.ts`, `frontend/src/api/client.ts` et `api/tests.py`, sinon le contrat n'est plus audité.

Vérification : `python manage.py test api` (30 tests, frontière des rôles route par route).
