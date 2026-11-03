-- Migration 001 : Schéma initial pour app-migrations
-- Appliquée par pre_start.sh au premier démarrage

-- Table de versionnement du schéma
CREATE TABLE IF NOT EXISTS schema_version (
    version INTEGER PRIMARY KEY,
    applied_at TIMESTAMPTZ DEFAULT NOW()
);

-- Table métier : compteur de visites (comme application-exemple)
CREATE TABLE IF NOT EXISTS visites (
    id SERIAL PRIMARY KEY,
    le TIMESTAMPTZ DEFAULT NOW()
);

-- Insérer version initiale (idempotent)
INSERT INTO schema_version (version) VALUES (1)
ON CONFLICT (version) DO NOTHING;

-- Index pour performances (optionnel)
CREATE INDEX IF NOT EXISTS idx_visites_le ON visites(le);