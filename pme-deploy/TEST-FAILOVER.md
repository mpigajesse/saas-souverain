# Test de failover — cluster à 3 nœuds

Procédure de test de la bascule primaire → standby sur le banc à **3 nœuds**, et
de la reconstruction sans divergence. Complète `COMPTES-ET-TESTS.md`, qui couvre
la réplication et les invariants métier mais pas la promotion.

> ### ⚠️ À lire avant de lancer quoi que ce soit
>
> La bascule automatique est désormais soumise à une **garde de quorum**
> (`spike/crates/ss-consensus/src/quorum.rs`). Elle ne s'arme que si
> `CLUSTER_PEERS` est renseigné sur chaque nœud — **sans cette variable, aucune
> promotion automatique n'a lieu**, quel que soit le nombre de machines.
>
> Voir l'étape 0 bis. Le test A vérifie que la garde tient ; le test B reste la
> procédure de bascule maîtrisée. Ce qui subsiste comme limite est décrit en fin
> de document.

---

## Repères du banc

| | U7 | Fedora | DEATHSTROKE |
| :--- | :--- | :--- | :--- |
| Adresse réelle | `10.174.40.45` | `10.174.40.77` | `10.174.40.57` |
| Adresse annoncée au portail | `10.174.40.45` | `10.174.40.77` | `192.168.145.1` ⚠️ |
| Hôte | Windows 11 25H2 — `cmd` admin | Linux — `bash` | Windows — `cmd` admin |
| Rôle au départ | **primaire** | standby | standby |
| Interface métier | http://10.174.40.45:9001 | http://10.174.40.77:9001 | http://10.174.40.57:9001 |
| pgAdmin | http://10.174.40.45:5050 | http://10.174.40.77:5050 | http://10.174.40.57:5050 |
| `failover_count` | 55 | 0 | 55 |

Console éditeur : http://10.174.40.37:5173 — les rôles remontent dans `/devices`.

### ⚠️ Corriger l'adresse annoncée par DEATHSTROKE

La machine est bien sur le LAN en **`10.174.40.57`** (vérifié : `:9001` répond
`303`, `:5050` répond `302`, TCP `5432` ouvert). Mais elle s'est **annoncée** au
portail sous `192.168.145.1`, une adresse d'adaptateur VMware host-only.

Cause : la machine étant sous Windows, c'est `templates/tenants/install-windows.bat.tmpl:69`
qui a choisi l'adresse. Il retient l'IPv4 dont le préfixe est le plus long
(`Sort-Object PrefixLength | Select-Object -Last 1`) et n'exclut que `127.*` et
`169.*`. Or un adaptateur VMware host-only est en /24, exactement comme le LAN :
le départage est alors arbitraire, et c'est l'adaptateur virtuel qui l'a emporté.
`_resoudre_adresse` (`devices/api_register.py:89`) respecte ensuite toute IP
réelle annoncée, et l'a enregistrée telle quelle.

Conséquence : la réplication WAL fonctionne (elle part du standby **vers** le
primaire, l'adresse annoncée n'y joue aucun rôle), mais la console éditeur, la
découverte de pairs et l'accès à son interface pointent dans le vide. Dans son
`.env` :

```
NODE_ADDR=10.174.40.57:9001
```

puis `docker compose up -d` pour réannoncer. Vérifier ensuite dans `/devices` de
la console éditeur que l'adresse a changé.

> Le ping ICMP vers `10.174.40.57` échoue : c'est un filtrage pare-feu, pas un
> défaut de joignabilité. Tester avec les ports (`curl :9001`, TCP 5432), pas
> avec `ping`.

Les adresses sont celles du réseau `YaPasGiga` (bail DHCP d'1 h, préfixe qui
change à chaque réattache du hotspot). **Si le réseau a changé depuis, tout ce
document est à réadresser** — relever les IP avant de commencer.

---

## Étape 0 — Vérifier l'état de départ

Le test n'a de sens que si l'on part d'**un** primaire et de **deux** standbys.
Sur **chaque** machine :

```
docker exec pg-node psql -U metier -d metier -tAc "SELECT pg_is_in_recovery()"
```

| Attendu | U7 | Fedora | DEATHSTROKE |
| :--- | :--- | :--- | :--- |
| `pg_is_in_recovery()` | `f` | `t` | `t` |

Puis, depuis le primaire, confirmer que les **deux** standbys sont bien attachés :

