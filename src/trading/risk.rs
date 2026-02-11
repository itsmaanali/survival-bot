use anyhow::Result;
use sqlx::PgPool;
use tracing::{info, warn};

use crate::binance::BinanceClient;
use crate::db::models::Position;
use crate::db::queries;

/// Risk management: stop-loss/take-profit checking and position limits.
pub struct RiskManager;

impl RiskManager {
    /// Check all open positions for stop-loss or take-profit triggers.
    /// Returns positions that need to be closed.
    pub async fn check_positions(
        pool: &PgPool,
        binance: &BinanceClient,
    ) -> Result<Vec<(Position, String)>> {
        let positions = queries::get_open_positions(pool).await?;
        let mut to_close: Vec<(Position, String)> = Vec::new();

        for pos in positions {
            // Get current price
            let ticker = match binance.get_ticker(&pos.symbol).await {
                Ok(t) => t,
                Err(e) => {
                    warn!(symbol = %pos.symbol, error = %e, "Failed to get ticker for risk check");
                    continue;
                }
            };

            let current_price: f64 = ticker.last_price.parse().unwrap_or(0.0);
            if current_price <= 0.0 {
                continue;
            }

            // Update position's current price in DB
            let _ = queries::update_position_price(pool, pos.id, current_price).await;

            // Check stop-loss
            if let Some(stop_loss) = pos.stop_loss {
                if current_price <= stop_loss {
                    info!(
                        symbol = %pos.symbol,
                        current_price,
                        stop_loss,
                        "🛑 Stop-loss triggered"
                    );
                    to_close.push((pos, "STOP_LOSS".to_string()));
                    continue;
                }
            }

            // Check take-profit
            if let Some(take_profit) = pos.take_profit {
                if current_price >= take_profit {
                    info!(
                        symbol = %pos.symbol,
                        current_price,
                        take_profit,
                        "🎯 Take-profit triggered"
                    );
                    to_close.push((pos, "TAKE_PROFIT".to_string()));
                    continue;
                }
            }
        }

        Ok(to_close)
    }

    /// Check if we can open a new position (max 2 open at a time)
    pub async fn can_open_position(pool: &PgPool) -> Result<bool> {
        let count = queries::count_open_positions(pool).await?;
        Ok(count < 2)
    }

    /// Validate a proposed stop-loss: must be within 5% of entry price
    pub fn validate_stop_loss(entry_price: f64, stop_loss: f64) -> f64 {
        let max_loss_pct = 0.05; // 5%
        let min_stop = entry_price * (1.0 - max_loss_pct);

        if stop_loss < min_stop {
            warn!(
                proposed = stop_loss,
                enforced = min_stop,
                "Stop-loss too wide — enforcing 5% max"
            );
            min_stop
        } else {
            stop_loss
        }
    }
}
