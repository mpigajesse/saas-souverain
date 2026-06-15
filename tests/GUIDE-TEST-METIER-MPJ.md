# Guide de test métier — Tenant MPJ

> Document **vivant** : on remplit les colonnes « Résultat observé » au fur et à mesure.
> Sert de base au **rapport final** (preuves de souveraineté, réplication, cohérence).
>
> - **Date de campagne** : _________________
> - **Testeur** : _________________
> - **Version image `ss-node`** : `192.168.200.1:5000/ss-node:dev` (digest `565872a0…`)
> - **Cluster** : Kali `192.168.200.128:9001` (primaire) · Ubuntu `192.168.200.130:9001` (standby)

---

## 0. Acteurs du test

| Identifiant | Nom | Rôle | Mot de passe | Connexion |
|---|---|---|---|---|
| `admin` | Administrateur PME | Admin | *(logs Kali)* | http://192.168.200.128:9001/login |
| `alice.martin` | Alice Martin | Employé | `Employe1!` | idem |
| `bob.dupont` | Bob Dupont | Employé | `Employe2!` | idem |

**Droits employé** : tableau de bord, articles (voir/créer/supprimer), mouvements (saisir/voir).
**Interdits employé** : page Cluster, gestion des utilisateurs (admin uniquement).

> Les deux employés se connectent **sur le primaire (Kali)**. Le standby (Ubuntu) redirige
> vers le primaire — aucune écriture possible en lecture seule (`standby.signal`).

---

## 1. Pré-vol — la base de test est-elle saine ?

> **Campagne pré-vol validée le 15/06/2026 ~06:54** — cluster sain sur mesure réelle.
> Note : les 2 nœuds tournent l'image `ss-node` digest `565872a0…` ; le primaire rapporte
> `streaming_standby_count = 1`. Avant la mise à jour du binaire sur Kali, le SaaS affichait
> correctement « ✗ Réplication interrompue » (preuve du correctif de reporting, voir §4).

| # | Commande | Attendu | Résultat observé | OK ? |
|---|---|---|---|---|
| P1 | **Kali** : `... SELECT username, role, is_active FROM users ...` | 3 comptes (admin, alice, bob) actifs | alice.martin/employee/t · bob.dupont/employee/t · admin/admin/t | ☑ |
| P2 | **Ubuntu** : `... SELECT username, role FROM users ...` | mêmes 3 comptes (réplication users) | admin · alice.martin · bob.dupont — identiques | ☑ |
| P3 | **Kali** : `... SELECT ... FROM pg_stat_replication;` | 1 ligne `state=streaming` | `walreceiver · streaming · async` (1 row) | ☑ |
| P4 | **SaaS** : page **Clusters** | « ✓ Cluster sain » + « Réplication streaming → » | ✓ Cluster sain · 0 alerte · streaming OK | ☑ |

---

## 2. Scénarios via l'interface web (employés)

### Test 1 — Un employé crée 2 articles et les approvisionne ✅ VALIDÉ 15/06 07:08

**Acteur** : Bob (`bob.dupont`) · **But** : prouver la réplication primaire→standby d'écritures métier.

> Réalisé par Bob (employé) — la création n'est pas tracée par utilisateur, les 2 employés
> ont les mêmes droits. Alice agira sur ces données en T2 (preuve de base partagée).

> ℹ️ La table `articles` **ne stocke pas de quantité** : le stock est la **somme des mouvements**.
> Un article créé démarre donc à **stock 0** — Alice enregistre ensuite une **entrée** pour l'approvisionner.

**Manip UI (a) — créer les 2 articles** : connexion Alice → menu **Articles** → **+ Nouvel article** (à répéter 2 fois) :

| Champ | Article 1 | Article 2 |
|---|---|---|
| Code article * | `REF-CAH-A5` | `REF-STY-BL` |
| Désignation * | `Cahiers A5` | `Stylos bille bleus` |
| Catégorie | `Fournitures` | `Fournitures` |
| Unité de mesure * | `Unité (pcs)` | `Boîte` |
| Prix unitaire (DH) | `12.50` | `45.00` |
| Seuil d'alerte | `10` | `20` |

**Manip UI (b) — approvisionner chaque article** : menu **Mouvements** (2 entrées) :

