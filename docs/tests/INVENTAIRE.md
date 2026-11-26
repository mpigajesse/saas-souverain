# A-03 — Inventaire réel du banc AMANE

- **Relevé** : 2026-11-25 18:23 → 18:32 UTC (horodatage brut : `artifacts-a03/date.txt`)
- **Périmètre** : machines, conteneurs, images, ports, réseaux, volumes, versions et ressources **tels qu'observés**, avant toute campagne.
- **Aucune modification** : uniquement `docker ps/inspect/stats/logs/system df`, `curl` en lecture, `free/nproc/df`, `ps`, et écriture des preuves sous `docs/tests/`. Aucun `docker stop/rm`, aucun `down`, aucun redémarrage, aucun `sudo`.
- Les sorties brutes sont conservées dans [`artifacts-a03/`](artifacts-a03/) ; chaque tableau ci-dessous est reproductible par la commande indiquée.

> **Note de méthode** : le modèle proposé dans le brief, `docker inspect --format '{{index .RepoDigests 0}}' <conteneur>`, échoue avec `map has no entry for key "RepoDigests"` : ce champ appartient à l'**image**, pas au conteneur. Les digests ci-dessous sont obtenus avec `docker image inspect --format '{{index .RepoDigests 0}}' "$(docker inspect --format '{{.Config.Image}}' <conteneur>)"` ; ils concordent avec le suffixe `@sha256:…` de `.Config.Image` pour les 8 conteneurs de la pile. Voir `artifacts-a03/images-digests.txt`.

## 1. Conteneurs

Commande : `docker ps -a --format '{{.Names}}\t{{.Image}}\t{{.Status}}\t{{.Ports}}'` → `artifacts-a03/docker-ps-a.txt`. **25 conteneurs** : 7 `Up` (dont 1 `Restarting`) et 18 `Exited`.

### 1.1 Pile PME — projet Compose `amane-pme` (8 conteneurs)

| Conteneur | Image configurée (épinglée) | Statut au relevé | Ports publiés | Digest complet (`RepoDigests`) |
|---|---|---|---|---|
| `amane-etcd` | `100.64.0.6:5000/amane/etcd@sha256:9344cfb9…` | `Up 4 days (unhealthy)` | `0.0.0.0:2379-2380->2379-2380/tcp`, `[::]:2379-2380->2379-2380/tcp` | `100.64.0.6:5000/amane/etcd@sha256:9344cfb9cbe4df0635478b6a2b62765330128fbdf3ca8fc9f2edac262552f700` |
| `amane-postgresql` | `100.64.0.6:5000/amane/postgresql@sha256:17bf371c…` | `Up 4 days` | `0.0.0.0:5432->5432/tcp`, `[::]:5432->5432/tcp`, `0.0.0.0:8008->8008/tcp`, `[::]:8008->8008/tcp`, `8080/tcp` (non publié) | `100.64.0.6:5000/amane/postgresql@sha256:17bf371c924196e4671f19d129fa86a80f7dcea7b2666cedca955ec0fba0b72b` |
| `amane-orchestrateur` | `100.64.0.6:5000/amane/orchestrateur@sha256:6a082c40…` | `Up 4 days` | `50051/tcp` (exposé, non publié) | `100.64.0.6:5000/amane/orchestrateur@sha256:6a082c400a2809e927e98bc94f2494e80291df92aa987458f7cdcf351fa63c35` |
| `amane-haproxy` | `100.64.0.6:5000/amane/haproxy@sha256:43f51ba5…` | `Up 4 days` | aucun (port interne `5000`) | `100.64.0.6:5000/amane/haproxy@sha256:43f51ba59dfffa4095c3dfd8363d11a3795369e985249c6d715f468c7def12d2` |
| `ss-node` | `100.64.0.6:5000/amane/noeud@sha256:cea1862c…` | `Up 4 days` | `0.0.0.0:9001->9001/tcp`, `[::]:9001->9001/tcp` | `100.64.0.6:5000/amane/noeud@sha256:cea1862ce5bbcbbc78a111ed77ee22d19bf629c355ab8fc77393188d78387cd4` |
| `amane-application` | `100.64.0.6:5000/editeurs/listmonk@sha256:3089042a…` | `Restarting (1)` — **5230 redémarrages** au relevé | aucun (image expose `9000/tcp`, `LISTMONK_APP__ADDRESS=0.0.0.0:9000`) | `100.64.0.6:5000/editeurs/listmonk@sha256:3089042a786beddb096410da3818b98ce9fe35bdc0a4ad4550f92be674b17291` |
| `pgadmin` | `100.64.0.6:5000/amane/pgadmin@sha256:2fbd14f8…` | `Up 4 days` | `443/tcp`, `0.0.0.0:5050->80/tcp`, `[::]:5050->80/tcp` | `100.64.0.6:5000/amane/pgadmin@sha256:2fbd14f87cfc030917cc3602acea551e2b2d7167501ac54f09af17b19c094645` |
| `amane-init` | `100.64.0.6:5000/amane/orchestrateur@sha256:6a082c40…` | `Exited (1) 4 days ago` (`restart: "no"`) | aucun | `100.64.0.6:5000/amane/orchestrateur@sha256:6a082c400a2809e927e98bc94f2494e80291df92aa987458f7cdcf351fa63c35` |

