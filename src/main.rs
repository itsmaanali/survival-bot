mod api;
mod binance;
mod config;
mod db;
mod market;
mod openclaw;
mod scheduler;
mod trading;

use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::info;

use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::binance::BinanceClient;
use crate::config::Config;
use crate::db::models::CycleUpdate;
use crate::openclaw::DiscordClient;

/// Shared application state passed to all handlers and the scheduler
pub struct AppState {
    pub pool: sqlx::PgPool,
    pub config: Arc<Config>,
    pub binance: BinanceClient,
    pub discord: DiscordClient,
    pub broadcast_tx: broadcast::Sender<CycleUpdate>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load config
    let config = Config::from_env()?;
    let config = Arc::new(config);

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "survival_bot=info,tower_http=info".into()),
        )
        .init();

    info!("🚀 Survival Trading Bot v{}", env!("CARGO_PKG_VERSION"));
    info!("Connecting to database...");

    // Database connection pool
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await?;

    info!("✅ Database connected");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    info!("✅ Migrations applied");

    // Initialize clients
    let binance = BinanceClient::new(
        &config.binance_base_url,
        &config.binance_api_key,
        &config.binance_secret_key,
    );

    let discord = DiscordClient::new(
        &config.discord_bot_token,
        &config.discord_channel_id,
        &config.openclaw_user_id,
    );

    // Broadcast channel for WebSocket updates
    let (broadcast_tx, _) = broadcast::channel::<CycleUpdate>(100);

    // Shared state
    let state = Arc::new(AppState {
        pool: pool.clone(),
        config: config.clone(),
        binance: binance.clone(),
        discord: discord.clone(),
        broadcast_tx: broadcast_tx.clone(),
    });

    // CORS layer
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build router
    let app = Router::new()
        .route("/health", get(api::routes::health))
        .route("/status", get(api::routes::status))
        .route("/trades", get(api::routes::trades))
        .route("/balance", get(api::routes::balance_history))
        .route("/cycles", get(api::routes::cycles))
        .route("/positions", get(api::routes::positions))
        .route("/trigger", post(api::routes::trigger))
        .route("/kill", post(api::routes::kill))
        .route("/ws", get(api::websocket::ws_handler))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Start scheduler in background
    let scheduler_pool = pool.clone();
    let scheduler_config = config.clone();
    let scheduler_binance = binance.clone();
    let scheduler_discord = discord.clone();
    let scheduler_broadcast = broadcast_tx.clone();

    tokio::spawn(async move {
        scheduler::start_scheduler(
            scheduler_config,
            scheduler_pool,
            scheduler_binance,
            scheduler_discord,
            scheduler_broadcast,
        )
        .await;
    });

    // Start HTTP server
    let addr = format!("{}:{}", config.api_host, config.api_port);
    info!("🌐 API server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
