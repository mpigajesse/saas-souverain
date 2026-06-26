# Runbook de Démonstration — Soutenance

**Framework SaaS Souverain** · Jesse MPIGA-ODOUMBA · AL BARAA CONSULTING · 01/07/2026
Durée cible : **~10 min** (slide 10 du support). Objectif : montrer que **déployer est facile**, puis expliquer **réplication & résilience**.

---

## Scénario narratif

> Deux PME illustrent le cycle de vie :
> - **PME nouvelle — « Yasmine Argan »** (fondée par Yasmine El Idrissi, jeune entrepreneuse marocaine — coopérative d'huile d'argan) : peu de moyens, **une seule machine (Debian)**. Sert à montrer l'inscription + l'installation en quelques minutes.
> - **PME existante — « MPJ »** : cluster **2 nœuds** déjà en place (Kali primaire + Ubuntu standby). Sert à expliquer la **réplication** et la **résilience à la panne**.

Message au jury : *« Une PME démarre avec une machine ; quand elle grandit, elle ajoute un nœud et obtient la réplication automatique — la même commande d'installation s'en charge. »*

---

## ACTE 0 — Préparation AVANT le jury (checklist)

> ⚠️ À faire **avant** d'entrer, jamais en direct. Une démo fluide se prépare.

### 🔌 Réseau — à valider en premier

**Chaque PME a son propre LAN** (réaliste). Yasmine Argan vit sur **`192.168.10.0/24`** (`ens37` = 192.168.10.128) ; MPJ sur `192.168.200.0/24`. Le SaaS éditeur, lui, doit être **joignable depuis chaque LAN PME** — comme un SaaS public en production.

L'hôte VMware a une patte virtuelle sur le VMnet `192.168.10.0/24` (typiquement **`192.168.10.1`**). C'est par cette IP que Yasmine Argan atteint le SaaS et le registre — **sans quitter son réseau**.

- [ ] **Trouver l'IP de l'hôte sur ce LAN** (depuis la Debian) :
  ```bash
  ip route | grep default          # passerelle du LAN PME
  ping -c1 192.168.10.1            # l'hôte VMware répond-il ?
  curl -I http://192.168.10.1:8000 # le SaaS répond-il ?  (sinon tester .2)
  ```
  → noter cette IP comme **`SAAS_IP`** (192.168.10.1 attendu).
- [ ] **Django accepte cette IP** : dans le `.env` du SaaS, `ALLOWED_HOSTS` doit inclure `192.168.10.1` (le plus simple pour la démo : `ALLOWED_HOSTS=*`). Si POST refusé (CSRF), ajouter `CSRF_TRUSTED_ORIGINS=http://192.168.10.1:8000`. Redémarrer le SaaS.
- [ ] **Registre joignable** : `curl http://192.168.10.1:5000/v2/_catalog` depuis la Debian doit répondre (le registre écoute sur 0.0.0.0:5000).
- [ ] `ens33` (192.168.1.57, Internet) reste actif — utile pour installer Docker.

