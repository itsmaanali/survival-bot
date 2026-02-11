use anyhow::{Context, Result};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use tracing::{info, warn};

use super::types::*;

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone)]
pub struct BinanceClient {
    base_url: String,
    api_key: String,
    secret_key: String,
    http: reqwest::Client,
}

impl BinanceClient {
    pub fn new(base_url: &str, api_key: &str, secret_key: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: api_key.to_string(),
            secret_key: secret_key.to_string(),
            http: reqwest::Client::new(),
        }
    }

    /// Generate HMAC-SHA256 signature for Binance API
    fn sign(&self, query: &str) -> String {
        let mut mac =
            HmacSha256::new_from_slice(self.secret_key.as_bytes()).expect("HMAC key error");
        mac.update(query.as_bytes());
        hex::encode(mac.finalize().into_bytes())
    }

    /// Get current timestamp in milliseconds
    fn timestamp() -> u64 {
        chrono::Utc::now().timestamp_millis() as u64
    }

    /// Get USDC balance
    pub async fn get_usdc_balance(&self) -> Result<f64> {
        let timestamp = Self::timestamp();
        let query = format!("timestamp={}", timestamp);
        let signature = self.sign(&query);

        let url = format!(
            "{}/api/v3/account?{}&signature={}",
            self.base_url, query, signature
        );

        let resp = self
            .http
            .get(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .send()
            .await
            .context("Failed to fetch Binance account")?;

        let status = resp.status();
        let body = resp.text().await.context("Failed to read account response")?;

        if !status.is_success() {
            warn!(status = %status, body = %body, "Binance account request failed");
            anyhow::bail!("Binance account failed ({}): {}", status, body);
        }

        let account: AccountInfo = serde_json::from_str(&body)
            .context(format!("Failed to parse account info. Response: {}", &body[..body.len().min(500)]))?;

        let usdc_balance = account
            .balances
            .iter()
            .find(|b| b.asset == "USDC")
            .map(|b| b.free.parse::<f64>().unwrap_or(0.0))
            .unwrap_or(0.0);

        info!(balance = usdc_balance, "USDC balance fetched");
        Ok(usdc_balance)
    }

    /// Get all non-zero balances (for position reconciliation)
    pub async fn get_all_balances(&self) -> Result<Vec<(String, f64)>> {
        let timestamp = Self::timestamp();
        let query = format!("timestamp={}", timestamp);
        let signature = self.sign(&query);

        let url = format!(
            "{}/api/v3/account?{}&signature={}",
            self.base_url, query, signature
        );

        let resp = self
            .http
            .get(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .send()
            .await
            .context("Failed to fetch Binance account")?;

        let account: AccountInfo = resp.json().await?;

        let balances: Vec<(String, f64)> = account
            .balances
            .iter()
            .filter_map(|b| {
                let free: f64 = b.free.parse().unwrap_or(0.0);
                let locked: f64 = b.locked.parse().unwrap_or(0.0);
                let total = free + locked;
                if total > 0.0 && b.asset != "USDC" {
                    Some((b.asset.clone(), total))
                } else {
                    None
                }
            })
            .collect();

        Ok(balances)
    }

    /// Get 24h ticker data for USDC trading pairs
    pub async fn get_tickers(&self) -> Result<Vec<Ticker24h>> {
        let url = format!("{}/api/v3/ticker/24hr", self.base_url);

        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .context("Failed to fetch tickers")?;

        let all_tickers: Vec<Ticker24h> = resp.json().await?;

        // Filter to only USDC pairs
        let usdc_tickers: Vec<Ticker24h> = all_tickers
            .into_iter()
            .filter(|t| t.symbol.ends_with("USDC"))
            .collect();

        info!(count = usdc_tickers.len(), "USDC tickers fetched");
        Ok(usdc_tickers)
    }

    /// Get 24h ticker for a specific symbol
    pub async fn get_ticker(&self, symbol: &str) -> Result<Ticker24h> {
        let url = format!(
            "{}/api/v3/ticker/24hr?symbol={}",
            self.base_url, symbol
        );

        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .context("Failed to fetch ticker")?;

        let ticker: Ticker24h = resp.json().await?;
        Ok(ticker)
    }

    /// Execute a market buy order (denominated in USDC)
    pub async fn market_buy(&self, symbol: &str, usdc_amount: f64) -> Result<OrderResponse> {
        info!(symbol, usdc_amount, "Executing market BUY");

        let timestamp = Self::timestamp();
        let query = format!(
            "symbol={}&side=BUY&type=MARKET&quoteOrderQty={:.2}&timestamp={}",
            symbol, usdc_amount, timestamp
        );
        let signature = self.sign(&query);

        let url = format!(
            "{}/api/v3/order?{}&signature={}",
            self.base_url, query, signature
        );

        let resp = self
            .http
            .post(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .send()
            .await
            .context("Failed to execute market buy")?;

        let status = resp.status();
        let body = resp.text().await?;

        if !status.is_success() {
            warn!(status = %status, body = %body, "Binance order failed");
            anyhow::bail!("Binance order failed ({}): {}", status, body);
        }

        let order: OrderResponse =
            serde_json::from_str(&body).context("Failed to parse order response")?;

        info!(
            order_id = order.order_id,
            executed_qty = %order.executed_qty,
            "Market BUY executed"
        );
        Ok(order)
    }

    /// Execute a market sell order (denominated in coin quantity)
    pub async fn market_sell(&self, symbol: &str, quantity: f64) -> Result<OrderResponse> {
        info!(symbol, quantity, "Executing market SELL");

        let timestamp = Self::timestamp();
        // Use appropriate precision for quantity
        let query = format!(
            "symbol={}&side=SELL&type=MARKET&quantity={:.8}&timestamp={}",
            symbol, quantity, timestamp
        );
        let signature = self.sign(&query);

        let url = format!(
            "{}/api/v3/order?{}&signature={}",
            self.base_url, query, signature
        );

        let resp = self
            .http
            .post(&url)
            .header("X-MBX-APIKEY", &self.api_key)
            .send()
            .await
            .context("Failed to execute market sell")?;

        let status = resp.status();
        let body = resp.text().await?;

        if !status.is_success() {
            warn!(status = %status, body = %body, "Binance sell order failed");
            anyhow::bail!("Binance sell order failed ({}): {}", status, body);
        }

        let order: OrderResponse =
            serde_json::from_str(&body).context("Failed to parse sell order response")?;

        info!(
            order_id = order.order_id,
            executed_qty = %order.executed_qty,
            "Market SELL executed"
        );
        Ok(order)
    }
}
