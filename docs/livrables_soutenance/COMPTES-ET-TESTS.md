# Comptes PME — Accès et Tests de Cluster

> ⚠️ **Schéma réel** (à connaître pour toutes les commandes ci-dessous) :
> - Table **`articles`** : clé métier = **`code`** (contrainte `UNIQUE`), `nom`, `unite`,
>   `categorie`, `prix_unitaire`, `seuil_alerte`, `actif`. **Pas** de colonne `quantite`
>   ni `reference` sur l'article.
> - Table **`mouvements_stock`** : `article_id`, **`type_mvt`** (`entree` / `sortie` /
>   `ajustement`), `quantite`, `reference`, `notes`, `created_at`.
> - **Stock courant** = somme des mouvements : `COALESCE(SUM(m.quantite),0)`.
>   Une **sortie est stockée en négatif** (l'UI le fait automatiquement ; en SQL brut,
>   insérer une quantité négative pour une sortie).
> - Connexion psql : `-U metier -d metier`.

## Comptes utilisateurs

| Identifiant    | Nom complet        | Rôle          | Mot de passe | URL de connexion                    |
|----------------|--------------------|---------------|--------------|-------------------------------------|
| `admin`        | Administrateur PME | Administrateur| *(voir encart)*| http://192.168.200.128:9001/login   |
| `alice.martin` | Alice Martin       | Employé       | `Employe1!`  | http://192.168.200.128:9001/login   |
| `bob.dupont`   | Bob Dupont         | Employé       | `Employe2!`  | http://192.168.200.128:9001/login   |