*État des conteneurs de la pile au relevé (`pilot-shot term`).*

### 1.2 Autres conteneurs présents sur la machine (17)

Ils ne font **pas** partie de la pile PME mais occupent le démon Docker ; aucun n'a été touché.

| Conteneurs (regroupés) | Image | Statut | Digest complet | Nb |
|---|---|---|---|---:|
| `tp3-cassandra-node1`, `tp3-cassandra-node2`, `tp3-cassandra-node3` | `cassandra:5.0.9` | `Exited (137) 4 hours ago` | `cassandra@sha256:7e2afd76d5afb991f64c5d9be7d1f8abf749e7ab0e9daaccfd6e865efcece7cc` | 3 |
| `cassandra-tp2` | `cassandra:5.0.9` | `Exited (143) 5 hours ago` | idem | 1 |
| `cassandra-node` | `cassandra:5.0.9` | `Exited (143) 12 days ago` | idem | 1 |
| `db_admistration-namenode-1`, `db_admistration-datanode1-1` … `datanode5-1`, `db_admistration-resourcemanager-1`, `db_admistration-nodemanager-1` | `apache/hadoop:3.3.6` | `Exited (143) 12 days ago` | `apache/hadoop@sha256:62a22627f35a1bbf29c721192c02fe311c93749cc43d51df01792b2ac40ff144` | 8 |
| `listmonk-test` | `3089042a786b` | `Exited (2) 12 days ago` | `100.64.0.6:5000/editeurs/listmonk@sha256:3089042a786beddb096410da3818b98ce9fe35bdc0a4ad4550f92be674b17291` | 1 |
| `amane-mailpit` | `axllent/mailpit` | `Exited (0) 12 days ago` | `axllent/mailpit@sha256:aef2aacc11f075ec0fb6874275ee4a421e688e11a96ebb296134d1735c7761ce` | 1 |
| `amane-registry` | `registry:2` | `Exited (2) 12 days ago` | `registry@sha256:a3d8aaa63ed8681a604f1dea0aa03f100d5895b6a58ace528858a7b332415373` | 1 |
| `amane-test-pg` | `postgres:17-alpine` | `Exited (0) About an hour ago` | `postgres@sha256:3540c0362c809df084ffac0ae69fe794515f9de92692f02178bcdce0ef0302ac` | 1 |

Labels Compose (`artifacts-a03/compose-labels.txt`) : projets `tp3-cassandra`, `tp2-cassandra`, `atelier-cassandra`, `db_admistration`, `deploiement_ia` ; quatre conteneurs sans projet (`listmonk-test`, `amane-mailpit`, `amane-registry`, `amane-test-pg`).

## 2. Réseaux et volumes

### 2.1 Réseaux (11)

Commande : `docker network ls` + `docker network inspect` → `artifacts-a03/network-ls.txt`, `artifacts-a03/reseaux-tous.txt`.

