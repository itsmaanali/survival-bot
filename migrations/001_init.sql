-- ============================================
-- Survival Trading Bot — Initial Schema
-- ============================================

-- Bot status: single-row table tracking alive/dead state
CREATE TABLE IF NOT EXISTS bot_status (
    id          SERIAL PRIMARY KEY,
    is_dead     BOOLEAN NOT NULL DEFAULT FALSE,
    death_reason TEXT,
    started_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Insert the initial alive state
INSERT INTO bot_status (is_dead) VALUES (FALSE)
ON CONFLICT DO NOTHING;

-- Positions: open and closed trading positions
CREATE TABLE IF NOT EXISTS positions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    symbol          VARCHAR(20) NOT NULL,
    side            VARCHAR(4) NOT NULL CHECK (side IN ('BUY', 'SELL')),
    quantity        DOUBLE PRECISION NOT NULL,
    entry_price     DOUBLE PRECISION NOT NULL,
    current_price   DOUBLE PRECISION,
    stop_loss       DOUBLE PRECISION,
    take_profit     DOUBLE PRECISION,
    status          VARCHAR(10) NOT NULL DEFAULT 'OPEN' CHECK (status IN ('OPEN', 'CLOSED')),
    pnl             DOUBLE PRECISION,
    opened_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    closed_at       TIMESTAMPTZ,
    close_reason    TEXT
);

CREATE INDEX idx_positions_status ON positions(status);
CREATE INDEX idx_positions_symbol ON positions(symbol);

-- Trades: individual buy/sell execution records
CREATE TABLE IF NOT EXISTS trades (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    position_id     UUID REFERENCES positions(id),
    symbol          VARCHAR(20) NOT NULL,
    side            VARCHAR(4) NOT NULL CHECK (side IN ('BUY', 'SELL')),
    quantity        DOUBLE PRECISION NOT NULL,
    price           DOUBLE PRECISION NOT NULL,
    usdc_amount     DOUBLE PRECISION NOT NULL,
    commission      DOUBLE PRECISION DEFAULT 0.0,
    executed_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_trades_symbol ON trades(symbol);
CREATE INDEX idx_trades_executed_at ON trades(executed_at DESC);

-- Cycle logs: full log of every 10-minute decision cycle
CREATE TABLE IF NOT EXISTS cycle_logs (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    cycle_number    SERIAL,
    balance_usdc    DOUBLE PRECISION NOT NULL,
    action          VARCHAR(10) NOT NULL CHECK (action IN ('BUY', 'SELL', 'HOLD', 'ERROR')),
    symbol          VARCHAR(20),
    confidence      INTEGER,
    reasoning       TEXT,
    raw_response    TEXT,
    fear_greed      INTEGER,
    execution_ms    INTEGER,
    result          TEXT,
    error           TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_cycle_logs_created_at ON cycle_logs(created_at DESC);

-- Balance history: time-series balance snapshots for charting
CREATE TABLE IF NOT EXISTS balance_history (
    id              SERIAL PRIMARY KEY,
    balance_usdc    DOUBLE PRECISION NOT NULL,
    open_positions  INTEGER NOT NULL DEFAULT 0,
    total_pnl       DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    recorded_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_balance_history_recorded_at ON balance_history(recorded_at DESC);