```
docker exec pg-node psql -U metier -d metier -c "SELECT client_addr, state, sync_state FROM pg_stat_replication"
```

Attendu : **deux lignes**, `state = streaming`. `sync_state` vaut `async` sur les
deux — c'est normal, aucun `synchronous_standby_names` n'est configuré dans
`pme-deploy/`, donc le primaire n'attend aucun standby pour valider une écriture.

Si une seule ligne apparaît, un des standbys ne réplique pas : le corriger avant
de continuer (voir Dépannage). Si deux machines répondent `f`, c'est déjà un
split-brain : reconstruire d'abord (étape 5).

Contrôle sans ligne de commande : ouvrir `:9001/login` sur chaque standby. Il doit
afficher **« Nœud secondaire »** et un bouton vers le primaire, pas un formulaire
de connexion. Ce bouton n'est rendu que si le récepteur WAL est réellement
connecté (`pg_stat_wal_receiver`) — c'est donc aussi une preuve que la réplication
est active.

### Capacité WAL — à vérifier à 3 nœuds

`pme-deploy/docker-compose.yml` porte désormais `max_wal_senders=5` et
`wal_keep_size=512` (Mo), au lieu de 3 et 64. Avec les anciennes valeurs, deux
standbys en streaming ne laissaient **qu'un seul** sender libre : un
`pg_basebackup` de reconstruction consommait le dernier et un second échouait.

Ces paramètres sont passés en ligne de commande à PostgreSQL, donc ils ne
prennent effet qu'après **recréation** du conteneur — un simple `restart` ne
suffit pas :

```bash
docker compose up -d --force-recreate postgres
docker exec pg-node psql -U metier -d metier -tAc "SHOW max_wal_senders"
```

→ doit répondre `5`.

---

## Étape 0 bis — Déclarer la composition du cluster

C'est ce qui **arme** la bascule automatique. Chaque nœud doit connaître la
liste des **autres** nœuds, dans son `.env` :

| Machine | `CLUSTER_PEERS` |
| :--- | :--- |
| U7 (`10.174.40.45`) | `10.174.40.77,10.174.40.57` |
| Fedora (`10.174.40.77`) | `10.174.40.45,10.174.40.57` |
| DEATHSTROKE (`10.174.40.57`) | `10.174.40.45,10.174.40.77` |

Le port PostgreSQL (5432) est ajouté automatiquement s'il est omis.

> ⚠️ **Lister aussi les nœuds éteints.** Cette liste donne la taille du cluster,
> donc le seuil de majorité. En oublier un abaisse le seuil et rouvre exactement
> le risque de split-brain que la garde doit fermer.

`NODE_ADDR` doit également être correct sur chaque machine : c'est lui qui
identifie le nœud dans l'élection (voir l'avertissement sur DEATHSTROKE plus
haut). Sans lui, le nœud refuse toute promotion automatique.

Appliquer puis vérifier :

```bash
docker compose up -d
docker logs ss-node | grep Quorum
```

Attendu sur un standby correctement configuré :

```
  Quorum   : 10.174.40.77:5432 (moi) + 2 pair(s) déclaré(s) — bascule automatique armée
```

Les deux autres réponses possibles signalent une bascule automatique **inactive** :

```
  Quorum   : 2 nœud(s) déclaré(s) — sous le seuil de 3, bascule MANUELLE uniquement (`ss-node failover`)
  Quorum   : NODE_ADDR absent — bascule automatique désactivée, promouvoir avec `ss-node failover`
```

---

## Étape 1 — Poser un repère métier

Sur le **primaire** (U7), se connecter à http://10.174.40.45:9001 (identifiant
`jean`) et créer un article reconnaissable :

- Nom : `TEST-FAILOVER-3N`
- Référence : `REF-FAILOVER-3N`
- Quantité : `42`

Vérifier qu'il a traversé sur **les deux** standbys :

```
docker exec pg-node psql -U metier -d metier -c "SELECT nom, reference, quantite FROM articles ORDER BY created_at DESC LIMIT 3"
```

L'article doit apparaître en moins d'une seconde, sans aucune action. C'est la
preuve « métier » de la réplication — plus parlante que les vues système.

---

## Test A — Bascule automatique sous garde de quorum

Objectif : vérifier qu'à l'arrêt du primaire, **un seul** des deux standbys se
promeut, et que l'autre refuse explicitement.