| Réseau | ID (12 car.) | Driver | Internal | Sous-réseau / passerelle | Membres vus par `inspect` |
|---|---|---|---|---|---|
| `amane-listmonk-net` | `cbdc4fbefff9` | bridge | non | `172.18.0.0/16` / `172.18.0.1` | aucun |
| `amane-pme_amane` | `e16380dab405` | bridge | **non** | `172.22.0.0/16` / `172.22.0.1` | `amane-orchestrateur` .2, `amane-etcd` .3, `ss-node` .4, `amane-postgresql` .5, `amane-haproxy` .6, `pgadmin` .7 |
| `amane-pme_amane-application` | `35983ce23e18` | bridge | **oui** | `172.23.0.0/16` / `172.23.0.1` | `ss-node` .3, `amane-haproxy` .4 (et `amane-application` : endpoint déclaré, **aucune IP attribuée**, `EndpointID` vide car il redémarre) |
| `atelier-cassandra_default` | `6f30a6e861fe` | bridge | non | `172.21.0.0/16` / `172.21.0.1` | aucun actif |
| `bridge` | `7337d3d70da1` | bridge | non | `172.17.0.0/16` / `172.17.0.1` | aucun |
| `db_admistration_default` | `4795d203b7f8` | bridge | non | `172.19.0.0/16` / `172.19.0.1` | aucun actif |
| `deploiement_ia_default` | `fe60174ae406` | bridge | non | `172.20.0.0/16` / `172.20.0.1` | aucun actif |
| `host` | `ce7c0057ca5e` | host | — | pas d'IPAM | aucun |
| `none` | `29a06f3db6ae` | null | — | pas d'IPAM | aucun |
| `tp2-cassandra_default` | `c87b365247dc` | bridge | non | `172.24.0.0/16` / `172.24.0.1` | aucun actif |
| `tp3-cassandra_net` | `3d972887c7ec` | bridge | non | `172.25.0.0/16` / `172.25.0.1` | aucun actif |

Affectation et passerelles de la pile (`artifacts-a03/container-networks.txt`) :

| Conteneur | Réseau(s) | IP | Passerelle effective |
|---|---|---|---|
| `amane-etcd` | `amane-pme_amane` | `172.22.0.3/16` | `172.22.0.1` |
| `amane-postgresql` | `amane-pme_amane` | `172.22.0.5/16` | `172.22.0.1` |
| `amane-orchestrateur` | `amane-pme_amane` | `172.22.0.2/16` | `172.22.0.1` |
| `amane-haproxy` | `amane-pme_amane` + `amane-pme_amane-application` | `172.22.0.6/16`, `172.23.0.4/16` | `172.22.0.1` ; aucun gateway sur le réseau interne (`internal: true`) |
| `ss-node` | `amane-pme_amane` + `amane-pme_amane-application` | `172.22.0.4/16`, `172.23.0.3/16` | `172.22.0.1` ; aucun gateway sur le réseau interne |
| `amane-application` | `amane-pme_amane-application` seul | aucune IP attribuée | aucune (conteneur en redémarrage) |
| `pgadmin` | `amane-pme_amane` | `172.22.0.7/16` | `172.22.0.1` |
| `amane-init` | `amane-pme_amane` (exited) | aucune | aucune |

### 2.2 Volumes (27)

Commande : `docker volume ls` + `docker system df -v` → `artifacts-a03/volume-ls.txt`, `artifacts-a03/docker-system-df-v.txt`.

| Volume | Liens | Taille | Appartient à la pile |
|---|---:|---:|---|
| `amane-pme_etcd-data` | 2 | 128,9 Mo | oui |
| `amane-pme_etcd-conf` | 2 | 644 o | oui |
| `amane-pme_pg-data` | 1 | 277,7 Mo | oui |
| `amane-pme_secrets` | 5 | 322 o | oui |
| `amane-pme_secrets-application` | 3 | 297 o | oui |
| `amane-pme_haproxy` | 3 | 1,101 ko | oui |
| `amane-pme_amane-bin` | 2 | 19,16 Mo | oui |
| `amane-pme_node-data` | 1 | 7,792 ko | oui |
| `amane-pme_pgadmin-data` | 1 | 169,3 ko | oui |
| `amane-registry-data` | 1 | 0 o | non (registre arrêté) |
| `atelier-cassandra_cassandra_data` | 1 | 33,9 Mo | non |
| `chatbot-bi_clickhousedata` | 0 | 648,5 Mo | non |
| `chatbot-bi_miniodata` | 0 | 2,234 Go | non |
| `chatbot-bi_neo4jdata` | 0 | 547,9 Mo | non |
| `chatbot-bi_pgdata` | 0 | 214,3 Mo | non |
| `maree_redpanda-data` | 0 | 285,2 Mo | non |
| `maree-simulation_redpanda-data` | 0 | 285,4 Mo | non |
| `maree-simulation_sim-data` | 0 | 0 o | non |
| `maree-simulation_sim-logs` | 0 | 4,911 Mo | non |
| `c9877250a974…beba` | 1 | 74,06 Mo | non (anonyme) |
| `d0296b74d86a…a2a` | 1 | 0 o | non (anonyme) |
| `f7dd3306f546…610f` | 1 | 0 o | non (anonyme) |
| `073386217ef5…fffb` | 1 | 0 o | non (anonyme) |
| `157032ccd946…5b06` | 1 | 0 o | non (anonyme) |
| `33f603274cd3…cb38e` | 1 | 0 o | non (anonyme) |
| `43ae67d60ec1…414d4b` | 1 | 0 o | non (anonyme) |
| `7aec4dc68213…f97` | 1 | 0 o | non (anonyme) |

