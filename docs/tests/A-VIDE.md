# A-04 — AMANE à vide : mesures réelles sur 10 minutes (2e exécution)

- **Fenêtre** : 2026-11-25 **19:20:30 → 19:30:30 UTC**, 61 points espacés de 10 s, soit **600 s** (fin réelle d'exécution 19:30:32).
- **Commandes** : `python3 tools/a-vide-mesure.py` puis `python3 tools/a-vide-analyse.py` (scripts conservés dans `tools/`).
- **Périmètre** : les 7 conteneurs de la pile (`amane-etcd`, `amane-postgresql`, `amane-orchestrateur`, `amane-haproxy`, `ss-node`, `amane-application`, `pgadmin`) et l'hôte.
- **Aucune modification de la pile**, aucun build, aucun test de charge, aucun redémarrage provoqué pendant la fenêtre. Les captures d'écran ont été prises avant/après la fenêtre, pas pendant.
- **Données brutes** : [`a-vide/stats.csv`](a-vide/stats.csv) (424 lignes = 61 × 7 − 3 relevés manquants pour `amane-application`, en redémarrage), [`a-vide/hote.csv`](a-vide/hote.csv) (61 lignes), [`a-vide/meta.txt`](a-vide/meta.txt). Synthèses : [`resume-conteneurs.csv`](a-vide/resume-conteneurs.csv), [`resume-hote.txt`](a-vide/resume-hote.txt), [`evolution.txt`](a-vide/evolution.txt), [`analyse-stdout.txt`](a-vide/analyse-stdout.txt), [`sous-fenetres.txt`](a-vide/sous-fenetres.txt).
- **Historique** : l'exécution précédente (18:53:10 → 19:03:10 UTC, 194,43 MiB en moyenne) est conservée intacte dans [`artifacts-a04-precedent/`](artifacts-a04-precedent/) ; celle-ci en est une reprise demandée, avec des conditions hôtes dégradées (voir §7).

> **Méthode** : `docker stats --no-stream` donne des compteurs **réseau/bloc cumulatifs depuis le démarrage du conteneur** ; les débits ci-dessous sont donc des deltas entre le premier et le dernier point, divisés par 600 s. Le CPU % est la valeur instantanée renvoyée à chaque relevé. Côté hôte : `/proc/loadavg`, `/proc/meminfo` (`MemAvailable`), `/proc/diskstats` (disques NVMe entiers seulement, pour éviter le double comptage des partitions ; secteurs de 512 octets) et `sensors -u`. `iostat` n'est pas installé sur cette machine.

*État des conteneurs avant la mesure (`pilot-shot term`).*

## 1. État initial — important pour interpréter la référence

| Constat | Preuve | Effet sur la mesure |
|---|---|---|
| `amane-etcd` `unhealthy` | Healthcheck `etcdctl endpoint health` toutes les 5 s ; `FailingStreak` = **39 396** après la fenêtre (`a-vide/evenements-docker-fenetre.txt`) | Le « à vide » inclut un healthcheck qui échoue en permanence : ce n'est **pas** un régime nominal. |
| `amane-application` en boucle de redémarrage | `RestartCount` 5286 → **5296** pendant la fenêtre (`a-vide/restart-avant.txt`, `restart-apres.txt`) ; 3 relevés sur 61 n'ont pas de ligne pour ce conteneur | 0 octet, 0 PID à chaque point : l'application métier ne tourne **pas**. La référence mesure l'infrastructure seule. |
| `ss-node` ne sert pas son interface | État `Up 4 days`, aucun trafic applicatif dans les compteurs | Cohérent avec « à vide ». |
| 3 VM QEMU `amane-bench` allumées | 3 × 4096 Mo, laissées en service (A-00), non sollicitées par ce worker | L'hôte n'est pas à vide : charge, RAM et I/O disque incluent ces VM. |
| **Activité externe pendant la fenêtre** | Commit git `edb485a` à **19:29:43 UTC** (autre acteur) ; 235 événements Docker dans la fenêtre, dont la pile `a06-*` (`a06-postgres`, `a06-listmonk`, `a06-mailpit`, `a06-odoo`) et des conteneurs jetables (`wizardly_mcnulty`, `hardcore_fermi`, `cranky_ptolemy`, `amazing_mendel`) — voir `a-vide/evenements-docker-fenetre.txt` | **Contamination de la mesure** : pics de charge, de température et d'I/O hôte, et saut de mémoire des conteneurs de la pile à partir de 19:25:30. Aucune commande de ce worker n'a provoqué cette activité. |

*Interface pgadmin avant la mesure (`pilot-shot web`, seule interface joignable : `:5050`).*

## 2. Consommation par conteneur — min / moyenne / max (fenêtre complète)

| Conteneur | Points | CPU % (min / moy / max) | Mémoire MiB (min / moy / max) | PIDs moy. |
|---|---:|---:|---:|---:|
| `amane-etcd` | 61 | 0,21 / **1,91** / 15,66 | 51,66 / **61,26** / 79,24 | 14,3 |
| `amane-postgresql` | 61 | 0,00 / **0,51** / 11,62 | 22,36 / **44,48** / 68,08 | 10,6 |
| `amane-orchestrateur` | 61 | 0,00 / **0,05** / 0,19 | 27,66 / **27,88** / 28,73 | 13,0 |
| `amane-haproxy` | 61 | 0,00 / **0,00** / 0,05 | 5,72 / **5,75** / 5,98 | 9,0 |
| `ss-node` | 61 | 0,00 / **0,00** / 0,00 | 3,93 / **5,33** / 7,26 | 10,0 |
| `pgadmin` | 61 | 0,01 / **0,04** / 1,05 | 80,91 / **83,09** / 86,22 | 16,3 |
| `amane-application` | 58 | 0,00 / **0,00** / 0,00 | 0,00 / **0,00** / 0,00 | 0,0 |
| **Total pile** | 61 | 0,26 / **2,52** / 15,74 | 192,59 / **227,80** / 273,42 | 73,2 |

*Synthèse min/moyenne/max (`pilot-shot term`).*

**Débits moyens sur la fenêtre** (delta des compteurs cumulatifs, en ko/s) :

| Conteneur | Réseau entrant | Réseau sortant | Bloc lecture | Bloc écriture |
|---|---:|---:|---:|---:|
| `amane-etcd` | 0,333 | 0,000 | 8,500 | 0,000 |
| `amane-postgresql` | 0,000 | 0,000 | 41,667 | 0,000 |
| `amane-haproxy` | 0,000 | 0,000 | 1,833 | 0,000 |
| `ss-node` | 0,000 | 0,000 | 1,667 | 0,000 |
| `amane-orchestrateur` | 0,017 | 0,017 | 0,000 | 0,000 |
| `pgadmin` | 0,083 | 15,500 | 0,000 | 0,000 |
| `amane-application` | 0,000 | 0,000 | 0,000 | 0,000 |

Contrairement à l'exécution précédente, des lectures bloc apparaissent (~53,7 ko/s cumulés, ~32 Mo sur la fenêtre) et `pgadmin` émet ~15,5 ko/s. **Aucune écriture bloc imputable à la pile**. Ces lectures corrélées avec l'activité externe sont à mettre sur le compte de la pression mémoire/page cache de l'hôte, pas d'une activité métier AMANE.

## 3. Évolution sur la fenêtre (moyennes par bucket de 6 relevés)

| Bucket UTC | Mémoire pile MiB | CPU pile % | CPU etcd % | Charge 1 min | RAM dispo. GiB | Disque lecture Mo/s | Disque écriture Mo/s | Package °C |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| 19:20* | 194,9 | 2,98 | 2,58 | 6,10 | 13,72 | 2,23 | 1,29 | 60,3 |
| 19:21 | 196,1 | 0,69 | 0,42 | 5,35 | 13,71 | 1,80 | 1,22 | 60,3 |
| 19:22 | 195,0 | 3,62 | 3,20 | 4,26 | 13,70 | 1,93 | 1,27 | 59,8 |
| 19:23 | 197,1 | 2,56 | 2,02 | 3,18 | 13,70 | 1,75 | 1,32 | 60,7 |
| 19:24 | 195,0 | 3,21 | 0,74 | 2,65 | 13,65 | 2,44 | 3,09 | 62,7 |
| 19:25 | 233,7 | 2,28 | 1,98 | 5,50 | 13,17 | 10,25 | 20,13 | 71,0 |
| 19:26 | 265,8 | 1,72 | 1,42 | 6,86 | 13,56 | 5,18 | 51,43 | 68,5 |
| 19:27 | 264,0 | 2,74 | 2,25 | 5,61 | 13,58 | 3,08 | 1,40 | 62,2 |
| 19:28 | 265,7 | 3,62 | 3,16 | 3,33 | 13,53 | 5,28 | 2,20 | 61,3 |
| 19:29 | 264,8 | 2,07 | 1,62 | 2,74 | 13,43 | 2,49 | 1,36 | 62,7 |
| 19:30† | 263,0 | 0,78 | 0,49 | 2,52 | 13,35 | 0,00 | 0,00 | 60,0 |

\* premier bucket de 6 points, 19:20:30 → 19:21:20 ; † dernier bucket incomplet (un seul point, 19:30:30), les débits disque y sont nuls par construction.

*Évolution par minute (`pilot-shot term`).*

Lecture : la mémoire de la pile est **plate autour de 195 MiB jusqu'à 19:24**, puis fait un saut durable à ~265 MiB à partir de 19:26. Le saut se répartit entre `amane-postgresql` (+43 MiB, 9 → 12 PID), `amane-etcd` (+14 MiB), `pgadmin` (+4 MiB, 15 → 18 PID) et `ss-node` (+3 MiB). La cause précise n'est pas établie (aucune connexion externe à `amane-postgresql` visible dans les événements Docker), mais la corrélation temporelle avec le déploiement `a06-*` est forte. Les pics CPU sont `amane-etcd` **15,66 % (19:22:40)**, 13,12 % (19:21:20), 10,44 % (19:24:10) et `amane-postgresql` **11,62 % (19:25:20)** — compatibles avec le healthcheck en échec et les tâches de fond, pas avec une requête applicative.

## 4. Hôte pendant la fenêtre

| Mesure | Min | Moyenne | Max |
|---|---:|---:|---:|
| Charge 1 min | 2,25 | 4,52 | 8,09 |
| Charge 5 min | 3,47 | 4,17 | 4,97 |
| Charge 15 min | 3,03 | 3,39 | 3,76 |
| RAM disponible | 12,45 GiB | 13,57 GiB | 13,74 GiB |
| Température package (coretemp) | 59,0 °C | 62,9 °C | 76,0 °C |
| Température cœur max | 59,0 °C | 62,85 °C | 77,0 °C |
| NVMe `nvme0n1` (composite) | 46,85 °C | 49,70 °C | 51,85 °C |
| Sonde châssis `dell_smm` | 59,0 °C | 61,89 °C | 72,0 °C |

- Disque physique : **3,720 Mo/s lus** et **7,819 Mo/s écrits** en moyenne sur 600 s (≈ 2,23 GB lus, 4,69 GB écrits), occupation moyenne **8,27 %** des deux NVMe.
- 8 cœurs logiques ; la charge 1 min moyenne de 4,52 est ~56 % de la capacité CPU de la machine, alors que la pile ne consomme que 2,52 % d'un cœur (0,32 % des 8 cœurs). Le reste vient de l'hôte : 3 VM QEMU, la pile `a06-*` déployée pendant la fenêtre, Firefox/qBittorrent et les autres conteneurs.
- `ps` juste après la fenêtre : 3 QEMU à ~34–42 % CPU chacun, un `odoo` python à ~57 %. **Ces chiffres hôte ne sont pas un plancher AMANE.**

## 5. Sous-fenêtres (analyse post-hoc, pas un protocole distinct)

| Sous-fenêtre | Points / durée | Pile mémoire MiB (min / moy / max) | Pile CPU % (moy) | Charge 1 min (moy) | RAM dispo. GiB (moy) | Package °C (moy / max) | Disque lu / écrit Mo/s |
|---|---|---:|---:|---:|---:|---:|---:|
| Avant le saut, 19:20:30 → 19:25:20 | 30 / 290 s | 192,59 / **195,63** / 199,65 | 2,61 | 4,31 | 13,70 | 60,8 / 71,0 | 1,966 / 1,561 |
| Après le saut, 19:25:30 → 19:30:30 | 31 / 300 s | 204,61 / **258,94** / 273,42 | 2,43 | 4,73 | 13,45 | 65,0 / 76,0 | 5,393 / 13,940 |

Détail par conteneur dans [`a-vide/sous-fenetres.txt`](a-vide/sous-fenetres.txt). La première sous-fenêtre recoupe l'exécution précédente (195,63 MiB contre 194,43 MiB) : **le plancher mémoire crédible de la pile est d'environ 195 MiB (0,19 GiB)**. La seconde ne doit pas être utilisée comme référence « à vide ».

*Résumé hôte et sous-fenêtres (`pilot-shot term`).*

## 6. Empreinte disque des données

`docker system df` et `docker system df -v` → [`a-vide/system-df.txt`](a-vide/system-df.txt), [`a-vide/system-df-v.txt`](a-vide/system-df-v.txt).

| Type | Total | Actifs | Taille | Récupérable |
|---|---:|---:|---:|---:|
| Images | 18 | 12 | 23,1 Go | 6,155 Go (26 %) |
| Conteneurs | 25 | 7 | 18,53 Mo | 18,21 Mo (98 %) |
| Volumes locaux | 27 | 19 | 4,755 Go | 4,221 Go (88 %) |
| Cache de build | 0 | 0 | 0 B | 0 B |

Volumes de la pile (`amane-pme_*`), d'après `docker system df -v` : **425 940 156 octets ≈ 425,94 Mo = 406,21 MiB** — `pg-data` 277,7 Mo, `etcd-data` 128,9 Mo, `amane-bin` 19,16 Mo, `pgadmin-data` 170 ko, le reste < 2 ko. La donnée utile de la pile tient dans moins d'un demi-gigaoctet ; les 4,755 Go de volumes Docker sont presque entièrement dus à d'autres projets. `/var/lib/docker/volumes` n'est pas accessible à l'utilisateur `jose` : la taille est relevée via l'API Docker, pas par `du`.

*Empreinte disque Docker (`pilot-shot term`).*

*Volumes de la pile (`pilot-shot term`).*

## 7. Ce qui consomme sans rien faire, et pourquoi

1. **`pgadmin` est le premier poste mémoire** : 83,09 MiB en moyenne sur la fenêtre (80,95 MiB sur la sous-fenêtre calme), soit ~41 % du plancher de 195 MiB, pour 0,04–0,06 % de CPU. C'est un outil d'administration web (Python/Flask) résident, sans rôle dans le fonctionnement autonome du cluster.
2. **`amane-etcd` est le premier poste CPU** : 1,91 % en moyenne mais jusqu'à 15,66 % sur un point. Le healthcheck `etcdctl endpoint health` échoue 39 396 fois de suite (toutes les 5 s, timeout 3 s) : chaque tentative consomme un processus et du CPU, et Raft continue de ticker (heartbeats, quorum) sans aucune écriture métier. Le chiffre mesuré est un **plancher dégradé**, pas un plancher sain.
3. **`amane-orchestrateur` (27,88 MiB)** garde sessions et heartbeat à 2 s vers etcd ; **`amane-postgresql` (22,68 MiB sur la sous-fenêtre calme)** paie checkpointer/autovacuum/WAL et ses pics à 11,62 % ; **`amane-haproxy` (5,75 MiB)** et **`ss-node` (3,94 MiB)** sont des boucles événementielles quasi inactives.
4. **`amane-application` ne consomme rien** parce qu'elle n'est pas démarrée : 10 redémarrages pendant la fenêtre, 0 PID et 0 octet aux relevés où elle existe, absente de 3 relevés. Ce n'est pas une économie : c'est un service en échec, indisponible pour un test de charge.
5. **Réseau/disque** : hors contention externe, la pile échange quelques centaines d'octets par seconde et n'écrit aucun bloc. Le coût à vide est presque entièrement de la mémoire résidente et du CPU de supervision, pas de l'I/O.

## 8. Ce que cela implique pour une PME de 2 ou 3 serveurs

- **Le coût fixe de la pile est marginal quand la mesure est propre** : ~195 MiB (0,19 GiB) et 1 à 3 % d'un cœur, soit < 0,4 % des 8 cœurs de l'hôte de mesure. Répliqué sur 3 serveurs, le surcoût d'etcd + Patroni + HAProxy + nœud reste très inférieur à celui du système d'exploitation et des applications métier. **Le dimensionnement doit être dicté par la charge de pointe, pas par ce plancher.**
- **Ne pas utiliser la fenêtre complète de cette exécution comme référence de charge** : la mémoire passe de 195 à 259 MiB en moyenne et l'hôte monte à 76 °C à cause d'un déploiement `a06-*` concurrent. Utiliser la sous-fenêtre calme (§5) ou l'exécution archivée (18:53 → 19:03), en conservant la mention « plancher dégradé ».
- **Deux défauts à corriger avant la campagne de charge** : (a) healthcheck etcd en échec permanent, qui gonfle la CPU et ses pics ; (b) application en boucle de redémarrage. Tant que (b) n'est pas réglé, un test de charge mesurerait l'infrastructure seule et non le parcours applicatif.
- **Retirer pgadmin du déploiement PME** ferait gagner ~80 MiB par cluster (~41 % du plancher) sans perte de fonction : c'est l'optimisation la plus rentable, avant tout réglage de JVM/Go.
- **Le quorum coûte peu, mais exige de la propreté opérationnelle** : avec 2 serveurs + témoin ou 3 serveurs, le surcoût Raft/Patroni est de quelques pourcents d'un cœur ; un healthcheck qui échoue en boucle le transforme en bruit CPU et en logs. La surveillance doit alerter sur l'état du healthcheck, quasiment pas sur la consommation à vide.
- **Isoler le banc de mesure** : cet hôte porte 3 VM de 4 Go, 25 conteneurs Docker au total (7 de la pile) et une session de travail active. La charge hôte (4,52), l'I/O (3,7 / 7,8 Mo/s) et la température (62,9 °C) ne sont pas représentatives d'une PME. Pour les tests suivants, utiliser le banc de VM dédié (`docs/tests/BANC-VM.md`) ou, à défaut, journaliser `docker events` et `git log` pendant la fenêtre pour détecter et qualifier toute activité parasite.

## Preuves

- `docs/tests/a-vide/stats.csv`, `docs/tests/a-vide/hote.csv`, `docs/tests/a-vide/meta.txt`, `docs/tests/a-vide/mesure-stdout.txt`
- `docs/tests/a-vide/resume-conteneurs.csv`, `docs/tests/a-vide/resume-hote.txt`, `docs/tests/a-vide/evolution.txt`, `docs/tests/a-vide/analyse-stdout.txt`, `docs/tests/a-vide/sous-fenetres.txt`
- `docs/tests/a-vide/system-df.txt`, `docs/tests/a-vide/system-df-v.txt`, `docs/tests/a-vide/etat-conteneurs.txt`, `docs/tests/a-vide/sensors-initial.txt`, `docs/tests/a-vide/date-debut.txt`, `docs/tests/a-vide/restart-avant.txt`, `docs/tests/a-vide/restart-apres.txt`, `docs/tests/a-vide/evenements-docker-fenetre.txt`
- Exécution précédente archivée : `docs/tests/artifacts-a04-precedent/`
- Captures : `docs/tests/captures/a04-r2-{conteneurs,stats-apercu,system-df,pgadmin,volumes,resume,evolution}.png`
- Scripts reproductibles : `tools/a-vide-mesure.py`, `tools/a-vide-analyse.py`

```json
{"task": "A-04", "verdict_self": "pass", "claims": [{"id": "c1", "what": "séries brutes 61 points × 7 conteneurs (3 relevés manquants)", "check": "wc -l < docs/tests/a-vide/stats.csv", "expect": "425"}, {"id": "c2", "what": "série hôte 61 points", "check": "wc -l < docs/tests/a-vide/hote.csv", "expect": "62"}, {"id": "c3", "what": "fenêtre de 600 s", "check": "python3 -c \"import csv;r=list(csv.DictReader(open('docs/tests/a-vide/hote.csv')));print(int(r[-1]['epoch'])-int(r[0]['epoch']))\"", "expect": "600"}, {"id": "c4", "what": "mémoire moyenne de la pile sur la fenêtre complète", "check": "grep -F '192.59 / 227.80 / 273.42' docs/tests/a-vide/analyse-stdout.txt", "expect": "mémoire MiB min/moy/max : 192.59 / 227.80 / 273.42"}, {"id": "c5", "what": "plancher mémoire sur la sous-fenêtre avant contention", "check": "grep -F '192.59/195.63/199.65' docs/tests/a-vide/sous-fenetres.txt", "expect": "PILE mem 192.59/195.63/199.65 MiB"}, {"id": "c6", "what": "captures visuelles produites", "check": "ls docs/tests/captures/a04-r2-*.png | wc -l", "expect": "7"}]}
```
