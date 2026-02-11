use anyhow::Result;
use chrono::Utc;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{error, info, warn};

use crate::binance::BinanceClient;
use crate::config::Config;
use crate::db::models::*;
use crate::db::queries;
use crate::market::fetch_fear_greed_index;
use crate::openclaw::{build_prompt, parse_decision, DiscordClient};
use crate::trading::risk::RiskManager;
use crate::trading::strategy::PositionSizer;

/// The core trading engine. Stateless — reads all state fresh each cycle.
pub struct TradingEngine {
    config: Arc<Config>,
    pool: PgPool,
    binance: BinanceClient,
    discord: DiscordClient,
    broadcast_tx: broadcast::Sender<CycleUpdate>,
}

impl TradingEngine {
    pub fn new(
        config: Arc<Config>,
        pool: PgPool,
        binance: BinanceClient,
        discord: DiscordClient,
        broadcast_tx: broadcast::Sender<CycleUpdate>,
    ) -> Self {
        Self {
            config,
            pool,
            binance,
            discord,
            broadcast_tx,
        }
    }

    /// Execute a single 10-minute trading cycle
    pub async fn run_cycle(&self) -> Result<()> {
        let cycle_start = std::time::Instant::now();
        info!("━━━ Starting trading cycle ━━━");

        // 1. Check if bot is dead
        let status = queries::get_bot_status(&self.pool).await?;
        if status.is_dead {
            warn!("Bot is DEAD — skipping cycle. Reason: {:?}", status.death_reason);
            return Ok(());
        }

        // 2. Fetch USDC balance from Binance
        let balance = match self.binance.get_usdc_balance().await {
            Ok(b) => b,
            Err(e) => {
                error!(error = %e, "Failed to fetch balance");
                self.log_error_cycle(0.0, &e.to_string()).await;
                return Ok(());
            }
        };

        info!(balance, "Current USDC balance");

        // 3. Check if balance is zero → bot dies
        if balance <= 0.0 {
            warn!("💀 Balance is ZERO — bot is DEAD");
            queries::kill_bot(&self.pool, "Balance reached zero").await?;
            self.log_error_cycle(balance, "Balance reached zero — bot terminated").await;
            return Ok(());
        }

        // 4. Check if below minimum reserve
        if balance < self.config.min_balance_usdc {
            warn!(balance, min = self.config.min_balance_usdc, "Below minimum reserve — HOLD");
            self.log_hold_cycle(balance, "Below minimum reserve").await;
            return Ok(());
        }

        // 5. Check stop-loss / take-profit on existing positions
        let positions_to_close = RiskManager::check_positions(&self.pool, &self.binance).await?;
        for (pos, reason) in &positions_to_close {
            info!(symbol = %pos.symbol, reason, "Closing position");
            self.close_position(pos, reason).await?;
        }

        // 6. Get open positions (refreshed after closures)
        let open_positions = queries::get_open_positions(&self.pool).await?;

        // 7. Fetch market data
        let tickers = self.binance.get_tickers().await.unwrap_or_default();
        let fear_greed = fetch_fear_greed_index().await;
        let consecutive_losses = queries::get_consecutive_losses(&self.pool).await.unwrap_or(0);

        // 8. Build prompt for OpenClaw
        let prompt = build_prompt(balance, &open_positions, &tickers, fear_greed, consecutive_losses, &self.config.openclaw_user_id);

        // 9. Send to OpenClaw and get decision
        let raw_response = match self.discord.ask(&prompt).await {
            Ok(Some(resp)) => resp,
            Ok(None) => {
                warn!("No response from OpenClaw — defaulting to HOLD");
                self.log_hold_cycle(balance, "OpenClaw timeout").await;
                return Ok(());
            }
            Err(e) => {
                error!(error = %e, "Discord communication error");
                self.log_error_cycle(balance, &e.to_string()).await;
                return Ok(());
            }
        };

        // 10. Parse decision
        let decision = parse_decision(&raw_response);
        info!(
            action = %decision.action,
            confidence = decision.confidence,
            symbol = ?decision.symbol,
            "Trading decision received"
        );

        // 11. Execute decision
        let (result, error) = match decision.action {
            TradingAction::Hold => {
                info!("📊 Decision: HOLD");
                (Some("HOLD".to_string()), None)
            }
            TradingAction::Buy => {
                match self.execute_buy(&decision, balance).await {
                    Ok(msg) => (Some(msg), None),
                    Err(e) => (None, Some(e.to_string())),
                }
            }
            TradingAction::Sell => {
                match self.execute_sell(&decision).await {
                    Ok(msg) => (Some(msg), None),
                    Err(e) => (None, Some(e.to_string())),
                }
            }
        };

        // 12. Log the cycle
        let execution_ms = cycle_start.elapsed().as_millis() as i32;
        queries::insert_cycle_log(
            &self.pool,
            balance,
            &decision.action.to_string(),
            decision.symbol.as_deref(),
            Some(decision.confidence),
            Some(&decision.reasoning),
            Some(&raw_response),
            Some(fear_greed),
            execution_ms,
            result.as_deref(),
            error.as_deref(),
        )
        .await?;

        // 13. Record balance snapshot
        let updated_balance = self.binance.get_usdc_balance().await.unwrap_or(balance);
        let open_count = queries::count_open_positions(&self.pool).await.unwrap_or(0) as i32;
        let total_pnl = queries::get_total_pnl(&self.pool).await.unwrap_or(0.0);
        queries::insert_balance_snapshot(&self.pool, updated_balance, open_count, total_pnl).await?;

        // 14. Broadcast to WebSocket subscribers
        let cycle_count = queries::count_cycles(&self.pool).await.unwrap_or(0) as i32;
        let update = CycleUpdate {
            cycle_number: cycle_count,
            balance_usdc: updated_balance,
            action: decision.action.to_string(),
            symbol: decision.symbol,
            confidence: Some(decision.confidence),
            reasoning: Some(decision.reasoning),
            pnl: total_pnl,
            fear_greed: Some(fear_greed),
            timestamp: Utc::now(),
        };
        let _ = self.broadcast_tx.send(update);

        info!(
            execution_ms,
            balance = updated_balance,
            "━━━ Cycle complete ━━━"
        );
        Ok(())
    }

