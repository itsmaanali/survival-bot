use anyhow::Result;
use serde::Deserialize;
use tracing::{info, warn};

#[derive(Debug, Deserialize)]
struct FearGreedResponse {
    data: Vec<FearGreedData>,
}

#[derive(Debug, Deserialize)]
struct FearGreedData {
    value: String,
    value_classification: String,
}

/// Fetch the current Fear & Greed Index from alternative.me.
/// Returns 50 (neutral) on any failure — non-critical data.
pub async fn fetch_fear_greed_index() -> i32 {
    match fetch_inner().await {
        Ok(value) => {
            info!(value, "Fear & Greed Index fetched");
            value
        }
        Err(e) => {
            warn!(error = %e, "Failed to fetch Fear & Greed Index — defaulting to 50");
            50
        }
    }
}

async fn fetch_inner() -> Result<i32> {
    let url = "https://api.alternative.me/fng/?limit=1";
    let client = reqwest::Client::new();
    let resp = client.get(url).send().await?;
    let data: FearGreedResponse = resp.json().await?;

    let value = data
        .data
        .first()
        .map(|d| d.value.parse::<i32>().unwrap_or(50))
        .unwrap_or(50);

    Ok(value)
}
