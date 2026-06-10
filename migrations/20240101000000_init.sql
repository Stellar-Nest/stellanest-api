-- Stellanest initial schema migration

-- Users table
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    stellar_account VARCHAR(56) NOT NULL UNIQUE,
    kyc_status TEXT,
    kyc_tier INTEGER,
    trading_tier INTEGER,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- City indices table
CREATE TABLE IF NOT EXISTS city_indices (
    city_code VARCHAR(10) PRIMARY KEY,
    name TEXT NOT NULL,
    country TEXT NOT NULL,
    flag_emoji TEXT,
    description TEXT,
    current_value DOUBLE PRECISION NOT NULL DEFAULT 0,
    change_24h DOUBLE PRECISION,
    change_7d DOUBLE PRECISION,
    change_30d DOUBLE PRECISION,
    change_1y DOUBLE PRECISION,
    volatility_30d DOUBLE PRECISION,
    volume_24h DOUBLE PRECISION,
    open_interest DOUBLE PRECISION,
    data_source_count INTEGER,
    stellar_asset_code TEXT,
    stellar_asset_issuer TEXT,
    status TEXT DEFAULT 'active',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Index history table
CREATE TABLE IF NOT EXISTS index_history (
    id SERIAL PRIMARY KEY,
    time TIMESTAMPTZ NOT NULL,
    city_code VARCHAR(10) NOT NULL REFERENCES city_indices(city_code),
    open DOUBLE PRECISION,
    high DOUBLE PRECISION,
    low DOUBLE PRECISION,
    close DOUBLE PRECISION,
    volume DOUBLE PRECISION,
    source_count INTEGER
);

CREATE INDEX IF NOT EXISTS idx_index_history_city_time ON index_history (city_code, time DESC);

-- Positions table
CREATE TABLE IF NOT EXISTS positions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    user_wallet VARCHAR(56) NOT NULL,
    city_code VARCHAR(10),
    direction TEXT NOT NULL,
    leverage INTEGER NOT NULL DEFAULT 1,
    entry_price DOUBLE PRECISION NOT NULL,
    collateral DOUBLE PRECISION NOT NULL,
    size DOUBLE PRECISION NOT NULL,
    liquidation_price DOUBLE PRECISION,
    current_price DOUBLE PRECISION,
    unrealized_pnl DOUBLE PRECISION,
    realized_pnl DOUBLE PRECISION,
    funding_paid DOUBLE PRECISION,
    health_factor DOUBLE PRECISION,
    status TEXT DEFAULT 'open',
    close_price DOUBLE PRECISION,
    close_reason TEXT,
    soroban_position_id TEXT,
    open_tx_hash TEXT,
    close_tx_hash TEXT,
    opened_at TIMESTAMPTZ DEFAULT NOW(),
    closed_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_positions_user ON positions (user_id);
CREATE INDEX IF NOT EXISTS idx_positions_status ON positions (status);

-- Orders table
CREATE TABLE IF NOT EXISTS orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    user_wallet VARCHAR(56) NOT NULL,
    city_code VARCHAR(10),
    side TEXT NOT NULL,
    type TEXT NOT NULL,
    price DOUBLE PRECISION,
    size DOUBLE PRECISION NOT NULL,
    filled DOUBLE PRECISION DEFAULT 0,
    status TEXT DEFAULT 'open',
    stellar_tx_hash TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    filled_at TIMESTAMPTZ,
    cancelled_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_orders_user ON orders (user_id);
CREATE INDEX IF NOT EXISTS idx_orders_status ON orders (status);
CREATE INDEX IF NOT EXISTS idx_orders_city ON orders (city_code);

-- Trades table
CREATE TABLE IF NOT EXISTS trades (
    id SERIAL PRIMARY KEY,
    time TIMESTAMPTZ NOT NULL,
    city_code VARCHAR(10) NOT NULL,
    price DOUBLE PRECISION NOT NULL,
    size DOUBLE PRECISION NOT NULL,
    side TEXT NOT NULL,
    buyer_wallet VARCHAR(56),
    seller_wallet VARCHAR(56),
    tx_hash TEXT
);

CREATE INDEX IF NOT EXISTS idx_trades_city_time ON trades (city_code, time DESC);

-- Oracle submissions table
CREATE TABLE IF NOT EXISTS oracle_submissions (
    id BIGSERIAL PRIMARY KEY,
    oracle_wallet VARCHAR(56) NOT NULL,
    city_code VARCHAR(10) NOT NULL,
    price DOUBLE PRECISION NOT NULL,
    confidence INTEGER NOT NULL,
    source TEXT NOT NULL,
    accepted BOOLEAN,
    timestamp TIMESTAMPTZ NOT NULL,
    submitted_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_oracle_submissions_city ON oracle_submissions (city_code, submitted_at DESC);

-- Liquidations table
CREATE TABLE IF NOT EXISTS liquidations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    position_id UUID REFERENCES positions(id),
    user_wallet VARCHAR(56) NOT NULL,
    city_code VARCHAR(10),
    direction TEXT,
    entry_price DOUBLE PRECISION,
    liquidation_price DOUBLE PRECISION,
    collateral DOUBLE PRECISION,
    collateral_seized DOUBLE PRECISION,
    penalty DOUBLE PRECISION,
    tx_hash TEXT,
    liquidated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_liquidations_position ON liquidations (position_id);
