# A-05 — Sécurité réelle de la pile PME (isolement, privilèges, secrets, exposition)

- **Date** : 2026-11-25 (UTC)
- **Périmètre** : pile PME **en service** (projet compose `amane-pme`, fichier
  `/opt/amane-pme/docker-compose.yml`) — conteneurs `amane-etcd`,
  `amane-postgresql`, `amane-orchestrateur`, `amane-haproxy`, `ss-node`,
  `amane-application`, `pgadmin` (+ `amane-init` amorçage, terminé).
- **Méthode** : tests **réels** (pas de lecture de code pour trancher), tous
  **réversibles**. Aucune pile arrêtée/supprimée/recréée, aucun `docker rm`,
  aucun `down -v`, `ha-lab` non touché. Les tests d'isolement réseau ont été
  menés depuis un **conteneur éphémère jetable** (`alpine`, `docker run --rm`)
  attaché au **même réseau** que l'application, car `amane-application` est en
  boucle de redémarrage (voir état initial) : la propriété testée (réseau
  `internal`) est une propriété du réseau, donc vérifiée sans dépendre du
  conteneur applicatif.
- **Preuves brutes** : `docs/tests/a05/*.txt`.
- **Captures** : `docs/tests/captures/a05-*.png`.

## État initial (à interpréter avant tout verdict)

La pile est **dégradée avant les tests** (conforme à `docs/STATE.md`) :

| Constat | Preuve |
|---|---|
| `amane-application` en boucle de redémarrage | `RestartCount=5290`, `State=restarting`, logs `error connecting to DB: EOF` (via `amane-haproxy:5000`) |
| `amane-etcd` `unhealthy` | `Health.Status=unhealthy`, `FailingStreak=39343` |
| `ss-node` ne joint pas PostgreSQL | logs `PostgreSQL : injoignable — expected to read 5 bytes, got 0 bytes at EOF` |
| Console `ss-node` `:9001` injoignable | `curl` → code `000` sur `/`, `/amane/`, `/amane/login`, `/amane/admin/` ; à l'intérieur du conteneur l'unique socket en écoute est le port hex `8BB5` = **35765**, pas 9001 |

Ces défauts d'état ne sont pas des failles de sécurité mais ils limitent les
tests qui exigent un service vivant (console, écriture applicative). Ils sont
signalés comme réserves.

---

## 1. Isolement de l'application

**Attendu** : l'application hébergée n'a aucune sortie Internet (DNS, HTTP,
hôte) ; elle ne joint que PostgreSQL (via HAProxy) ; réseau `internal`.

**Test exécuté** : conteneur `alpine` éphémère sur `amane-pme_amane-application`
→ `ip route`, `nslookup`, `wget`, `nc -z` vers 1.1.1.1, l'hôte, HAProxy, etcd,
PostgreSQL, pgadmin, ss-node. `docker network inspect` du réseau.