| Champ | Entrée 1 | Entrée 2 |
|---|---|---|
| Article * | `Cahiers A5` (REF-CAH-A5) | `Stylos bille bleus` (REF-STY-BL) |
| Type * | `Entrée (réception, achat)` | `Entrée (réception, achat)` |
| Quantité * | `100` | `50` |
| Référence | `BC-2026-001` | `BC-2026-002` |

**Vérification (Ubuntu, lecture seule)** :
```bash
docker exec pg-node psql -U metier -d metier -c \
  "SELECT a.code, a.nom, a.seuil_alerte, COALESCE(SUM(m.quantite),0) AS stock
   FROM articles a LEFT JOIN mouvements_stock m ON m.article_id = a.id
   WHERE a.code IN ('REF-CAH-A5','REF-STY-BL')
   GROUP BY a.code, a.nom, a.seuil_alerte ORDER BY a.code;"
```

- **Attendu** : sur Ubuntu, les **2 articles** présents — stock `REF-CAH-A5 = 100`, `REF-STY-BL = 50` — en < 1 s, sans action côté standby.
- **Résultat observé** (Ubuntu .130, 15/06 07:08) :
  ```
      code    |        nom         | seuil_alerte | stock
  ------------+--------------------+--------------+-------
   REF-CAH-A5 | Cahiers A5         |           10 |   100
   REF-STY-BL | Stylos bille bleus |           20 |    50
  (2 rows)
  ```
  Historique mouvements (Kali) : REF-STY-BL Entrée +50 (BC-2026-002) · REF-CAH-A5 Entrée +100 (BC-2026-001).
- **Verdict** : ☑ **OK** — réplication confirmée, stocks identiques sur le standby. 📸 captures UI disponibles.

---

### Test 2 — Alice enregistre 2 sorties (dont une sous le seuil d'alerte) ✅ VALIDÉ 15/06

**Acteur** : Alice (`alice.martin`) · **But** : un employé voit/agit sur les données créées par un autre (Bob) + déclencher une alerte de stock bas.

**Manip UI** : connexion Alice → menu **Mouvements** (2 sorties) :

| Champ | Sortie 1 | Sortie 2 |
|---|---|---|
| Article * | `Cahiers A5` (REF-CAH-A5) | `Stylos bille bleus` (REF-STY-BL) |
| Type * | `Sortie (vente, consommation)` | `Sortie (vente, consommation)` |
| Quantité * | `15` | `35` |
| Référence | `BL-2026-001` | `BL-2026-002` |
| Notes | `Distribué formation` | `Commande service achat` |

> La sortie 2 fait passer les stylos de 50 → **15**, soit **sous le seuil d'alerte (20)** → l'article
> doit apparaître en alerte stock bas sur le tableau de bord.

**Vérification (Kali puis Ubuntu)** :
```bash
docker exec pg-node psql -U metier -d metier -c "
SELECT a.code, m.type_mvt, m.quantite, m.reference, m.notes
FROM mouvements_stock m JOIN articles a ON a.id = m.article_id
ORDER BY m.created_at DESC LIMIT 4;"
```

- **Attendu** : Alice voit les articles créés par Bob ; les 2 mouvements sont enregistrés
  (`quantite = -15` et `-35`, une sortie étant stockée en négatif) et répliqués sur Ubuntu.
- **Résultat observé** (Ubuntu .130) :
  ```
      code    | type_mvt | quantite |  reference  |        notes
  ------------+----------+----------+-------------+------------------------
   REF-STY-BL | sortie   |      -35 | BL-2026-002 | Commande service achat
   REF-CAH-A5 | sortie   |      -15 | BL-2026-001 | Distribué formation
   REF-STY-BL | entree   |       50 | BC-2026-002 |
   REF-CAH-A5 | entree   |      100 | BC-2026-001 |
  (4 rows)
  ```
- **Verdict** : ☑ **OK** — sorties répliquées (négatif), Alice agit sur les données de Bob. 📸 captures UI.

---

### Test 3 — Cohérence + alerte stock bas ✅ VALIDÉ 15/06

**Acteur** : Bob · **But** : base unique partagée, pas de copies divergentes.

**Manip UI** : reconnexion Bob → menu **Articles** → lignes Cahiers A5 et Stylos bille bleus.

