use crate::binance::types::Ticker24h;
use crate::db::models::Position;

/// Build a structured prompt for OpenClaw with all market context
pub fn build_prompt(
    balance_usdc: f64,
    open_positions: &[Position],
    top_tickers: &[Ticker24h],
    fear_greed_index: i32,
    consecutive_losses: i64,
    openclaw_user_id: &str,
) -> String {
    let mut prompt = String::with_capacity(4096);

    // Header — mention OpenClaw so it triggers a response
    prompt.push_str(&format!("<@{}> 🤖 **SURVIVAL TRADING BOT — CYCLE ANALYSIS REQUEST**\n\n", openclaw_user_id));

    // Balance & Status
    prompt.push_str(&format!("💰 **Available USDC Balance:** ${:.2}\n", balance_usdc));
    prompt.push_str(&format!("📊 **Fear & Greed Index:** {}/100\n", fear_greed_index));
    prompt.push_str(&format!("📉 **Consecutive Losses:** {}\n\n", consecutive_losses));

    // Open Positions
    if open_positions.is_empty() {
        prompt.push_str("📂 **Open Positions:** None\n\n");
    } else {
        prompt.push_str("📂 **Open Positions:**\n");
        for pos in open_positions {
            let current = pos.current_price.unwrap_or(pos.entry_price);
            let pnl_pct = ((current - pos.entry_price) / pos.entry_price) * 100.0;
            prompt.push_str(&format!(
                "  • {} | Entry: ${:.6} | Current: ${:.6} | P&L: {:.2}% | SL: {} | TP: {}\n",
                pos.symbol,
                pos.entry_price,
                current,
                pnl_pct,
                pos.stop_loss
                    .map(|v| format!("${:.6}", v))
                    .unwrap_or_else(|| "N/A".to_string()),
                pos.take_profit
                    .map(|v| format!("${:.6}", v))
                    .unwrap_or_else(|| "N/A".to_string()),
            ));
        }
        prompt.push('\n');
    }

    // Top Movers (top 10 by volume)
    prompt.push_str("📈 **Top USDC Pairs (by 24h volume):**\n");
    let mut sorted_tickers = top_tickers.to_vec();
    sorted_tickers.sort_by(|a, b| {
        let vol_a: f64 = a.quote_volume.parse().unwrap_or(0.0);
        let vol_b: f64 = b.quote_volume.parse().unwrap_or(0.0);
        vol_b.partial_cmp(&vol_a).unwrap_or(std::cmp::Ordering::Equal)
    });

    for ticker in sorted_tickers.iter().take(10) {
        prompt.push_str(&format!(
            "  • {} | Price: ${} | 24h Change: {}% | Volume: ${}\n",
            ticker.symbol, ticker.last_price, ticker.price_change_percent, ticker.quote_volume
        ));
    }
    prompt.push('\n');

    // Rules
    prompt.push_str("⚠️ **RULES (MUST FOLLOW):**\n");
    prompt.push_str("1. This is a SURVIVAL game. If balance reaches $0, the bot dies forever.\n");
    prompt.push_str("2. Only HALAL spot trading. No leverage, no shorting, no derivatives.\n");
    prompt.push_str("3. Max 2 open positions at any time.\n");
    prompt.push_str("4. Max 10% of tradeable balance per trade.\n");
    prompt.push_str("5. Always set stop-loss (max 5% below entry) and take-profit.\n");
    prompt.push_str("6. If Fear & Greed < 25 (Extreme Fear), be very conservative.\n");

    if consecutive_losses >= 3 {
        prompt.push_str("7. ⚡ ULTRA-CONSERVATIVE MODE: 3+ consecutive losses. Only trade with extremely high confidence.\n");
    }

    prompt.push('\n');

    // Required Response Format
    prompt.push_str("📋 **RESPOND WITH EXACTLY THIS JSON FORMAT (no markdown, no extra text):**\n");
    prompt.push_str("```json\n");
    prompt.push_str("{\n");
    prompt.push_str("  \"action\": \"BUY\" | \"SELL\" | \"HOLD\",\n");
    prompt.push_str("  \"symbol\": \"BTCUSDC\" (required if BUY/SELL),\n");
    prompt.push_str("  \"confidence\": 0-100,\n");
    prompt.push_str("  \"reasoning\": \"Brief explanation of your decision\",\n");
    prompt.push_str("  \"stop_loss\": 50000.00 (required if BUY, price to cut losses),\n");
    prompt.push_str("  \"take_profit\": 55000.00 (required if BUY, price to take profit)\n");
    prompt.push_str("}\n");
    prompt.push_str("```\n");

    prompt
}
