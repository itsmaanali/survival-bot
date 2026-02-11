use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ─── Bot Status ──────────────────────────────────────────

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct BotStatus {
    pub id: i32,
    pub is_dead: bool,
    pub death_reason: Option<String>,
    pub started_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ─── Position ────────────────────────────────────────────

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Position {
    pub id: Uuid,
    pub symbol: String,
    pub side: String,
    pub quantity: f64,
    pub entry_price: f64,
    pub current_price: Option<f64>,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    pub status: String,
    pub pnl: Option<f64>,
    pub opened_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub close_reason: Option<String>,
}

// ─── Trade ───────────────────────────────────────────────

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Trade {
    pub id: Uuid,
    pub position_id: Option<Uuid>,
    pub symbol: String,
    pub side: String,
    pub quantity: f64,
    pub price: f64,
    pub usdc_amount: f64,
    pub commission: Option<f64>,
    pub executed_at: DateTime<Utc>,
}

// ─── Cycle Log ───────────────────────────────────────────

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct CycleLog {
    pub id: Uuid,
    pub cycle_number: i32,
    pub balance_usdc: f64,
    pub action: String,
    pub symbol: Option<String>,
    pub confidence: Option<i32>,
    pub reasoning: Option<String>,
    pub raw_response: Option<String>,
    pub fear_greed: Option<i32>,
    pub execution_ms: Option<i32>,
    pub result: Option<String>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
}

// ─── Balance History ─────────────────────────────────────

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct BalanceHistory {
    pub id: i32,
    pub balance_usdc: f64,
    pub open_positions: i32,
    pub total_pnl: f64,
    pub recorded_at: DateTime<Utc>,
}

// ─── Trading Decision (from OpenClaw) ────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradingDecision {
    pub action: TradingAction,
    pub symbol: Option<String>,
    pub confidence: i32,
    pub reasoning: String,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum TradingAction {
    Buy,
    Sell,
    Hold,
}

impl std::fmt::Display for TradingAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TradingAction::Buy => write!(f, "BUY"),
            TradingAction::Sell => write!(f, "SELL"),
            TradingAction::Hold => write!(f, "HOLD"),
        }
    }
}

// ─── Cycle Update (broadcast via WebSocket) ──────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleUpdate {
    pub cycle_number: i32,
    pub balance_usdc: f64,
    pub action: String,
    pub symbol: Option<String>,
    pub confidence: Option<i32>,
    pub reasoning: Option<String>,
    pub pnl: f64,
    pub fear_greed: Option<i32>,
    pub timestamp: DateTime<Utc>,
}

// ─── API Response Types ──────────────────────────────────

#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub is_alive: bool,
    pub balance_usdc: f64,
    pub total_pnl: f64,
    pub open_positions: i32,
    pub total_trades: i64,
    pub total_cycles: i64,
    pub win_rate: f64,
    pub uptime_hours: f64,
    pub last_cycle_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: DateTime<Utc>,
    pub version: String,
}
