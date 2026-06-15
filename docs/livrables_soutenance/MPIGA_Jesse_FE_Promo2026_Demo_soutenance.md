# Runbook de Démonstration — Soutenance

**Framework SaaS Souverain** · Jesse MPIGA-ODOUMBA · AL BARAA CONSULTING · 01/07/2026
Durée cible : **~10 min** (slide 10 du support). Objectif : montrer que **déployer est facile**, puis expliquer **réplication & résilience**.

---

## Scénario narratif

> Deux PME illustrent le cycle de vie :
> - **PME nouvelle — « Boulangerie Atlas »** : peu de moyens, **une seule machine (Debian)**. Sert à montrer l'inscription + l'installation en quelques minutes.
> - **PME existante — « MPJ »** : cluster **2 nœuds** déjà en place (Kali primaire + Ubuntu standby). Sert à expliquer la **réplication** et la **résilience à la panne**.

Message au jury : *« Une PME démarre avec une machine ; quand elle grandit, elle ajoute un nœud et obtient la réplication automatique — la même commande d'installation s'en charge. »*

---

## ACTE 0 — Préparation AVANT le jury (checklist)

> ⚠️ À faire **avant** d'entrer, jamais en direct. Une démo fluide se prépare.

### 🔌 Réseau — LE point bloquant à régler en premier

Le SaaS et le registre Docker sont sur **192.168.200.1** (VMnet `192.168.200.0/24`, le même que MPJ).
La VM Debian doit avoir **une patte sur ce réseau**, sinon elle ne joint ni le SaaS ni le registre.

État Debian (rappel) : `ens33` = 192.168.1.57 (LAN + Internet) · `ens37` = **192.168.10.128 ❌ mauvais VMnet**.

- [ ] **Rattacher `ens37` au VMnet de MPJ** (VMware → Debian → Adaptateur réseau 2 → même VMnet que Kali/Ubuntu).
- [ ] Renouveler le bail : `sudo dhclient -r ens37 && sudo dhclient ens37` → `ens37` obtient **192.168.200.x**.
- [ ] Vérifier la liaison : `ping -c1 192.168.200.1` puis `curl -I http://192.168.200.1:8000`.
- [ ] **Noter l'IP 192.168.200.x de la Debian** → c'est l'IP de l'interface métier en démo (`http://<ip>:9001`).
- [ ] Garder `ens33` (192.168.1.57, Internet) actif — utile si Docker doit s'installer.

### Préparation Docker (sur la Debian, via Internet ens33)

- [ ] **Installer Docker à l'avance** : `curl -fsSL https://get.docker.com | sh` (évite l'attente live).
- [ ] Autoriser le registre éditeur : ajouter `192.168.200.1:5000` dans `/etc/docker/daemon.json` → `{"insecure-registries":["192.168.200.1:5000"]}` puis `sudo systemctl restart docker`.
- [ ] **Pré-tirer l'image** : `docker pull 192.168.200.1:5000/ss-node:dev`.
- [ ] **Aucun stack PME installé** : `/opt/elbaraa-pme` absent (sinon `docker compose down` + purge avant la démo).

### SaaS & cluster existant

- [ ] **SaaS éditeur** joignable : `http://192.168.200.1:8000` (Tableau de bord visible).
- [ ] **Registre Docker** up : image `ss-node:dev` présente sur `192.168.200.1:5000`.
- [ ] **Cluster MPJ** en marche : Kali (.128) + Ubuntu (.130) → Clusters « ✓ Cluster sain ».
- [ ] **Onglets navigateur** ouverts : (1) inscription SaaS, (2) Parc machines, (3) Clusters.
- [ ] **Plan B** : captures d'écran dans `tests/captures-GUIDE-TEST-METIER-MPJ/` si le réseau flanche.
- [ ] Terminaux lisibles : **police agrandie** (Ctrl+ +), fond clair si projecteur pâle.

---

## ACTE 1 — Création du compte (SaaS éditeur) · ~1 min 30

**Écran : navigateur sur le SaaS.**

1. Aller sur la page d'inscription : `http://192.168.200.1:8000/tenants/inscription/`
2. Remplir le formulaire :
   | Champ | Valeur démo |
   |---|---|
   | Nom de l'entreprise | `Boulangerie Atlas` |
   | Email | `contact@boulangerie-atlas.ma` |
   | Mot de passe | `Atlas2026!` |
   | Téléphone | `+212 6 00 00 00 00` |
   | Nombre d'employés | `1` |
3. Valider → **automatiquement** : compte créé + **licence d'essai 30 jours (Starter, 1 poste)** + connexion + page **Bienvenue**.

> **Narration :** *« En moins d'une minute, la PME a un compte, une licence d'essai, et — vous allez le voir — son installateur prêt à télécharger. L'éditeur, lui, ne verra jamais les données métier : il ne gère que le compte et la licence. »*

