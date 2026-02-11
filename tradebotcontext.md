# Chat Conversation

Note: _This is purely the output of the chat conversation and does not contain any raw data, codebase snippets, etc. used to generate the output._

### User Input


create this SURVIVAL TRADING BOT

Technical Architecture & System Design

OpenClaw + Rust + Binance + Next.js

Version
1.0
Date
February 2026
Author
Maan
Status
Draft
Classification
Confidential


CONFIDENTIAL — For Internal Use Only

Table of Contents




1. System Architecture Overview
1.1 Executive Summary
The Survival Trading Bot is an autonomous cryptocurrency trading system built on a survival-game mechanic: starting with 100 SAR (~$27 USDC), the bot must grow or preserve its capital through halal spot trading. If the balance reaches zero, the bot terminates permanently. The system uses OpenClaw (Claude via Discord) as its decision-making brain, a Rust backend for high-performance execution, Binance for order management, and a Next.js dashboard for real-time monitoring.
1.2 Architectural Style
The system follows a simplified event-driven monolith architecture deployed on a single VPS. This approach was chosen over microservices for several reasons: the system has a single bounded context (trading), the team size is one developer, operational simplicity is critical for a real-money system, and the 10-minute cycle does not require extreme horizontal scalability.



Design Philosophy
Simple, effective, easy to modify. A single Rust binary handles all backend logic. The frontend is a statically-deployed Next.js app on Vercel. No Kubernetes, no service mesh, no over-engineering.


1.3 System Context
The system interacts with four external services:

External System
Purpose
Protocol
Frequency
Binance API
Exchange: balances, prices, order execution
REST (HMAC-SHA256)
Every 10 min + on trade
Discord API
Communication channel to OpenClaw (Claude)
REST + Polling
Every 10 min
Fear & Greed API
Market sentiment indicator
REST (public)
Every 10 min
PostgreSQL
Persistent storage for trades, logs, state
TCP (local)
Every 10 min + queries

1.4 Component Diagram
The system comprises five major components:

Component
Technology
Responsibility
Deployment
Trading Engine
Rust (Tokio)
Scheduling, data aggregation, trade execution
VPS
API Server
Rust (Axum)
REST endpoints, WebSocket server for dashboard
VPS (same binary)
Decision Engine
OpenClaw (Claude)
Market analysis, buy/sell/hold decisions
Discord (external)
Dashboard
Next.js + React + Tailwind
Real-time monitoring, kill switch, trade history
Vercel
Database
PostgreSQL 16
Trade logs, positions, balance history, bot state
VPS (Docker)

1.5 Data Flow Architecture
Every 10 minutes, the following sequence executes:

Cron trigger fires in the Rust scheduler
Rust fetches current USDC balance, open positions, and 24h ticker data from Binance
Rust fetches the Fear & Greed Index from the public API
All data is formatted into a structured prompt and sent to OpenClaw via Discord
OpenClaw analyzes conditions and responds with a JSON decision (BUY/SELL/HOLD)
Rust parses the response, validates it, and executes the trade on Binance if applicable
The cycle result, balance, and decision reasoning are logged to PostgreSQL
A WebSocket broadcast notifies the dashboard of the update

2. Technology Stack Recommendations
2.1 Backend: Rust
Language: Rust (2021 edition). Chosen for microsecond-level latency on trade execution, memory safety without garbage collection pauses (critical during volatile markets), strong type system that catches errors at compile time, and excellent async ecosystem via Tokio.

Crate
Version
Purpose
tokio
1.35
Async runtime with full feature set
axum
0.7
HTTP framework with WebSocket support
reqwest
0.11
HTTP client for Binance and Discord APIs
sqlx
0.7
Compile-time checked SQL queries with PostgreSQL
serde / serde_json
1.0
JSON serialization/deserialization
hmac / sha2
0.12 / 0.10
HMAC-SHA256 for Binance API authentication
chrono
0.4
Date/time handling
tracing
0.1
Structured logging
anyhow / thiserror
1.0
Error handling (anyhow for app, thiserror for libraries)
ßtokio-cron-scheduler
0.10
10-minute interval scheduling