Total des volumes `amane-pme_*` : **≈ 425,9 Mo** (`artifacts-a03/calcul-part-pile.txt`). Montages déclarés : `artifacts-a03/services-detail.txt` (ex. `amane-pme_secrets:/amane/secrets:ro` pour PostgreSQL et le nœud, `amane-pme_secrets-application:/amane/secrets-application:ro` pour l'application — séparation issue de l'étape 2.6/ADR-007).

## 3. Services de la pile PME — rôle, ports, santé, ressources, utilisateur, capabilities

Sources : `artifacts-a03/services-detail.txt`, `artifacts-a03/services-inspect.txt`, `artifacts-a03/capabilities-effectives.txt`, `artifacts-a03/processus-utilisateurs.txt`, `artifacts-a03/ports-exposes.txt`, `artifacts-a03/logs-tail.txt`.

| Service | Rôle constaté | Ports exposés / publiés | Santé (`docker inspect …State.Health.Status`) | Limites déclarées | Utilisateur d'exécution | Capabilities |
|---|---|---|---|---|---|---|
| `amane-etcd` | Consensus Raft, membership, leases, secrets et topologie Patroni | exposés `2379/tcp`, `2380/tcp` ; publiés sur `0.0.0.0` et `[::]` | **`unhealthy`** (seul service à déclarer une sonde) | aucune : `Memory=0`, `NanoCpus=0`, `CpuQuota=0`, `CpusetCpus=''`, `PidsLimit=null` | `Config.User=''` → défaut image = **root** (`docker top` : UID 0) | `CapAdd`/`CapDrop` vides, `Privileged=false` ; `CapEff=0x00000000a80425fb` (jeu Docker par défaut) |
| `amane-postgresql` | Spilo/Patroni : base répliquée, API REST, rôles `amane`/`application` | exposés `5432`, `8008`, `8080` ; publiés `5432`, `8008` | **aucun objet Health** → pas de statut Docker | idem (aucune) | `Config.User=''` ; init `root`, processus Postgres/Patroni **UID 101** | `CapAdd`/`CapDrop` vides, non privilégié ; `CapEff=0` sur `patroni` |
| `amane-orchestrateur` | `init`, secrets via etcd, superviseur Patroni, config HAProxy, synchrone-auto, base de l'application, gRPC | exposé `50051/tcp`, non publié | aucun objet Health | idem (aucune) | **`amane`** (UID 10001, conforme au Dockerfile) | `CapAdd`/`CapDrop` vides, non privilégié ; `CapEff=0` |
| `amane-haproxy` | Routage vers le primaire PostgreSQL ; config rechargée par l'orchestrateur | aucun port exposé/publié ; joint en interne sur `amane-haproxy:5000` | aucun objet Health | idem (aucune) | **`haproxy`** (UID 99) | `CapAdd`/`CapDrop` vides, non privilégié ; `CapEff=0` |
| `ss-node` | Reverse proxy de l'application, console `/amane/`, enrôlement, sauvegardes chiffrées, garde de quorum en lecture | exposé et publié `9001/tcp` | aucun objet Health | idem (aucune) | `Config.User=''` → défaut image = **root** (`docker top` : UID 0 ; `run --mode active`) | `CapAdd`/`CapDrop` vides, non privilégié ; `CapEff=0x00000000a80425fb` |
| `amane-application` | Listmonk, SaaS hébergé, seul sur le réseau `internal: true` | image expose `9000/tcp` ; aucun port publié ; `LISTMONK_APP__ADDRESS=0.0.0.0:9000` | aucun objet Health ; `status=restarting`, **5230** redémarrages, `ExitCode=1` | idem (aucune) | `Config.User=''` → **root** (`Launching listmonk with user=[root] group=[root]`) | `CapAdd`/`CapDrop` vides, non privilégié ; caps non mesurables (redémarrage permanent) |

