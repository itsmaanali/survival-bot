use serde::{Deserialize, Serialize};

// ─── Account Info ────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountInfo {
    pub balances: Vec<AssetBalance>,
}

#[derive(Debug, Deserialize)]
pub struct AssetBalance {
    pub asset: String,
    pub free: String,
    pub locked: String,
}

// ─── Ticker 24h ──────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ticker24h {
    pub symbol: String,
    pub price_change: String,
    pub price_change_percent: String,
    pub last_price: String,
    pub high_price: String,
    pub low_price: String,
    pub volume: String,
    pub quote_volume: String,
}

// ─── Order Response ──────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderResponse {
    pub symbol: String,
    pub order_id: u64,
    pub status: String,
    pub side: String,
    #[serde(rename = "type")]
    pub order_type: String,
    pub executed_qty: String,
    pub cummulative_quote_qty: String,
    pub fills: Vec<OrderFill>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderFill {
    pub price: String,
    pub qty: String,
    pub commission: String,
    pub commission_asset: String,
}

// ─── Computed types for internal use ─────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct ExecutedTrade {
    pub symbol: String,
    pub side: String,
    pub quantity: f64,
    pub avg_price: f64,
    pub usdc_amount: f64,
    pub commission: f64,
}

impl OrderResponse {
    /// Convert to an ExecutedTrade with computed averages
    pub fn to_executed_trade(&self) -> ExecutedTrade {
        let quantity: f64 = self.executed_qty.parse().unwrap_or(0.0);
        let usdc_amount: f64 = self.cummulative_quote_qty.parse().unwrap_or(0.0);
        let avg_price = if quantity > 0.0 {
            usdc_amount / quantity
        } else {
            0.0
        };
        let commission: f64 = self
            .fills
            .iter()
            .map(|f| f.commission.parse::<f64>().unwrap_or(0.0))
            .sum();

        ExecutedTrade {
            symbol: self.symbol.clone(),
            side: self.side.clone(),
            quantity,
            avg_price,
            usdc_amount,
            commission,
        }
    }
}
