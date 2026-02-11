use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct Config {
    // Database
    pub database_url: String,

    // Binance
    pub binance_api_key: String,
    pub binance_secret_key: String,
    pub binance_base_url: String,

    // Discord / OpenClaw
    pub discord_bot_token: String,
    pub discord_channel_id: String,
    pub openclaw_user_id: String,

    // Server
    pub api_host: String,
    pub api_port: u16,

    // Trading
    pub min_balance_usdc: f64,

    // Kill switch
    pub kill_secret: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok(); // Load .env if present, ignore if missing

        Ok(Config {
            database_url: std::env::var("DATABASE_URL")
                .context("DATABASE_URL not set")?,
            binance_api_key: std::env::var("BINANCE_API_KEY")
                .context("BINANCE_API_KEY not set")?,
            binance_secret_key: std::env::var("BINANCE_SECRET_KEY")
                .context("BINANCE_SECRET_KEY not set")?,
            binance_base_url: std::env::var("BINANCE_BASE_URL")
                .unwrap_or_else(|_| "https://api.binance.com".to_string()),
            discord_bot_token: std::env::var("DISCORD_BOT_TOKEN")
                .context("DISCORD_BOT_TOKEN not set")?,
            discord_channel_id: std::env::var("DISCORD_CHANNEL_ID")
                .context("DISCORD_CHANNEL_ID not set")?,
            openclaw_user_id: std::env::var("OPENCLAW_USER_ID")
                .context("OPENCLAW_USER_ID not set")?,
            api_host: std::env::var("API_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            api_port: std::env::var("API_PORT")
                .unwrap_or_else(|_| "3001".to_string())
                .parse()
                .context("API_PORT must be a valid port number")?,
            min_balance_usdc: std::env::var("MIN_BALANCE_USDC")
                .unwrap_or_else(|_| "5.0".to_string())
                .parse()
                .context("MIN_BALANCE_USDC must be a valid number")?,
            kill_secret: std::env::var("KILL_SECRET")
                .unwrap_or_else(|_| "changeme".to_string()),
        })
    }
}
