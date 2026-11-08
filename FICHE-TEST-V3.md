# Fiche test V3 — application Newsletter hébergée par AMANE

Objectif de la séance : faire tourner une application tierce (**Newsletter**), sans la modifier, sur le cluster d'une PME orchestré par AMANE. Le parcours couvre l'ajout au catalogue, le rattachement à la licence, l'installation des machines, l'enrôlement, l'accès à l'interface de l'application par le nœud AMANE, et une sauvegarde chiffrée sur le relais.

Fiche établie le 2026-11-05, pour la V3 poussée sur `main3` (étapes 1 à 10). Elle s'appuie sur le journal `docs/refonte-v3/AMANE-Refonte-V3.md` (annexe « Validation sur le banc »).

---

## 0. Le banc du jour

Adresses relevées par `tailscale status` le 2026-11-05. **Les relever de nouveau en début de séance** : si l'adresse de l'éditeur change, `RELAY_URL` du `.env` du portail doit suivre (voir § 1).

| Machine | Adresse Tailscale | Système | Rôle dans l'essai | État |
|---|---|---|---|---|
| siradiou | `100.64.0.6` | Windows | **Éditeur** : portail (`:8000`, console `:5173`), registre (`:5000`), relais (`:8080`) | En ligne |
| jose | `100.64.0.4` | Linux | **Serveur 1** de la PME (premier installé : il génère la DEK) | En ligne |
| enorah | `100.64.0.7` | Windows | **Serveur 2** de la PME (enrôlé auprès du serveur 1) | En ligne |
| celestine | `100.64.0.5` | Windows | Témoin, si elle revient en ligne | Hors ligne |
| nael | `100.64.0.2` | Linux | Témoin de secours | Hors ligne depuis 5 jours |

Tenant utilisé : **AMANE Test** (licence de 3 postes ; le témoin n'en consomme pas). Aucune machine ne lui est rattachée à ce jour.

**Limite de cet essai à deux serveurs.** Sans témoin, deux serveurs n'ont pas de quorum etcd tolérant la perte de l'un d'eux : la **bascule automatique n'est pas testable** (ADR-004). L'espace PME affichera l'alerte « Installez un témoin sur une troisième machine » : c'est attendu. Le test de bascule (§ 8) ne se fait que si celestine ou nael revient. Le poste éditeur ne peut pas servir de témoin : ses ports 2379, 5432 et 8008 sont pris par la pile du portail.

---

## 1. Avant la séance — poste éditeur (siradiou)

```powershell
tailscale ip -4                       # doit donner 100.64.0.6 ; sinon, prévenir pour réaligner RELAY_URL
cd C:\Users\Siradiou\saas-souverain
docker compose up -d                  # portail, PostgreSQL, etcd, registre
docker start ss-relay                 # s'il est arrêté
curl http://100.64.0.6:8080/health    # doit contenir "authentification":true
curl http://100.64.0.6:5000/v2/       # registre : {}
```

Dans `.env` : `RELAY_URL=http://100.64.0.6:8080` et `RELAY_SECRET` renseigné (ne jamais le copier ailleurs). Après tout changement du `.env` : `docker compose up -d --force-recreate backend`.

**Depuis jose et enorah**, vérifier que l'éditeur est joignable (sinon : pare-feu Windows de siradiou, ports 8000, 5000, 8080 entrants sur l'interface Tailscale) :

```bash
curl -s http://100.64.0.6:8000/ -o /dev/null -w "%{http_code}\n"   # portail
curl -s http://100.64.0.6:5000/v2/                                  # registre
curl -s http://100.64.0.6:8080/health                               # relais
```

Sous Windows (enorah), même chose avec `curl.exe`.

---

## 2. Préparer l'application Newsletter — poste éditeur

### 2.1 Placer les sources

Déposer le `Dockerfile` et les sources de l'application dans le dépôt, par exemple sous **`ha-lab/application-newsletter/`**, à côté de l'application d'exemple (`ha-lab/application-exemple/`).

### 2.2 Vérifier qu'elle peut entrer dans AMANE sans modification

