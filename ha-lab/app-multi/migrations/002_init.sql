-- Migration 002 : Schéma pour app-multi (web + worker + Redis)
-- Appliquée par pre_start.sh au premier démarrage
-- Doit être idempotente et créer schema_version si absent

-- Table de versionnement du schéma (créée si pas existante)
CREATE TABLE IF NOT EXISTS schema_version (
    version INTEGER PRIMARY KEY,
    applied_at TIMESTAMPTZ DEFAULT NOW()
);

-- Insérer version 2 (idempotent)
INSERT INTO schema_version (version) VALUES (2)
ON CONFLICT (version) DO NOTHING;

-- Table pour stocker les résultats des jobs traités par le worker
CREATE TABLE IF NOT EXISTS jobs_results (
    id BIGSERIAL PRIMARY KEY,
    job_id TEXT NOT NULL,
    task TEXT NOT NULL,
    payload JSONB,
    result JSONB,
    status TEXT NOT NULL,  -- 'success' | 'failed'
    error TEXT,
    duration_ms BIGINT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Index pour requêtes fréquentes
CREATE INDEX IF NOT EXISTS idx_jobs_results_job_id ON jobs_results(job_id);
CREATE INDEX IF NOT EXISTS idx_jobs_results_status ON jobs_results(status);
CREATE INDEX IF NOT EXISTS idx_jobs_results_created_at ON jobs_results(created_at);