    /// Execute a BUY decision
    async fn execute_buy(&self, decision: &TradingDecision, balance: f64) -> Result<String> {
        let symbol = decision.symbol.as_ref().unwrap(); // Validated by parser

        // Check position limits
        if !RiskManager::can_open_position(&self.pool).await? {
            info!("Max positions reached (2) — skipping BUY");
            return Ok("SKIPPED: max positions".to_string());
        }

        // Calculate position size
        let usdc_amount =
            PositionSizer::calculate(balance, decision.confidence, self.config.min_balance_usdc);

        if usdc_amount <= 0.0 {
            info!("Position size too small — skipping BUY");
            return Ok("SKIPPED: insufficient size".to_string());
        }

        info!(symbol, usdc_amount, "Executing BUY");

        // Execute on Binance
        let order = self.binance.market_buy(symbol, usdc_amount).await?;
        let trade = order.to_executed_trade();

        // Validate and enforce stop-loss
        let stop_loss = decision.stop_loss.map(|sl| {
            RiskManager::validate_stop_loss(trade.avg_price, sl)
        }).unwrap_or(trade.avg_price * 0.95); // Default 5% stop-loss

        // Record position
        let position_id = queries::insert_position(
            &self.pool,
            symbol,
            "BUY",
            trade.quantity,
            trade.avg_price,
            Some(stop_loss),
            decision.take_profit,
        )
        .await?;

        // Record trade
        queries::insert_trade(
            &self.pool,
            Some(position_id),
            symbol,
            "BUY",
            trade.quantity,
            trade.avg_price,
            trade.usdc_amount,
            trade.commission,
        )
        .await?;

        info!(
            symbol,
            qty = trade.quantity,
            price = trade.avg_price,
            "✅ BUY executed"
        );
        Ok(format!(
            "BUY {} @ ${:.6} (${:.2} USDC)",
            symbol, trade.avg_price, trade.usdc_amount
        ))
    }

    /// Execute a SELL decision
    async fn execute_sell(&self, decision: &TradingDecision) -> Result<String> {
        let symbol = decision.symbol.as_ref().unwrap();

        // Find the open position for this symbol
        let position = match queries::get_position_by_symbol(&self.pool, symbol).await? {
            Some(p) => p,
            None => {
                info!(symbol, "No open position to sell — skipping");
                return Ok("SKIPPED: no open position".to_string());
            }
        };

        info!(symbol, qty = position.quantity, "Executing SELL");

        // Execute on Binance
        let order = self.binance.market_sell(symbol, position.quantity).await?;
        let trade = order.to_executed_trade();

        // Calculate PnL
        let pnl = (trade.avg_price - position.entry_price) * position.quantity;

        // Close position
        queries::close_position(&self.pool, position.id, pnl, "SELL_DECISION").await?;

        // Record trade
        queries::insert_trade(
            &self.pool,
            Some(position.id),
            symbol,
            "SELL",
            trade.quantity,
            trade.avg_price,
            trade.usdc_amount,
            trade.commission,
        )
        .await?;

        let result_str = if pnl >= 0.0 { "WIN" } else { "LOSS" };
        info!(
            symbol,
            qty = trade.quantity,
            price = trade.avg_price,
            pnl,
            result = result_str,
            "✅ SELL executed"
        );
        Ok(format!(
            "SELL {} @ ${:.6} (PnL: ${:.4} {})",
            symbol, trade.avg_price, pnl, result_str
        ))
    }

    /// Close a position triggered by risk management
    async fn close_position(&self, position: &Position, reason: &str) -> Result<()> {
        let order = self
            .binance
            .market_sell(&position.symbol, position.quantity)
            .await?;
        let trade = order.to_executed_trade();

        let pnl = (trade.avg_price - position.entry_price) * position.quantity;
        queries::close_position(&self.pool, position.id, pnl, reason).await?;

        queries::insert_trade(
            &self.pool,
            Some(position.id),
            &position.symbol,
            "SELL",
            trade.quantity,
            trade.avg_price,
            trade.usdc_amount,
            trade.commission,
        )
        .await?;

        info!(
            symbol = %position.symbol,
            reason,
            pnl,
            "Position closed by risk manager"
        );
        Ok(())
    }

    /// Log a HOLD cycle (for timeouts, low balance, etc.)
    async fn log_hold_cycle(&self, balance: f64, reason: &str) {
        let _ = queries::insert_cycle_log(
            &self.pool, balance, "HOLD", None, None,
            Some(reason), None, None, 0, Some("HOLD"), None,
        )
        .await;

        let open_count = queries::count_open_positions(&self.pool).await.unwrap_or(0) as i32;
        let total_pnl = queries::get_total_pnl(&self.pool).await.unwrap_or(0.0);
        let _ = queries::insert_balance_snapshot(&self.pool, balance, open_count, total_pnl).await;
    }

    /// Log an error cycle
    async fn log_error_cycle(&self, balance: f64, error: &str) {
        let _ = queries::insert_cycle_log(
            &self.pool, balance, "ERROR", None, None,
            None, None, None, 0, None, Some(error),
        )
        .await;
    }
}