| Exigence | Pourquoi | À vérifier dans la Newsletter |
|---|---|---|
| Un seul conteneur, une interface HTTP sur un port fixe | Le nœud relaie ce port | Port : `____` |
| Un chemin qui répond 2xx ou 3xx | Sonde de santé du nœud | Chemin : `____` (à défaut `/`) |
| PostgreSQL lu depuis une variable d'environnement (ex. `DATABASE_URL`), tables créées par l'application au démarrage | AMANE crée la base et injecte ses identifiants | Variable : `____` |
| Ou bien : fichiers (SQLite, pièces jointes) dans un dossier déclaré en volume | Sinon perdus à chaque redémarrage | Dossier : `____` |
| **Aucun accès à Internet** pendant le fonctionnement : ni CDN, ni police distante, ni API externe | Réseau de l'application interne, sans sortie (ADR-008) | |
| Chemins relatifs, ou respect de `X-Forwarded-Host` | L'interface est servie par le nœud sur le port 9001 | |
| Pas de WebSocket | Le nœud ne les relaie pas encore | |

> **Point d'attention propre à une newsletter : l'envoi des courriels ne fonctionnera pas.** L'application n'a aucune sortie réseau, donc ni SMTP ni API d'envoi. Cet essai valide l'interface, l'enregistrement des abonnés dans PostgreSQL, le partage des données entre serveurs et la sauvegarde. Un envoi de courriels depuis un SaaS confiné est une question d'architecture à ouvrir (relais sortant contrôlé par la PME ?), pas un réglage.

### 2.3 Construire et pousser l'image

```powershell
cd C:\Users\Siradiou\saas-souverain\ha-lab\application-newsletter
docker build -t localhost:5000/editeurs/newsletter:1.0 .
docker push localhost:5000/editeurs/newsletter:1.0
docker inspect --format "{{index .RepoDigests 0}}" localhost:5000/editeurs/newsletter:1.0
#   → localhost:5000/editeurs/newsletter@sha256:<64 caractères>   (noter le digest)
docker inspect --format "{{json .Config.Entrypoint}} {{json .Config.Cmd}} {{json .Config.ExposedPorts}}" localhost:5000/editeurs/newsletter:1.0
#   → commande de démarrage et port, à reporter dans le manifeste
```

### 2.4 Essai rapide hors AMANE (facultatif, 2 minutes)

```powershell
docker run --rm -d --name newsletter-essai -p 8099:<PORT> localhost:5000/editeurs/newsletter:1.0
curl http://localhost:8099<CHEMIN_DE_SANTE>       # doit répondre 200 (ou une redirection)
docker logs newsletter-essai
docker stop newsletter-essai
```

Si l'application exige sa base pour démarrer, cet essai échouera : ce n'est pas bloquant, AMANE la lui fournira.

### 2.5 Rédiger le manifeste

L'image s'écrit **sans l'hôte du registre** (le portail le préfixe par l'adresse que verra la PME). Variante avec PostgreSQL (recommandée) :

```json
{
  "type_artefact": "docker",
  "image": "editeurs/newsletter@sha256:<DIGEST>",
  "commande": ["<commande>", "<arguments éventuels>"],
  "port": 8080,
  "sante": "/",
  "variables": {},
  "base_de_donnees": {
    "variables": {
      "DATABASE_URL": "postgres://{utilisateur}:{mot_de_passe}@{hote}:{port}/{base}"
    }
  },
  "volumes": []
}
```

Variante sans PostgreSQL (SQLite ou fichiers) : supprimer `base_de_donnees` et déclarer le dossier, par exemple `"volumes": [{"nom": "donnees", "chemin": "/app/data"}]`. Attention : un volume reste **local à chaque serveur** ; les données ne sont alors ni partagées entre les deux serveurs ni sauvegardées sur le relais. Le test du § 6.3 n'aurait plus de sens.

Règles vérifiées par le portail : `commande` est une liste non vide (reprendre `Entrypoint` + `Cmd`) ; les noms de variables sont en MAJUSCULES et ne commencent pas par `AMANE_` ; espaces réservés autorisés : `{utilisateur}`, `{mot_de_passe}`, `{hote}`, `{port}`, `{base}`.