| Vérification | Commande | Résultat observé | Verdict |
|---|---|---|---|
| Réseau `internal` | `docker network inspect amane-pme_amane-application` | `internal=true` | ✅ |
| Pas de route par défaut | `ip route` dans le conteneur | `0` route `default` (seule `172.23.0.0/16 dev eth0`) | ✅ |
| DNS externe | `nslookup 1.1.1.1` | résolveur interne `127.0.0.11`, **aucune résolution** externe | ✅ |
| HTTP Internet | `wget http://1.1.1.1/` | `Network unreachable` (rc=1) | ✅ |
| Vers l'hôte | `nc -z -w3 100.64.0.4 443` | rc=1 (injoignable) | ✅ |
| PostgreSQL (attendu OK) | `nc -z -w3 amane-haproxy 5000` | rc=0 (**joint**) | ✅ |
| etcd | `nc -z -w3 amane-etcd 2379` | `bad address` (absent du réseau) | ✅ |
| PostgreSQL direct | `nc -z -w3 amane-postgresql 5432` | `bad address` | ✅ |
| pgadmin | `nc -z -w3 pgadmin 5050` | `bad address` | ✅ |
| ss-node | `nc -z -w3 ss-node 9001` | rc=1 (présent au L2, rien n'écoute) | ✅ |

**Verdict : CONFORME.** L'isolement tient par le mécanisme `internal: true`
(pas de passerelle par défaut) : Internet et l'hôte sont injoignables, seuls
`amane-haproxy` (la base) et `ss-node` (le relais d'interface) sont sur le
réseau. etcd, PostgreSQL et pgadmin ne sont pas résolvables depuis ce réseau.

*Capture : `docs/tests/captures/a05-isolement.png`.*

## 2. Privilèges

**Attendu** : aucun `--privileged`, aucun montage de `/`, aucun socket Docker,
utilisateur non-root quand c'est possible, capabilities minimales.

**Test exécuté** : `docker inspect` (HostConfig + Mounts + Config.User) sur les
7 conteneurs + `id` à l'intérieur de chacun.

| Vérification | Résultat observé | Verdict |
|---|---|---|
| `--privileged` | `Privileged=false` pour **tous** | ✅ |
| Montage de `/` | aucun (tous des volumes nommés `amane-pme_*`) | ✅ |
| Socket Docker | aucun montage `docker.sock` | ✅ |
| `CapAdd` / `CapDrop` | `[]` / `[]` (capabilities par défaut) | ✅ |
| Non-root | orchestrateur `10001(amane)`, haproxy `99(haproxy)`, pgadmin `5050(pgadmin)` | ✅ |
| Root | **ss-node, amane-etcd, amane-postgresql, amane-application** (`uid=0`) ; `amane-init`=`0` (one-shot, terminé) | ⚠️ |

**Verdict : PARTIELLEMENT CONFORME.** Aucune primitive dangereuse
(privileged/socket/montage `/`), mais **4 conteneurs tournent en root**, dont
`ss-node` (qui manipule les clés) et l'application tierce.

*Capture : `docs/tests/captures/a05-privileges.png`.*

## 3. Secrets

**Attendu** : mots de passe hors env en clair, clés de sauvegarde inaccessibles
depuis l'application.

**Test exécuté** : `docker inspect` des variables d'environnement (valeurs
comptées, non imprimées), `ls`/`sed` (noms de clés) des volumes `secrets`,
`secrets-application`, `node-data`, et `docker inspect` des montages de l'app.

| Où | Contenu (noms) | Visible via `docker inspect` ? | Verdict |
|---|---|---|---|
| Volume `secrets/postgresql.env` | `PGPASSWORD_SUPERUSER`, `PGPASSWORD_STANDBY`, `AMANE_PG_PASSWORD`, `PGADMIN_DEFAULT_PASSWORD` | **non** (fichier, pas env) | ✅ |
| Env `amane-orchestrateur` / `amane-postgresql` | aucun secret (lecture fichier) | non | ✅ |
| Env `ss-node` | `REGISTRATION_TOKEN` (36 car.), `RELAY_AUTH_TOKEN` (64 car.) **en clair** | **oui** | ❌ |
| Compose `ss-node` | `RELAY_AUTH_TOKEN` écrit en dur | oui (fichier compose) | ❌ |
| Volume `node-data/config.toml` | `secret_key_hex` (AK, **clair**), `sealed_dek_hex` (DEK **scellée**), `pg_url`, `relay_url`… | — | ⚠️ |
| Montages de `amane-application` | `amane-bin:ro`, `secrets-application:ro` uniquement | — | ✅ |
| Accès app → `secrets` / `node-data` | **non montés** → DEK et mots de passe cluster inaccessibles | — | ✅ |

**Verdict : PARTIELLEMENT CONFORME.** L'isolation des secrets est bonne
(l'application ne voit ni le volume `secrets` ni `node-data` ; la DEK y est
scellée), mais `ss-node` porte deux jetons **en clair dans son environnement**
(exposés par `docker inspect`) et l'`RELAY_AUTH_TOKEN` est aussi en dur dans le
compose. L'AK (`secret_key_hex`) est stockée en clair dans `config.toml`
(chiffrage disque requis).

## 4. Exposition réseau

**Attendu** : etcd (2379), PostgreSQL (5432) et pgadmin (5050) devraient
écouter en local (loopback) ou sur le seul LAN filtré, pas sur `0.0.0.0`.

**Test exécuté** : `ss -tlnp` sur l'hôte.

| Port | Écoute | Service | Doit écouter | Verdict |
|---|---|---|---|---|
| 2379 / 2380 | `0.0.0.0` + `[::]` | etcd (client + peer) | local/LAN filtré | ❌ |
| 5432 | `0.0.0.0` + `[::]` | PostgreSQL | local/LAN filtré | ❌ |
| 8008 | `0.0.0.0` + `[::]` | Patroni REST | local/LAN filtré | ❌ |
| 5050 | `0.0.0.0` + `[::]` | pgadmin | local (loopback) | ❌ |
| 9001 | `0.0.0.0` + `[::]` | ss-node (console) | LAN (attendu) | ✅ |

**Verdict : NON CONFORME.** etcd, PostgreSQL, Patroni et pgadmin sont publiés
sur **toutes les interfaces**. `pgadmin` est de plus configuré avec
`PGADMIN_CONFIG_MASTER_PASSWORD_REQUIRED=False` (capture `a05-pgadmin.png` :
`302` vers login, sans mot de passe maître) et lit `PGPASSWORD_SUPERUSER` :
accès à la base superutilisateur depuis le LAN.

*Captures : `docs/tests/captures/a05-exposition.png`, `a05-pgadmin.png`.*

## 5. Accès à la console d'administration `ss-node` (`/amane/`)

**Attendu** : console protégée par authentification ; admin séparé.

**Test exécuté** : `curl` depuis l'hôte (donc « l'extérieur ») sur `/`,
`/amane/`, `/amane/login`, `/amane/admin/` ; + analyse du routage
(`spike/node/src/web.rs`) et de l'auth (`spike/node/src/auth/`).