Politique de redémarrage des six services : `unless-stopped` partout. **Aucune limite CPU, mémoire ou PID n'est déclarée** dans l'un des six conteneurs : tous peuvent consommer la RAM et les 8 cœurs de l'hôte.

### État de service observé (au-delà du statut Docker)

- etcd : `curl http://127.0.0.1:2379/version` → timeout à 5 s ; `etcdctl endpoint status` → `DeadlineExceeded`. Journaux : élections répétées au term 450, pair `100.64.0.7:2380` en `no route to host` / `i/o timeout`, `failed to publish local member to cluster`.
- PostgreSQL/Patroni : `patroni 4.0.4`, `postgres (PostgreSQL) 17.5` ; `GET /patroni` sur `127.0.0.1:8008` → `Connection reset by peer`, et depuis le conteneur → `Connection refused`. Journaux : `waiting on etcd` en boucle.
- Orchestrateur : journaux `membres Patroni illisibles`, `secrets du cluster indisponibles`, `dial tcp 172.22.0.3:2379: i/o timeout`.
- HAProxy : `haproxy version is 3.0.30-47d806b` ; journaux `Server primaire/… is DOWN`, **`proxy 'primaire' has no server available!`** puis rechargements de config.
- `ss-node` : `curl http://127.0.0.1:9001/` → `Connection reset by peer` ; journaux `PostgreSQL : injoignable — expected to read 5 bytes, got 0 bytes at EOF` et `SaaS : injoignable — http://100.64.0.6:5173/api/devices/register/`.
- `amane-application` : `v6.2.0 (ef0a7587 2026-06-26T10:21:29Z)` puis `error connecting to DB: EOF` toutes les ~60 s.
- pgAdmin : `curl http://127.0.0.1:5050/` → **302** (interface accessible).
- Tentative de capture web de `http://127.0.0.1:9001/` : `net::ERR_CONNECTION_RESET` — aucune image produite ; la capture terminal de l'erreur est conservée.

## 4. Versions réelles

| Composant | Version mesurée | Commande / preuve |
|---|---|---|
| etcd | **3.5.5** (Git SHA `19002cfc6`, compilé en Go 1.16.15) | `docker exec amane-etcd etcd --version` — `artifacts-a03/version-etcd.txt` |
| PostgreSQL | **17.5** (`Ubuntu 17.5-1.pgdg22.04+1`) | `docker exec amane-postgresql postgres --version` — `artifacts-a03/version-postgresql.txt` |
| Patroni | **4.0.4** | `docker exec amane-postgresql patroni --version` ; variable image `PATRONIVERSION=4.0.4` |
| Orchestrateur Go | binaire compilé avec **Go 1.26.10** (`go.mod` demande `go 1.26.5`, Dockerfile `golang:1.26-alpine`) | `strings /usr/local/bin/orchestrator \| grep -E '^go1\.'` dans le conteneur — `artifacts-a03/version-go.txt` |
| ss-node (Rust) | **ss-node 0.1.0**, compilé avec **rustc 1.88.0** (commit `/rustc/6b00bc3880198600130e1cf62b8f8a93494488cc`, tag 1.88.0) | `docker exec ss-node ss-node --version` ; `strings` du binaire + vérification du hash du tag Rust — `artifacts-a03/version-ssnode.txt`, `version-rust2.txt` |
| Listmonk | **v6.2.0**, revision `ef0a75872463f10a4848af6c547d1c057405453a`, image `org.opencontainers.image.version=6.2.0` générée le 2026-06-26 | `docker logs amane-application` + labels de l'image — `artifacts-a03/logs-tail.txt`, `images-config.txt` |
| HAProxy | **3.0.30-47d806b** | `docker logs amane-haproxy` |
| pgAdmin | **8.14** (`APP_RELEASE=8`, `APP_REVISION=14`, commit `9e58717`, 2024-12-09) | `docker exec pgadmin grep -E '^APP_(RELEASE|REVISION)' /pgadmin4/version.py` — `artifacts-a03/version-pgadmin.txt` |
| Bibliothèques notables de `ss-node` | axum **0.7.9**, tokio **1.52.3**, sqlx **0.8.6**, reqwest **0.12.28** | chemins `/usr/local/cargo/registry/src/…` dans le binaire — `artifacts-a03/version-rust.txt` |

