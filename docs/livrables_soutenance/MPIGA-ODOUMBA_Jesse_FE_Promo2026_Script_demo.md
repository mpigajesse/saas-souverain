# Script de Démonstration Live — أمان (Amān)

> Soutenance — Jesse MPIGA-ODOUMBA · Promotion 2026 · cible ~5 min (slot démo)
> **Principe** : **yasmine** = parcours de déploiement à chaud · **MPJ** = cluster déjà en place (réplication, fencing, agent).

---

## ⏱️ Avant de commencer (checklist, à faire AVANT la soutenance)

- [ ] SaaS éditeur lancé : `python manage.py runserver 0.0.0.0:8000` (un seul, port 8000 libre).
- [ ] **Pare-feu Windows** réactivé (plus besoin du serveur de transfert).
- [ ] MPJ : kali (primaire) + ubuntu (standby) démarrés, réplication saine (`pg_stat_replication` = 1 ligne streaming).
- [ ] yasmine : Debian prête, mais **logiciel pas encore "présenté"** (on montre le parcours).
- [ ] Relais joignable, `blob-stats` répond (2 tenants).
- [ ] Onglets navigateur préparés : portail SaaS, page inscription, page Relais, UI nœud MPJ, UI nœud yasmine.
- [ ] Codes de récupération notés (MPJ `EF18-…`, yasmine `64BD-…`) au cas où.

---

## SÉQUENCE 1 — Le parcours client : déploiement (≈ 2 min) — **tenant yasmine**

> *Objectif : montrer qu'une PME s'inscrit et déploie seule, sans expertise.*

1. **Page d'inscription** → `http://192.168.200.1:8000/tenants/inscription/`
   > « Une PME s'inscrit en ligne. Elle renseigne son nom, son email, son nombre d'employés. »
   *(Le formulaire s'affiche toujours, même si je suis déjà connecté — corrigé pour la démo.)*

2. **Page Bienvenue** (après inscription / espace yasmine) → montrer :
   - Les **3 étapes** (télécharger l'installateur, l'exécuter, vérifier le parc).
   - L'**encart 🔑 clé de récupération** : « la PME devra conserver sa clé ; sans elle, même nous ne pouvons rien déchiffrer ».
   > « Tout est automatisé : l'installateur embarque Docker, configure et démarre أمان. »

3. **Interface d'أمان sur yasmine** → `http://<ip-yasmine>:9001`, login **admin** →
   - Sidebar **« أمان (Amān) · souverain »**.
   - Menu **Administration → Clé de récupération** : la clé `64BD-…` s'affiche + **Télécharger (.txt)**.
   > « La clé est générée **sur la machine de la PME**, jamais transmise. C'est la promesse zero-knowledge, concrète. »

---

## SÉQUENCE 2 — Le cœur technique : réplication & résilience (≈ 2 min) — **tenant MPJ**

> *Objectif : prouver la haute disponibilité sur 2 machines réelles.*

4. **Réplication en direct** :
   - Sur **kali (primaire)** UI → créer/modifier un article (ou montrer un stock existant).
   - Sur **ubuntu (standby)** → la donnée apparaît (réplique lecture seule). *(ou requête `pg_stat_replication` sur kali = `streaming`.)*
   > « Toute écriture est copiée en moins d'une seconde. Si une machine tombe, les données sont déjà ailleurs. »

5. **Tableau de bord SaaS véridique** → `http://192.168.200.1:8000/devices/clusters/`
   > « Le portail reflète l'état **réel** de la réplication, mesuré — il ne ment jamais. »

6. **(Optionnel, si le temps le permet) Fencing** — *à ne montrer que si répété et fiable* :
   - Promouvoir ubuntu (`SELECT pg_promote();`) → rallumer kali → kali affiche **⛔ NŒUD CLÔTURÉ**.
   > « L'ancien primaire revenu se clôture tout seul : pas de split-brain. »
   *(⚠️ Plan B : si risqué en live, montrer la capture/log `⛔ NŒUD CLÔTURÉ` déjà obtenue.)*

---

## SÉQUENCE 3 — Souveraineté & supervision (≈ 1 min) — **les 2 tenants + relais**

7. **Page Relais zero-knowledge** → montrer les **2 tenants** avec leurs blobs :
   > « MPJ et yasmine, deux entreprises sur des réseaux isolés. Le relais stocke leurs coffres **chiffrés** — `blob_count` visible, contenu **jamais** lisible. »

8. **Agent IA** → montrer le diagnostic (score de risque, recommandation) :
   > « Un agent IA supervise les 3 acteurs — il lit des métriques d'infrastructure, jamais les données métier. C'est le lien avec ma spécialité IA & Big Data. »

---

## 🎯 Phrase de clôture de la démo
> « En résumé : la PME s'installe seule, ses données restent chez elle, elles sont répliquées et résilientes, sauvegardées chiffrées chez l'éditeur — qui ne peut **rien** lire. Souveraineté prouvée, pas promise. »

---

## 🛟 Plans B (si un live échoue)

| Risque | Repli |
|--------|-------|
| Réseau VM instable | Captures d'écran préparées de chaque étape |
| Fencing trop risqué en live | Montrer le log `⛔ NŒUD CLÔTURÉ` déjà capturé |
| Relais injoignable | `curl .../api/blob-stats` capturé d'avance |
| Inscription plante | Montrer un tenant déjà créé (bienvenue) |

> **Règle d'or de la démo** : ne jamais lancer en live une action non répétée. Tout ce qui est montré « à chaud » (déploiement yasmine) doit avoir été testé juste avant. Le reste (réplication MPJ) est **déjà en place** → on le parcourt, on ne le (re)construit pas.

---

## 🔄 Réinitialiser yasmine pour rejouer la démo « de zéro »

> À exécuter **sur la VM yasmine** entre deux passages, pour repartir d'un état vierge (teste l'auto-dérivation du relais + l'encart d'identifiants + la génération de la clé).