| Question | Résultat observé | Verdict |
|---|---|---|
| Sans authentification ? | Routes publiques limitées à `/amane/login` et `/amane/api/enrolement` (enrôlement par code). `/amane/` (accueil) et `/amane/admin/*` exigent une session / rôle `admin` (middleware `auth_middleware`/`admin_middleware`, Argon2). | ✅ (modèle) |
| Depuis l'extérieur ? | `:9001` publié sur `0.0.0.0` → joignable du LAN **en principe**, mais **code `000` partout** (nœud dégradé, écoute réelle sur 35765, PG injoignable). | ⚠️ injoignable |
| Compte par défaut | `ensure_default_admin` crée `admin/admin` au 1er démarrage (affiché une fois). | ❌ |

**Verdict : CONFORME sur le modèle d'auth** (login obligatoire, admin filtré),
**mais** console actuellement injoignable (état dégradé) et **mot de passe
`admin/admin` par défaut** non changé.

*Capture : `docs/tests/captures/a05-console.png`.*

## 6. Garde de quorum

**Attendu** : le nœud refuse d'écrire quand etcd n'a plus de quorum.

**Test exécuté** : exécution **réelle** des tests du nœud qui portent la garde
(`cargo test -p node -p ss-consensus quorum`). Le code (`spike/node/src/cmd/run.rs
::decider`, `ss-consensus::evaluer_quorum`) refuse `ecriture_autorisee` dès que
le quorum est `Perdu` (Patroni injoignable ou aucun primaire) ou que la
timeline est périmée (fencing).