## 5. Ressources de l'hôte et part consommée par la pile

Sources : `artifacts-a03/host-ressources.txt`, `consommation-pile.txt`, `cgroup-memoire.txt`, `calcul-part-pile.txt`, `docker-system-df-v.txt`.

| Mesure | Valeur |
|---|---|
| Hôte | `Linux GENTLEMAN31 7.2.11-arch1-1`, x86_64 |
| RAM | **38 GiB** totale (25 GiB utilisés, 676 MiB libres, 12 GiB cache, 13 GiB disponibles) ; swap 18 GiB (11 GiB utilisés) |
| CPU | **8 cœurs** (`nproc`) |
| Disque `/` | 206 G, **164 G utilisés, 33 G libres, 84 %** (`/var/lib/docker` sur le même système de fichiers) |
| Docker | 18 images (23,1 Go dont 6,155 Go récupérables), 25 conteneurs (18,53 Mo), 27 volumes (4,755 Go dont 4,221 Go non attachés), build cache 0 |
| RAM pile (6 services, cgroup `memory.current`) | **192 589 824 octets = 183,7 MiB = 0,472 % de la RAM hôte** : etcd 71,4 MiB ; PostgreSQL 56,1 ; orchestrateur 33,0 ; ss-node 17,1 ; HAProxy 6,0. Application : 0 (en redémarrage) |
| RAM pile + pgadmin | 222 216 192 octets = 211,9 MiB = 0,545 % (pgadmin 28,3 MiB) |
| CPU instantané (`docker stats --no-stream`) | etcd 7,12 %, orchestrateur 0,04 %, pgadmin 0,03 %, les autres ≈ 0 % ; **échantillon instantané, pas une moyenne** |
| Disque pile | images de la pile ≈ **3,63 Go** (etcd 273 Mo, PostgreSQL 2,25 Go, orchestrateur 38,1 Mo, HAProxy 46,9 Mo, nœud 221 Mo, listmonk 49,7 Mo, pgadmin 751 Mo) + volumes `amane-pme_*` **425,9 Mo** |
| Contexte | 3 VM du banc (§ 6) réservent 3 × 4 GiB = **12 GiB** et 3 × 2 vCPU = **6 vCPU** |
| Limites | aucune limite mémoire/CPU/PID déclarée dans les conteneurs de la pile ; `docker stats` affiche `limite = 38,95 GiB` (= RAM hôte) |

*Ressources hôte relevées (`pilot-shot term`).*

*Consommation instantanée des conteneurs actifs (`pilot-shot term`).*

## 6. Machines — banc de VM réelles (complément « machines »)

Le banc A-00 est **toujours démarré** au relevé. Comptage robuste (piège `pgrep` consigné dans `AGENTS.md`) : `ps -eo comm=,args= | awk '$1 ~ /^qemu-system/ && index($0,"amane-bench")' | wc -l` → **3**. Voir `artifacts-a03/vm-et-ports-hote.txt`.

| VM | RAM / vCPU | Ports hôte | État HTTP au relevé |
|---|---|---|---|
| `amane-bench-m1` (`10.77.0.11`) | 4096 Mo / 2 | SSH `127.0.0.1:2211`, app `127.0.0.1:9011` | `2211` ouvert, `9011` → **200** |
| `amane-bench-m2` (`10.77.0.12`) | 4096 Mo / 2 | SSH `127.0.0.1:2212`, app `127.0.0.1:9012` | `2212` ouvert, `9012` → **200** |
| `amane-bench-m3` (`10.77.0.13`) | 4096 Mo / 2 | SSH `127.0.0.1:2213`, app `127.0.0.1:9013` | `2213` ouvert, `9013` → **200** |