### 2.6 Ajouter au catalogue et valider

1. Console éditeur : `http://localhost:5173/applications` (compte éditeur).
2. **Nouvelle application** : nom `Newsletter`, éditeur (le groupe), version `1.0`, coller le manifeste.
3. **Valider** : le portail vérifie que l'image existe dans le registre. Le statut passe à « validée » et le manifeste est figé. En cas de refus, le message dit quel champ corriger.

### 2.7 Rattacher à la licence d'AMANE Test

Console éditeur : `http://localhost:5173/licenses`, licence active d'AMANE Test, champ **Application** : remplacer « Compteur (exemple) » par **Newsletter 1.0**, puis enregistrer.

### 2.8 « Recompiler » les installeurs : rien à recompiler

Les installeurs et la pile sont **générés par le portail à chaque téléchargement**. L'installeur télécharge lui-même la pile (`docker-compose.yml`) au moment où il s'exécute, avec l'application rattachée à la licence à cet instant. Il suffit donc de télécharger les installeurs **après** le § 2.7.

Contrôle depuis le poste éditeur (le jeton d'enregistrement d'AMANE Test est visible dans la console, fiche du tenant) :

```powershell
curl.exe -s "http://100.64.0.6:8000/api/tenants/pme/compose/?token=<JETON_ENREGISTREMENT>&type=serveur" -o pile.yml
Select-String -Path pile.yml -Pattern "image:|RELAY_URL|RELAY_AUTH_TOKEN"
```

Attendu : un service `amane-application` en `100.64.0.6:5000/editeurs/newsletter@sha256:<DIGEST>`, toutes les images en `100.64.0.6:5000/…@sha256:`, `RELAY_URL` en `100.64.0.6:8080`, et un `RELAY_AUTH_TOKEN` de 64 caractères. Supprimer `pile.yml` ensuite : il contient le jeton du relais d'AMANE Test.

---

## 3. Télécharger les installeurs

Depuis chaque machine de la PME, dans un navigateur : `http://100.64.0.6:5173/mon-espace`, compte de la PME **AMANE Test**.

- jose (Linux) : installeur **serveur, Linux** (`install-amane-test.sh`).
- enorah (Windows) : installeur **serveur, Windows** (`install-amane-test.bat`).

Les installeurs détectent seuls l'adresse de la machine (adresse source de la route vers le portail, donc l'adresse Tailscale). Ils installent Docker si besoin, déclarent le registre `100.64.0.6:5000`, ouvrent les ports (2379, 2380, 5432, 8008, 9001, 5050), vérifient le chiffrement du disque, téléchargent la pile et la démarrent.

---

## 4. Serveur 1 — jose (`100.64.0.4`, Linux)

```bash
sudo bash install-amane-test.sh
```

L'installeur doit annoncer « Premier serveur du cluster. » et finir en affichant le mot de passe pgAdmin : le noter.

Vérifications (quelques minutes après la fin) :

```bash
sudo docker ps --format "table {{.Names}}\t{{.Status}}"
#   amane-etcd, amane-orchestrateur, amane-postgresql, amane-haproxy, ss-node, amane-application, pgadmin
sudo docker exec amane-postgresql patronictl list          # 1 membre, Leader, nommé « jose »
sudo docker logs ss-node 2>&1 | grep -A1 "Identifiant"     # identifiant et mot de passe admin de la console du nœud
sudo docker logs ss-node 2>&1 | grep "Application"         # « Application : disponible »
```

- Console du nœud : `http://100.64.0.4:9001/amane`. Se connecter en `admin` avec le mot de passe ci-dessus. La console redirige vers **la page de récupération** : choisir **« Plus tard »** pour l'instant (§ 7).
- Application : `http://100.64.0.4:9001/` doit afficher **l'interface de la Newsletter**, servie par le nœud AMANE.

---

## 5. Serveur 2 — enorah (`100.64.0.7`, Windows)

