use anyhow::Result;
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use super::models::*;

// ─── Bot Status ──────────────────────────────────────────

pub async fn get_bot_status(pool: &PgPool) -> Result<BotStatus> {
    let status = sqlx::query_as::<_, BotStatus>("SELECT * FROM bot_status LIMIT 1")
        .fetch_one(pool)
        .await?;
    Ok(status)
}

pub async fn kill_bot(pool: &PgPool, reason: &str) -> Result<()> {
    sqlx::query("UPDATE bot_status SET is_dead = TRUE, death_reason = $1, updated_at = $2")
        .bind(reason)
        .bind(Utc::now())
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn revive_bot(pool: &PgPool) -> Result<()> {
    sqlx::query(
        "UPDATE bot_status SET is_dead = FALSE, death_reason = NULL, updated_at = $1",
    )
    .bind(Utc::now())
    .execute(pool)
    .await?;
    Ok(())
}

// ─── Positions ───────────────────────────────────────────

pub async fn get_open_positions(pool: &PgPool) -> Result<Vec<Position>> {
    let positions =
        sqlx::query_as::<_, Position>("SELECT * FROM positions WHERE status = 'OPEN' ORDER BY opened_at DESC")
            .fetch_all(pool)
            .await?;
    Ok(positions)
}

pub async fn get_position_by_symbol(pool: &PgPool, symbol: &str) -> Result<Option<Position>> {
    let position = sqlx::query_as::<_, Position>(
        "SELECT * FROM positions WHERE symbol = $1 AND status = 'OPEN' LIMIT 1",
    )
    .bind(symbol)
    .fetch_optional(pool)
    .await?;
    Ok(position)
}

pub async fn insert_position(
    pool: &PgPool,
    symbol: &str,
    side: &str,
    quantity: f64,
    entry_price: f64,
    stop_loss: Option<f64>,
    take_profit: Option<f64>,
) -> Result<Uuid> {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO positions (id, symbol, side, quantity, entry_price, stop_loss, take_profit, status, opened_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, 'OPEN', $8)",
    )
    .bind(id)
    .bind(symbol)
    .bind(side)
    .bind(quantity)
    .bind(entry_price)
    .bind(stop_loss)
    .bind(take_profit)
    .bind(Utc::now())
    .execute(pool)
    .await?;
    Ok(id)
}