> 🛠️ **AVANT LA SOUTENANCE — reconstruire et republier l'image (à NE PAS oublier).**
> Le tag `ss-node:dev` est mutable : si l'image du registre n'a pas été reconstruite après une modif du cœur Rust, la PME tournera un **ancien binaire** (c'est exactement ce qui faisait disparaître l'encart d'identifiants au login). Depuis l'hôte Windows, dans `E:\SaaS souverain\spike\` :
> ```powershell
> powershell -ExecutionPolicy Bypass -File build-node.ps1 --no-cache
> ```
> Cela reconstruit `192.168.200.1:5000/ss-node:dev` **et** le pousse. Vérifier ensuite côté registre PME : `curl http://192.168.10.1:5000/v2/ss-node/tags/list`. L'installateur fait désormais `docker compose pull ss-node` au démarrage, donc la PME prendra automatiquement cette image fraîche.

### Préparation Docker (sur la Debian)

- [ ] **Installer Docker à l'avance** (via Internet ens33) : `curl -fsSL https://get.docker.com | sh`.
- [ ] Autoriser le registre éditeur : `/etc/docker/daemon.json` → `{"insecure-registries":["192.168.10.1:5000"]}` puis `sudo systemctl restart docker`.
- [ ] **Pré-tirer l'image** : `docker pull 192.168.10.1:5000/ss-node:dev`.
- [ ] **Aucun stack PME installé** : `/opt/elbaraa-pme` absent (sinon `docker compose down` + purge avant).

### SaaS & cluster existant

- [ ] **SaaS éditeur** joignable depuis la Debian : `http://192.168.10.1:8000` (Tableau de bord visible).
- [ ] **Cluster MPJ** en marche : Kali (.128) + Ubuntu (.130) → Clusters « ✓ Cluster sain ».
- [ ] **Onglets navigateur** ouverts : (1) inscription SaaS, (2) Parc machines, (3) Clusters.
- [ ] **Plan B** : captures d'écran dans `tests/captures-GUIDE-TEST-METIER-MPJ/` si le réseau flanche.
- [ ] Terminaux lisibles : **police agrandie** (Ctrl+ +), fond clair si projecteur pâle.

> ℹ️ `SAAS_IP` = IP de l'hôte sur le LAN de la PME (attendu 192.168.10.1). Remplacer `192.168.10.1`
> par la valeur réelle si `.1` ne répond pas (en NAT VMware, tester `.2`).

---

## ACTE 1 — Création du compte (SaaS éditeur) · ~1 min 30

**Écran : navigateur sur le SaaS.**

1. Aller sur la page d'inscription : `http://192.168.200.1:8000/tenants/inscription/`
2. Remplir le formulaire :
   | Champ | Valeur démo |
   |---|---|
   | Nom de l'entreprise | `Yasmine Argan` |
   | Email | `contact@yasmine-argan.ma` |
   | Mot de passe | `Yasmine2026!` |
   | Téléphone | `+212 6 00 00 00 00` |
   | Nombre d'employés | `1` |
3. Valider → **automatiquement** : compte créé + **licence d'essai 30 jours (Starter, 1 poste)** + connexion + page **Bienvenue**.

> **Narration :** *« En moins d'une minute, la PME a un compte, une licence d'essai, et — vous allez le voir — son installateur prêt à télécharger. L'éditeur, lui, ne verra jamais les données métier : il ne gère que le compte et la licence. »*

---

## ACTE 2 — Installation sur la machine PME (Debian) · ~2 min 30

**Écran : page Bienvenue (SaaS), puis terminal Debian.**

1. **Depuis le navigateur de la Debian**, ouvrir le SaaS via `http://192.168.10.1:8000` (important : c'est cette URL — l'IP de l'hôte sur le LAN de la PME — qui sera injectée comme `SAAS_URL` et registre dans le script). Sur la page **Bienvenue** → section installation → **Linux** → télécharger `install-yasmine-argan.sh`.
   *(le script embarque déjà le token, l'URL du SaaS = 192.168.10.1:8000, le registre 192.168.10.1:5000, l'URL du relais et l'image)*
2. Transférer/ouvrir le script sur la **Debian**, puis :
   ```bash
   sudo bash install-yasmine-argan.sh
   ```
3. **Commenter les 5 étapes** qui défilent :
   | Étape | Ce que fait l'installeur | À dire |
   |---|---|---|
   | [1/5] | Détecte l'IP réseau | « il se configure seul » |
   | [2/5] | Installe Docker s'il manque + autorise le registre éditeur | « aucune compétence Docker requise côté PME » |
   | [3/5] | **Détecte le rôle** : aucun nœud → **PRIMAIRE** | « première machine = primaire, automatiquement » |
   | [4/5] | Dérive les mots de passe du token, génère `.env` + `docker-compose.yml` | « tout est pré-configuré » |
   | [5/5] | `docker compose up -d` + `ss-node init --first` (génère la **DEK**) | « la clé de chiffrement naît ici, chez la PME — jamais chez l'éditeur » |

4. À la fin, l'installeur affiche :
   ```
   Installation terminée ! (Yasmine Argan — primary)
   Interface : http://<ip-debian>:9001
   pgAdmin   : http://<ip-debian>:5050
   ```

> **Narration :** *« Une seule commande. Docker, PostgreSQL, le logiciel métier, le service au démarrage : tout est installé et configuré. La clé de chiffrement (la DEK) est générée sur place, dans le périmètre de la PME. »*

---

## ACTE 3 — Le logiciel métier fonctionne · ~1 min 30

**Écran : navigateur sur `http://<ip-debian>:9001`.**

1. Récupérer le mot de passe admin du premier démarrage :
   ```bash
   docker compose -f /opt/elbaraa-pme/docker-compose.yml logs ss-node | grep -A5 "PREMIER DÉMARRAGE"
   ```
2. Se connecter à l'interface → créer **1 article** (ex. `ARG-50` / `Huile d'argan 50 ml` / unité / seuil 20) + **1 entrée** de stock (qté 200).
3. Revenir sur le SaaS (`http://192.168.10.1:8000`) :
   - **Parc machines** : la machine `Yasmine Argan` apparaît (1 poste, primaire `192.168.10.128`, en ligne).
   - **Clusters** : « **⚠ Bascule manuelle** » — *une seule machine, pas de redondance*.

> **Narration :** *« Le logiciel tourne, les données sont là — et restent là. Mais avec une seule machine, pas de redondance : le SaaS le signale honnêtement. C'est exactement la situation d'une PME qui démarre avec peu de moyens. Voyons ce qui se passe quand elle ajoute une deuxième machine. »*

---

## ACTE 4 — Réplication & résilience (PME MPJ, 2 nœuds) · ~3 min 30

**Écran : SaaS Clusters + 2 terminaux (Kali primaire, Ubuntu standby).**

> Transition : *« La PME MPJ, elle, a deux machines. Voici la réplication en action. »*

1. **SaaS Clusters → MPJ** : « ✓ Cluster sain · Réplication streaming → ».
2. **Créer les deux employés (manuel, depuis l'admin MPJ)** — sur **Kali** (primaire), connecté en **admin** sur `http://192.168.200.128:9001`, menu **Utilisateurs → « + Nouvel utilisateur »**. Créer ces deux comptes :

   | Identifiant | Nom complet | Rôle | Mot de passe |
   |---|---|---|---|
   | `alice.martin` | Alice Martin | Employé | `Employe1!` |
   | `bob.dupont` | Bob Dupont | Employé | `Employe2!` |

   → **Réplication des comptes** : sur **Ubuntu** (standby), sans aucune action, vérifier qu'ils sont déjà là :
   ```bash
   docker exec pg-node psql -U metier -d metier -c \
     "SELECT username, role FROM users ORDER BY username;"
   ```
   → `admin`, `alice.martin`, `bob.dupont` — identiques sur le standby. *Même les comptes employés sont répliqués instantanément.*
3. **Réplication métier live** — sur **Kali**, se reconnecter en **Alice** (`alice.martin` / `Employe1!`) → créer **1 article** + **1 entrée** de stock ; puis en **Bob** (`bob.dupont` / `Employe2!`) → **1 sortie** de stock *(base partagée : Bob agit sur les données saisies par Alice)*. Sur **Ubuntu** :
   ```bash
   docker exec pg-node psql -U metier -d metier -c \
     "SELECT code, nom FROM articles ORDER BY created_at DESC LIMIT 3;"
   ```
   → la donnée créée sur Kali apparaît sur Ubuntu en < 1 s.
4. **Panne du standby** — sur **Ubuntu** : `cd /opt/elbaraa-pme && docker compose down`
   → après ~60 s, **SaaS Clusters** : « **✗ Réplication interrompue** » *(détection honnête — notre correctif)*.
5. **Reprise** — sur **Ubuntu** : `docker compose up -d`
   → logs : `started streaming WAL from primary` ; **SaaS** repasse « ✓ Cluster sain » ; rattrapage WAL automatique.

> **Narration :** *« Toute écriture est répliquée en moins d'une seconde. Si une machine tombe, les données sont déjà ailleurs, et le tableau de bord de l'éditeur le détecte immédiatement — il ne ment jamais. Au redémarrage, la machine rattrape automatiquement ce qu'elle a manqué. »*

⚠️ **Sécurité démo :** ne couper **que le standby** (Ubuntu). Ne jamais couper le primaire en direct (le fencing n'est pas encore en place — risque de split-brain).

### ▸ Atomicité & cohérence (optionnel — si le jury creuse)

> Toutes ces commandes se lancent **sur Kali** (primaire), sauf la dernière vérif sur Ubuntu. ⚠️ Schéma réel : `articles.code` (UNIQUE), table `mouvements_stock`, colonne `type_mvt`.

**a) Unicité en concurrence** — deux fois la même référence `code` : la 1ʳᵉ passe, la 2ᵉ est rejetée.
```bash
# Alice crée l'article REF-DEMO
docker exec pg-node psql -U metier -d metier -c "INSERT INTO articles (code, nom, unite, seuil_alerte, actif) VALUES ('REF-DEMO','Cahier A5','unité',5,true);"
# Bob tente la MÊME référence → ERREUR de contrainte UNIQUE
docker exec pg-node psql -U metier -d metier -c "INSERT INTO articles (code, nom, unite, seuil_alerte, actif) VALUES ('REF-DEMO','Cahier A5 (doublon)','unité',5,true);"
```
→ attendu : `ERROR: duplicate key value violates unique constraint "articles_code_key"`.

**b) Deux mouvements simultanés** sur le même article — aucune corruption, les deux sont horodatés.
```bash
docker exec pg-node psql -U metier -d metier -c "INSERT INTO mouvements_stock (article_id,type_mvt,quantite,notes) SELECT id,'sortie',5,'Alice — bureau 1' FROM articles WHERE code='REF-DEMO';"
docker exec pg-node psql -U metier -d metier -c "INSERT INTO mouvements_stock (article_id,type_mvt,quantite,notes) SELECT id,'sortie',3,'Bob — bureau 2' FROM articles WHERE code='REF-DEMO';"
docker exec pg-node psql -U metier -d metier -c "SELECT type_mvt,quantite,notes,created_at FROM mouvements_stock m JOIN articles a ON a.id=m.article_id WHERE a.code='REF-DEMO' ORDER BY created_at DESC;"
```

**c) Le doublon n'a jamais été répliqué** — sur **Ubuntu** : un seul `REF-DEMO`.
```bash
docker exec pg-node psql -U metier -d metier -c "SELECT code, nom FROM articles WHERE code='REF-DEMO';"
```

> **Narration :** *« La contrainte d'unicité est vérifiée sur le primaire AVANT l'écriture du journal WAL. Le doublon est donc rejeté à la source : il n'entre jamais dans le journal, donc n'est jamais répliqué. Le standby ne peut pas diverger. C'est PostgreSQL natif, éprouvé — nous n'avons réimplémenté aucune logique de cohérence à la main. »*

---

## ACTE 5 — Chute · ~15 s

> *« De la création du compte à un cluster résilient — déploiement en quelques minutes, et l'éditeur n'a jamais vu une seule donnée métier. C'est ça, le Framework SaaS Souverain. »*

---

## Minutage récapitulatif

| Acte | Contenu | Durée |
|---|---|---|
| 1 | Création de compte (SaaS) | 1 min 30 |
| 2 | Installation (Debian, 1 commande) | 2 min 30 |
| 3 | Logiciel métier + parc/cluster | 1 min 30 |
| 4 | Réplication & résilience (MPJ) | 3 min 30 |
| 5 | Conclusion | 15 s |
| | **TOTAL** | **~9 min 15** |

---

## Antisèche commandes (à garder sous les yeux)

```bash
# Debian (nouvelle PME) — installation
sudo bash install-yasmine-argan.sh
docker compose -f /opt/elbaraa-pme/docker-compose.yml logs ss-node | grep -A5 "PREMIER DÉMARRAGE"

# MPJ — vérif réplication des COMPTES employés (sur Ubuntu .130)
docker exec pg-node psql -U metier -d metier -c "SELECT username, role FROM users ORDER BY username;"

# MPJ — vérif réplication MÉTIER (sur Ubuntu .130)
docker exec pg-node psql -U metier -d metier -c "SELECT code, nom FROM articles ORDER BY created_at DESC LIMIT 3;"

# MPJ — panne / reprise du standby (sur Ubuntu .130)
docker compose down
docker compose up -d
```

## URLs

> SaaS vu par **Yasmine Argan** : `192.168.10.1` · SaaS vu par **MPJ** : `192.168.200.1` (même SaaS, IP de l'hôte propre à chaque LAN).

| Quoi | URL |
|---|---|
| Inscription PME (depuis la Debian) | `http://192.168.10.1:8000/tenants/inscription/` |
| Parc machines | `http://192.168.10.1:8000/devices/` |
| Clusters | (menu « Clusters » du SaaS) |
| Interface métier (Debian) | `http://192.168.10.128:9001` |
| Interface métier (MPJ primaire) | `http://192.168.200.128:9001` |

---

## 🔄 Réinitialisation après répétition (repartir vierge pour le jour J)

> Objectif : effacer la PME de test **Yasmine Argan** pour que la vraie démo reparte de zéro
> (inscription + 1ʳᵉ installation). **Ne pas toucher à MPJ** — il sert aussi le jour J.

### A. Côté machine PME (Debian)

```bash
cd /opt/elbaraa-pme
sudo docker compose down                 # stoppe pg-node, ss-node, pgadmin
sudo systemctl disable elbaraa-pme.service 2>/dev/null || true
sudo rm -f /etc/systemd/system/elbaraa-pme.service
sudo systemctl daemon-reload
# Purge des données (DEK, base, config) → force un "premier install" propre
sudo rm -rf /opt/pme-node/pg-data /opt/pme-node/node-data
sudo rm -rf /opt/elbaraa-pme
# ⚠️ Rafraîchir l'image : le tag :dev est mutable. Si on garde un ancien cache,
# yasmine tournerait un binaire périmé (encart d'identifiants absent, etc.).
sudo docker rmi 192.168.10.1:5000/ss-node:dev 2>/dev/null || true
sudo docker pull 192.168.10.1:5000/ss-node:dev
```
> On garde Docker, mais on **re-tire l'image** : depuis la correction de l'installateur, `install-*.sh` fait de toute façon `docker compose pull ss-node` avant de démarrer, donc la dernière image du registre est toujours utilisée. **Pré-requis absolu** : que le registre contienne bien la dernière image (voir l'encadré « Avant la soutenance » ci-dessous).

### B. Côté SaaS éditeur — supprimer le tenant de test

**Méthode simple (UI)** : SaaS → **Tenants** → *Yasmine Argan* → **Supprimer** (cascade : licences + machines).
Puis Django admin → **Users** → supprimer le compte `contact` (sinon l'email unique bloque le ré-essai).

**Méthode shell (sûre, tout en un)** :
```bash
python manage.py shell -c "
from tenants.models import Tenant
for t in Tenant.objects.filter(name__icontains='Yasmine'):
    u = t.user
    t.delete()            # cascade : devices + licenses
    if u: u.delete()      # supprime aussi le compte Django (email unique)
print('Tenant de test supprimé.')
"
```

### C. Vérification « état jour J »

- [ ] SaaS → **Tenants** : seul **MPJ** reste (plus de Yasmine Argan).
- [ ] SaaS → **Parc machines** : seules les 2 machines MPJ.
- [ ] SaaS → **Clusters** : MPJ « ✓ Cluster sain ».
- [ ] Debian : `docker ps` ne montre plus pg-node/ss-node/pgadmin · `/opt/elbaraa-pme` absent.
- [ ] *(optionnel)* MPJ : supprimer l'article de répétition créé pendant la démo réplication, si vous voulez un historique propre.

---

## Plan B (si le réseau lâche pendant la démo)

1. Basculer sur les **captures d'écran** préparées (dossier `tests/captures-GUIDE-TEST-METIER-MPJ/`).
2. Commenter le flux à partir des captures — le récit reste le même.
3. Ne **jamais** déboguer en direct devant le jury : annoncer « je vous montre les captures de la même manipulation réalisée hier », puis continuer.