> **Mot de passe admin** : au tout premier démarrage, il s'affiche dans un encart
> « 🔑 Première connexion » directement sur la page de login (et dans les logs Kali :
> `docker compose logs ss-node | grep -A5 "PREMIER DÉMARRAGE"`).
> L'encart **disparaît automatiquement dès la première connexion d'un admin**
> (sécurité : le mot de passe n'est plus exposé aux employés). Le changer via :
> Administration → Utilisateurs → Mot de passe.

---

## Droits par rôle

| Fonctionnalité                  | Employé | Admin |
|---------------------------------|---------|-------|
| Tableau de bord stock           | ✓       | ✓     |
| Voir les articles               | ✓       | ✓     |
| Créer un article                | ✓       | ✓     |
| Supprimer un article            | ✓       | ✓     |
| Enregistrer un mouvement        | ✓       | ✓     |
| Voir les mouvements             | ✓       | ✓     |
| Page Cluster (statut PG)        | ✗       | ✓     |
| Gérer les utilisateurs          | ✗       | ✓     |
| Créer / désactiver un compte    | ✗       | ✓     |
| Changer le mot de passe         | ✗       | ✓     |

> Le nœud standby (Ubuntu 192.168.200.130) affiche une page de redirection
> vers le primaire — aucune connexion possible en lecture seule.

---

## Tests de cluster — Scénarios

### Test 1 — Réplication d'un article créé par Alice

**Objectif** : un article créé par Alice sur le primaire apparaît sur le standby.

```bash
# Sur Kali — avant (count articles)
docker exec pg-node psql -U metier -d metier -c "SELECT COUNT(*) FROM articles;"

# Alice se connecte sur http://192.168.200.128:9001
# → Articles → Créer un article : code "REF-CAH-A5", nom "Cahiers A5", seuil 10
# → Mouvements → Entrée : REF-CAH-A5, quantité 100

# Sur Ubuntu — vérifier réplication (~1 seconde)
docker exec pg-node psql -U metier -d metier -c "SELECT code, nom FROM articles ORDER BY created_at DESC LIMIT 3;"
```

**Résultat attendu** : même article visible sur Ubuntu sans aucune action.

---

### Test 2 — Mouvement de stock par Bob, visible par Alice

**Objectif** : Bob sort du stock, Alice voit le stock mis à jour.

```bash
# Bob se connecte sur http://192.168.200.128:9001
# → Mouvements → Sortie : REF-CAH-A5, quantité 15, note "Distribué formation"

# Vérifier sur Kali — les mouvements et le stock courant calculé
docker exec pg-node psql -U metier -d metier -c "
SELECT a.code, a.nom, m.type_mvt, m.quantite AS qte_mvt, m.notes
FROM mouvements_stock m JOIN articles a ON a.id = m.article_id
ORDER BY m.created_at DESC LIMIT 5;"

# Stock courant de l'article (somme des mouvements)
docker exec pg-node psql -U metier -d metier -c "
SELECT a.code, a.nom, COALESCE(SUM(m.quantite),0) AS stock
FROM articles a LEFT JOIN mouvements_stock m ON m.article_id = a.id
WHERE a.code = 'REF-CAH-A5'
GROUP BY a.code, a.nom;"

# Vérifier réplication sur Ubuntu
docker exec pg-node psql -U metier -d metier -c "
SELECT a.code, a.nom, COALESCE(SUM(m.quantite),0) AS stock
FROM articles a LEFT JOIN mouvements_stock m ON m.article_id = a.id
WHERE a.code = 'REF-CAH-A5'
GROUP BY a.code, a.nom;"
```

**Résultat attendu** : stock = 100 − 15 = **85**, identique sur Kali et Ubuntu.

---

### Test 3 — Contrainte d'unicité (doublons impossibles)

**Objectif** : la base empêche deux articles avec le même code.

```bash
# Tenter d'insérer un code déjà existant
docker exec pg-node psql -U metier -d metier -c "
INSERT INTO articles (code, nom, unite, seuil_alerte, actif)
VALUES ('REF-CAH-A5', 'Cahiers copie', 'unité', 5, true);"
```

**Résultat attendu** :
```
ERROR:  duplicate key value violates unique constraint "articles_code_key"
DETAIL:  Key (code)=(REF-CAH-A5) already exists.
```

> La contrainte `UNIQUE(code)` est définie au niveau PostgreSQL — elle s'applique
> sur le primaire **avant** toute réplication. Aucun doublon ne peut exister, même si
> Alice et Bob essaient d'insérer le même code simultanément.

---

### Test 4 — Accès concurrent : Alice et Bob modifient en même temps

**Objectif** : deux mouvements simultanés sur le même article ne corrompent pas le stock.

```bash
# Simuler deux sorties simultanées (lancer les deux rapidement)
# Rappel : une SORTIE est stockée en quantité NÉGATIVE.
docker exec pg-node psql -U metier -d metier -c "
INSERT INTO mouvements_stock (article_id, type_mvt, quantite, notes)
SELECT id, 'sortie', -5, 'Alice — bureau 1' FROM articles WHERE code='REF-CAH-A5';"

docker exec pg-node psql -U metier -d metier -c "
INSERT INTO mouvements_stock (article_id, type_mvt, quantite, notes)
SELECT id, 'sortie', -3, 'Bob — bureau 2' FROM articles WHERE code='REF-CAH-A5';"

# Vérifier les deux mouvements enregistrés
docker exec pg-node psql -U metier -d metier -c "
SELECT m.type_mvt, m.quantite, m.notes, m.created_at
FROM mouvements_stock m JOIN articles a ON a.id = m.article_id
WHERE a.code = 'REF-CAH-A5'
ORDER BY m.created_at DESC LIMIT 5;"
```

> PostgreSQL traite les transactions en série via MVCC — les deux INSERTs
> réussissent, les deux mouvements sont horodatés séparément (stock = 85 − 5 − 3 = 77).

---

### Test 5 — Réplication en temps réel (watch)

**Objectif** : observer la réplication à la milliseconde.

```bash
# Terminal 1 — Ubuntu : surveiller le count en continu
watch -n 1 "docker exec pg-node psql -U metier -d metier -c 'SELECT (SELECT COUNT(*) FROM articles) AS articles, (SELECT COUNT(*) FROM mouvements_stock) AS mouvements;'"

# Terminal 2 — Kali : faire une modification
docker exec pg-node psql -U metier -d metier -c "
INSERT INTO mouvements_stock (article_id, type_mvt, quantite, notes)
SELECT id, 'entree', 100, 'Réappro test temps réel' FROM articles LIMIT 1;"
```

**Résultat attendu** : le compteur sur Ubuntu s'incrémente en < 1 seconde.

---

### Test 6 — Absence d'un nœud puis rattrapage WAL

**Objectif** : Ubuntu éteint, Kali continue à écrire, Ubuntu rattrape au redémarrage.

```bash
# 1. Éteindre Ubuntu
docker compose down   # sur Ubuntu

# 2. Sur Kali : insérer des données pendant l'absence
docker exec pg-node psql -U metier -d metier -c "
INSERT INTO articles (code, nom, unite, seuil_alerte, actif)
VALUES ('REF-ABSENT-01', 'Article absent', 'unité', 5, true);"

# 3. Rallumer Ubuntu (pg_basebackup si pg-data effacé, sinon WAL replay)
docker compose up -d   # sur Ubuntu

# 4. Vérifier le rattrapage
docker exec pg-node psql -U metier -d metier -c "SELECT code, nom FROM articles WHERE code='REF-ABSENT-01';"
```

> Si la réplication streaming reprend (standby.signal présent), le WAL manquant
> est rejoué automatiquement. Sinon, pg_basebackup repart de zéro depuis Kali.

---

## Architecture — Pourquoi les données restent cohérentes

```
Kali (Primaire PG)                Ubuntu (Standby PG)
─────────────────                 ─────────────────────
  INSERT article                        
  → WAL record écrit              ← WAL stream (TCP)
  → Contrainte UNIQUE vérifiée    → WAL appliqué
  → Transaction commitée          → Données identiques
  → Réponse à l'utilisateur       → Lecture seule
```

**Garanties** :

| Mécanisme          | Ce qu'il garantit                                              |
|--------------------|----------------------------------------------------------------|
| `UNIQUE(code)`     | Aucun doublon de code article, même en concurrence            |
| `NOT NULL` + `CHECK`| Données obligatoires et valides avant insertion (`quantite != 0`, `type_mvt` contrôlé) |
| Transactions ACID  | Pas de lecture partielle, pas de corruption                   |
| WAL streaming sync | Ubuntu reçoit chaque écriture de Kali en < 100 ms             |
| `standby.signal`   | Ubuntu refuse les écritures — ne peut pas diverger            |
| Epoch token        | L'ancien primaire est bloqué après promotion du standby       |

**Cas du doublon concurrent** :
- Alice tente d'insérer le code REF-001 à T=0ms
- Bob tente d'insérer le code REF-001 à T=1ms
- PostgreSQL sérialise : Alice réussit, Bob reçoit une erreur de contrainte
- Aucun doublon n'atteint jamais le WAL → Ubuntu ne reçoit jamais le doublon