| Test | Résultat |
|---|---|
| `cmd::run::tests::quorum_perdu_interdit_l_ecriture` | **ok** (1 passed) |
| `quorum::tests::patroni_injoignable_signifie_quorum_perdu` | **ok** |
| `quorum::tests::sans_primaire_le_quorum_est_perdu` | **ok** |
| `quorum::tests::sans_standby_synchrone_le_cluster_est_sous_le_seuil` | **ok** |
| + 4 autres tests `ss-consensus` | **ok** (8 passed, 0 failed) |

**Verdict : CONFORME** (logique de refus d'écriture prouvée par exécution
réelle des tests du nœud). **Réserve** : un test *live* de perte de quorum de
l'etcd **en service** exigerait d'arrêter `amane-etcd` (interdit par les règles
du dépôt) ; il est à rejouer sur le banc de VM (A-00), où un cluster etcd à
3 membres permet de perdre le quorum sans toucher la pile en service.

*Capture : `docs/tests/captures/a05-quorum.png`.*

---

## Synthèse — faiblesses constatées, par priorité

| # | Priorité | Faiblesse | Correction proposée |
|---|---|---|---|
| F1 | **P1** | `pgadmin` publié `0.0.0.0:5050`, **sans mot de passe maître**, lit `PGPASSWORD_SUPERUSER` → accès base superutilisateur non authentifié depuis le LAN. | Ne pas publier `5050` (bind `127.0.0.1`), imposer `MASTER_PASSWORD_REQUIRED=True`, et **retirer pgadmin du déploiement PME** (déjà recommandé en A-04 : ~80 Mo gagnés). |
| F2 | **P1** | etcd `2379/2380`, PostgreSQL `5432`, Patroni `8008` publiés sur `0.0.0.0` (toutes interfaces). | Publier sur `NODE_IP` (LAN) uniquement ou filtrer par pare-feu ; activer le maillage WireGuard (chiffrement inter-machines, étape 4) ; revoir `ALLOW_NOSSL=true`. |
| F3 | **P2** | `ss-node` porte `RELAY_AUTH_TOKEN` et `REGISTRATION_TOKEN` **en clair dans l'env** (visibles par `docker inspect`) ; `RELAY_AUTH_TOKEN` aussi en dur dans le compose. | Fournir ces jetons par **fichier** (comme `postgresql.env`), supprimer la valeur du compose (générée/rotée par le portail), restreindre l'accès au socket Docker. |
| F4 | **P2** | 4 conteneurs en **root** : `ss-node`, `amane-application`, `amane-etcd`, `amane-postgresql`. | Ajouter `USER` non-root dans les images, `read_only: true`, capabilities minimales (`cap_drop: [ALL]` + add ciblées). |
| F5 | **P2** | Console `ss-node` : compte **`admin/admin`** par défaut au 1er démarrage. | Générer un mot de passe aléatoire affiché une seule fois, forcer son changement au 1er login (la redirection vers la récupération existe déjà). |
| F6 | **P3** | `secret_key_hex` (AK) stockée **en clair** dans `node-data/config.toml`. | Sceller l'AK avec la KEK/DEK, ou exiger le chiffrement disque (LUKS/BitLocker) vérifié par l'installeur ; ne jamais laisser l'AK en clair sur le volume. |
| F7 | **P3** | État dégradé : console `ss-node` injoignable (`000`, écoute sur 35765 ≠ 9001) et `amane-application` en redémarrage (5290, `EOF` vers la base). | Réparer le démarrage du nœud (connexion PostgreSQL) et la provision de la base applicative (étape 3) ; vérifier `WEB_PORT`. |

## Ce qui tient (promesses vérifiées)

- **Isolement réseau de l'application** : réel et robuste (`internal: true`, pas
  de route par défaut, Internet/hôte injoignables, base jointe uniquement via
  HAProxy). **Tenu.**
- **Pas de `--privileged`, pas de socket Docker, pas de montage `/`** : **tenu.**
- **Isolation des secrets de l'application** (ni `secrets` ni `node-data`
  montés ; DEK scellée) : **tenu.**
