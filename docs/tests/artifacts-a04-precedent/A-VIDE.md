# A-04 — AMANE à vide : mesures réelles sur 10 minutes

- **Fenêtre** : 2026-11-25 **18:53:10 → 19:03:10 UTC**, 61 points espacés de 10 s, soit **600 s** (fin réelle d'exécution 19:03:12).
- **Commandes** : `python3 tools/a-vide-mesure.py` puis `python3 tools/a-vide-analyse.py` (scripts conservés dans `tools/`).
- **Périmètre** : les 7 conteneurs de la pile (`amane-etcd`, `amane-postgresql`, `amane-orchestrateur`, `amane-haproxy`, `ss-node`, `amane-application`, `pgadmin`) et l'hôte.
- **Aucune modification de la pile**, aucun build, aucun test de charge, aucun redémarrage provoqué pendant la fenêtre. Les captures d'écran ont été prises avant/après la fenêtre, pas pendant.
- **Données brutes** : [`a-vide/stats.csv`](a-vide/stats.csv) (427 lignes = 61 × 7 conteneurs), [`a-vide/hote.csv`](a-vide/hote.csv) (61 lignes), [`a-vide/meta.txt`](a-vide/meta.txt). Synthèses : [`resume-conteneurs.csv`](a-vide/resume-conteneurs.csv), [`resume-hote.txt`](a-vide/resume-hote.txt), [`evolution.txt`](a-vide/evolution.txt), [`analyse-stdout.txt`](a-vide/analyse-stdout.txt).

> **Méthode** : `docker stats --no-stream` donne des compteurs **réseau/bloc cumulatifs depuis le démarrage du conteneur** ; les débits ci-dessous sont donc des deltas entre le premier et le dernier point, divisés par 600 s. Le CPU % est la valeur instantanée renvoyée à chaque relevé. Côté hôte : `/proc/loadavg`, `/proc/meminfo` (`MemAvailable`), `/proc/diskstats` (disques NVMe entiers seulement, pour éviter le double comptage des partitions ; secteurs de 512 octets) et `sensors -u`. `iostat` n'est pas installé sur cette machine.

*État des conteneurs avant la mesure (`pilot-shot term`).*

## État initial — important pour interpréter la référence

La pile n'est **pas saine** pendant la mesure (conformément à `docs/STATE.md`) :

| Constat | Preuve | Effet sur la mesure |
|---|---|---|
| `amane-etcd` `unhealthy` | Healthcheck `etcdctl endpoint health` toutes les 5 s, timeout 3 s, **39 192 échecs consécutifs** (`docker inspect`) | Le « à vide » inclut un healthcheck qui échoue en permanence : ce n'est **pas** un régime nominal. |
| `amane-application` en boucle de redémarrage | 8 événements `die` + 8 `start` **pendant la fenêtre** (`a-vide/evenements-application.txt`) ; `RestartCount` = 5270 à 19:04 | 0 octet, 0 PID à chaque point : l'application métier ne tourne **pas**. La référence mesure l'infrastructure seule. |
| `ss-node` ne sert pas son interface | `curl http://127.0.0.1:9001/amane/` → code `000` | Aucun trafic applicatif, cohérent avec « à vide ». |
| 3 VM QEMU `amane-bench` allumées | 3 × 4096 Mo, laissées en service (A-00) | L'hôte n'est pas à vide : la charge, la RAM disponible et l'I/O disque incluent ces VM. Elles n'ont pas été arrêtées (règles du dépôt). |
| Activité git externe pendant la fenêtre | `git log -1` : commit `bc6329e` à **18:56:13 UTC**, par un autre acteur ; aucun `git commit` lancé par ce worker | Activité hôte parasite, sans rapport avec la pile ; à signaler avant toute comparaison fine des pics (18:56:20, CPU etcd 6,31 %). |

*Interface pgadmin avant la mesure (`pilot-shot web`, la seule interface joignable : `:5050` → 302).*

## 1. Consommation par conteneur — min / moyenne / max

| Conteneur | Points | CPU % (min / moy / max) | Mémoire MiB (min / moy / max) | PIDs moy. |
|---|---:|---:|---:|---:|
| `amane-etcd` | 61 | 0,25 / **0,93** / 14,60 | 51,61 / **53,93** / 59,09 | 14,1 |
| `amane-postgresql` | 61 | 0,00 / **0,35** / 2,59 | 22,35 / **22,51** / 23,34 | 9,0 |
| `amane-orchestrateur` | 61 | 0,00 / **0,05** / 0,10 | 27,98 / **28,03** / 28,54 | 13,0 |
| `amane-haproxy` | 61 | 0,00 / **0,00** / 0,00 | 5,72 / **5,74** / 5,97 | 9,0 |
| `ss-node` | 61 | 0,00 / **0,02** / 0,09 | 3,93 / **4,03** / 4,68 | 10,0 |
| `pgadmin` | 61 | 0,01 / **0,03** / 0,05 | 80,19 / **80,19** / 80,44 | 12,0 |
| `amane-application` | 61 | 0,00 / **0,00** / 0,00 | 0,00 / **0,00** / 0,00 | 0,0 |
| **Total pile** | 61 | 0,34 / **1,37** / 14,69 | 191,82 / **194,43** / 200,18 | 68,1 |

*Synthèse min/moyenne/max (`pilot-shot term`).*

**Débits moyens sur la fenêtre** (delta des compteurs cumulatifs, en ko/s) :

| Conteneur | Réseau entrant | Réseau sortant | Bloc lecture | Bloc écriture |
|---|---:|---:|---:|---:|
| `amane-etcd` | 0,167 | 0,000 | 0,000 | 0,000 |
| `amane-haproxy` | 0,000 | 0,167 | 0,000 | 0,000 |
| `amane-orchestrateur` | 0,017 | 0,000 | 0,000 | 0,000 |
| `amane-postgresql` | 0,000 | 0,000 | 0,000 | 0,000 |
| `pgadmin` | 0,000 | 0,000 | 0,000 | 0,000 |
| `ss-node` | 0,000 | 0,000 | 0,000 | 0,000 |
| `amane-application` | 0,000 | 0,000 | 0,000 | 0,000 |

Autrement dit : **aucun I/O bloc imputable à la pile pendant 10 minutes**, et tout au plus ~100 kB échangés par `amane-etcd` et `amane-haproxy`. Le trafic réseau résiduel est le battement de cœur et le healthcheck, pas de l'activité métier.

## 2. Évolution sur la fenêtre (moyennes par minute)

| Minute UTC | Mémoire pile MiB | CPU pile % | CPU etcd % | Charge 1 min | RAM dispo. GiB | Disque lecture Mo/s | Disque écriture Mo/s | Package °C |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| 18:53 | 194,5 | 1,08 | 0,66 | 3,39 | 14,82 | 3,26 | 1,58 | 60,5 |
| 18:54 | 195,4 | 0,88 | 0,47 | 2,45 | 14,82 | 3,61 | 1,22 | 60,3 |
| 18:55 | 194,5 | 1,08 | 0,84 | 1,63 | 14,80 | 2,68 | 1,19 | 60,8 |
| 18:56 | 194,5 | 1,85 | 1,44 | 1,53 | 14,83 | 3,04 | 1,19 | 60,7 |
| 18:57 | 194,1 | 0,83 | 0,44 | 1,48 | 14,83 | 3,24 | 1,19 | 60,7 |
| 18:58 | 193,2 | 3,24 | 2,77 | 2,80 | 14,84 | 3,44 | 1,18 | 60,0 |
| 18:59 | 195,1 | 1,17 | 0,75 | 3,05 | 14,85 | 2,42 | 1,13 | 60,3 |
| 19:00 | 194,4 | 1,85 | 1,25 | 3,16 | 14,82 | 2,30 | 1,66 | 60,3 |
| 19:01 | 194,4 | 0,91 | 0,43 | 2,38 | 14,85 | 2,46 | 1,17 | 60,2 |
| 19:02 | 194,6 | 0,92 | 0,35 | 2,40 | 14,86 | 2,45 | 1,18 | 60,2 |
| 19:03* | 192,8 | 0,54 | 0,37 | 2,01 | 14,88 | 0,00 | 0,00 | 59,0 |

\* bucket incomplet : un seul point (19:03:10), les débits disque y sont donc nuls par construction.

*Évolution par minute (`pilot-shot term`).*

Lecture : la mémoire de la pile est **plate** (191,8 → 200,2 MiB sur toute la fenêtre, ±2 %), la température du package reste dans une bande de 4 °C (59–63 °C). Seule la CPU d'`amane-etcd` est irrégulière, avec des pics à 6,31 % (18:56:20), **14,60 % (18:58:30)** et 5,57 % (19:00:40), et des pics plus faibles côté PostgreSQL (2,59 % à 19:00:50). Ces pics périodiques sont compatibles avec le healthcheck en échec (processus `etcdctl` lancé toutes les 5 s, 3 s de timeout) et avec les tâches de fond PostgreSQL ; ils ne correspondent à aucune requête applicative.

## 3. Hôte pendant la fenêtre

| Mesure | Min | Moyenne | Max |
|---|---:|---:|---:|
| Charge 1 min | 1,31 | 2,42 | 3,74 |
| Charge 5 min | 2,32 | 2,79 | 3,56 |
| Charge 15 min | 2,84 | 3,03 | 3,35 |
| RAM disponible | 14,70 GiB | 14,83 GiB | 14,89 GiB |
| Température package (coretemp) | 59,0 °C | 60,4 °C | 63,0 °C |
| Température cœur max | 59,0 °C | 60,4 °C | 63,0 °C |
| NVMe `nvme0n1` (composite) | 49,85 °C | 50,85 °C | 52,85 °C |
| Sonde châssis `dell_smm` | 59,0 °C | 59,5 °C | 61,0 °C |

- Disque physique : **2,886 Mo/s lus** et **1,221 Mo/s écrits** en moyenne sur 600 s (≈ 2,47 GB lus, 0,73 GB écrits), occupation moyenne **6,99 %** des deux NVMe.
- 8 cœurs logiques ; la charge 1 min moyenne de 2,42 est donc d'environ 30 % de la capacité CPU de la machine, **alors que la pile ne consomme que 1,37 % d'un cœur** (0,17 % des 8 cœurs). Le reste vient de l'hôte, en particulier des 3 VM du banc et des autres projets Docker présents.
- Attribution indicative (après la fenêtre, échantillon de 10 s dans `a-vide/attribution-io.txt`) : les 3 QEMU ont écrit ~4,44 Mo sur ~37,4 Mo d'I/O hôte mesurés sur la même période, soit ~12 %. Ce n'est **pas** une attribution exhaustive, mais elle confirme que l'I/O disque de l'hôte n'est pas produit par la pile (dont les compteurs de blocs sont restés constants).

## 4. Empreinte disque des données

`docker system df` et `docker system df -v` → [`a-vide/system-df.txt`](a-vide/system-df.txt), [`a-vide/system-df-v.txt`](a-vide/system-df-v.txt).

| Type | Total | Actifs | Taille | Récupérable |
|---|---:|---:|---:|---:|
| Images | 18 | 12 | 23,1 Go | 6,155 Go (26 %) |
| Conteneurs | 25 | 7 | 18,53 Mo | 18,21 Mo (98 %) |
| Volumes locaux | 27 | 19 | 4,755 Go | 4,221 Go (88 %) |
| Cache de build | 0 | 0 | 0 B | 0 B |

Volumes de la pile (`amane-pme_*`) : **425 939 856 octets = 425,94 Mo = 406,21 MiB** (détail : `pg-data` 277,7 Mo, `etcd-data` 128,9 Mo, `amane-bin` 19,16 Mo, `pgadmin-data` 169,7 ko, le reste < 2 ko). La donnée utile de la pile tient donc dans moins d'un demi-gigaoctet ; les 4,755 Go de volumes Docker sont presque entièrement dus à d'autres projets.

*Empreinte disque Docker (`pilot-shot term`).*

## 5. Ce qui consomme sans rien faire, et pourquoi

1. **`pgadmin` est le premier poste mémoire** : 80,19 MiB, soit **41 % de la mémoire de la pile**, pour 0,03 % de CPU. C'est un outil d'administration web (Python/Flask) chargé en permanence, alors qu'aucun utilisateur ne l'utilise. Il n'a aucune fonction dans le fonctionnement autonome du cluster.
2. **`amane-etcd` est le premier poste CPU** : 0,93 % en moyenne mais jusqu'à 14,60 % sur un point. Le healthcheck `etcdctl endpoint health` échoue toutes les 5 s (timeout 3 s) depuis 39 192 fois : chaque tentative consomme un processus et du CPU, et le Raft continue de ticker (élections/heartbeats, quorum) même sans écriture. Le chiffre « à vide » mesuré ici est donc un **plancher dégradé**, pas un plancher sain.
3. **`amane-orchestrateur` (28,03 MiB)** garde ses sessions et son heartbeat à 2 s vers etcd ; **`amane-postgresql` (22,51 MiB)** paie ses processus de fond (checkpointer, autovacuum, WAL) et ses pics à 2,59 % ; **`amane-haproxy` (5,74 MiB)** et **`ss-node` (4,03 MiB)** sont des boucles événementielles quasi inactives.
4. **`amane-application` ne consomme rien** parce qu'elle n'est pas démarrée : 8 redémarrages pendant la fenêtre, 0 PID et 0 octet à chaque relevé. Ce n'est pas une économie : c'est un service en échec qui n'est pas disponible pour comparer un test de charge.
5. **Réseau/disque** : la pile échange ~100 kB et n'écrit aucun bloc sur 10 minutes. Le « coût à vide » est donc presque entièrement de la mémoire résidente et du CPU de supervision, pas de l'I/O.

## 6. Ce que cela implique pour une PME de 2 ou 3 serveurs

- **Le coût fixe de la pile est marginal** : 194,43 MiB (**0,190 GiB**, ~0,49 % des 38,95 GiB de l'hôte de mesure) et 1,37 % d'un cœur, soit 0,17 % des 8 cœurs. Répliqué sur 3 serveurs, le surcoût d'etcd + Patroni + HAProxy + nœud reste très inférieur à celui du système d'exploitation et des applications métier. **Le dimensionnement doit être dicté par la charge de pointe, pas par ce plancher.**
- **Mais ne pas comparer un test de charge à cette référence telle quelle.** Deux défauts à corriger avant la campagne : (a) le healthcheck etcd en échec permanent gonfle la CPU et ses pics (jusqu'à 14,60 %) ; (b) l'application en boucle de redémarrage n'est pas servie. Tant que (b) n'est pas réglé, un test de charge mesurerait l'infrastructure seule et non le parcours applicatif.
- **Retirer pgadmin du déploiement PME** ferait gagner ~80 MiB par cluster (~41 % de la mémoire de la pile) sans perte de fonction : c'est l'optimisation la plus rentable, avant tout réglage de JVM/Go.
- **Le quorum coûte peu, mais exige de la propreté opérationnelle** : avec 2 serveurs + témoin ou 3 serveurs, le surcoût Raft/Patroni est de quelques pourcents d'un cœur ; en revanche un healthcheck qui échoue en boucle transforme ce faible coût en bruit CPU et en logs. La surveillance doit alerter sur l'état du healthcheck, et quasiment pas sur la consommation à vide.
- **Isoler le banc de mesure** : ici l'hôte porte 3 VM de 4 Go et 17 autres conteneurs ; la charge hôte (2,42) et l'I/O disque (2,9 Mo/s en lecture) ne sont pas représentatives d'une PME. Pour les tests de charge suivants, utiliser le banc de VM dédié (`docs/tests/BANC-VM.md`) ou garantir l'absence d'autres charges, sinon les deltas seraient attribués à tort à AMANE.

## Preuves

- `docs/tests/a-vide/stats.csv`, `docs/tests/a-vide/hote.csv`, `docs/tests/a-vide/meta.txt`
- `docs/tests/a-vide/resume-conteneurs.csv`, `docs/tests/a-vide/resume-hote.txt`, `docs/tests/a-vide/evolution.txt`, `docs/tests/a-vide/analyse-stdout.txt`
- `docs/tests/a-vide/system-df.txt`, `docs/tests/a-vide/system-df-v.txt`, `docs/tests/a-vide/etat-conteneurs.txt`, `docs/tests/a-vide/sensors-initial.txt`, `docs/tests/a-vide/date-debut.txt`, `docs/tests/a-vide/evenements-application.txt`, `docs/tests/a-vide/attribution-io.txt`
- Captures : `docs/tests/captures/a04-avide-{conteneurs,stats-apercu,system-df,pgadmin,resume,evolution}.png`
- Scripts reproductibles : `tools/a-vide-mesure.py`, `tools/a-vide-analyse.py`

```json
{"task": "A-04", "verdict_self": "pass", "claims": [{"id": "c1", "what": "séries brutes 61 points × 7 conteneurs", "check": "wc -l < docs/tests/a-vide/stats.csv", "expect": "428"}, {"id": "c2", "what": "série hôte 61 points", "check": "wc -l < docs/tests/a-vide/hote.csv", "expect": "62"}, {"id": "c3", "what": "fenêtre de 600 s", "check": "python3 -c \"import csv;r=list(csv.DictReader(open('docs/tests/a-vide/hote.csv')));print(int(r[-1]['epoch'])-int(r[0]['epoch']))\"", "expect": "600"}, {"id": "c4", "what": "mémoire moyenne de la pile", "check": "grep -F '194.43' docs/tests/a-vide/analyse-stdout.txt", "expect": "mémoire MiB min/moy/max : 191.82 / 194.43 / 200.18"}, {"id": "c5", "what": "captures visuelles produites", "check": "ls docs/tests/captures/a04-avide-*.png | wc -l", "expect": "6"}]}
```