**Vérification (stock calculé des 2 articles)** :
```bash
docker exec pg-node psql -U metier -d metier -c "
SELECT a.code, a.seuil_alerte, COALESCE(SUM(m.quantite),0) AS stock,
       (COALESCE(SUM(m.quantite),0) < a.seuil_alerte) AS sous_seuil
FROM articles a LEFT JOIN mouvements_stock m ON m.article_id = a.id
WHERE a.code IN ('REF-CAH-A5','REF-STY-BL')
GROUP BY a.code, a.seuil_alerte ORDER BY a.code;"
```

- **Attendu** :
  - `REF-CAH-A5` : stock = **85** (100 − 15), au-dessus du seuil (10) ;
  - `REF-STY-BL` : stock = **15** (50 − 35), **sous le seuil (20)** → `sous_seuil = t`, alerte stock bas.
  - Bob voit les sorties saisies par Alice → base unique partagée entre les 2 employés.
- **Résultat observé** (Ubuntu .130) :
  ```
      code    |        nom         | seuil_alerte | stock
  ------------+--------------------+--------------+-------
   REF-CAH-A5 | Cahiers A5         |           10 |    85
   REF-STY-BL | Stylos bille bleus |           20 |    15   ← sous seuil (alerte)
  (2 rows)
  ```
- **Verdict** : ☑ **OK** — stocks 85 / 15 cohérents et répliqués, alerte stock bas sur REF-STY-BL. 📸 captures UI.
- **Bonus tableau de bord** (UI Alice) : bandeau « ⚠️ 1 article en dessous du seuil », compteur « En alerte stock : 1 »,
  REF-STY-BL badge **Alerte** (15 < 20), REF-CAH-A5 badge **OK**. L'alerte de seuil est rendue dans l'interface.

---

## 3. Scénarios avancés (admin / SQL)

### Test 4 — Contrainte d'unicité (doublons impossibles)

```bash
docker exec pg-node psql -U metier -d metier -c "
INSERT INTO articles (code, nom, unite, seuil_alerte)
VALUES ('REF-CAH-A5', 'Cahiers copie', 'unité', 5);"
```
- **Attendu** : `ERROR: duplicate key value violates unique constraint "articles_code_key"`.
- **Résultat observé** (Kali .128) :
  ```
  ERROR:  duplicate key value violates unique constraint "articles_code_key"
  DETAIL:  Key (code)=(REF-CAH-A5) already exists.
  ```
- **Verdict** : ☑ **OK** — doublon refusé avant réplication, aucun doublon n'atteint le WAL ni le standby.

> La contrainte `UNIQUE(code)` est vérifiée sur le primaire **avant** toute réplication —
> aucun doublon n'atteint jamais le WAL ni le standby.

---

### Test 5 — Accès concurrent (MVCC)

Deux sorties quasi simultanées sur le même article (sortie = quantité **négative**) :
```bash
docker exec pg-node psql -U metier -d metier -c "
INSERT INTO mouvements_stock (article_id, type_mvt, quantite, notes)
SELECT id, 'sortie', -5, 'Alice — bureau 1' FROM articles WHERE code='REF-CAH-A5';"

docker exec pg-node psql -U metier -d metier -c "
INSERT INTO mouvements_stock (article_id, type_mvt, quantite, notes)
SELECT id, 'sortie', -3, 'Bob — bureau 2' FROM articles WHERE code='REF-CAH-A5';"
```
- **Attendu** : les mouvements enregistrés, horodatés séparément, sans corruption.
- **Résultat observé** (Ubuntu .130) :
  ```
      code    | stock | nb_mouvements
  ------------+-------+---------------
   REF-CAH-A5 |    74 |             5
  (1 row)
  ```
  > 3 sorties exécutées (−5, −3, −3 — l'insert −3 a été lancé deux fois), + entrée 100 − sortie 15 (T2)
  > = **74**, 5 mouvements. Tous coexistent sans corruption et sont répliqués sur le standby.
- **Verdict** : ☑ **OK** — concurrence sans corruption, stock cohérent (74) et répliqué.

---

### Test 6 — Absence d'un nœud puis rattrapage WAL ⭐ ✅ VALIDÉ 15/06 11:22

**But** : rejoue le cycle panne→reprise. **Lié directement au correctif de reporting SaaS.**

> ⚠️ Couper **uniquement le standby** (Ubuntu). Ne PAS couper le primaire tant que le
> fencing n'est pas en place (c'est ce qui a causé le split-brain initial).

