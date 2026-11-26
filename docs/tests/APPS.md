# A-06 — Les trois SaaS configurés, chacun avec un vrai processus métier

**Date** : 2026-11-25
**Tâche** : A-06
**Sources** : `docs/compatibilite-saas.md`, `ha-lab/listmonk/manifeste.json`,
`ha-lab/app-multi/manifeste.json`.

## Méthode et environnement dédié

La pile PME en service (`amane-*`) est **laissée intacte** : elle était déjà
dégradée avant la campagne (cf. `docs/STATE.md` : `amane-etcd` unhealthy,
`amane-application` en redémarrage, PostgreSQL Patroni sans leader car etcd
injoignable). Pour ne pas la toucher et pour obtenir des mesures `docker stats`
propres, chaque SaaS a été déployé dans un **environnement isolé dédié** :

| Conteneur | Rôle | Image |
|---|---|---|
| `a06-postgres` | PostgreSQL 17 dédié (bases `listmonk`, `odoo`, `appmulti`) | `postgres:17-alpine` |
| `a06-mailpit` | Sidecar SMTP local (capture e-mails Listmonk) | `axllent/mailpit:latest` |
| `a06-listmonk` | Listmonk v6.2.0 | `100.64.0.6:5000/editeurs/listmonk@sha256:3089042a…` |
| `a06-odoo` | Odoo 17 | `odoo:17@sha256:8779f8157…` |
| `a06-redis` | Redis (queue app-multi) | `redis:7-alpine` |
| `a06-appweb` | app-multi mode web | `amane/app-multi:a06` (build local) |
| `a06-appworker` | app-multi mode worker | `amane/app-multi:a06` (build local) |

Réseau `a06-net` (bridge dédié). Ports publiés sur `127.0.0.1` uniquement :
Listmonk `9000`, Odoo `8069`, app-multi web `8081`, mailpit UI `58025`,
PostgreSQL `55432`. Aucune image d'application modifiée ; schémas/bases dédiés ;
aucun `docker rm`, aucun `down -v`.

![État de la pile dédiée A-06](captures/a06-pile-dediee.png)
*Pile dédiée A-06 (7 conteneurs) — la pile en service `amane-*` n'est pas touchée.*

---

## 1. Listmonk — processus newsletter complet

**Image** : `100.64.0.6:5000/editeurs/listmonk@sha256:3089042a786beddb096410da3818b98ce9fe35bdc0a4ad4550f92be674b17291` (v6.2.0).

### Processus exécuté

1. **Install + upgrade** (hooks `pre_start` du manifeste) sur base dédiée
   `listmonk`, avec `LISTMONK_ADMIN_USER=admin` →
   `creating superadmin user 'admin'` / `setup complete`.
2. **Démarrage** du serveur Listmonk → `GET /health` = **200**.
3. **SMTP configuré en base** (jsonb `settings.smtp`) pointé vers le sidecar
   local `a06-mailpit:1025` (100 % local, aucun egress).
4. **Liste** créée via `POST /api/lists` → id **3** « A06 Newsletter ».
5. **Abonné réel** créé via `POST /api/subscribers` → id **3**
   `abonne@a06.local`, statut `enabled`, souscription `confirmed`.
6. **Campagne** créée via `POST /api/campaigns` → id **2** « Campagne A06 »,
   puis démarrée (`PUT /api/campaigns/2/status` → `running`).
7. **Envoi déclenché** : la campagne passe `finished`, **1 message envoyé**,
   capturé par le sidecar SMTP local (mailpit).

### Preuves

SQL (base `listmonk`) — `docs/tests/artifacts-a06/listmonk-sql.txt` :

```
liste       : id=3  name='A06 Newsletter'  type=public  status=active
abonné      : id=3  email='abonne@a06.local'  name='Abonne Test A06'  status=enabled
souscription: subscriber_id=3  list_id=3  status=confirmed
campagne    : id=2  name='Campagne A06'  status=finished  sent=1  to_send=1
```

Message capturé par mailpit (API `/api/v1/messages`) :

