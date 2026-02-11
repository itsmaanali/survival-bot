/// Position sizing based on OpenClaw's confidence level.
/// Higher confidence = larger position (up to 10% of tradeable balance).
pub struct PositionSizer;

impl PositionSizer {
    /// Calculate position size in USDC based on confidence level.
    /// Returns 0.0 if confidence is below 70 (forced HOLD).
    pub fn calculate(balance_usdc: f64, confidence: i32, min_balance: f64) -> f64 {
        // Reserve minimum balance for infrastructure costs
        let tradeable = (balance_usdc - min_balance).max(0.0);

        if tradeable <= 0.0 {
            return 0.0;
        }

        let percentage = match confidence {
            90..=100 => 0.10, // 10%
            80..=89 => 0.06,  // 6%
            70..=79 => 0.03,  // 3%
            _ => 0.0,         // Below 70: forced HOLD
        };

        let size = tradeable * percentage;

        // Enforce minimum order size (Binance requires ~$5 minimum for most pairs)
        if size < 5.0 {
            return 0.0;
        }

        size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_high_confidence() {
        let size = PositionSizer::calculate(100.0, 95, 5.0);
        assert!((size - 9.5).abs() < 0.01); // 10% of 95 tradeable
    }

    #[test]
    fn test_medium_confidence() {
        let size = PositionSizer::calculate(100.0, 85, 5.0);
        assert!((size - 5.7).abs() < 0.01); // 6% of 95 tradeable
    }

    #[test]
    fn test_low_confidence_forced_hold() {
        let size = PositionSizer::calculate(100.0, 60, 5.0);
        assert_eq!(size, 0.0);
    }

    #[test]
    fn test_below_minimum_balance() {
        let size = PositionSizer::calculate(4.0, 95, 5.0);
        assert_eq!(size, 0.0);
    }
}
