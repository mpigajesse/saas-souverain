# Réinstallation complète du parc PME

Procédure de remise à zéro des machines PME et de réinstallation ordonnée depuis
le portail éditeur. À utiliser quand les `.env`, les références d'image et les
`pg_hba.conf` des nœuds ont trop divergé pour être corrigés un par un —
typiquement après plusieurs changements d'adresse réseau.

Pour le test de bascule une fois le parc remonté, voir `TEST-FAILOVER.md`.

---

## Repères

| | Valeur |
| :--- | :--- |
| Éditeur (console) | `http://10.174.40.37:5173` |
| Éditeur (API) | `http://10.174.40.37:8000` |
| Registre Docker | `10.174.40.37:5000` |
| Sous-réseau | `10.174.40.0/24` |
| Ordre d'installation | **U7 → Fedora → 3ᵉ machine** |

| Machine | Système | Shell | Dossier d'installation |
| :--- | :--- | :--- | :--- |
| U7 | Windows | `cmd` administrateur | `%PROGRAMDATA%\ElBaraaConsult\PME` |
| Fedora | Linux | `bash` + `sudo` | `/opt/elbaraa-pme` |
| 3ᵉ machine | selon système | — | idem selon système |

> **Les adresses bougent.** Le hotspot utilisé sur ce banc redistribue un
> préfixe différent à chaque réattache au réseau opérateur (`10.128.177.0/24`,
> puis `10.156.179.0/24`, puis `10.174.40.0/24` — seul le dernier octet reste
> stable). **Relever l'IP de l'éditeur avant de commencer** et remplacer partout
> dans ce document si elle a changé :
>
> ```powershell
> ipconfig | findstr IPv4
> ```

---

## Phase 0 — Vérifier l'éditeur

Rien ne sert de lancer une installation si le portail n'est pas joignable : les
installeurs téléchargent leur `docker-compose.yml` depuis lui, et les nœuds s'y
annoncent.

```bash
docker compose ps                       # 8 services « running »
curl -s -o /dev/null -w "%{http_code}\n" http://10.174.40.37:5173/          # 200
curl -s -o /dev/null -w "%{http_code}\n" http://10.174.40.37:5173/api/auth/me/   # 401 (attendu)
curl -s http://10.174.40.37:5000/v2/ss-node/tags/list                        # {"name":"ss-node","tags":["v2"]}
```

**Vider la table des appareils**, sinon les machines réinstallées hériteront de
rôles et d'adresses fantômes :

```bash
docker exec saas_editor_backend python manage.py shell -c "from devices.models import Device; n,_ = Device.objects.all().delete(); print(n, 'lignes supprimees')"
```

> **Supprimer, jamais désactiver.** `devices/api_register.py` refuse
> l'enregistrement d'un appareil dont `is_active` est faux (HTTP 403,
> `device_disabled`) et ne le réactive pas. Passer les devices à
> `is_active=False` verrouillerait définitivement les machines ;
> `get_or_create` recrée en revanche proprement une ligne supprimée.

Vérifier :

```bash
docker exec saas_editor_backend python manage.py shell -c "from devices.models import Device; print(Device.objects.count(), 'devices')"
```

→ `0 devices`.

---

## Phase 1 — Suppression complète

À faire sur les trois machines, dans n'importe quel ordre. Le `-v` est **voulu
ici** : on repart de zéro, identité du nœud comprise.

### U7 — Windows (`cmd` administrateur)

Une commande par ligne — `cmd` n'accepte pas deux commandes collées, et
`schtasks` exige un espace avant chaque commutateur (`schtasks/delete` est lu
comme un seul mot et refusé).

```
cd /d "%PROGRAMDATA%\ElBaraaConsult\PME"
docker compose down -v
cd /d C:\
rmdir /s /q "%PROGRAMDATA%\ElBaraaConsult"
schtasks /delete /tn "ElBaraaConsultPME" /f
```

Si `schtasks` répond que la tâche est introuvable, c'est sans conséquence.

Contrôle :

```
dir "%PROGRAMDATA%\ElBaraaConsult"
docker ps -a
docker volume ls
```

Le premier doit répondre « chemin introuvable ». S'il reste des volumes :

```
docker volume rm pme_pg-data pme_node-data pme_pgadmin-data
```

### Fedora — Linux

```bash
cd /opt/elbaraa-pme
sudo docker compose down -v
sudo systemctl disable --now elbaraa-pme.service
sudo rm -f /etc/systemd/system/elbaraa-pme.service
sudo systemctl daemon-reload
sudo rm -rf /opt/elbaraa-pme
```

Contrôle :

```bash
sudo docker ps -a
sudo docker volume ls
```

### 3ᵉ machine

Rien à faire si elle n'a jamais été installée. Si une installation a échoué en
cours de route, appliquer le bloc correspondant à son système.

> Les `daemon.json` conservent les anciennes entrées dans `insecure-registries`.
> Inoffensif : l'installeur ajoute la nouvelle de façon idempotente.

---

## Phase 2 — Installation, une machine à la fois

**L'ordre est impératif.** La première machine lit `node_count = 0` et se déclare
**primaire** ; les suivantes la détectent et se déclarent **standby**. Lancées en
parallèle, elles se croiraient toutes primaires — split-brain immédiat.

**Ne pas passer à la machine suivante avant d'avoir vu la précédente « En ligne »
dans la console.**

### 1. U7 — le primaire

1. Depuis le navigateur d'U7 : `http://10.174.40.37:5173` → connexion →
   **Mon espace** → télécharger l'installeur Windows.

   **Retélécharger impérativement**, ne pas réutiliser un ancien fichier : le
   portail dérive l'hôte du registre et le `SAAS_URL` de la requête entrante.