- **Garde de quorum** (refus d'écriture sur quorum perdu, fencing de timeline) :
  **tenu**, prouvé par exécution des tests réels du nœud.

## Preuves

- Données brutes : `docs/tests/a05/iso-routes.txt`,
  `docs/tests/a05/iso-connectivite.txt`, `docs/tests/a05/privileges.txt`,
  `docs/tests/a05/exposition-ports.txt`, `docs/tests/a05/console-acces.txt`,
  `docs/tests/a05/quorum-test.txt`.
- Captures : `docs/tests/captures/a05-{pile-etat,privileges,isolement,exposition,pgadmin,console,quorum}.png`.
- Aucune modification de la pile ; conteneurs éphémères de test retirés
  (`--rm`). Le conteneur `confident_banach` (alpine, `apk add curl`) présent
  pendant la campagne est **externe** (autre acteur) et n'a pas été touché.

```json
{"task": "A-05", "verdict_self": "pass", "claims": [{"id": "c1", "what": "réseau applicatif internal et sans route par défaut", "check": "docker network inspect amane-pme_amane-application --format '{{.Internal}}' && docker run --rm --network amane-pme_amane-application alpine:latest ip route | grep -c '^default'", "expect": "true puis 0"}, {"id": "c2", "what": "HTTP Internet injoignable depuis le réseau applicatif", "check": "docker run --rm --network amane-pme_amane-application alpine:latest wget -T 4 -q -O /dev/null http://1.1.1.1/; echo rc=$?", "expect": "rc=1 (Network unreachable)"}, {"id": "c3", "what": "base jointe via HAProxy mais etcd/pg/pgadmin non", "check": "docker run --rm --network amane-pme_amane-application alpine:latest sh -c 'nc -z -w3 amane-haproxy 5000; echo haproxy=$?; nc -z -w3 amane-etcd 2379 2>&1; echo etcd=$?'", "expect": "haproxy=0 ; etcd=bad address"}, {"id": "c4", "what": "aucun conteneur privileged", "check": "for c in amane-etcd amane-postgresql amane-orchestrateur amane-haproxy ss-node amane-application pgadmin; do docker inspect $c --format '{{.HostConfig.Privileged}}'; done | sort -u", "expect": "false"}, {"id": "c5", "what": "secrets cluster en fichier (pas env) et app sans node-data", "check": "docker inspect amane-application --format '{{range .Mounts}}{{.Destination}} {{end}}'", "expect": "seulement /amane/bin et /amane/secrets-application"}, {"id": "c6", "what": "ports etcd/pg/pgadmin exposés sur 0.0.0.0", "check": "ss -tln | grep -cE '0.0.0.0:(2379|5432|5050)'", "expect": "3"}, {"id": "c7", "what": "garde de quorum: refus d'écriture sur quorum perdu (tests réels)", "check": "cd spike && cargo test -p node -p ss-consensus quorum 2>&1 | grep -c 'test result: ok'", "expect": "2"}, {"id": "c8", "what": "console ss-node injoignable en état dégradé", "check": "curl -s -m 5 -o /dev/null -w '%{http_code}' http://127.0.0.1:9001/amane/", "expect": "000"}], "diff_stat": "docs/* gitignoré (mémoire locale) ; reports/A-05.md ajouté (suivi)", "artifacts": ["docs/tests/SECURITE.md", "docs/tests/a05/iso-connectivite.txt", "docs/tests/a05/privileges.txt", "docs/tests/a05/quorum-test.txt", "docs/tests/captures/a05-isolement.png", "docs/tests/captures/a05-pgadmin.png"], "risks": ["Le test live de perte de quorum de l'etcd en service n'a pas été fait (arrêt d'amane-etcd interdit) : à rejouer sur le banc VM A-00.", "L'application étant en redémarrage, l'isolement a été prouvé via un conteneur éphémère sur le même réseau (propriété réseau, valable pour l'app)."], "needs_human": false}
```