```
news@a06.local -> abonne@a06.local | "Bienvenue A06"
news@a06.local -> abonne@a06.local | "Confirm subscription"
```

![Message Listmonk capturé par le sidecar SMTP (mailpit)](captures/a06-listmonk-mailpit.png)
*Le message de campagne « Bienvenue A06 » arrive bien dans le sidecar SMTP local.*

![Campagne terminée — preuve SQL](captures/a06-listmonk-sql.png)
*La campagne id=2 est `finished`, `sent=1`.*

### Mesures (`docker stats`, pendant requêtes API)

| Conteneur | CPU | Mémoire | % RAM |
|---|---|---|---|
| `a06-listmonk` | 0,02 % | 26,52 MiB | 0,07 % |
| `a06-postgres` | 5,49 % | 49,29 MiB | 0,12 % |

Latence `GET /health` : **0,5–0,6 ms** (3 mesures).

### Verdict : **utilisable**

Processus newsletter complet prouvé de bout en bout (liste → abonné → campagne →
envoi → capture SMTP), sans egress. Réserve mineure : l'endpoint de test SMTP
`/api/settings/smtp/test` renvoie une erreur de lecture de port (`dial tcp :0`)
alors que l'envoi réel fonctionne — cosmétique, sans impact sur le processus.

---

## 2. Odoo 17 — base initialisée, contact + devis réels

**Image** : `odoo:17@sha256:8779f8157ddcc37b84014c94646073f46fd4ad776b7a5d95fc3b0dc73a24a8fa`
(v17.0-20260908, digest conforme à `docs/compatibilite-saas.md`).

### Processus exécuté

1. **Base dédiée** `odoo` créée dans le PostgreSQL dédié.
2. **Initialisation** : `odoo -d odoo -i base --stop-after-init` →
   `Registry loaded`, **114 tables** créées (conforme doc).
3. **Démarrage serveur** → `GET /web/health` = **200** `{"status": "pass"}`.
4. **Contact réel** créé via ORM (JSON-RPC `res.partner.create`) → id **41**
   « Client A06 », `client@a06.local`.
5. **Module `sale` installé** via ORM (`button_immediate_install`) → `installed`.
6. **Devis réel** créé via ORM (`sale.order.create`) → id **21**, ref `S00021`,
   partenaire 41, état `draft`.

### Preuves

SQL (base `odoo`) — `docs/tests/artifacts-a06/odoo-sql.txt` :

```
res_partner : id=41  name='Client A06'  email='client@a06.local'  phone='0600000000'
sale_order  : id=21  name='S00021'  partner_id=41  state=draft  amount_total=0.00
/web/health : {"status": "pass"}  (HTTP 200)
```

![Login Odoo 17](captures/a06-odoo-login.png)
*Odoo 17 démarré, page de login servie.*

![Devis S00021 — preuve SQL](captures/a06-odoo-sql.png)
*Le devis id=21 (S00021) et le contact id=41 existent en base.*

### Mesures (`docker stats`, pendant appels `/web/health`)

| Conteneur | CPU | Mémoire | % RAM |
|---|---|---|---|
| `a06-odoo` | 0,01 % | 150 MiB | 0,38 % |
| `a06-postgres` | 21,11 % | 130,2 MiB | 0,33 % |

Latence `GET /web/health` : **43–51 ms** (3 mesures).

### Verdict : **utilisable**

Base initialisée (114 tables), `/web/health` 200, contact et devis créés via
l'ORM et prouvés en SQL. Le pic CPU PostgreSQL (21 %) vient de l'installation du
module `sale` (création de nombreuses tables) ; au repos Odoo est à 0,01 % CPU.

---

## 3. app-multi — 3 services, 50+ jobs, zéro perte