```bash
cd /opt/elbaraa-pme
sudo docker compose down -v
sudo docker run --rm -v /opt/pme-node:/d alpine sh -c "rm -rf /d/* /d/.[!.]*"
# ↑ efface pg-data ET node-data → config, DEK, clé, mot de passe admin : tout repart à zéro
sudo systemctl disable --now elbaraa-pme.service
sudo rm -rf /opt/elbaraa-pme /etc/systemd/system/elbaraa-pme.service
sudo systemctl daemon-reload
```

**Puis, côté portail éditeur** : supprimer l'ancien tenant yasmine (`/admin/` Django → Tenants) **ou** réinscrire avec un autre email.

**(Optionnel) Relais** : purger les blobs orphelins de l'ancien yasmine :
```bash
sudo ls /opt/elbaraa-relay/blobs          # repérer le(s) dossier(s) de l'ancien tenant
sudo rm -rf /opt/elbaraa-relay/blobs/<id-orphelin>
```

**Réinstallation propre** (le test à blanc) :
1. Inscription yasmine sur le portail → télécharger l'installateur.
2. Sur yasmine : `chmod +x install-*.sh && sudo ./install-*.sh`
3. Vérifier `Relais (auto) : http://192.168.10.10:8080` (auto-dérivé, **sans config**).
4. `http://<ip-yasmine>:9001` → encart **identifiants** sur le login → connexion → **Clé de récupération** affichée.

> ✅ Si les 3 (auto-dérivation, encart identifiants, clé) marchent **sans toucher à aucune adresse**, la chaîne de déploiement autonome est validée. Répétable à volonté.

`★ Le point qui rend le test "de zéro" réel` : effacer **node-data** (pas seulement pg-data). Sinon l'ancienne config (tenant_id, DEK, clé, mot de passe admin) persiste et la génération fraîche n'est pas testée.