2.2 Frontend: Next.js on Vercel
Framework: Next.js 14 with App Router, deployed to Vercel. This gives you zero-config deployment, automatic HTTPS, edge caching for static assets, and instant rollbacks. The dashboard is a single-page app that connects to the Rust backend via REST and WebSocket.

Library
Purpose
React 18
UI rendering
TailwindCSS
Utility-first styling, dark theme
Recharts
Balance history and P&L charts
Native WebSocket
Real-time updates from Rust backend
SWR or React Query
Data fetching with automatic revalidation

2.3 Database: PostgreSQL 16
PostgreSQL was chosen for ACID compliance (critical for financial records), excellent JSON support for storing raw OpenClaw responses, mature tooling and easy backup/restore, and the fact that the dataset is small enough that a single instance handles everything. The database runs in a Docker container on the same VPS as the Rust backend, eliminating network latency for queries.
2.4 Infrastructure Summary

Layer
Technology
Cost
Justification
Compute
VPS (Hetzner CX22 or DigitalOcean Basic)
$4–6/mo
2 vCPU, 4GB RAM — more than enough
Frontend Hosting
Vercel (Hobby plan)
Free
Auto-deploy from Git, CDN, HTTPS
Database
PostgreSQL 16 (Docker on VPS)
$0 (included)
Runs alongside Rust backend
Domain + SSL
Cloudflare (free tier)
Free
DNS, SSL termination, basic DDoS protection
Monitoring
Uptime Kuma (self-hosted)
$0 (included)
Health checks, alerts via Discord webhook




Total Monthly Cost
Approximately $5–10/month for the entire system. The bot needs to make at least $10/month just to break even on infrastructure and api requests.


3. Detailed Component Design
3.1 API Layer
The Rust backend exposes a lightweight REST API via Axum. All endpoints are unauthenticated for v1 (the dashboard is not publicly exposed; access is controlled via VPS firewall rules and Cloudflare Access if needed).

Method
Endpoint
Description
Response
GET
/health
Liveness check
{ status, timestamp }
GET
/status
Bot status, balance, P&L, position count
StatusResponse
GET
/trades
Recent 50 trades with details
Trade[]
GET
/balance
Balance history (last 100 snapshots)
BalanceSnapshot[]
POST
/trigger
Manually trigger a trading cycle
Confirmation string
POST
/kill
Emergency kill switch — marks bot as dead
Confirmation string
GET
/ws
WebSocket upgrade for real-time cycle updates
WebSocket stream

3.2 Business Logic Layer
Trading Engine
The TradingEngine struct orchestrates each 10-minute cycle. It is stateless: all state is read from the database and Binance at the start of each cycle, and written back after execution. This makes the system resilient to restarts.

Key design decision: The engine creates a fresh instance per cycle rather than maintaining long-lived state. This prevents stale data bugs and makes the system trivially restartable.
Decision Parsing
OpenClaw responses are parsed with a two-stage approach: first, a regex extracts JSON from potential markdown code blocks; second, serde_json deserializes into a strongly-typed TradingDecision struct. If parsing fails, the engine defaults to HOLD — the safest action.
Position Sizing
Position size scales with confidence level from OpenClaw:

Confidence
Position Size (% of tradeable balance)
Max USDC
90–100%
10%
~$2.20
80–89%
6%
~$1.32
70–79%
3%
~$0.66
Below 70%
0% (forced HOLD)
$0

3.3 Data Layer
Database Schema
Five tables handle all persistent state:

Table
Purpose
Key Fields
bot_status
Single-row table tracking alive/dead state
is_dead, death_reason, updated_at
positions
Open and closed trading positions
symbol, quantity, entry_price, stop_loss, take_profit, status
trades
Individual buy/sell execution records
symbol, side, quantity, price, usdc_amount, executed_at
cycle_logs
Full log of every 10-minute decision cycle
balance, action, confidence, reasoning, result
balance_history
Time-series balance snapshots for charting
balance_usdc, recorded_at

