# Survival Trading Bot — Technical Architecture & System Design

> **Version:** 1.0 | **Date:** February 2026 | **Author:** Maan | **Status:** Draft

---

## Table of Contents

1. [System Architecture Overview](#1-system-architecture-overview)
2. [Technology Stack Recommendations](#2-technology-stack-recommendations)
3. [Detailed Component Design](#3-detailed-component-design)
4. [Design Patterns & Best Practices](#4-design-patterns--best-practices)
5. [Non-Functional Requirements](#5-non-functional-requirements)
6. [Integration Strategy](#6-integration-strategy)
7. [Data Architecture](#7-data-architecture)
8. [Deployment Architecture](#8-deployment-architecture)
9. [Monitoring & Observability](#9-monitoring--observability)
10. [Risk Assessment & Mitigation](#10-risk-assessment--mitigation)

---

## 1. System Architecture Overview

### 1.1 Executive Summary

The Survival Trading Bot is an autonomous cryptocurrency trading system built on a survival-game mechanic: starting with 100 SAR (~$27 USDC), the bot must grow or preserve its capital through halal spot trading. If the balance reaches zero, the bot terminates permanently. The system uses OpenClaw (Claude via Discord) as its decision-making brain, a Rust backend for high-performance execution, Binance for order management, and a Next.js dashboard for real-time monitoring.

### 1.2 Architectural Style

The system follows a **simplified event-driven monolith** architecture deployed on a single VPS. This approach was chosen over microservices for several reasons:

- The system has a single bounded context (trading)
- The team size is one developer
- Operational simplicity is critical for a real-money system
- The 10-minute cycle does not require extreme horizontal scalability

> **Design Philosophy:** Simple, effective, easy to modify. A single Rust binary handles all backend logic. The frontend is a statically-deployed Next.js app on Vercel. No Kubernetes, no service mesh, no over-engineering.

### 1.3 System Context

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           SURVIVAL TRADING SYSTEM                           │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   ┌──────────────┐     ┌──────────────┐     ┌──────────────────────────┐   │
│   │   TRIGGER    │────▶│   RUST API   │────▶│       OPENCLAW           │   │
│   │  (10 min)    │     │   BACKEND    │     │    (Trading Brain)       │   │
│   └──────────────┘     └──────┬───────┘     └────────────┬─────────────┘   │
│                               │                          │                  │
│                               ▼                          ▼                  │
│                        ┌──────────────┐          ┌──────────────┐          │
│                        │   BINANCE    │◀─────────│   DECISION   │          │
│                        │   EXCHANGE   │          │  BUY/SELL/   │          │
│                        └──────────────┘          │    HOLD      │          │
│                               │                  └──────────────┘          │
│                               ▼                                             │
│                        ┌──────────────┐     ┌──────────────────────────┐   │
│                        │  POSTGRESQL  │────▶│       DASHBOARD          │   │
│                        │   DATABASE   │     │   (Next.js / Vercel)     │   │
│                        └──────────────┘     └──────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

The system interacts with four external services:

| External System | Purpose | Protocol | Frequency |
|---|---|---|---|
| Binance API | Exchange: balances, prices, order execution | REST (HMAC-SHA256) | Every 10 min + on trade |
| Discord API | Communication channel to OpenClaw (Claude) | REST + Polling | Every 10 min |
| Fear & Greed API | Market sentiment indicator | REST (public) | Every 10 min |
| PostgreSQL | Persistent storage for trades, logs, state | TCP (local) | Every 10 min + queries |

### 1.4 Component Diagram

| Component | Technology | Responsibility | Deployment |
|---|---|---|---|
| Trading Engine | Rust (Tokio) | Scheduling, data aggregation, trade execution | VPS |
| API Server | Rust (Axum) | REST endpoints, WebSocket server for dashboard | VPS (same binary) |
| Decision Engine | OpenClaw (Claude) | Market analysis, buy/sell/hold decisions | Discord (external) |
| Dashboard | Next.js + React + Tailwind | Real-time monitoring, kill switch, trade history | Vercel |
| Database | PostgreSQL 16 | Trade logs, positions, balance history, bot state | VPS (Docker) |

### 1.5 Data Flow Architecture

Every 10 minutes, the following sequence executes:

1. Cron trigger fires in the Rust scheduler
2. Rust fetches current USDC balance, open positions, and 24h ticker data from Binance
3. Rust fetches the Fear & Greed Index from the public API
4. All data is formatted into a structured prompt and sent to OpenClaw via Discord
5. OpenClaw analyzes conditions and responds with a JSON decision (BUY/SELL/HOLD)
6. Rust parses the response, validates it, and executes the trade on Binance if applicable
7. The cycle result, balance, and decision reasoning are logged to PostgreSQL
8. A WebSocket broadcast notifies the dashboard of the update

```
Every 10 Minutes:

  CRON TRIGGER
       │
       ▼
  RUST BACKEND
       │
       ├──▶ Fetch balance from Binance
       ├──▶ Fetch open positions
       ├──▶ Fetch prices (BTC, ETH, SOL, LTC)
       ├──▶ Fetch market data (24h change, volume)
       ├──▶ Fetch Fear & Greed Index
       │
       ▼
  BUILD CONTEXT PAYLOAD
       │
       ▼
  SEND TO OPENCLAW (Discord)
       │
       ▼
  OPENCLAW ANALYZES → Returns JSON { action, symbol, amount, confidence, reasoning }
       │
       ▼
  RUST PARSES & VALIDATES
       │
       ├──▶ BUY  → Execute market buy on Binance
       ├──▶ SELL → Execute market sell on Binance
       ├──▶ HOLD → Do nothing
       │
       ▼
  LOG TO POSTGRESQL
       │
       ▼
  BROADCAST VIA WEBSOCKET → Dashboard updates
```

---

## 2. Technology Stack Recommendations

### 2.1 Backend: Rust

**Language:** Rust (2021 edition). Chosen for microsecond-level latency on trade execution, memory safety without garbage collection pauses (critical during volatile markets), strong type system that catches errors at compile time, and excellent async ecosystem via Tokio.

| Crate | Version | Purpose |
|---|---|---|
| tokio | 1.35 | Async runtime with full feature set |
| axum | 0.7 | HTTP framework with WebSocket support |
| reqwest | 0.11 | HTTP client for Binance and Discord APIs |
| sqlx | 0.7 | Compile-time checked SQL queries with PostgreSQL |
| serde / serde_json | 1.0 | JSON serialization/deserialization |
| hmac / sha2 | 0.12 / 0.10 | HMAC-SHA256 for Binance API authentication |
| chrono | 0.4 | Date/time handling |
| tracing | 0.1 | Structured logging |
| anyhow / thiserror | 1.0 | Error handling (anyhow for app, thiserror for libraries) |
| tokio-cron-scheduler | 0.10 | 10-minute interval scheduling |

### 2.2 Frontend: Next.js on Vercel

**Framework:** Next.js 14 with App Router, deployed to Vercel. This gives you zero-config deployment, automatic HTTPS, edge caching for static assets, and instant rollbacks. The dashboard is a single-page app that connects to the Rust backend via REST and WebSocket.

| Library | Purpose |
|---|---|
| React 18 | UI rendering |
| TailwindCSS | Utility-first styling, dark theme |
| Recharts | Balance history and P&L charts |
| Native WebSocket | Real-time updates from Rust backend |
| SWR or React Query | Data fetching with automatic revalidation |

### 2.3 Database: PostgreSQL 16

PostgreSQL was chosen for ACID compliance (critical for financial records), excellent JSON support for storing raw OpenClaw responses, mature tooling and easy backup/restore, and the fact that the dataset is small enough that a single instance handles everything. The database runs in a Docker container on the same VPS as the Rust backend, eliminating network latency for queries.

### 2.4 Infrastructure Summary

| Layer | Technology | Cost | Justification |
|---|---|---|---|
| Compute | VPS (Hetzner CX22 or DigitalOcean Basic) | $4–6/mo | 2 vCPU, 4GB RAM — more than enough |
| Frontend Hosting | Vercel (Hobby plan) | Free | Auto-deploy from Git, CDN, HTTPS |
| Database | PostgreSQL 16 (Docker on VPS) | $0 (included) | Runs alongside Rust backend |
| Domain + SSL | Cloudflare (free tier) | Free | DNS, SSL termination, basic DDoS protection |
| Monitoring | Uptime Kuma (self-hosted) | $0 (included) | Health checks, alerts via Discord webhook |

> **Total Monthly Cost:** Approximately $5–10/month for the entire system. The bot needs to make at least $10/month just to break even on infrastructure.

---

## 3. Detailed Component Design

### 3.1 API Layer

The Rust backend exposes a lightweight REST API via Axum. All endpoints are unauthenticated for v1 (the dashboard is not publicly exposed; access is controlled via VPS firewall rules and Cloudflare Access if needed).

| Method | Endpoint | Description | Response |
|---|---|---|---|
| GET | `/health` | Liveness check | `{ status, timestamp }` |
| GET | `/status` | Bot status, balance, P&L, position count | `StatusResponse` |
| GET | `/trades` | Recent 50 trades with details | `Trade[]` |
| GET | `/balance` | Balance history (last 100 snapshots) | `BalanceSnapshot[]` |
| POST | `/trigger` | Manually trigger a trading cycle | Confirmation string |
| POST | `/kill` | Emergency kill switch — marks bot as dead | Confirmation string |
| GET | `/ws` | WebSocket upgrade for real-time cycle updates | WebSocket stream |

### 3.2 Business Logic Layer

#### Trading Engine

The `TradingEngine` struct orchestrates each 10-minute cycle. It is **stateless**: all state is read from the database and Binance at the start of each cycle, and written back after execution. This makes the system resilient to restarts.

**Key design decision:** The engine creates a fresh instance per cycle rather than maintaining long-lived state. This prevents stale data bugs and makes the system trivially restartable.

#### Decision Parsing

OpenClaw responses are parsed with a two-stage approach: first, a regex extracts JSON from potential markdown code blocks; second, `serde_json` deserializes into a strongly-typed `TradingDecision` struct. If parsing fails, the engine defaults to HOLD — the safest action.

#### Position Sizing

Position size scales with confidence level from OpenClaw:

| Confidence | Position Size (% of tradeable balance) | Max USDC |
|---|---|---|
| 90–100% | 10% | ~$2.20 |
| 80–89% | 6% | ~$1.32 |
| 70–79% | 3% | ~$0.66 |
| Below 70% | 0% (forced HOLD) | $0 |

### 3.3 Data Layer

#### Database Schema

Five tables handle all persistent state:

| Table | Purpose | Key Fields |
|---|---|---|
| `bot_status` | Single-row table tracking alive/dead state | `is_dead`, `death_reason`, `updated_at` |
| `positions` | Open and closed trading positions | `symbol`, `quantity`, `entry_price`, `stop_loss`, `take_profit`, `status` |
| `trades` | Individual buy/sell execution records | `symbol`, `side`, `quantity`, `price`, `usdc_amount`, `executed_at` |
| `cycle_logs` | Full log of every 10-minute decision cycle | `balance`, `action`, `confidence`, `reasoning`, `result` |
| `balance_history` | Time-series balance snapshots for charting | `balance_usdc`, `recorded_at` |

#### Caching Strategy

No application-level cache is needed for v1. The 10-minute cycle frequency is low enough that direct database and API queries are sufficient. If the dashboard sees heavy traffic, SWR/React Query on the frontend provides client-side caching with stale-while-revalidate semantics.

### 3.4 Integration Layer

#### Binance Integration

All Binance API calls use HMAC-SHA256 authentication. The client is configured with spot-trading-only permissions and zero withdrawal permissions for security. Market buys use `quoteOrderQty` (specifying USDC amount) while sells use `quantity` (specifying coin amount).

#### OpenClaw (Discord) Integration

Communication with OpenClaw happens via Discord's REST API. The Rust backend sends a message to a dedicated channel, then polls for a response from OpenClaw's user ID. The polling timeout is 60 seconds with 2-second intervals. If no response arrives, the cycle defaults to HOLD.

> **Important:** This Discord-based integration means OpenClaw must be active in the channel. If OpenClaw goes offline or Discord has an outage, the bot simply holds all positions — the safest default.

---

## 4. Design Patterns & Best Practices

### 4.1 Architectural Patterns

| Pattern | Where Used | Why |
|---|---|---|
| Shared State (`Arc<AppState>`) | Across all handlers and scheduler | Thread-safe shared access to config, DB pool, broadcast channel |
| Repository Pattern | `db` module (`queries.rs`) | Abstracts SQL behind clean async functions |
| Strategy Pattern | Position sizing based on confidence | Easily swappable sizing algorithms |
| Observer Pattern | WebSocket broadcast channel | Dashboard subscribes to real-time cycle updates |
| Fail-Safe Default | Parser and engine | Any error defaults to HOLD (do nothing) |

### 4.2 Project Structure

The Rust project is organized by domain concern. Each module has a `mod.rs` that re-exports public types:

```
survival-bot/
├── Cargo.toml
├── .env
├── src/
│   ├── main.rs              → Entry point, server setup, state initialization
│   ├── config.rs            → Environment variable loading
│   ├── lib.rs
│   │
│   ├── api/
│   │   ├── mod.rs
│   │   ├── routes.rs        → HTTP routes (health, status, trades, kill)
│   │   └── websocket.rs     → WebSocket handler for dashboard
│   │
│   ├── binance/
│   │   ├── mod.rs
│   │   ├── client.rs        → Exchange client, HMAC auth, order execution
│   │   ├── types.rs         → Binance API response types
│   │   └── orders.rs        → Market buy/sell helpers
│   │
│   ├── openclaw/
│   │   ├── mod.rs
│   │   ├── discord.rs       → Discord messaging, polling for responses
│   │   ├── parser.rs        → JSON extraction and validation
│   │   └── prompts.rs       → Prompt template builder
│   │
│   ├── trading/
│   │   ├── mod.rs
│   │   ├── engine.rs        → Main cycle orchestration
│   │   ├── strategy.rs      → Position sizing logic
│   │   └── risk.rs          → Stop-loss, take-profit, max position checks
│   │
│   ├── market/
│   │   ├── mod.rs
│   │   ├── data.rs          → Fear & Greed Index fetcher
│   │   └── indicators.rs    → Market data aggregation
│   │
│   ├── db/
│   │   ├── mod.rs
│   │   ├── models.rs        → Structs matching DB tables
│   │   └── queries.rs       → All SQL operations
│   │
│   └── scheduler/
│       ├── mod.rs
│       └── cron.rs          → 10-minute interval loop
│
└── migrations/
    └── 001_initial.sql
```

### 4.3 Error Handling Strategy

The system uses a layered error handling approach:

- **Library-level errors** use `thiserror` for typed, specific error variants (e.g., `BinanceError`, `ParseError`)
- **Application-level errors** use `anyhow` for ergonomic error chaining with context
- **Critical principle:** No error should cause a trade to execute incorrectly. Errors always result in HOLD.
- All errors are logged with `tracing` and stored in `cycle_logs` for post-mortem analysis

---

## 5. Non-Functional Requirements

### 5.1 Scalability

Scalability is not a primary concern for this system. The bot executes one cycle every 10 minutes, generating at most 144 database rows per day. A single VPS with 2 vCPU and 4GB RAM can handle this workload for years without any scaling intervention.

**If scaling becomes necessary:** The stateless engine design means you could run multiple bot instances with different coin sets. The database would be the only shared resource, and PostgreSQL handles concurrent writes well at this volume.

### 5.2 Performance

| Metric | Target | Rationale |
|---|---|---|
| Cycle execution time | < 10 seconds | Well within the 10-minute window |
| Trade execution latency | < 500ms | Market orders execute near-instantly on Binance |
| API response time | < 100ms | Dashboard queries hit local PostgreSQL |
| WebSocket broadcast | < 50ms | In-process broadcast channel, near-zero latency |
| OpenClaw response time | < 30 seconds | Discord polling with 60s timeout |

### 5.3 Security

#### API Key Security

- Binance API keys are restricted to spot trading only with **zero withdrawal permissions**
- All secrets are stored in environment variables, never in code or version control
- The `.env` file has `600` permissions (owner read/write only)

#### Network Security

- VPS firewall allows only ports 22 (SSH), 80/443 (HTTP/HTTPS via reverse proxy), and 3001 (restricted to Vercel IP ranges)
- PostgreSQL is bound to `localhost` only — no external access
- All external API calls use HTTPS/TLS
- Cloudflare provides DDoS protection and SSL termination

#### Application Security

- Input validation on all OpenClaw responses before trade execution
- The kill switch endpoint should be protected with a shared secret header for v1
- CORS is configured to allow only the Vercel dashboard domain in production

### 5.4 Reliability & Availability

The system is designed for "good enough" reliability, not five-nines. The consequences of downtime are minimal: the bot simply misses cycles and holds its positions.

| Scenario | Impact | Recovery |
|---|---|---|
| VPS crash/reboot | Missed cycles during downtime | systemd auto-restarts Docker containers; stateless engine resumes on next tick |
| Binance API outage | Cannot fetch data or execute trades | Cycle fails gracefully, defaults to HOLD, retries next tick |
| Discord/OpenClaw offline | Cannot get trading decisions | 60s timeout triggers, defaults to HOLD |
| PostgreSQL crash | Cannot log data; cycle may fail | Docker restart policy recovers; WAL ensures no data loss |
| Rust backend panic | Current cycle aborted | Process supervisor (systemd) restarts within seconds |

#### Backup Strategy

- Daily `pg_dump` to a local backup directory
- Weekly backup upload to cloud storage (Backblaze B2, ~$0.005/GB)
- Database is small (< 100MB even after months), so full backups are trivial

### 5.5 Maintainability

- Rust's type system and compile-time checks catch most bugs before deployment
- Structured logging via `tracing` makes debugging straightforward
- The modular project structure allows changing any component without affecting others
- Easy modification: swap OpenClaw for direct Claude API, add new coins, change intervals — each requires editing a single module

---

## 6. Integration Strategy

### 6.1 Binance API Integration

The Binance client handles three types of operations:

| Operation | Endpoint | Auth | Notes |
|---|---|---|---|
| Account balance | `GET /api/v3/account` | HMAC-SHA256 | Returns all asset balances; filter for USDC |
| 24h ticker | `GET /api/v3/ticker/24hr` | None | Public endpoint, no auth needed |
| Market buy | `POST /api/v3/order` | HMAC-SHA256 | Uses `quoteOrderQty` for USDC-denominated buys |
| Market sell | `POST /api/v3/order` | HMAC-SHA256 | Uses `quantity` for coin-denominated sells |

> **Rate Limits:** Binance allows 1,200 requests per minute. At 4–6 requests per 10-minute cycle, the bot uses less than 0.5% of the limit. Rate limiting is not a concern.

### 6.2 OpenClaw (Discord) Integration

The communication pattern is request-response over Discord messages:

1. Rust bot sends a structured prompt to the designated Discord channel
2. OpenClaw (monitoring the channel) analyzes and responds with JSON
3. Rust polls for responses after the sent message ID, filtering by OpenClaw's user ID
4. If no valid response within 60 seconds, the cycle defaults to HOLD

#### Future Migration Path

The Discord-based integration is a v1 approach. A cleaner v2 would call the Claude API directly via the Anthropic SDK, eliminating Discord as a middleman. The `openclaw` module is isolated, so this migration requires changing only `discord.rs` — no other code is affected.

### 6.3 Dashboard ↔ Backend Communication

The Next.js dashboard on Vercel communicates with the Rust backend on the VPS via two channels:

- **REST API:** For initial data loading and actions (kill switch, manual trigger). The `NEXT_PUBLIC_API_URL` environment variable points to the VPS domain.
- **WebSocket:** For real-time cycle notifications. The dashboard opens a persistent connection and receives a broadcast on every cycle completion.

---

## 7. Data Architecture

### 7.1 Entity Relationship Overview

The schema is intentionally simple with five tables. The `positions` table is the central entity: `trades` reference positions, `cycle_logs` capture the decision context, and `balance_history` provides the time-series data for charting.

```
bot_status (1 row)
    │
    │  (checked each cycle)
    │
positions ◄──── trades
    │               │
    │               │ (references position_id)
    │               │
cycle_logs      balance_history
    │               │
    │               │ (independent time-series)
    └───────────────┘
```

### 7.2 Full Schema

```sql
-- Bot status
CREATE TABLE bot_status (
    id SERIAL PRIMARY KEY,
    is_dead BOOLEAN NOT NULL DEFAULT FALSE,
    death_reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO bot_status (is_dead) VALUES (FALSE);

-- Positions (open trades)
CREATE TABLE positions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    symbol VARCHAR(20) NOT NULL,
    quantity DECIMAL(20, 8) NOT NULL,
    entry_price DECIMAL(20, 8) NOT NULL,
    current_price DECIMAL(20, 8),
    stop_loss_price DECIMAL(20, 8),
    take_profit_price DECIMAL(20, 8),
    status VARCHAR(20) NOT NULL DEFAULT 'OPEN',
    opened_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    closed_at TIMESTAMPTZ,
    close_price DECIMAL(20, 8),
    pnl_usdc DECIMAL(20, 8),
    pnl_percent DECIMAL(10, 4)
);

-- Trade history
CREATE TABLE trades (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    position_id UUID REFERENCES positions(id),
    symbol VARCHAR(20) NOT NULL,
    side VARCHAR(10) NOT NULL,
    quantity DECIMAL(20, 8) NOT NULL,
    price DECIMAL(20, 8) NOT NULL,
    usdc_amount DECIMAL(20, 8) NOT NULL,
    executed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Cycle logs
CREATE TABLE cycle_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    balance_usdc DECIMAL(20, 8) NOT NULL,
    tradeable_balance DECIMAL(20, 8) NOT NULL,
    fear_greed_index INTEGER,
    action VARCHAR(10) NOT NULL,
    symbol VARCHAR(20),
    confidence INTEGER,
    reasoning TEXT,
    result VARCHAR(50),
    error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Balance snapshots
CREATE TABLE balance_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    balance_usdc DECIMAL(20, 8) NOT NULL,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_positions_status ON positions(status);
CREATE INDEX idx_trades_executed_at ON trades(executed_at);
CREATE INDEX idx_cycle_logs_created_at ON cycle_logs(created_at);
CREATE INDEX idx_balance_history_recorded_at ON balance_history(recorded_at);
```

### 7.3 Data Retention

| Data Type | Retention | Reasoning |
|---|---|---|
| `cycle_logs` | 90 days | Decision history for analysis; older data archived to JSON export |
| `balance_history` | Forever | Critical for long-term performance tracking; rows are tiny |
| `trades` | Forever | Financial records should be permanent |
| `positions` | Forever | Both open and closed positions kept for P&L calculation |
| `bot_status` | Single row | Only current state matters |

### 7.4 Data Consistency

Trade execution and database logging happen in sequence within the same async task. If the trade succeeds but the database write fails, the position will be out of sync. To mitigate this, the bot reconciles positions with Binance account data at the start of each cycle, using Binance as the source of truth for balances and using the local database as the source of truth for entry prices and stop-loss/take-profit levels.

---

## 8. Deployment Architecture

### 8.1 Environment Strategy

| Environment | Purpose | Infrastructure |
|---|---|---|
| Development | Local development and testing | `cargo run` + local PostgreSQL (Docker) |
| Testnet | Paper trading with Binance testnet | Same VPS or local; `BINANCE_BASE_URL` points to testnet |
| Production | Live trading with real funds | VPS + Vercel |

### 8.2 VPS Setup

The VPS runs three Docker containers managed by `docker-compose`:

1. **survival-bot:** The Rust binary compiled in a multi-stage Dockerfile (build stage: `rust:latest`, runtime stage: `debian:slim`)
2. **postgres:** PostgreSQL 16 Alpine with a persistent volume
3. **caddy** (optional): Reverse proxy for HTTPS termination if not using Cloudflare

#### Docker Compose

```yaml
version: '3.8'

services:
  bot:
    build: ./survival-bot
    env_file: .env
    depends_on:
      - postgres
    restart: unless-stopped

  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_USER: survival
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD}
      POSTGRES_DB: survival_bot
    volumes:
      - postgres_data:/var/lib/postgresql/data
    ports:
      - "127.0.0.1:5432:5432"

volumes:
  postgres_data:
```

#### Deployment Workflow

For simplicity, deployment is a manual SSH-and-pull process:

```bash
ssh vps
cd /opt/survival-bot
git pull origin main
docker-compose build bot
docker-compose up -d
```

For v2, a GitHub Actions workflow can automate this: push to `main` triggers a build, SSH deploy, and health check.

### 8.3 Vercel Frontend Deployment

The Next.js dashboard deploys automatically on every push to the `main` branch. The only required environment variable is `NEXT_PUBLIC_API_URL` pointing to the VPS backend (e.g., `https://bot.yourdomain.com`). Vercel handles builds, CDN distribution, and SSL automatically.

### 8.4 Environment Variables

```bash
# Database
DATABASE_URL=postgres://survival:password@localhost:5432/survival_bot

# Binance API
BINANCE_API_KEY=your_api_key_here
BINANCE_SECRET_KEY=your_secret_key_here
BINANCE_BASE_URL=https://api.binance.com

# Discord (for OpenClaw)
DISCORD_BOT_TOKEN=your_discord_bot_token
DISCORD_CHANNEL_ID=123456789012345678
OPENCLAW_USER_ID=987654321098765432

# Rust logging
RUST_LOG=info
```

---

## 9. Monitoring & Observability

### 9.1 Logging

The Rust backend uses the `tracing` crate with structured logging. Log levels are controlled via the `RUST_LOG` environment variable. In production, logs are written to stdout and captured by Docker's logging driver. For persistent logs, Docker is configured with the `json-file` driver with a 10MB max size and 3 rotated files.

### 9.2 Health Monitoring

| Check | Tool | Frequency | Alert Channel |
|---|---|---|---|
| HTTP health endpoint | Uptime Kuma | Every 1 minute | Discord webhook |
| Balance threshold | Custom (in trading engine) | Every cycle | Discord webhook + dashboard |
| Consecutive failures | Custom (`cycle_logs` query) | Every cycle | Discord webhook |
| VPS resource usage | Docker stats + htop | Manual / cron | Discord webhook if threshold exceeded |

### 9.3 Key Metrics to Track

- **Balance over time** (primary survival metric)
- **Win rate** (profitable trades / total trades)
- **Average profit per winning trade** vs. average loss per losing trade
- **OpenClaw response latency** and failure rate
- **Cycle execution time**
- **Consecutive losses streak** (triggers conservative mode)

### 9.4 Discord Alert Examples

The bot sends alerts to a dedicated Discord monitoring channel:

```
🚨 BALANCE ALERT: $12.50 USDC (below $15 threshold)
❌ CYCLE FAILED: Binance API timeout (3 consecutive failures)
☠️ BOT DEAD: Balance reached $0.00 at 2026-02-11T14:30:00Z
✅ TRADE EXECUTED: BUY 0.0003 BTC @ $67,234 ($2.17 USDC)
```

---

## 10. Risk Assessment & Mitigation

### 10.1 Technical Risks

| Risk | Severity | Likelihood | Mitigation |
|---|---|---|---|
| OpenClaw returns invalid/unparseable JSON | Medium | Medium | Regex + serde parsing with fallback to HOLD on any error |
| Binance API key compromised | Critical | Low | No withdrawal permissions; IP whitelist on Binance; rotate keys quarterly |
| Discord rate limiting blocks messages | Low | Low | Bot sends 1 message per 10 min; well within limits; add retry with backoff |
| VPS provider outage | Medium | Low | Positions are on Binance (safe); bot resumes on recovery; no stop-loss execution during downtime is the real risk |
| PostgreSQL data corruption | High | Very Low | Daily backups; WAL mode; Docker volume on reliable storage |

### 10.2 Financial Risks

| Risk | Severity | Mitigation |
|---|---|---|
| Flash crash wipes balance | Critical | 5% stop-loss per position; max 2 positions; max 10% per trade |
| Slow bleed from bad decisions | High | Consecutive loss tracking triggers ultra-conservative mode |
| Exchange insolvency | Critical | Only keep trading capital on Binance; withdraw profits periodically |
| API cost exceeds returns | Medium | $5 reserve deducted from tradeable balance; bot pauses if balance < $5 |

### 10.3 Dependency Risks

| Dependency | Risk | Mitigation |
|---|---|---|
| OpenClaw/Discord | Service unavailable | Timeout + HOLD default; future migration to direct Claude API |
| Binance | API changes or regional restrictions | Abstract behind `BinanceClient` trait; swappable to other exchanges |
| Fear & Greed API | Service down | Default to 50 (neutral) on failure; non-critical data |
| VPS provider | Price increase or discontinuation | Docker-based; migrate to any provider in < 1 hour |

---

## Quick Start Checklist

- [ ] Create Binance account and generate API keys (spot trading only, **no withdrawal**)
- [ ] Set up Discord server with OpenClaw
- [ ] Create Discord bot for message sending
- [ ] Provision VPS (Hetzner CX22 recommended)
- [ ] Install Docker & Docker Compose on VPS
- [ ] Configure `.env` file with all credentials
- [ ] Run `docker-compose up -d` on VPS
- [ ] Deploy Next.js dashboard to Vercel
- [ ] **Start with Binance testnet for paper trading**
- [ ] Validate full cycle end-to-end
- [ ] Fund account with 100 SAR (~$27 USDC)
- [ ] Switch `BINANCE_BASE_URL` to production
- [ ] Start the bot and watch it fight for survival

---


---

> **Final Note:** This architecture prioritizes simplicity and reliability over sophistication. Every design decision was made to minimize what can go wrong when real money is on the line. The system can be understood, modified, and debugged by a single developer. Start with paper trading on Binance testnet, validate the full cycle end-to-end, then switch to production with real funds.

---

*Survival Trading Bot — OpenClaw + Rust + Binance + Next.js Dashboard*