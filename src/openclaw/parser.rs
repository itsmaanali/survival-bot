use crate::db::models::{TradingAction, TradingDecision};
use regex::Regex;
use tracing::{info, warn};

/// Parse OpenClaw's response into a TradingDecision.
/// Uses two-stage parsing: regex to extract JSON from markdown, then serde.
/// Falls back to HOLD on any error.
pub fn parse_decision(raw_response: &str) -> TradingDecision {
    let default_hold = TradingDecision {
        action: TradingAction::Hold,
        symbol: None,
        confidence: 0,
        reasoning: "Failed to parse OpenClaw response — defaulting to HOLD".to_string(),
        stop_loss: None,
        take_profit: None,
    };

    // Stage 1: Try to extract JSON from markdown code blocks
    let json_str = extract_json(raw_response).unwrap_or_else(|| raw_response.trim().to_string());

    // Stage 2: Parse JSON
    match serde_json::from_str::<TradingDecision>(&json_str) {
        Ok(decision) => {
            info!(
                action = %decision.action,
                confidence = decision.confidence,
                symbol = ?decision.symbol,
                "Parsed trading decision"
            );

            // Validate: BUY/SELL must have a symbol
            if decision.action != TradingAction::Hold && decision.symbol.is_none() {
                warn!("BUY/SELL decision missing symbol — defaulting to HOLD");
                return default_hold;
            }

            decision
        }
        Err(e) => {
            warn!(error = %e, raw = %raw_response, "Failed to parse decision JSON");
            default_hold
        }
    }
}

/// Extract JSON from potential markdown code blocks
fn extract_json(text: &str) -> Option<String> {
    // Try to find JSON in ```json ... ``` blocks
    let re = Regex::new(r"```(?:json)?\s*\n?([\s\S]*?)\n?\s*```").ok()?;
    if let Some(caps) = re.captures(text) {
        return Some(caps[1].trim().to_string());
    }

    // Try to find a raw JSON object
    let re_obj = Regex::new(r"\{[\s\S]*\}").ok()?;
    if let Some(m) = re_obj.find(text) {
        return Some(m.as_str().trim().to_string());
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_json_block() {
        let response = r#"
Here's my analysis:
```json
{
  "action": "BUY",
  "symbol": "BTCUSDC",
  "confidence": 85,
  "reasoning": "Strong uptrend",
  "stop_loss": 42000.0,
  "take_profit": 48000.0
}
```
        "#;

        let decision = parse_decision(response);
        assert_eq!(decision.action, TradingAction::Buy);
        assert_eq!(decision.symbol.as_deref(), Some("BTCUSDC"));
        assert_eq!(decision.confidence, 85);
    }

    #[test]
    fn test_parse_raw_json() {
        let response = r#"{"action":"HOLD","confidence":50,"reasoning":"Market uncertain"}"#;
        let decision = parse_decision(response);
        assert_eq!(decision.action, TradingAction::Hold);
    }

    #[test]
    fn test_parse_invalid_defaults_to_hold() {
        let decision = parse_decision("I think you should buy Bitcoin!");
        assert_eq!(decision.action, TradingAction::Hold);
    }

    #[test]
    fn test_buy_without_symbol_defaults_to_hold() {
        let response = r#"{"action":"BUY","confidence":90,"reasoning":"Go all in"}"#;
        let decision = parse_decision(response);
        assert_eq!(decision.action, TradingAction::Hold);
    }
}