---

## ACTE 2 — Installation sur la machine PME (Debian) · ~2 min 30

**Écran : page Bienvenue (SaaS), puis terminal Debian.**

1. **Depuis le navigateur de la Debian**, ouvrir le SaaS via `http://192.168.200.1:8000` (important : c'est cette URL qui sera injectée comme `SAAS_URL` et registre dans le script). Sur la page **Bienvenue** → section installation → **Linux** → télécharger `install-boulangerie-atlas.sh`.
   *(le script embarque déjà le token, l'URL du SaaS = 192.168.200.1:8000, l'URL du relais et l'image du registre)*
2. Transférer/ouvrir le script sur la **Debian**, puis :
   ```bash
   sudo bash install-boulangerie-atlas.sh
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
   Installation terminée ! (Boulangerie Atlas — primary)
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
2. Se connecter à l'interface → créer **1 article** (ex. `PAIN-01` / `Baguette` / unité / seuil 20) + **1 entrée** de stock (qté 200).
3. Revenir sur le SaaS :
   - **Parc machines** : la machine `Boulangerie Atlas` apparaît (1 poste, primaire, en ligne).
   - **Clusters** : « **⚠ Bascule manuelle** » — *une seule machine, pas de redondance*.

> **Narration :** *« Le logiciel tourne, les données sont là — et restent là. Mais avec une seule machine, pas de redondance : le SaaS le signale honnêtement. C'est exactement la situation d'une PME qui démarre avec peu de moyens. Voyons ce qui se passe quand elle ajoute une deuxième machine. »*

---

## ACTE 4 — Réplication & résilience (PME MPJ, 2 nœuds) · ~3 min 30

**Écran : SaaS Clusters + 2 terminaux (Kali primaire, Ubuntu standby).**

> Transition : *« La PME MPJ, elle, a deux machines. Voici la réplication en action. »*

1. **SaaS Clusters → MPJ** : « ✓ Cluster sain · Réplication streaming → ».
2. **Réplication live** — sur **Kali**, créer une donnée (UI métier ou SQL), puis sur **Ubuntu** :
   ```bash
   docker exec pg-node psql -U metier -d metier -c \
     "SELECT code, nom FROM articles ORDER BY created_at DESC LIMIT 3;"
   ```
   → la donnée créée sur Kali apparaît sur Ubuntu en < 1 s.
3. **Panne du standby** — sur **Ubuntu** : `cd /opt/elbaraa-pme && docker compose down`
   → après ~60 s, **SaaS Clusters** : « **✗ Réplication interrompue** » *(détection honnête — notre correctif)*.
4. **Reprise** — sur **Ubuntu** : `docker compose up -d`
   → logs : `started streaming WAL from primary` ; **SaaS** repasse « ✓ Cluster sain » ; rattrapage WAL automatique.

> **Narration :** *« Toute écriture est répliquée en moins d'une seconde. Si une machine tombe, les données sont déjà ailleurs, et le tableau de bord de l'éditeur le détecte immédiatement — il ne ment jamais. Au redémarrage, la machine rattrape automatiquement ce qu'elle a manqué. »*

⚠️ **Sécurité démo :** ne couper **que le standby** (Ubuntu). Ne jamais couper le primaire en direct (le fencing n'est pas encore en place — risque de split-brain).

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
sudo bash install-boulangerie-atlas.sh
docker compose -f /opt/elbaraa-pme/docker-compose.yml logs ss-node | grep -A5 "PREMIER DÉMARRAGE"

# MPJ — vérif réplication (sur Ubuntu .130)
docker exec pg-node psql -U metier -d metier -c "SELECT code, nom FROM articles ORDER BY created_at DESC LIMIT 3;"

# MPJ — panne / reprise du standby (sur Ubuntu .130)
docker compose down
docker compose up -d
```

## URLs

| Quoi | URL |
|---|---|
| Inscription PME | `http://192.168.200.1:8000/tenants/inscription/` |
| Parc machines | `http://192.168.200.1:8000/devices/` |
| Clusters | (menu « Clusters » du SaaS) |
| Interface métier (Debian) | `http://<ip-debian-192.168.200.x>:9001` *(IP obtenue après rattachement au VMnet MPJ)* |
| Interface métier (MPJ primaire) | `http://192.168.200.128:9001` |

---

## Plan B (si le réseau lâche pendant la démo)

1. Basculer sur les **captures d'écran** préparées (dossier `tests/captures-GUIDE-TEST-METIER-MPJ/`).
2. Commenter le flux à partir des captures — le récit reste le même.
3. Ne **jamais** déboguer en direct devant le jury : annoncer « je vous montre les captures de la même manipulation réalisée hier », puis continuer.
