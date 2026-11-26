-- Table de test dédiée A-07 (supprimée en fin de tâche)
DROP TABLE IF EXISTS a07_replication_test;
CREATE TABLE a07_replication_test (
  id serial PRIMARY KEY,
  genere_le timestamptz NOT NULL DEFAULT clock_timestamp(),
  libelle text NOT NULL
);
INSERT INTO a07_replication_test (libelle)
SELECT 'ligne-' || g FROM generate_series(1,1000) g;
SELECT count(*) AS lignes_inserees FROM a07_replication_test;
SELECT pg_current_wal_lsn() AS lsn_primaire_apres_ecriture;