Ouvrir les logs des deux standbys, chacun dans sa fenêtre :

```
docker logs -f ss-node
```

Puis, sur le **primaire** :

```
docker stop pg-node
```

Noter l'heure. Les deux standbys détectent la coupure de la même façon, à ~15 s
(3 ticks de 5 s, `WAL_FAILOVER_THRESHOLD`) :

```
  [tick N] ALERTE : WAL receiver inactif (1/3) — primaire injoignable ?
  [tick N+1] ALERTE : WAL receiver inactif (2/3) — primaire injoignable ?
  [tick N+2] ALERTE : WAL receiver inactif (3/3) — primaire injoignable ?
```

À partir du 3e tick, chacun sonde les autres nœuds et applique la garde. Les
deux issues sont **complémentaires** — c'est tout l'objet du test.

Sur le standby dont le WAL est le plus avancé :

```
  [tick N+2] Quorum atteint (2/3 nœuds) — FAILOVER AUTOMATIQUE
  [tick N+2] Époque 3 -> 4 (fencing posé)
  [tick N+2] ✓ FAILOVER : nœud promu en primaire !
```

Sur l'autre :

```
  [tick N+2] promotion refusée — le standby 10.174.40.77:5432 est plus à jour — il assurera la promotion
```

Le message de refus n'est journalisé qu'**une fois** tant que le motif ne change
pas : la boucle réévalue toutes les 5 s, mais ne réécrit la ligne que si la
situation évolue.

### Vérifications

| Contrôle | Attendu |
| :--- | :--- |
| `SELECT pg_is_in_recovery()` sur les 3 nœuds | exactement **un** `f` |
| `/devices` sur la console éditeur | un `primary`, deux `standby` |
| `grep epoch /opt/pme-node/node-data/config.toml` sur le promu | valeur **incrémentée** |
| `grep epoch ...` sur le standby resté passif | valeur **inchangée** |

Le contrôle de l'époque est le plus important : c'est lui qui distingue cette
version de la précédente, où la promotion automatique laissait le cluster sans
fencing.

### Variante — partition plutôt qu'arrêt

Couper le réseau du primaire au lieu d'arrêter son conteneur (sur le primaire,
`docker network disconnect pme-deploy_pme-net pg-node`). Les deux standbys se
voient toujours mutuellement : le résultat doit être identique.

Puis couper un standby du reste (`docker network disconnect` sur ce standby) et
arrêter le primaire. Le standby isolé doit refuser :

```
  [tick N] promotion refusée — pas de quorum : 1/3 nœud(s) joignable(s), 2 requis — ce nœud est probablement isolé du cluster
```

C'est le cas qui justifie la garde : sans elle, un nœud isolé se promouvait et
divergait.

### Ensuite

Le cluster a un nouveau primaire et deux nœuds à rattacher : enchaîner sur B5
puis l'étape 5. Un standby non promu ne peut pas suivre le nouveau primaire —
sa timeline a divergé.

---

## Test B — Failover contrôlé à 3 nœuds (procédure sûre)

Le principe : **désigner le successeur à l'avance** et neutraliser la supervision
de l'autre standby avant de couper le primaire.

La garde de quorum du test A choisit le successeur toute seule, sur le critère
« WAL le plus avancé ». Le test B sert quand c'est **l'opérateur** qui veut
choisir — bascule planifiée, maintenance, ou successeur imposé pour une autre
raison que la fraîcheur des données.

### B1 — Neutraliser le standby non-successeur

Successeur retenu : **Fedora**. Sur **DEATHSTROKE** :

```bash
docker stop ss-node
```

Seul `ss-node` est arrêté : `pg-node` continue de répliquer, donc DEATHSTROKE
reste à jour et ne peut plus se promouvoir (la boucle de supervision ne tourne
plus). C'est le point clé de la procédure.

### B2 — Simuler la panne du primaire

Sur **U7** :

```
docker stop pg-node
```

Noter l'heure : point de départ de la mesure du temps de bascule.

### B3 — Promouvoir Fedora

Deux voies. Préférer la **manuelle**, qui seule incrémente l'époque :

```bash
docker exec ss-node ss-node failover
```

La commande enchaîne (`spike/node/src/cmd/failover.rs`) :

1. contrôle que le nœud est bien en `recovery` — refuse s'il est déjà primaire
2. `pg_promote()`
3. incrémente l'**époque** et la sauve dans `config.toml` — c'est le *fencing* de
   l'ancien primaire, qui l'empêchera d'écrire s'il revient