2. Clic droit sur `install-pme-test.bat` → **Exécuter en tant qu'administrateur**.

3. L'installeur doit afficher `Aucun nœud enregistré — ce PC devient le PRIMAIRE`.
   **S'il demande l'IP d'un primaire, arrêter** : la table `devices` n'était pas
   vide.

4. Vérifier :

   ```
   docker compose ps
   docker exec ss-node printenv SAAS_URL
   ```

   `SAAS_URL` doit contenir `10.174.40.37`. Puis, dans `/devices` sur la console :
   U7 **En ligne** en moins de 2 minutes.

### 2. Fedora — premier standby

1. Télécharger l'installeur Linux depuis `http://10.174.40.37:5173` → *Mon espace*.

2. ```bash
   sudo bash install-pme-test.sh
   ```

3. Il liste U7 et propose son IP — accepter avec Entrée.

4. Suivre la synchronisation :

   ```bash
   sudo docker compose -f /opt/elbaraa-pme/docker-compose.yml logs -f postgres
   ```

   Attendu : `[standby] Repertoire vide - pg_basebackup depuis <IP U7>:5432`.

5. Vérifier :

   ```bash
   docker exec pg-node psql -U metier -d metier -tAc "SELECT pg_is_in_recovery()"
   ```

   → `t`, et Fedora **En ligne** dans la console.

### 3. La 3ᵉ machine — second standby

Même déroulé, avec l'installeur correspondant à son système. Elle verra deux
nœuds et se déclarera standby.

---

## Phase 3 — Contrôles finaux

| Contrôle | Attendu |
| :--- | :--- |
| `/devices` sur la console | 3 machines, noms distincts, toutes **En ligne** |
| Rôles | 1 `primary` (U7) + 2 `standby` |
| `SELECT pg_is_in_recovery()` | `f` sur U7, `t` sur les deux autres |
| `pg_stat_replication` sur U7 | 2 lignes en `streaming` |
| `:9001` des standbys | page **« Nœud secondaire »** + bouton vers U7 |

Le seuil « En ligne » est de **2 minutes** (`ONLINE_THRESHOLD`,
`devices/views_web.py:11`) et le battement de cœur repasse toutes les 60 s. Un
nœud qui reste hors ligne au-delà n'atteint pas le portail — vérifier son
`SAAS_URL`.

---

## Comptes de l'interface métier

Les volumes ayant été détruits, le mot de passe `admin` est **régénéré** au
premier démarrage d'U7 et affiché **une seule fois** dans les logs. Le relever
immédiatement :

```
docker logs ss-node | findstr /C:"Mot de passe"
```

```bash
sudo docker logs ss-node | grep -B4 -A2 "Mot de passe"
```

Créer les comptes employés **sur le primaire uniquement** (c'est une écriture) :

```
docker exec ss-node ss-node adduser --username jean --name "Jean Dupont" --role employee --password "Employe1!"
```

`--role` vaut `admin` ou `employee`. La requête est un *upsert* : relancée sur un
`--username` existant, elle remplace le mot de passe — c'est aussi la procédure
de réinitialisation.

Les comptes sont répliqués vers les standbys, mais **s'y connecter est
impossible** : ouvrir une session écrit une ligne dans `sessions`, interdit sur
un réplica en lecture seule. Le standby affiche une page de redirection vers le
primaire. Consulter les données d'un standby via pgAdmin `:5050` ou `psql`.

---

## Pièges connus

| Piège | Symptôme | Réponse |
| :--- | :--- | :--- |
| **`.env.txt`** | les modifications du `.env` restent sans effet | Notepad ajoute `.txt` quand le nom commence par un point. Vérifier avec `dir /a`, et éditer avec un éditeur qui préserve le nom |
| **`restart` au lieu de `up -d`** | le conteneur garde l'ancienne configuration | les valeurs d'`env_file` sont figées à la **création** du conteneur. Toujours `docker compose up -d <service>` |
| **Standby non enrôlé** | `ss-node` redémarre en boucle, `:9001` muet, logs « Noeud non enrôlé » | les installeurs ne lancent `ss-node init --first` que si `NODE_ROLE=primary`. Contournement : `docker exec ss-node ss-node init --first` puis `docker compose up -d ss-node` — le nœud génère alors sa propre DEK au lieu de recevoir celle du cluster, à corriger dans l'installeur |
| **`pg-node is unhealthy`** | `dependency postgres failed to start` après ~190 s | le standby boucle sur `[standby] Attente du primaire...`. Vérifier avec `docker logs --tail 40 pg-node` — primaire éteint, `PRIMARY_IP` périmé, ou `pg_hba.conf` du primaire n'autorisant pas le sous-réseau |
| **`pg_hba.conf` figé** | réplication refusée après changement de réseau | le fichier est écrit **une seule fois**, à l'initialisation. Sur un `pg-data` conservé, ajouter la ligne à la main puis `SELECT pg_reload_conf()` |
| **`grep` sur Windows** | `'grep' n'est pas reconnu` | utiliser `findstr`, ou `docker volume ls --format "{{.Name}}"`. Dans `docker exec pg-node grep …`, le `grep` tourne dans le conteneur et fonctionne depuis `cmd` |
| **`docker-compose.yml` parasite** | `service "backend" is not running` | un `docker-compose.yml` traîne dans `C:\Users\<user>`. Se placer dans le projet, ou viser le conteneur par son nom (`docker exec saas_editor_backend …`) |
| **Ordre d'installation non respecté** | plusieurs primaires | réinstaller en séquence, en attendant « En ligne » entre chaque machine |