Caching Strategy
No application-level cache is needed for v1. The 10-minute cycle frequency is low enough that direct database and API queries are sufficient. If the dashboard sees heavy traffic, SWR/React Query on the frontend provides client-side caching with stale-while-revalidate semantics.
3.4 Integration Layer
Binance Integration
All Binance API calls use HMAC-SHA256 authentication. The client is configured with spot-trading-only permissions and zero withdrawal permissions for security. Market buys use quoteOrderQty (specifying USDC amount) while sells use quantity (specifying coin amount).
OpenClaw (Discord) Integration
Communication with OpenClaw happens via Discord's REST API. The Rust backend sends a message to a dedicated channel, then polls for a response from OpenClaw's user ID. The polling timeout is 60 seconds with 2-second intervals. If no response arrives, the cycle defaults to HOLD.



Important
This Discord-based integration means OpenClaw must be active in the channel. If OpenClaw goes offline or Discord has an outage, the bot simply holds all positions — the safest default.


4. Design Patterns & Best Practices
4.1 Architectural Patterns

Pattern
Where Used
Why
Shared State (Arc<AppState>)
Across all handlers and scheduler
Thread-safe shared access to config, DB pool, broadcast channel
Repository Pattern
db module (queries.rs)
Abstracts SQL behind clean async functions
Strategy Pattern
Position sizing based on confidence
Easily swappable sizing algorithms
Observer Pattern
WebSocket broadcast channel
Dashboard subscribes to real-time cycle updates
Fail-Safe Default
Parser and engine
Any error defaults to HOLD (do nothing)

4.2 Project Structure
The Rust project is organized by domain concern. Each module has a mod.rs that re-exports public types:

src/main.rs          → Entry point, server setup, state initialization
src/config.rs        → Environment variable loading
src/api/             → HTTP routes and WebSocket handler
src/binance/         → Exchange client, types, order execution
src/openclaw/        → Discord messaging, prompt building, response parsing
src/trading/         → Engine orchestration, strategy, risk management
src/market/          → External market data (Fear & Greed)
src/db/              → Models, queries, migrations
src/scheduler/       → 10-minute interval cron loop
4.3 Error Handling Strategy
The system uses a layered error handling approach:

Library-level errors use thiserror for typed, specific error variants (e.g., BinanceError, ParseError)
Application-level errors use anyhow for ergonomic error chaining with context
Critical principle: No error should cause a trade to execute incorrectly. Errors always result in HOLD.
All errors are logged with tracing and stored in cycle_logs for post-mortem analysis

5. Non-Functional Requirements
5.1 Scalability
Scalability is not a primary concern for this system. The bot executes one cycle every 10 minutes, generating at most 144 database rows per day. A single VPS with 2 vCPU and 4GB RAM can handle this workload for years without any scaling intervention.

If scaling becomes necessary: The stateless engine design means you could run multiple bot instances with different coin sets. The database would be the only shared resource, and PostgreSQL handles concurrent writes well at this volume.
5.2 Performance

Metric
Target
Rationale
Cycle execution time
< 10 seconds
Well within the 10-minute window
Trade execution latency
< 500ms
Market orders execute near-instantly on Binance
API response time
< 100ms
Dashboard queries hit local PostgreSQL
WebSocket broadcast
< 50ms
In-process broadcast channel, near-zero latency
OpenClaw response time
< 30 seconds
Discord polling with 60s timeout

5.3 Security
API Key Security
Binance API keys are restricted to spot trading only with zero withdrawal permissions
All secrets are stored in environment variables, never in code or version control
The .env file has 600 permissions (owner read/write only)
Network Security
VPS firewall allows only ports 22 (SSH), 80/443 (HTTP/HTTPS via reverse proxy), and 3001 (restricted to Vercel IP ranges)
PostgreSQL is bound to localhost only — no external access
All external API calls use HTTPS/TLS
Cloudflare provides DDoS protection and SSL termination
Application Security
Input validation on all OpenClaw responses before trade execution
The kill switch endpoint should be protected with a shared secret header for v1
CORS is configured to allow only the Vercel dashboard domain in production
5.4 Reliability & Availability
The system is designed for "good enough" reliability, not five-nines. The consequences of downtime are minimal: the bot simply misses cycles and holds its positions.

Scenario
Impact
Recovery
VPS crash/reboot
Missed cycles during downtime
systemd auto-restarts the Docker containers; stateless engine resumes on next tick
Binance API outage
Cannot fetch data or execute trades
Cycle fails gracefully, defaults to HOLD, retries next tick
Discord/OpenClaw offline
Cannot get trading decisions
60s timeout triggers, defaults to HOLD
PostgreSQL crash
Cannot log data; cycle may fail
Docker restart policy recovers; WAL ensures no data loss
Rust backend panic
Current cycle aborted
Process supervisor (systemd) restarts within seconds