4. annonce au relais
5. revérifie que la DEK s'ouvre toujours après promotion

Sorties attendues :

```
[failover] This node is a standby. Attempting promotion...
[failover] PostgreSQL promoted to primary successfully.
[failover] Incrementing epoch N -> N+1
[failover] Relay unreachable: ... (non-blocking)
[failover] DEK integrity: OK
[failover] This node is now PRIMARY at epoch N+1.
```

`Relay unreachable` est **normal** tant que le relais zero-knowledge (port 8080)
n'est pas démarré — l'annonce est non bloquante.

Si l'on laisse au contraire la promotion automatique se déclencher (15 s), elle
fonctionne mais **n'incrémente pas l'époque** : `run.rs:330` appelle
`promote_standby()` sans toucher à `config.epoch`. Le nouveau primaire garde
l'époque de l'ancien, et le *fencing* décrit à l'étape 5 ne protège plus rien.
C'est la raison de préférer la voie manuelle.

### B4 — Vérifier la bascule

| Contrôle | Où | Attendu |
| :--- | :--- | :--- |
| `SELECT pg_is_in_recovery()` | Fedora | `f` |
| `SELECT pg_is_in_recovery()` | DEATHSTROKE | `t` (toujours standby) |
| Page `:9001/login` | Fedora | le **formulaire de connexion** remplace « Nœud secondaire », **sans redémarrage** |
| Connexion `jean` | Fedora | fonctionne — la session peut enfin s'écrire |
| Article `TEST-FAILOVER-3N` | Fedora | présent, quantité 42 |
| `/devices` | console éditeur | Fedora passe en `primary` tout seul |

Le changement de page est le meilleur indicateur du **temps de bascule vu du
métier** : `pg_is_in_recovery()` est relu à chaque requête HTTP, aucun
redémarrage n'est nécessaire.

La re-déclaration dans `/devices` est automatique : `run.rs` compare le rôle
PostgreSQL réel au `NODE_MODE` déclaré et corrige l'annonce au portail.

### B5 — Rattacher DEATHSTROKE au nouveau primaire

C'est l'étape **propre au cluster à 3 nœuds**, et elle est contre-intuitive :
DEATHSTROKE était à jour, mais Fedora a changé de **timeline** en se promouvant.
Un standby ne peut pas suivre un primaire dont la timeline a divergé de la sienne.

Modifier `PRIMARY_IP` dans le `.env` **ne suffit pas** : `primary_conninfo` est
écrit dans `postgresql.auto.conf` par le `pg_basebackup -R` d'origine, et
`entrypoint.sh:8` ne reclone que si le répertoire est **vide**. Il faut donc
reconstruire — même procédure qu'à l'étape 5, avec `PRIMARY_IP=10.174.40.77`.

Ne pas se contenter de redémarrer `ss-node` sur DEATHSTROKE : il repartirait en
standby d'un primaire qui n'existe plus, WAL receiver inactif, et se promouvrait
tout seul au bout de 15 s.

---

## Étape 5 — Reconstruire un nœud en standby

> ⚠️ **L'étape qui a produit un split-brain la dernière fois.**
> Ne pas redémarrer l'ancien primaire tel quel : son `.env` porte encore
> `NODE_ROLE=primary` et son `PGDATA` est celui d'un ancien primaire. Il
> repartirait en primaire, et le banc aurait de nouveau **deux primaires
> divergents**.

À appliquer sur **chaque** nœud à remettre en standby — après le test A, c'est
deux machines sur trois.

```bash
cd /opt/elbaraa-pme            # U7 : %PROGRAMDATA%\ElBaraaConsult\PME
docker compose stop
docker volume ls --format "{{.Name}}"
docker volume rm elbaraa-pme_pg-data
```

Adapter le nom au préfixe de projet affiché par la commande précédente.
**Seul `pg-data` doit être supprimé.** Ne jamais utiliser `docker compose down -v` :
cela emporterait aussi `node-data`, qui contient `config.toml`, la clé X25519 du
nœud, sa DEK scellée et le journal. `NodeConfig::load` ne recrée rien — le nœud
redémarrerait en boucle sur « Impossible de lire la config », et l'installeur ne
lance `ss-node init --first` que sur le primaire.

Éditer le `.env` :