**Image** : `amane/app-multi:a06` (build local depuis `ha-lab/app-multi/`,
code source non modifié ; l'image `amane/app-multi@sha256:8fd78…` du manifeste
n'étant pas présente sur le disque).

### Processus exécuté

1. **Base dédiée** `appmulti` + **migration** `002_init.sql` appliquée
   (`schema_version` = 2, table `jobs_results` créée).
2. **Redis** démarré (`a06-redis`, `PING` → `PONG`).
3. **Service web** démarré (`--mode web`) → `GET /sante` = **200**.
4. **Service worker** démarré (`--mode worker`) → `Worker démarré - attente jobs...`.
5. **60 jobs** poussés via `POST /enqueue` (30 `echo` + 30 `compute`), puis
   vagues de charge supplémentaires (40 + 3) → **103 jobs au total**.
6. **`GET /stats`** → `{"queue_length":0}` (file entièrement consommée).
7. **Preuve SQL** : `jobs_results` contient **103 lignes, 103 success,
   103 job_id distincts** → aucun job perdu, worker bien consommateur.

> Note technique : l'entrypoint de l'image est `/bin/sh -c` ; le worker doit
> être lancé avec une **chaîne de commande unique**
> (`"/application --mode worker --redis …"`), sinon `sh` n'avance pas les
> arguments et le process retombe en mode web. Les migrations ont été appliquées
> manuellement car `pre_start.sh` attend des variables `AMANE_APP_DB_*` non
> fournies dans ce banc (workaround documenté, sans modifier l'image).

### Preuves

SQL (base `appmulti`) — `docs/tests/artifacts-a06/appmulti-sql.txt` :

```
jobs_results : total=103  success=103  distinct_jobs=103
répartition  : echo=…  compute=…  (30/30 sur la 1re vague de 60)
/stats       : {"queue_length":0}
```

![jobs_results — preuve SQL](captures/a06-appmulti-sql.png)
*103 jobs en base, tous `success`, file vide.*

![Page d'accueil app-multi web](captures/a06-appmulti-web.png)
*Service web app-multi (mode web) opérationnel.*

### Mesures (`docker stats`, pendant vague de charge)

| Conteneur | CPU | Mémoire | % RAM |
|---|---|---|---|
| `a06-appweb` | 1,86 % | 3,426 MiB | 0,01 % |
| `a06-appworker` | 0,31 % | 2,91 MiB | 0,01 % |
| `a06-redis` | 0,56 % | 4,551 MiB | 0,01 % |
| `a06-postgres` | 0,35 % | 136,9 MiB | 0,34 % |

Latence `POST /enqueue` : **1,5–2,3 ms** (3 mesures).

### Verdict : **utilisable**

Les trois services (web, worker, redis) tournent, 103 jobs poussés et 100 %
traités (`jobs_results` = 103, file vide). Réserve : l'image du manifeste
(`amane/app-multi@sha256:8fd78…`) et `amane/redis@sha256:8da19…` ne sont pas
sur le disque ; build local depuis la source et `redis:7-alpine` utilisés. Le
hook `pre_start` du manifeste n'est pas fonctionnel sans les variables
`AMANE_APP_DB_*` (workaround : migration manuelle).

---

## Synthèse

| SaaS | Processus prouvé | verdict |
|---|---|---|
| **Listmonk** | liste → abonné → campagne → envoi → capture SMTP | **utilisable** |
| **Odoo 17** | base 114 tables → `/web/health` 200 → contact + devis (ORM) | **utilisable** |
| **app-multi** | 3 services → 103 jobs → 0 perdu (SQL) | **utilisable** |

Toutes les preuves (SQL, HTTP, `docker stats`, captures) sont dans
`docs/tests/artifacts-a06/` et `docs/tests/captures/a06-*.png`.

## Réserves / risques

- La pile en service `amane-*` est **dégradée indépendamment** de cette tâche
  (etcd unhealthy → PostgreSQL Patroni sans leader). Elle n'a pas été touchée.
- Images app-multi/redis du manifeste absentes du disque → build local + image
  redis publique. À figer (digest) pour une reproductibilité stricte.
- `pre_start.sh` d'app-multi dépend de variables `AMANE_APP_DB_*` non injectées
  dans ce banc ; migrations appliquées manuellement.