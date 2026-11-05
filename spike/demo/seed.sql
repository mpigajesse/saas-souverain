-- Stock fictif de la PME de démo « Atlas Distribution ».
INSERT INTO articles (code, nom, categorie, unite, prix_unitaire, seuil_alerte) VALUES
 ('ATL-001', 'Huile de tournesol 5 L', 'Épicerie', 'bidon', 89.0, 20),
 ('ATL-002', 'Farine de blé 25 kg', 'Épicerie', 'sac', 210.0, 15),
 ('ATL-003', 'Sucre en morceaux 1 kg', 'Épicerie', 'paquet', 14.5, 50),
 ('ATL-004', 'Eau minérale 1,5 L (pack de 6)', 'Boissons', 'pack', 33.0, 30),
 ('ATL-005', 'Thé vert 500 g', 'Épicerie', 'boîte', 42.0, 25),
 ('ATL-006', 'Savon de Marseille 400 g', 'Entretien', 'pièce', 18.0, 40),
 ('ATL-007', 'Lessive 3 kg', 'Entretien', 'baril', 75.0, 10),
 ('ATL-008', 'Riz long grain 5 kg', 'Épicerie', 'sac', 64.0, 20)
ON CONFLICT (code) DO NOTHING;

INSERT INTO mouvements_stock (article_id, type_mvt, quantite, reference, notes, created_at)
SELECT a.id, m.type_mvt, m.quantite, m.reference, 'Données fictives', NOW() - (m.jours || ' days')::interval
FROM articles a
JOIN (VALUES
 ('ATL-001','entree',120,'BL-2026-101','12'), ('ATL-001','sortie',-85,'FAC-2026-334','3'),
 ('ATL-002','entree',60,'BL-2026-102','11'),  ('ATL-002','sortie',-48,'FAC-2026-335','2'),
 ('ATL-003','entree',300,'BL-2026-103','10'), ('ATL-003','sortie',-140,'FAC-2026-340','1'),
 ('ATL-004','entree',200,'BL-2026-104','9'),  ('ATL-004','sortie',-185,'FAC-2026-341','1'),
 ('ATL-005','entree',80,'BL-2026-105','8'),
 ('ATL-006','entree',150,'BL-2026-106','7'),  ('ATL-006','sortie',-60,'FAC-2026-342','2'),
 ('ATL-007','entree',40,'BL-2026-107','6'),   ('ATL-007','sortie',-33,'FAC-2026-343','1'),
 ('ATL-008','entree',90,'BL-2026-108','5'),   ('ATL-008','ajustement',-2,'INV-2026-01','4')
) AS m(code, type_mvt, quantite, reference, jours) ON m.code = a.code;