```
NODE_ROLE=standby
NODE_MODE=passive
PRIMARY_IP=10.174.40.77
PG_REPLICATION_SUBNET=10.174.40.0/24
```

Puis :

```bash
docker compose up -d
docker compose logs -f postgres
```

Attendu dans les logs :

```
[standby] Repertoire vide - pg_basebackup depuis 10.174.40.77:5432
```

**Reconstruire les deux nœuds l'un après l'autre**, pas en parallèle :
`max_wal_senders=3` ne laisse pas la marge pour deux clones simultanés en plus
des standbys déjà attachés (voir étape 0).

**Ce qui a été écrit sur un nœud après le dernier envoi WAL est perdu** :
`pg_basebackup` repart de zéro depuis le nouveau primaire. Exporter avant si
nécessaire :

```bash
docker exec pg-node psql -U metier -d metier -c "\copy (SELECT * FROM articles) TO '/tmp/articles.csv' CSV HEADER"
docker cp pg-node:/tmp/articles.csv ~/articles.csv
```

---

## Vérification finale

Le cluster est sain quand le primaire voit **deux** standbys :

```
# sur Fedora (nouveau primaire)
docker exec pg-node psql -U metier -d metier -c "SELECT client_addr, state, sync_state FROM pg_stat_replication"
```

→ deux lignes, `10.174.40.45` et l'IP corrigée de DEATHSTROKE, `state = streaming`.

```
# sur chaque standby
docker exec pg-node psql -U metier -d metier -tAc "SELECT pg_is_in_recovery()"
```

→ `t`, et `:9001/login` affiche « Nœud secondaire » avec un bouton pointant vers
`http://10.174.40.77:9001/login`.

Dans `/devices` de la console éditeur : un `primary`, deux `standby`.

Pour revenir à la configuration d'origine (U7 primaire), rejouer le test B en
inversant les machines.

---

## Dépannage

| Symptôme | Cause probable | Action |
| :--- | :--- | :--- |
| Les **deux** standbys passent `primary` après `docker stop pg-node` | `CLUSTER_PEERS` incomplet : chaque nœud se croit dans un cluster plus petit et le seuil de majorité tombe | `docker logs ss-node \| grep Quorum` sur chaque machine ; compléter l'étape 0 bis, puis reconstruire via l'étape 5 |
| **Aucun** standby ne se promeut | garde non armée : `CLUSTER_PEERS` vide, `NODE_ADDR` absent, ou moins de 3 nœuds déclarés | la ligne `Quorum :` au démarrage donne le motif exact ; à défaut, promouvoir avec `ss-node failover` |
| `promotion refusée — un primaire est vivant sur ...` | le WAL receiver local est tombé mais le primaire répond : c'est le lien de réplication qui est cassé, pas le primaire | vérifier le réseau entre ce standby et le primaire, `pg_hba.conf`, et `REPLICATION_PASSWORD` |
| `[standby] Attente du primaire...` en boucle | 5432 fermé sur le primaire, `pg_hba.conf` n'autorise pas le sous-réseau, ou `REPLICATION_PASSWORD` divergent | sur le primaire : `docker exec pg-node grep replication /var/lib/postgresql/data/pg_hba.conf` |
| `dependency failed to start: container pg-node is unhealthy` sur Fedora | SELinux (`enforcing` par défaut) bloque les montages : les scripts n'ont pas de suffixe `:z` et `/opt/pme-node` n'est pas en `container_file_t` | `sudo semanage fcontext -a -t container_file_t "/opt/pme-node(/.*)?"` puis `sudo restorecon -Rv /opt/pme-node` ; ajouter `,z` aux montages `./postgres/*` ; confirmer d'abord avec `sudo setenforce 0` |
| Un nœud n'apparaît pas dans `pg_stat_replication` | son `NODE_ADDR` / son IP n'est pas sur le LAN (cas DEATHSTROKE), ou firewalld bloque 5432 | `hostname -I` sur le nœud ; sur le primaire `sudo firewall-cmd --permanent --add-port=5432/tcp && sudo firewall-cmd --reload` |
| `pg_basebackup: error: could not connect` alors que le mot de passe est bon | `max_wal_senders=3` saturé par les 2 standbys + un clone | augmenter à 5 (étape 0), ou arrêter un standby le temps du clone |
| Les trois nœuds affichent le formulaire de connexion | split-brain à 3, **ou** PostgreSQL injoignable (`pg_is_in_recovery()` échoue et retombe sur `false`) | trancher avec `SELECT pg_is_in_recovery()` sur chaque machine |
| `Noeud non initialisé` sur `ss-node status` | le volume `node-data` a été détruit (`down -v`) | le nœud a perdu son identité et sa DEK — réenrôlement nécessaire |
| `failover` répond `already a primary` | le nœud n'était pas standby | revérifier l'état de départ (étape 0) |
| Connexion impossible sur un standby | comportement voulu : créer une session est une écriture, impossible sur un réplica en lecture seule | consulter les données via pgAdmin `:5050` ou `psql` |
| Identifiant refusé | `username` ≠ nom complet — correspondance exacte sur `username`, casse comprise | sur le primaire : `docker exec pg-node psql -U metier -d metier -c "SELECT username, role, is_active FROM users"` |
| `grep n'est pas reconnu` sur U7 | `grep` n'existe pas sur Windows | utiliser `docker volume ls --format "{{.Name}}"`, ou `findstr`. Dans `docker exec pg-node grep ...`, le `grep` tourne dans le conteneur et fonctionne depuis `cmd` |

