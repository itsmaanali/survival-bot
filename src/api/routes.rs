use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use chrono::Utc;
use std::sync::Arc;
use tracing::info;

use crate::db::models::*;
use crate::db::queries;

use super::super::AppState;

/// GET /health — Liveness check
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        timestamp: Utc::now(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// GET /status — Bot status, balance, P&L, position count
pub async fn status(State(state): State<Arc<AppState>>) -> Result<Json<StatusResponse>, StatusCode> {
    let bot = queries::get_bot_status(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let balance = queries::get_balance_history(&state.pool, 1)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .first()
        .map(|b| b.balance_usdc)
        .unwrap_or(0.0);

    let total_pnl = queries::get_total_pnl(&state.pool)
        .await
        .unwrap_or(0.0);

    let open_positions = queries::count_open_positions(&state.pool)
        .await
        .unwrap_or(0) as i32;

    let total_trades = queries::count_trades(&state.pool)
        .await
        .unwrap_or(0);

    let total_cycles = queries::count_cycles(&state.pool)
        .await
        .unwrap_or(0);

    let win_rate = queries::get_win_rate(&state.pool)
        .await
        .unwrap_or(0.0);

    let uptime_hours = (Utc::now() - bot.started_at).num_minutes() as f64 / 60.0;

    let last_cycle = queries::get_recent_cycles(&state.pool, 1)
        .await
        .unwrap_or_default()
        .first()
        .map(|c| c.created_at);

    Ok(Json(StatusResponse {
        is_alive: !bot.is_dead,
        balance_usdc: balance,
        total_pnl,
        open_positions,
        total_trades,
        total_cycles,
        win_rate,
        uptime_hours,
        last_cycle_at: last_cycle,
    }))
}

/// GET /trades — Recent 50 trades
pub async fn trades(State(state): State<Arc<AppState>>) -> Result<Json<Vec<Trade>>, StatusCode> {
    let trades = queries::get_recent_trades(&state.pool, 50)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(trades))
}

/// GET /balance — Balance history (last 100 snapshots)
pub async fn balance_history(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<BalanceHistory>>, StatusCode> {
    let history = queries::get_balance_history(&state.pool, 100)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(history))
}

/// GET /cycles — Recent 50 cycle logs
pub async fn cycles(State(state): State<Arc<AppState>>) -> Result<Json<Vec<CycleLog>>, StatusCode> {
    let logs = queries::get_recent_cycles(&state.pool, 50)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(logs))
}

/// GET /positions — Open positions
pub async fn positions(State(state): State<Arc<AppState>>) -> Result<Json<Vec<Position>>, StatusCode> {
    let positions = queries::get_open_positions(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(positions))
}

/// POST /trigger — Manually trigger a trading cycle
pub async fn trigger(State(state): State<Arc<AppState>>) -> &'static str {
    info!("🔧 Manual cycle trigger received");

    let pool = state.pool.clone();
    let config = state.config.clone();
    let binance = state.binance.clone();
    let discord = state.discord.clone();
    let broadcast_tx = state.broadcast_tx.clone();

    tokio::spawn(async move {
        let engine = crate::trading::TradingEngine::new(
            config, pool, binance, discord, broadcast_tx,
        );
        if let Err(e) = engine.run_cycle().await {
            tracing::error!(error = %e, "Manual cycle failed");
        }
    });

    "Cycle triggered"
}

/// POST /kill — Emergency kill switch
pub async fn kill(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<&'static str, StatusCode> {
    // Verify kill secret
    let secret = headers
        .get("X-Kill-Secret")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if secret != state.config.kill_secret {
        return Err(StatusCode::UNAUTHORIZED);
    }

    info!("🛑 KILL SWITCH ACTIVATED");

    queries::kill_bot(&state.pool, "Manual kill switch activated")
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok("Bot killed")
}