```bash
# 1. Éteindre le STANDBY (Ubuntu)
docker compose down           # sur Ubuntu

# 2. Pendant l'absence — observer le SaaS
#    → Attendu : page Clusters passe à « ✗ Réplication interrompue » sous ~60 s

# 3. Sur KALI : écrire pendant l'absence
docker exec pg-node psql -U metier -d metier -c "
INSERT INTO articles (code, nom, unite, seuil_alerte)
VALUES ('REF-ABSENT-01', 'Article absent', 'unité', 5);"

# 4. Rallumer le STANDBY (Ubuntu)
docker compose up -d          # sur Ubuntu

# 5. Vérifier le rattrapage sur Ubuntu
docker exec pg-node psql -U metier -d metier -c \
  "SELECT nom, code FROM articles WHERE code='REF-ABSENT-01';"
```

- **Attendu** :
  - pendant l'absence → SaaS « ✗ Réplication interrompue » (plus de faux « sain ») ;
  - au redémarrage → l'article écrit par Kali apparaît sur Ubuntu (rattrapage WAL) ;
  - SaaS repasse « ✓ Cluster sain » sous ~60 s.
- **Résultat observé** :
  - Pendant l'absence : SaaS « ✗ Réplication interrompue », standby `Hors ligne`, 1 alerte. ✅
  - Écriture pendant panne : `REF-ABSENT-02` inséré sur Kali (`INSERT 0 1`).
  - Redémarrage Ubuntu (logs) : `entering standby mode` → `started streaming WAL from primary at 0/11000000 on timeline 1` (rattrapage WAL, **sans** pg_basebackup, timeline conservée).
  - Rattrapage confirmé sur Ubuntu :
    ```
         code      |             nom
    ---------------+-----------------------------
     REF-ABSENT-01 | Article absent
     REF-ABSENT-02 | Article écrit pendant panne
    (2 rows)
    ```
  - SaaS après redémarrage : « ✓ Cluster sain · Réplication streaming → · 0 alerte ». ✅
- **Verdict** : ☑ **OK** — détection de panne (correctif SaaS) + rattrapage WAL automatique, sans divergence ni intervention. 📸 captures SaaS avant/après.

---

## 4. Synthèse pour le rapport final

| Test | Garantie démontrée | Verdict | Preuve (réf. capture/sortie) |
|---|---|---|---|
| 1 | Réplication streaming primaire→standby (2 articles + entrées) | ☑ OK | SQL Ubuntu 100/50 + captures UI |
| 2 | Base unique partagée entre opérateurs (Bob crée, Alice agit) | ☑ OK | SQL Ubuntu sorties −15/−35 + captures |
| 3 | Cohérence ACID + alerte seuil (UI) | ☑ OK | SQL stocks 85/15 + dashboard « Alerte » |
| 4 | Unicité `UNIQUE(code)` avant réplication | ☑ OK | erreur `articles_code_key` |
| 5 | Accès concurrent sérialisé (MVCC) | ☑ OK | SQL stock 74 / 5 mouvements |
| 6 | Rattrapage WAL + détection de panne (reporting SaaS) | ☑ OK | SaaS interrompu→sain + REF-ABSENT-02 rattrapé |

**Conclusion** : les 6 scénarios sont validés sur le banc 2 nœuds (Kali primaire ⚡ + Ubuntu standby).
La souveraineté des données métier (réplication streaming, cohérence transactionnelle, contraintes
d'intégrité, accès concurrent, résilience à la panne d'un nœud) est démontrée bout en bout. Le
reporting SaaS reflète désormais l'état **réel** de la réplication (`pg_stat_replication`), supprimant
le risque de dégradation silencieuse.

**Limites connues (hors périmètre de cette campagne)** :
- Failover automatique non re-testé ici (couper le primaire) — nécessite le **fencing** (jeton d'époque)
  pour éviter le split-brain rencontré le 15/06 au matin.
- IP DHCP à bail court sur le réseau 200.x — à fiabiliser (IP statiques / réservation) avant production.

---

## Annexe — schéma de cohérence

```
Kali (Primaire PG)                Ubuntu (Standby PG)
─────────────────                 ─────────────────────
  INSERT article
  → WAL record écrit              ← WAL stream (TCP)
  → Contrainte UNIQUE vérifiée    → WAL appliqué
  → Transaction commitée          → Données identiques
  → Réponse à l'utilisateur        → Lecture seule (standby.signal)
```

> Guide SQL exhaustif d'origine : `pme-deploy/COMPTES-ET-TESTS.md`.