---

## Pourquoi ces précautions

La réplication est **physique** (WAL) : le standby est une copie octet à octet du
primaire, en lecture seule imposée par PostgreSQL lui-même. Deux primaires qui
acceptent des écritures divergent immédiatement et **aucune fusion n'est
possible** — d'où l'obligation de reconstruire par `pg_basebackup` plutôt que de
« rebrancher » l'ancien primaire.

L'incrément d'époque à chaque promotion sert précisément à ça : un ancien
primaire qui revient porte une époque inférieure et se voit refuser les écritures
(*fencing*), au lieu de corrompre silencieusement les données.

### Comment la garde de quorum décide

`ss_consensus::decider_failover` (`spike/crates/ss-consensus/src/quorum.rs`) est
une **fonction pure** : le nœud collecte les observations, elle tranche. Quatre
contrôles, dans cet ordre :

1. **Taille du cluster.** Sous 3 nœuds déclarés, refus — c'est la règle de
   `CLAUDE.md`, et elle vaut aussi pour le banc à 2 machines, où la bascule
   automatique était auparavant active à tort.
2. **Primaire vivant.** Si un pair joignable répond qu'il n'est pas en recovery,
   refus : c'est le lien de réplication local qui est cassé, pas le primaire.
3. **Majorité stricte.** Le nœud doit joindre `taille / 2 + 1` machines,
   lui-même compris. Un nœud isolé par une partition se voit en minorité et
   s'abstient.
4. **Élection déterministe.** Parmi les standbys joignables, le WAL le plus
   avancé l'emporte (`pg_last_wal_receive_lsn()`), départagé à égalité stricte
   par l'adresse. Les deux nœuds comparent le même couple `(lsn, adresse)` et
   désignent donc le même gagnant, sans échanger de message dédié.

Le chemin automatique incrémente et persiste ensuite l'époque dans
`config.toml`, exactement comme `ss-node failover` — c'est ce qui fence
l'ancien primaire.

### Ce qui reste hors de portée sans DCS

La garde ferme le split-brain de partition et la double promotion, mais **ce
n'est pas du consensus**. Il n'y a pas de DCS dans `pme-deploy/` (réplication
streaming nue) ; le vrai quorum Raft vit côté mission C (`orchestrator-go/`,
etcd + Patroni). Deux limites subsistent :

- **Partition asymétrique.** Si les standbys ne joignent plus le primaire mais
  que les postes clients, eux, y accèdent encore, le contrôle 2 ne voit rien et
  le successeur se promeut pendant que l'ancien primaire continue d'accepter des
  écritures. Le fencing par époque est **consultatif** : il n'y a pas de STONITH,
  rien ne coupe physiquement l'ancien primaire.
- **Fenêtre de course.** Les sondes ne sont pas atomiques. Deux nœuds qui
  évaluent à quelques centaines de millisecondes d'écart peuvent observer des
  LSN différents. L'écart doit être franc pour que l'élection soit stable — c'est
  pourquoi le départage par adresse existe : il tranche les égalités parfaites,
  pas les quasi-égalités.

Pour une bascule dont l'issue doit être certaine — maintenance planifiée,
démonstration client — utiliser le test B, qui ne dépend d'aucune de ces
conditions.
