# Comptes PME — Accès et Tests de Cluster

## Comptes utilisateurs

| Identifiant    | Nom complet        | Rôle          | Mot de passe | URL de connexion                    |
|----------------|--------------------|---------------|--------------|-------------------------------------|
| `admin`        | Administrateur PME | Administrateur| *(voir logs)*| http://192.168.200.128:9001/login   |
| `alice.martin` | Alice Martin       | Employé       | `Employe1!`  | http://192.168.200.128:9001/login   |
| `bob.dupont`   | Bob Dupont         | Employé       | `Employe2!`  | http://192.168.200.128:9001/login   |

> **Mot de passe admin** : affiché au premier démarrage dans les logs Kali.
> Récupérer avec : `docker compose logs ss-node | grep -A5 "PREMIER DÉMARRAGE"`
> Le changer via : Administration → Utilisateurs → Mot de passe

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
# → Articles → Créer un article : "Cahiers A5", REF-CAH-A5, qté 100, seuil 10

# Sur Ubuntu — vérifier réplication (~1 seconde)
docker exec pg-node psql -U metier -d metier -c "SELECT nom, reference, quantite FROM articles ORDER BY created_at DESC LIMIT 3;"
```

**Résultat attendu** : même article visible sur Ubuntu sans aucune action.

---

### Test 2 — Mouvement de stock par Bob, visible par Alice

**Objectif** : Bob sort du stock, Alice voit le stock mis à jour.

```bash
# Bob se connecte sur http://192.168.200.128:9001
# → Mouvements → Sortie : REF-CAH-A5, quantité 15, note "Distribué formation"

# Vérifier sur Kali
docker exec pg-node psql -U metier -d metier -c "
SELECT a.nom, a.quantite, m.type_mouvement, m.quantite AS qte_mvt, m.notes
FROM mouvements m JOIN articles a ON a.id = m.article_id
ORDER BY m.created_at DESC LIMIT 5;"

# Vérifier réplication sur Ubuntu
docker exec pg-node psql -U metier -d metier -c "
SELECT a.nom, a.quantite FROM articles WHERE reference = 'REF-CAH-A5';"
```

---

### Test 3 — Contrainte d'unicité (doublons impossibles)

**Objectif** : la base empêche deux articles avec la même référence.

```bash
# Tenter d'insérer une référence déjà existante
docker exec pg-node psql -U metier -d metier -c "
INSERT INTO articles (nom, reference, quantite, seuil_alerte, actif)
VALUES ('Cahiers copie', 'REF-CAH-A5', 50, 5, true);"
```

**Résultat attendu** :
```
ERROR:  duplicate key value violates unique constraint "articles_reference_key"
DETAIL:  Key (reference)=(REF-CAH-A5) already exists.
```

> La contrainte `UNIQUE(reference)` est définie au niveau PostgreSQL — elle s'applique
> sur le primaire **avant** toute réplication. Aucun doublon ne peut exister, même si
> Alice et Bob essaient d'insérer la même référence simultanément.

---

### Test 4 — Accès concurrent : Alice et Bob modifient en même temps

**Objectif** : deux mouvements simultanés sur le même article ne corrompent pas le stock.

```bash
# Simuler deux sorties simultanées (lancer les deux rapidement)
docker exec pg-node psql -U metier -d metier -c "
INSERT INTO mouvements (article_id, type_mouvement, quantite, notes)
SELECT id, 'sortie', 5, 'Alice — bureau 1' FROM articles WHERE reference='REF-CAH-A5';"

docker exec pg-node psql -U metier -d metier -c "
INSERT INTO mouvements (article_id, type_mouvement, quantite, notes)
SELECT id, 'sortie', 3, 'Bob — bureau 2' FROM articles WHERE reference='REF-CAH-A5';"

# Vérifier les deux mouvements enregistrés
docker exec pg-node psql -U metier -d metier -c "
SELECT type_mouvement, quantite, notes, created_at
FROM mouvements m JOIN articles a ON a.id = m.article_id
WHERE a.reference = 'REF-CAH-A5'
ORDER BY created_at DESC LIMIT 5;"
```

> PostgreSQL traite les transactions en série via MVCC — les deux INSERTs
> réussissent, les deux mouvements sont horodatés séparément.

---

### Test 5 — Réplication en temps réel (watch)

**Objectif** : observer la réplication à la milliseconde.

```bash
# Terminal 1 — Ubuntu : surveiller le count en continu
watch -n 1 "docker exec pg-node psql -U metier -d metier -c 'SELECT COUNT(*) as articles, (SELECT COUNT(*) FROM mouvements) as mouvements FROM articles;'"

# Terminal 2 — Kali : faire une modification
docker exec pg-node psql -U metier -d metier -c "
INSERT INTO mouvements (article_id, type_mouvement, quantite, notes)
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
INSERT INTO articles (nom, reference, quantite, seuil_alerte, actif)
VALUES ('Article absent', 'REF-ABSENT-01', 50, 5, true);"

# 3. Rallumer Ubuntu (pg_basebackup si pg-data effacé, sinon WAL replay)
docker compose up -d   # sur Ubuntu

# 4. Vérifier le rattrapage
docker exec pg-node psql -U metier -d metier -c "SELECT nom, reference FROM articles WHERE reference='REF-ABSENT-01';"
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
| `UNIQUE(reference)`| Aucun doublon de référence, même en concurrence               |
| `NOT NULL` + `CHECK`| Données obligatoires et valides avant insertion              |
| Transactions ACID  | Pas de lecture partielle, pas de corruption                   |
| WAL streaming sync | Ubuntu reçoit chaque écriture de Kali en < 100 ms             |
| `standby.signal`   | Ubuntu refuse les écritures — ne peut pas diverger            |
| Epoch token        | L'ancien primaire est bloqué après promotion du standby       |

**Cas du doublon concurrent** :
- Alice tente d'insérer REF-001 à T=0ms
- Bob tente d'insérer REF-001 à T=1ms
- PostgreSQL sérialise : Alice réussit, Bob reçoit une erreur de contrainte
- Aucun doublon n'atteint jamais le WAL → Ubuntu ne reçoit jamais le doublon
