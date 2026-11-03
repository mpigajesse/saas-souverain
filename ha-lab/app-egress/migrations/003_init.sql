-- Migration 003 : Schéma pour app-egress (sortie réseau contrôlée)
-- Appliquée par pre_start.sh au premier démarrage

CREATE TABLE IF NOT EXISTS schema_version (
    version INTEGER PRIMARY KEY,
    applied_at TIMESTAMPTZ DEFAULT NOW()
);

INSERT INTO schema_version (version) VALUES (3)
ON CONFLICT (version) DO NOTHING;

CREATE TABLE IF NOT EXISTS egress_logs (
    id BIGSERIAL PRIMARY KEY,
    domain TEXT NOT NULL,
    status TEXT NOT NULL,  -- 'success' | 'blocked' | 'error'
    latency_ms BIGINT,
    via_proxy BOOLEAN DEFAULT false,
    error_msg TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_egress_logs_domain ON egress_logs(domain);
CREATE INDEX IF NOT EXISTS idx_egress_logs_status ON egress_logs(status);
CREATE INDEX IF NOT EXISTS idx_egress_logs_created_at ON egress_logs(created_at);