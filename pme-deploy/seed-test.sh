#!/bin/bash
# Seed de test PME — 2 employés + articles + mouvements
# Usage : bash seed-test.sh
# À exécuter sur le nœud PRIMAIRE (Kali)

set -e

COMPOSE_DIR="/opt/elbaraa-pme"

echo "=== Création des employés ==="

docker compose -f "$COMPOSE_DIR/docker-compose.yml" run --rm \
  -e TENANT_ID="${TENANT_ID:-}" \
  -e REGISTRATION_TOKEN="${REGISTRATION_TOKEN:-}" \
  ss-node adduser \
    --username alice.martin \
    --name "Alice Martin" \
    --role employee \
    --password "Employe1!"

docker compose -f "$COMPOSE_DIR/docker-compose.yml" run --rm \
  -e TENANT_ID="${TENANT_ID:-}" \
  -e REGISTRATION_TOKEN="${REGISTRATION_TOKEN:-}" \
  ss-node adduser \
    --username bob.dupont \
    --name "Bob Dupont" \
    --role employee \
    --password "Employe2!"

echo ""
echo "=== Seed articles + mouvements dans PostgreSQL ==="

docker exec pg-node psql -U pme -d pme_db -c "
-- Articles
INSERT INTO articles (nom, reference, quantite, seuil_alerte, actif)
VALUES
  ('Stylos bleus (boîte 50)',  'REF-STYLO-B50',  200, 20, true),
  ('Ramettes A4 80g (500f)',   'REF-PAPIER-A4',  150, 30, true),
  ('Classeurs noirs A4',       'REF-CLASS-NR',    80, 10, true)
ON CONFLICT (reference) DO NOTHING;
"

docker exec pg-node psql -U pme -d pme_db -c "
-- Mouvement d'entrée (réception commande)
INSERT INTO mouvements (article_id, type_mouvement, quantite, notes)
SELECT id, 'entree', 50, 'Réception commande fournisseur'
FROM articles WHERE reference = 'REF-STYLO-B50';

-- Mouvement de sortie (utilisation)
INSERT INTO mouvements (article_id, type_mouvement, quantite, notes)
SELECT id, 'sortie', 10, 'Remis au bureau RH'
FROM articles WHERE reference = 'REF-STYLO-B50';

INSERT INTO mouvements (article_id, type_mouvement, quantite, notes)
SELECT id, 'sortie', 5, 'Remis au service compta'
FROM articles WHERE reference = 'REF-PAPIER-A4';

-- Mise à jour des quantités
UPDATE articles SET quantite = quantite + 50 WHERE reference = 'REF-STYLO-B50';
UPDATE articles SET quantite = quantite - 10 WHERE reference = 'REF-STYLO-B50';
UPDATE articles SET quantite = quantite - 5  WHERE reference = 'REF-PAPIER-A4';
"

echo ""
echo "=== Données sur le PRIMAIRE (Kali) ==="
docker exec pg-node psql -U pme -d pme_db -c "
SELECT username, full_name, role, is_active FROM users ORDER BY created_at;
"
docker exec pg-node psql -U pme -d pme_db -c "
SELECT nom, reference, quantite FROM articles ORDER BY nom;
"
docker exec pg-node psql -U pme -d pme_db -c "
SELECT a.nom, m.type_mouvement, m.quantite, m.notes, m.created_at
FROM mouvements m JOIN articles a ON a.id = m.article_id
ORDER BY m.created_at;
"

echo ""
echo "=== Vérification réplication sur Ubuntu (192.168.200.130) ==="
echo "  Connecte-toi sur Ubuntu et lance :"
echo "  docker exec pg-node psql -U pme -d pme_db -c \"SELECT username, full_name FROM users;\""
echo "  docker exec pg-node psql -U pme -d pme_db -c \"SELECT nom, quantite FROM articles;\""
echo "  docker exec pg-node psql -U pme -d pme_db -c \"SELECT COUNT(*) as mouvements FROM mouvements;\""
echo ""
echo "✓ Seed terminé — connecte-toi avec alice.martin / Employe1! ou bob.dupont / Employe2!"