pub async fn close_position(
    pool: &PgPool,
    position_id: Uuid,
    pnl: f64,
    reason: &str,
) -> Result<()> {
    sqlx::query(
        "UPDATE positions SET status = 'CLOSED', pnl = $1, closed_at = $2, close_reason = $3 WHERE id = $4",
    )
    .bind(pnl)
    .bind(Utc::now())
    .bind(reason)
    .bind(position_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_position_price(pool: &PgPool, position_id: Uuid, price: f64) -> Result<()> {
    sqlx::query("UPDATE positions SET current_price = $1 WHERE id = $2")
        .bind(price)
        .bind(position_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn count_open_positions(pool: &PgPool) -> Result<i64> {
    let row: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM positions WHERE status = 'OPEN'")
            .fetch_one(pool)
            .await?;
    Ok(row.0)
}

// ─── Trades ──────────────────────────────────────────────

pub async fn insert_trade(
    pool: &PgPool,
    position_id: Option<Uuid>,
    symbol: &str,
    side: &str,
    quantity: f64,
    price: f64,
    usdc_amount: f64,
    commission: f64,
) -> Result<Uuid> {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO trades (id, position_id, symbol, side, quantity, price, usdc_amount, commission, executed_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
    )
    .bind(id)
    .bind(position_id)
    .bind(symbol)
    .bind(side)
    .bind(quantity)
    .bind(price)
    .bind(usdc_amount)
    .bind(commission)
    .bind(Utc::now())
    .execute(pool)
    .await?;
    Ok(id)
}

pub async fn get_recent_trades(pool: &PgPool, limit: i64) -> Result<Vec<Trade>> {
    let trades = sqlx::query_as::<_, Trade>(
        "SELECT * FROM trades ORDER BY executed_at DESC LIMIT $1",
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(trades)
}

pub async fn count_trades(pool: &PgPool) -> Result<i64> {
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM trades")
        .fetch_one(pool)
        .await?;
    Ok(row.0)
}

// ─── Cycle Logs ──────────────────────────────────────────

pub async fn insert_cycle_log(
    pool: &PgPool,
    balance_usdc: f64,
    action: &str,
    symbol: Option<&str>,
    confidence: Option<i32>,
    reasoning: Option<&str>,
    raw_response: Option<&str>,
    fear_greed: Option<i32>,
    execution_ms: i32,
    result: Option<&str>,
    error: Option<&str>,
) -> Result<Uuid> {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO cycle_logs (id, balance_usdc, action, symbol, confidence, reasoning, raw_response, fear_greed, execution_ms, result, error, created_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
    )
    .bind(id)
    .bind(balance_usdc)
    .bind(action)
    .bind(symbol)
    .bind(confidence)
    .bind(reasoning)
    .bind(raw_response)
    .bind(fear_greed)
    .bind(execution_ms)
    .bind(result)
    .bind(error)
    .bind(Utc::now())
    .execute(pool)
    .await?;
    Ok(id)
}

pub async fn get_recent_cycles(pool: &PgPool, limit: i64) -> Result<Vec<CycleLog>> {
    let logs = sqlx::query_as::<_, CycleLog>(
        "SELECT * FROM cycle_logs ORDER BY created_at DESC LIMIT $1",
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(logs)
}

pub async fn count_cycles(pool: &PgPool) -> Result<i64> {
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM cycle_logs")
        .fetch_one(pool)
        .await?;
    Ok(row.0)
}

pub async fn get_consecutive_losses(pool: &PgPool) -> Result<i64> {
    // Count consecutive cycles with negative PnL from the most recent
    let logs = sqlx::query_as::<_, CycleLog>(
        "SELECT * FROM cycle_logs WHERE action IN ('BUY', 'SELL') ORDER BY created_at DESC LIMIT 20",
    )
    .fetch_all(pool)
    .await?;

    let mut streak = 0i64;
    for log in &logs {
        if log.result.as_deref() == Some("LOSS") {
            streak += 1;
        } else {
            break;
        }
    }
    Ok(streak)
}

pub async fn get_win_rate(pool: &PgPool) -> Result<f64> {
    let total: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM cycle_logs WHERE action IN ('BUY', 'SELL') AND result IS NOT NULL",
    )
    .fetch_one(pool)
    .await?;

    if total.0 == 0 {
        return Ok(0.0);
    }

    let wins: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM cycle_logs WHERE action IN ('BUY', 'SELL') AND result = 'WIN'",
    )
    .fetch_one(pool)
    .await?;

    Ok(wins.0 as f64 / total.0 as f64 * 100.0)
}

// ─── Balance History ─────────────────────────────────────

pub async fn insert_balance_snapshot(
    pool: &PgPool,
    balance_usdc: f64,
    open_positions: i32,
    total_pnl: f64,
) -> Result<()> {
    sqlx::query(
        "INSERT INTO balance_history (balance_usdc, open_positions, total_pnl, recorded_at) VALUES ($1, $2, $3, $4)",
    )
    .bind(balance_usdc)
    .bind(open_positions)
    .bind(total_pnl)
    .bind(Utc::now())
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_balance_history(pool: &PgPool, limit: i64) -> Result<Vec<BalanceHistory>> {
    let history = sqlx::query_as::<_, BalanceHistory>(
        "SELECT * FROM balance_history ORDER BY recorded_at DESC LIMIT $1",
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(history)
}

pub async fn get_total_pnl(pool: &PgPool) -> Result<f64> {
    let row: (Option<f64>,) = sqlx::query_as(
        "SELECT SUM(pnl) FROM positions WHERE status = 'CLOSED'",
    )
    .fetch_one(pool)
    .await?;
    Ok(row.0.unwrap_or(0.0))
}
