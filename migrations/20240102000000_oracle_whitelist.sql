CREATE TABLE IF NOT EXISTS oracle_whitelist (
    wallet_address VARCHAR(56) PRIMARY KEY,
    label TEXT,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Seed with a placeholder for development
INSERT INTO oracle_whitelist (wallet_address, label) VALUES
    ('DEV_ORACLE_KEY', 'Development Oracle')
ON CONFLICT DO NOTHING;