Réseau interne des VM : `mcast=230.0.0.1:7777` (socket, `localaddr=127.0.0.1`) + NAT `user` pour les ports hôte ; LAN `10.77.0.11-13`. Ports éditeur documentés `3000`, `5173`, `8000` : **fermés** sur l'hôte (aucun conteneur correspondant).

## 7. Écarts avec la documentation

| # | Ce que le dépôt prétend | Ce que la machine montre | Portée |
|---:|---|---|---|
| 1 | `docs/STATE.md` (17:15) : application en `Restarting (1)`, etcd `unhealthy`, port 9001 muet | Confirmé et aggravé à 18:24 : **5230** redémarrages (~1/min), etcd `unhealthy` avec perte de quorum Raft, 9001 **et 8008** en `Connection reset`, 2379 en timeout. HAProxy : `no server available!`, Patroni : `waiting on etcd` | La pile en service est **hors de service fonctionnellement**, pas seulement « partiellement dégradée » |
| 2 | `README.md` §2 et `AGENTS.md` énumèrent 6 services + pgadmin | 8 conteneurs dans le projet `amane-pme` : s'ajoute **`amane-init`, `Exited (1)` depuis 4 jours** (attendu `restart: "no"`, mais son code de sortie 1 n'est consigné nulle part) | Inventaire documenté incomplet ; à ne pas confondre avec un service actif |
| 3 | `ACCES-INTERFACES.md` §2 présente 9001, 5050, 5432, 8008, 2379/2380 comme les ports de la machine PME | Les ports sont bien **publiés** (`docker ps`), mais seuls 5050 répond (302). 9001 et 8008 réinitialisent la connexion ; 2379 expire ; 5432 n'a pas été testé en authentification | La doc décrit la configuration attendue, pas l'état de service : à préciser |
| 4 | `CLAUDE.md` §« Stack locale » : `docker compose up -d` → Django `:8000`, React `:5173`, Patroni `:8008/:8009`, PG `:5432/:5433`, Grafana `:3000` | Aucun conteneur Django, React, Prometheus ou Grafana n'existe ; ports **3000, 5173, 8000 fermés**. Seule la pile PME et des projets tiers sont présents | La stack de développement décrite n'est pas déployée ici ; la doc mélange deux bancs |
| 5 | `ADR-004`/`ADR-006`, `README.md` §3 : quorum de trois membres etcd, bascule automatique, HAProxy mène toujours au primaire | Un seul membre etcd local, pair `100.64.0.7:2380` injoignable (`no route to host`), term 450, élections en boucle ; HAProxy n'a **aucun serveur disponible** | Garantie HA non effective sur la pile en service (le banc VM A-00, lui, est sain : 3 membres, leader Patroni) |
| 6 | `ADR-005` décision 3 / `README.md` : images servies par le registre de l'éditeur et épinglées par digest | Les 8 conteneurs de la pile sont effectivement épinglés par `@sha256:…` et `RepoDigests` concordent ; **mais `amane-registry` est `Exited (2)`** et `media/node-images/pile.json` n'existe pas dans le dépôt | Épinglage respecté ; manifeste de publication et registre absents de la machine (images présentes en cache local uniquement) |
| 7 | `AGENTS.md` : la pile en service contient `amane-application`, `amane-etcd`, … ; `docs/tests/BANC-VM.md` ne mentionne que ces services | **17 autres conteneurs** (Cassandra ×5, Hadoop ×8, mailpit, registry, test-pg, listmonk-test) et **11 réseaux / 27 volumes** cohabitent sur le même démon, dont 4,221 Go de volumes non attachés | L'inventaire « projet » ignorait l'environnement réel ; à prendre en compte pour toute mesure de ressources |
| 8 | `scripts/publier-pile.sh` : sources `spilo-17:4.0-p3`, `etcd:v3.5.5`, `haproxy:3.0-alpine`, `pgadmin4:8` | Patroni **4.0.4** / PostgreSQL **17.5**, etcd **3.5.5**, HAProxy **3.0.30**, pgAdmin **8.14** : cohérents. Mais `docker system df -v` date l'image PostgreSQL de **15 mois** et l'image etcd de **4 ans** | Pas d'écart de numéro de version ; ancienneté des images locale à signaler avant campagne |
| 9 | `spike/node/Dockerfile` : `FROM rust:1.88-bookworm` ; `orchestrator-go/go.mod` : `go 1.26.5` | `ss-node` compilé avec **rustc 1.88.0** (conforme) ; orchestrateur compilé avec **Go 1.26.10**, patch supérieur au `go 1.26.5` déclaré (compatible, mais non identique) | Toolchain Go réelle à enregistrer si une comparaison de build est prévue |
| 10 | `CLAUDE.md` §« Règles non négociables » / ADR-008 : confinement de l'application sur réseau interne ; comptes non privilégiés pour l'orchestrateur | Confinement **conforme** (`internal=true`, seuls `ss-node` et HAProxy y sont attachés). En revanche etcd et `ss-node` tournent en **root** avec `CapEff=0xa80425fb`, et l'application en root ; seuls orchestrateur (UID 10001) et HAProxy (UID 99) sont non privilégiés | À ne pas présenter comme une violation : le dépôt n'impose pas l'utilisateur du nœud, mais l'écart de durcissement est réel |
| 11 | `templates/tenants/docker-compose.yml.tmpl` : `user: "0"` pour `amane-init` uniquement, le reste sans `user:` explicite | `amane-init` est bien `User=0` ; aucun des six services déclare de limites mémoire/CPU/PID | Conforme au gabarit ; absence de limites à documenter comme choix de configuration, pas comme oubli de mesure |