Backup Strategy
Daily pg_dump to a local backup directory
Weekly backup upload to cloud storage (Backblaze B2, ~$0.005/GB)
Database is small (< 100MB even after months), so full backups are trivial
5.5 Maintainability
Rust's type system and compile-time checks catch most bugs before deployment
Structured logging via tracing makes debugging straightforward
The modular project structure allows changing any component without affecting others
Easy modification: swap OpenClaw for direct Claude API, add new coins, change intervals — each requires editing a single module

6. Integration Strategy
6.1 Binance API Integration
The Binance client handles three types of operations:

Operation
Endpoint
Auth
Notes
Account balance
GET /api/v3/account
HMAC-SHA256
Returns all asset balances; filter for USDC
24h ticker
GET /api/v3/ticker/24hr
None
Public endpoint, no auth needed
Market buy
POST /api/v3/order
HMAC-SHA256
Uses quoteOrderQty for USDC-denominated buys
Market sell
POST /api/v3/order
HMAC-SHA256
Uses quantity for coin-denominated sells




Rate Limits
Binance allows 1,200 requests per minute. At 4–6 requests per 10-minute cycle, the bot uses less than 0.5% of the limit. Rate limiting is not a concern.

6.2 OpenClaw (Discord) Integration
The communication pattern is request-response over Discord messages:

Rust bot sends a structured prompt to the designated Discord channel
OpenClaw (monitoring the channel) analyzes and responds with JSON
Rust polls for responses after the sent message ID, filtering by OpenClaw's user ID
If no valid response within 60 seconds, the cycle defaults to HOLD
Future Migration Path
The Discord-based integration is a v1 approach. A cleaner v2 would call the Claude API directly via the Anthropic SDK, eliminating Discord as a middleman. The openclaw module is isolated, so this migration requires changing only discord.rs — no other code is affected.
6.3 Dashboard ↔ Backend Communication
The Next.js dashboard on Vercel communicates with the Rust backend on the VPS via two channels:

REST API: For initial data loading and actions (kill switch, manual trigger). The NEXT_PUBLIC_API_URL environment variable points to the VPS domain.
WebSocket: For real-time cycle notifications. The dashboard opens a persistent connection and receives a broadcast on every cycle completion.

7. Data Architecture
7.1 Entity Relationship Overview
The schema is intentionally simple with five tables. The positions table is the central entity: trades reference positions, cycle_logs capture the decision context, and balance_history provides the time-series data for charting.
7.2 Data Retention

Data Type
Retention
Reasoning
cycle_logs
90 days
Decision history for analysis; older data archived to JSON export
balance_history
Forever
Critical for long-term performance tracking; rows are tiny
trades
Forever
Financial records should be permanent
positions
Forever
Both open and closed positions kept for P&L calculation
bot_status
Single row
Only current state matters

7.3 Data Consistency
Trade execution and database logging happen in sequence within the same async task. If the trade succeeds but the database write fails, the position will be out of sync. To mitigate this, the bot reconciles positions with Binance account data at the start of each cycle, using Binance as the source of truth for balances and using the local database as the source of truth for entry prices and stop-loss/take-profit levels.
8. Deployment Architecture
8.1 Environment Strategy

Environment
Purpose
Infrastructure
Development
Local development and testing
cargo run + local PostgreSQL (Docker)
Testnet
Paper trading with Binance testnet
Same VPS or local; BINANCE_BASE_URL points to testnet
Production
Live trading with real funds
VPS + Vercel

8.2 VPS Setup
The VPS runs three Docker containers managed by docker-compose:

survival-bot: The Rust binary compiled in a multi-stage Dockerfile (build stage: rust:latest, runtime stage: debian:slim)
postgres: PostgreSQL 16 Alpine with a persistent volume
caddy (optional): Reverse proxy for HTTPS termination if not using Cloudflare
Deployment Workflow
For simplicity, deployment is a manual SSH-and-pull process:

ssh vps
cd /opt/survival-bot
git pull origin main
docker-compose build bot
docker-compose up -d

