use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{error, info};

use crate::binance::BinanceClient;
use crate::config::Config;
use crate::db::models::CycleUpdate;
use crate::openclaw::DiscordClient;
use crate::trading::TradingEngine;

/// Start the 10-minute trading cycle scheduler.
/// Runs indefinitely, executing one cycle every 10 minutes.
pub async fn start_scheduler(
    config: Arc<Config>,
    pool: sqlx::PgPool,
    binance: BinanceClient,
    discord: DiscordClient,
    broadcast_tx: broadcast::Sender<CycleUpdate>,
) {
    let engine = TradingEngine::new(
        config,
        pool,
        binance,
        discord,
        broadcast_tx,
    );

    info!("⏰ Scheduler started — running every 10 minutes");

    // Run initial cycle immediately
    info!("Running initial cycle...");
    if let Err(e) = engine.run_cycle().await {
        error!(error = %e, "Initial cycle failed");
    }

    // Then loop every 10 minutes
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(600));
    interval.tick().await; // Skip the first immediate tick (already ran above)

    loop {
        interval.tick().await;
        info!("⏰ Scheduler tick — starting cycle");

        if let Err(e) = engine.run_cycle().await {
            error!(error = %e, "Cycle failed");
        }
    }
}