## 8. Captures et preuves

- `artifacts-a03/` : 37 fichiers de preuves brutes (36 `.txt` et 1 `.json` : ps, digests, réseaux, volumes, inspect, stats, cgroups, versions, logs, HTTP, VM).
- Captures terminal : [`captures/a03-conteneurs.png`](captures/a03-conteneurs.png), [`captures/a03-reseaux-volumes.png`](captures/a03-reseaux-volumes.png), [`captures/a03-hote.png`](captures/a03-hote.png), [`captures/a03-consommation.png`](captures/a03-consommation.png), [`captures/a03-interfaces-indisponibles.png`](captures/a03-interfaces-indisponibles.png).
- Capture web : [`captures/a03-pgadmin.png`](captures/a03-pgadmin.png) (interface pgAdmin joignable).

*Conteneurs de la pile et autres conteneurs au relevé.*

*Réseaux et volumes Docker au relevé.*

*Interface pgAdmin accessible sur `127.0.0.1:5050`.*

*Interfaces `:9001` (nœud) et `:8008` (Patroni) injoignables au relevé.*

Commandes de reproduction (extraits) :

```bash
docker ps -a --format '{{.Names}}\t{{.Image}}\t{{.Status}}\t{{.Ports}}'
docker image inspect --format '{{index .RepoDigests 0}}' "$(docker inspect --format '{{.Config.Image}}' amane-etcd)"
docker network ls && docker volume ls
docker network inspect amane-pme_amane amane-pme_amane-application
docker inspect --format '{{.State.Health.Status}}' amane-etcd
docker inspect --format 'user={{.Config.User}} mem={{.HostConfig.Memory}} capAdd={{.HostConfig.CapAdd}} capDrop={{.HostConfig.CapDrop}}' amane-orchestrateur
docker exec amane-etcd etcd --version
docker exec amane-postgresql postgres --version && docker exec amane-postgresql patroni --version
docker exec ss-node ss-node --version
docker logs --tail 30 amane-application
free -h && nproc && df -h / && docker system df
```

## 9. Limites de l'inventaire

- Les valeurs de santé, redémarrages et consommation sont des **instantanés** à 18:23–18:32 UTC ; le compteur de redémarrages continue d'augmenter.
- La sonde de santé de l'application n'a pas pu être examinée : le conteneur est en boucle de redémarrage, `docker top` et l'inspection des capabilities effectives sont refusés par le démon.
- Aucune mesure de latence réseau, de débit de réplication ni de failover n'a été faite : la tâche demandait un inventaire, pas un test de bascule.
- Les autres projets (Cassandra, Hadoop, chatbot-bi, maree, tp-deploiement-ia) n'ont été ni inventoriés en profondeur ni modifiés : seuls leurs conteneurs, réseaux et volumes sont comptés.