1. Clic droit sur `install-amane-test.bat`, **Exécuter en tant qu'administrateur**. Il doit annoncer « 1 serveur(s) déjà installé(s) : cette machine rejoindra le cluster ».
2. Vérifier qu'elle a rejoint le cluster (PowerShell) :

   ```powershell
   docker exec amane-postgresql patronictl list      # 2 membres : jose Leader, enorah Replica ou Sync Standby, en streaming
   docker exec amane-etcd etcdctl member list        # 2 membres started
   ```

3. **Enrôlement** : enorah n'a pas encore la DEK du cluster (elle ne peut ni sauvegarder ni ouvrir le journal).
   - Sur la console de jose : `http://100.64.0.4:9001/amane/admin/enrolement` : **générer un code** (usage unique, durée limitée).
   - Sur enorah :

     ```powershell
     docker exec ss-node ss-node enroler --depuis http://100.64.0.4:9001 --code <CODE>
     docker restart ss-node
     docker logs ss-node 2>&1 | Select-String "Journal|Application"
     ```

   Attendu : « DEK du cluster reçue et enregistrée. », puis, après redémarrage, un journal ouvert (plus de « en attente d'enrôlement »).

4. Console éditeur (`http://localhost:5173/devices`) : 2 serveurs actifs pour AMANE Test, avec l'état du disque (chiffré ou non). La licence compte 2 postes sur 3. L'espace PME affiche « Installez un témoin sur une troisième machine ».

---

## 6. Accès à l'application orchestrée par AMANE

### 6.1 L'interface par les deux serveurs

Depuis n'importe quelle machine du réseau Tailscale (y compris siradiou), dans un navigateur :

- `http://100.64.0.4:9001/` (jose) et `http://100.64.0.7:9001/` (enorah) : **même interface Newsletter**.
- `http://<serveur>:9001/amane` : console AMANE du nœud, séparée de l'application.

### 6.2 Les données sont partagées

1. Sur jose : créer un abonné (ou une campagne, une liste…).
2. Sur enorah : recharger. **L'abonné doit apparaître.** Les deux serveurs écrivent sur le même primaire PostgreSQL (par HAProxy), répliqué en synchrone sur l'autre.
3. Sur pgAdmin (`http://100.64.0.4:5050`, identifiants affichés par l'installeur) : la base `application` contient les tables de la Newsletter.

### 6.3 Le confinement tient

L'application ne doit pouvoir joindre ni Internet ni l'éditeur. Sur jose :

```bash
RESEAU=$(sudo docker network ls --format '{{.Name}}' | grep amane-application)
sudo docker run --rm --network "$RESEAU" alpine wget -qT5 -O- http://example.com && echo "FUITE" || echo "confiné : OK"
sudo docker run --rm --network "$RESEAU" alpine wget -qT5 -O- http://100.64.0.6:8000/ && echo "FUITE" || echo "confiné : OK"
```

Et l'application ne voit pas les secrets du cluster :

```bash
sudo docker inspect amane-application --format '{{range .Mounts}}{{.Destination}} {{end}}'   # pas de /amane/secrets
```

---

## 7. Sauvegarde et récupération

### 7.1 Sauvegarde chiffrée sur le relais

Sur **le primaire** (le `Leader` de `patronictl list`, a priori jose) :

```bash
sudo docker exec ss-node ss-node sauvegarder
#   Sauvegarde amane : sauvegarde-amane-<jour> (… octets chiffrés) déposée sur le relais
#   Sauvegarde application : sauvegarde-application-<jour> (…) déposée sur le relais
```

- Console éditeur, page **Relais** (`http://localhost:5173/relay`) : les nœuds d'AMANE Test apparaissent.
- Une lecture sans jeton est refusée (depuis n'importe quelle machine) :

  ```bash
  curl -s -o /dev/null -w "%{http_code}\n" http://100.64.0.6:8080/api/blobs/<TENANT_ID>/sauvegarde-application-<jour>   # 401
  ```

### 7.2 Restauration de la base de l'application

1. Supprimer un abonné dans la Newsletter.
2. Sur le primaire : `sudo docker exec ss-node ss-node restaurer --base application --jour <jour>`. Attendu : « Base application restaurée depuis la sauvegarde du <date> (jour <jour>) ».
3. Recharger l'interface : l'abonné est revenu.

### 7.3 Récupération de la DEK (facultatif dans cette séance)

Console de jose : `http://100.64.0.4:9001/amane/admin/recuperation` : passphrase du gérant (12 caractères au moins, saisie deux fois), puis **imprimer les trois feuilles** (une part chacune). Le test de perte totale (`ss-node recuperer`) est décrit dans l'annexe du journal (point 17) ; il demande de réinstaller les deux serveurs à neuf et se fera dans une séance dédiée.

---

## 8. Bascule — seulement si un troisième poste est disponible

Si celestine (`100.64.0.5`) ou nael (`100.64.0.2`) revient en ligne :

1. Espace PME : télécharger l'installeur **témoin** (Windows ou Linux) et le lancer en administrateur. `etcdctl member list` : 3 membres.
2. Couper brutalement l'alimentation du primaire. Sur l'autre serveur, `patronictl list` doit le montrer `Leader` en quelques secondes ; noter la durée.
3. L'application répond toujours par le serveur restant (`http://<serveur restant>:9001/`), et les abonnés sont intacts.
4. Rallumer : le poste revient en `Replica`, puis `Sync Standby`.

---

## 9. Grille de résultats

| # | Vérification | Résultat | Remarque |
|---|---|---|---|
| 1 | Éditeur joignable depuis jose et enorah (portail, registre, relais) | ☐ | |
| 2 | Image Newsletter poussée, digest noté | ☐ | |
| 3 | Manifeste accepté, application validée au catalogue | ☐ | |
| 4 | Newsletter rattachée à la licence ; pile servie avec `amane-application` et le jeton du relais | ☐ | |
| 5 | jose installé : 7 conteneurs, Leader, « Application : disponible » | ☐ | |
| 6 | Interface Newsletter sur `http://100.64.0.4:9001/` | ☐ | |
| 7 | enorah installée, dans le cluster (Patroni et etcd) | ☐ | |
| 8 | enorah enrôlée (DEK reçue, journal ouvert) | ☐ | |
| 9 | Interface Newsletter sur `http://100.64.0.7:9001/` | ☐ | |
| 10 | Donnée créée sur un serveur, visible sur l'autre | ☐ | |
| 11 | Application confinée (ni Internet, ni éditeur, ni secrets) | ☐ | |
| 12 | Sauvegarde déposée ; lecture sans jeton refusée | ☐ | |
| 13 | Restauration de la base `application` | ☐ | |
| 14 | Console éditeur : 2 serveurs, état du disque, alerte témoin | ☐ | |
| 15 | (Si 3e poste) bascule automatique, durée : ____ s | ☐ | |

---

## 10. En cas de problème

| Symptôme | Où regarder |
|---|---|
| L'installeur ne télécharge pas la pile (503) | Images non publiées : `media/node-images/pile.json` sur le poste éditeur |
| `pull` refusé : « http: server gave HTTP response to HTTPS client » | Registre non déclaré en `insecure-registries` (Docker Desktop : Settings, Docker Engine) |
| enorah ne rejoint pas le cluster | `docker logs amane-init` et `amane-orchestrateur` ; pare-feu Windows (2379, 2380, 5432, 8008) ; adresse annoncée dans `C:\ProgramData\Amane\PME\.env` (`NODE_IP` doit être en `100.64.0.x`) |
| « Application : INDISPONIBLE » | `docker logs amane-application` : commande ou port du manifeste, base attendue, accès Internet au démarrage |
| Page blanche ou styles manquants par le port 9001 | Chemins absolus ou ressources sur un CDN dans la Newsletter (§ 2.2) |
| Sauvegarde : 401 | Pile installée avant l'étape 10 : retélécharger la pile (§ 2.8) puis `docker compose up -d` dans le dossier d'installation |
| Sauvegarde : « DEK absente » | Machine non enrôlée (§ 5.3) |

Dossiers d'installation : `/opt/amane-pme` (Linux), `C:\ProgramData\Amane\PME` (Windows).

Consigner les résultats et les durées dans le journal `docs/refonte-v3/AMANE-Refonte-V3.md` (§ 6, nouvelle séance), puis régénérer le PDF.