For v2, a GitHub Actions workflow can automate this: push to main triggers a build, SSH deploy, and health check.
8.3 Vercel Frontend Deployment
The Next.js dashboard deploys automatically on every push to the main branch. The only required environment variable is NEXT_PUBLIC_API_URL pointing to the VPS backend (e.g., https://bot.yourdomain.com). Vercel handles builds, CDN distribution, and SSL automatically.

9. Monitoring & Observability
9.1 Logging
The Rust backend uses the tracing crate with structured logging. Log levels are controlled via the RUST_LOG environment variable. In production, logs are written to stdout and captured by Docker's logging driver. For persistent logs, Docker is configured with the json-file driver with a 10MB max size and 3 rotated files.
9.2 Health Monitoring

Check
Tool
Frequency
Alert Channel
HTTP health endpoint
Uptime Kuma
Every 1 minute
Discord webhook
Balance threshold
Custom (in trading engine)
Every cycle
Discord webhook + dashboard
Consecutive failures
Custom (cycle_logs query)
Every cycle
Discord webhook
VPS resource usage
Docker stats + htop
Manual / cron
Discord webhook if threshold exceeded

9.3 Key Metrics to Track
Balance over time (primary survival metric)
Win rate (profitable trades / total trades)
Average profit per winning trade vs. average loss per losing trade
OpenClaw response latency and failure rate
Cycle execution time
Consecutive losses streak (triggers conservative mode)
10. Risk Assessment & Mitigation
10.1 Technical Risks

Risk
Severity
Likelihood
Mitigation
OpenClaw returns invalid/unparseable JSON
Medium
Medium
Regex + serde parsing with fallback to HOLD on any error
Binance API key compromised
Critical
Low
No withdrawal permissions; IP whitelist on Binance; rotate keys quarterly
Discord rate limiting blocks messages
Low
Low
Bot sends 1 message per 10 min; well within limits; add retry with backoff
VPS provider outage
Medium
Low
Positions are on Binance (safe); bot resumes on recovery; no stop-loss execution during downtime is the real risk
PostgreSQL data corruption
High
Very Low
Daily backups; WAL mode; Docker volume on reliable storage

10.2 Financial Risks

Risk
Severity
Mitigation
Flash crash wipes balance
Critical
5% stop-loss per position; max 2 positions; max 10% per trade
Slow bleed from bad decisions
High
Consecutive loss tracking triggers ultra-conservative mode
Exchange insolvency
Critical
Only keep trading capital on Binance; withdraw profits periodically
API cost exceeds returns
$5 reserve
Reserve is deducted from tradeable balance; bot pauses if balance < $5

10.3 Dependency Risks

Dependency
Risk
Mitigation
OpenClaw/Discord
Service unavailable
Timeout + HOLD default; future migration to direct Claude API
Binance
API changes or regional restrictions
Abstract behind BinanceClient trait; swappable to other exchanges
Fear & Greed API
Service down
Default to 50 (neutral) on failure; non-critical data
VPS provider
Price increase or discontinuation
Docker-based; migrate to any provider in < 1 hour




Final Note
This architecture prioritizes simplicity and reliability over sophistication. Every design decision was made to minimize what can go wrong when real money is on the line. The system can be understood, modified, and debugged by a single developer. Start with paper trading on Binance testnet, validate the full cycle end-to-end, then switch to production with real funds.


— End of Document —


*Listed directory [tradebot](file:///Users/maanali/Desktop/tradebot) *

### Planner Response

Good, the workspace only has the [ARCHITECTURE.md](cci:7://file:///Users/maanali/Desktop/tradebot/ARCHITECTURE.md:0:0-0:0) file. This is a greenfield project. Let me create the task breakdown and implementation plan.

*Edited relevant file*

### Planner Response

Now let me write the implementation plan:

*Edited relevant file*

### User Input

This is a system-generated message that is sent due to the user's review policy. Proceed with the decision that you think is the most optimal here.

*Edited relevant file*

*Edited relevant file*

*Edited relevant file*

*Edited relevant file*

### Planner Response

Now let me create the database models, queries, and config module:

*Edited relevant file*

*Edited relevant file*

*Edited relevant file*

*Edited relevant file*

### Planner Response

Now let me build the Binance integration, OpenClaw, and market data modules:

*Edited relevant file*

*Edited relevant file*

*Edited relevant file*

*Edited relevant file*

*Edited relevant file*

*Edited relevant file*

*Edited relevant file*

*Edited relevant file*

*Edited relevant file*

### Planner Response

Now let me build the trading engine, strategy, risk, scheduler, API server, and main entry point:

*Edited relevant file*

*Edited relevant file*

*Edited relevant file*

*Edited relevant file*

*Edited relevant file*

*Edited relevant file*

*Edited relevant file*

*Edited relevant file*

### Planner Response

Now let me create `main.rs` and the Docker/deployment files:

*Edited relevant file*

*Edited relevant file*

*Edited relevant file*

*Edited relevant file*

### Planner Response

Now let me build the Next.js dashboard. Let me scaffold it:

*Edited relevant file*

*Edited relevant file*

*Edited relevant file*

*Edited relevant file*

*Edited relevant file*

*Edited relevant file*

*Edited relevant file*

### Planner Response

Now let me create the lib helpers, components, and the main dashboard page:

*Edited relevant file*

*Edited relevant file*

*Edited relevant file*

*Edited relevant file*

*Edited relevant file*

*Edited relevant file*

*Edited relevant file*

*Edited relevant file*

### Planner Response

Now let me create the main dashboard page and then move to verification:

*Edited relevant file*

### Planner Response

Now let me verify the Rust project compiles and the Next.js project builds:

*User accepted the command `cargo check 2>&1`*

*User accepted the command `npm install 2>&1`*

*Checked command status*

*Checked command status*

### Planner Response

One compile error (missing `Ok()` wrapper) and npm install was interrupted. Let me fix the Rust error and retry npm:

*Edited relevant file*

*User accepted the command `cd /Users/maanali/Desktop/tradebot/dashboard && npm install 2>&1`*

*Checked command status*

*User accepted the command `cargo check 2>&1`*

*Checked command status*

### Planner Response

Rust compiles cleanly (only warnings, no errors). Now let me check the Next.js build:

*User accepted the command `cd /Users/maanali/Desktop/tradebot/dashboard && npx next build 2>&1`*

*Checked command status*

### Planner Response

The `npx next build` exited without actually running because it prompted for confirmation. Let me use the correct approach with the locally installed next:

*User accepted the command `cd /Users/maanali/Desktop/tradebot/dashboard && ./node_modules/.bin/next build 2>&1`*

*Checked command status*

### Planner Response

npm install may have had an issue. Let me check and reinstall properly:

*User accepted the command `ls /Users/maanali/Desktop/tradebot/dashboard/node_modules/.bin/next 2>&1 || echo "NOT FOUND"`*

### Planner Response

npm install may have been interrupted earlier. Let me run it properly:

*User accepted the command `cd /Users/maanali/Desktop/tradebot/dashboard && rm -rf node_modules && npm install 2>&1`*

*Checked command status*

### Planner Response

npm install succeeded. Now let's build:

*User accepted the command `cd /Users/maanali/Desktop/tradebot/dashboard && npm run build 2>&1`*

*Checked command status*

### Planner Response

Both builds pass. Let me create the walkthrough and update the task:

*Edited relevant file*

*Edited relevant file*

### User Input

 check @[ARCHITECTURE.md] and verify if all architecture has been implemented

*Viewed [ARCHITECTURE.md](file:///Users/maanali/Desktop/tradebot/ARCHITECTURE.md) *

### Planner Response

Now I have the full architecture doc. Let me systematically audit every spec item against the implemented code:

### Planner Response

Let me now audit the implemented files against the spec. I need to check several key areas:

*Analyzed [main.rs](file:///Users/maanali/Desktop/tradebot/src/main.rs) *

*Analyzed [routes.rs](file:///Users/maanali/Desktop/tradebot/src/api/routes.rs) *

*Analyzed [engine.rs](file:///Users/maanali/Desktop/tradebot/src/trading/engine.rs) *

*Analyzed [001_init.sql](file:///Users/maanali/Desktop/tradebot/migrations/001_init.sql) *

### Planner Response

Good. Let me now check the remaining files that the architecture specifies — checking for missing files and features:

*Searched filesystem*

*Analyzed [queries.rs](file:///Users/maanali/Desktop/tradebot/src/db/queries.rs) *

*Analyzed [models.rs](file:///Users/maanali/Desktop/tradebot/src/db/models.rs) *

### Planner Response

Now let me check the missing spec files and a few remaining areas of concern:

*Analyzed [client.rs](file:///Users/maanali/Desktop/tradebot/src/binance/client.rs) *

*Analyzed [config.rs](file:///Users/maanali/Desktop/tradebot/src/config.rs) *

*Analyzed [strategy.rs](file:///Users/maanali/Desktop/tradebot/src/trading/strategy.rs) *

*Analyzed [risk.rs](file:///Users/maanali/Desktop/tradebot/src/trading/risk.rs) *

### Planner Response

Now let me check the remaining files and the [docker-compose.yml](cci:7://file:///Users/maanali/Desktop/tradebot/docker-compose.yml:0:0-0:0) to verify alignment with the spec:

### Planner Response

Now let me compile the comprehensive audit report. I have all the data I need.

*Edited relevant file*

### User Input

okay what do we need for it to run 

### Planner Response

Here's everything you need to get the bot running:

## 1. **External Accounts & Credentials**

| What | Where to Get | Notes |
|------|-------------|-------|
| **Binance Account** | [binance.com](https://binance.com) | Create API keys → **Spot trading only, NO withdrawal** |
| **Discord Bot** | [discord.com/developers](https://discord.com/developers/applications) | Create a bot, get the token, invite it to your server |
| **Discord Server + Channel** | Discord | A dedicated channel where OpenClaw (Claude) and your bot both have access |
| **OpenClaw User ID** | Right-click OpenClaw's profile in Discord → Copy User ID | Enable Developer Mode in Discord settings first |

## 2. **Infrastructure**

| What | Cost | Notes |
|------|------|-------|
| **VPS** (Hetzner CX22 or DigitalOcean) | ~$5/mo | 2 vCPU, 4GB RAM — needs Docker + Docker Compose installed |
| **Domain** (optional) | Free via Cloudflare | For HTTPS access to the API |
| **Vercel account** (for dashboard) | Free | Auto-deploys from Git |

## 3. **Local Setup (to test first)**

You need **Docker + Docker Compose** installed locally. Then:

```bash
# 1. Copy and fill in your credentials
cp .env.example .env
# Edit .env with your real values

# 2. Start PostgreSQL + bot
docker-compose up -d

# 3. For the dashboard
cd dashboard
npm install
npm run dev
```

## 4. **Your `.env` file needs these filled in:**

```bash
# REQUIRED — won't start without these
DATABASE_URL=postgres://survival:your_password@localhost:5432/survival_bot
BINANCE_API_KEY=your_binance_api_key
BINANCE_SECRET_KEY=your_binance_secret_key
DISCORD_BOT_TOKEN=your_discord_bot_token
DISCORD_CHANNEL_ID=your_channel_id
OPENCLAW_USER_ID=openclaw_user_id

# RECOMMENDED — use testnet first!
BINANCE_BASE_URL=https://testnet.binance.vision

# OPTIONAL — have safe defaults
API_HOST=0.0.0.0
API_PORT=3001
MIN_BALANCE_USDC=5.0
KILL_SECRET=your_secret_here
RUST_LOG=info
```

## 5. **Recommended Launch Order**

1. **Get all credentials** (Binance testnet keys, Discord bot token, channel ID, OpenClaw user ID)
2. **Fill in `.env`** with testnet Binance URL first
3. **`docker-compose up -d`** — starts PostgreSQL + bot locally
4. **Watch logs**: `docker-compose logs -f bot`
5. **Verify**: Hit `http://localhost:3001/health` — should return `{ "status": "ok" }`
6. **Test a cycle**: `curl -X POST http://localhost:3001/trigger`
7. **Once validated** → switch `BINANCE_BASE_URL` to `https://api.binance.com` and fund with $27 USDC

> ⚠️ **Start with Binance testnet** (`https://testnet.binance.vision`) so you can validate the full cycle without risking real money. The testnet gives you fake balance to trade with.

Do you have any of these credentials ready, or would you like help setting up any of them?