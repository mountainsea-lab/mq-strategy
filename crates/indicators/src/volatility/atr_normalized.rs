use nautilus_indicators::average::MovingAverageType;
use nautilus_indicators::indicator::Indicator;
use nautilus_indicators::volatility::atr::AverageTrueRange;
use nautilus_model::{data::Bar, enums::PriceType};
use std::fmt::Display;

/// An indicator that computes a normalized Average True Range (ATR) over a rolling window.
///
/// This indicator expresses market volatility in a dimensionless form by scaling
/// the ATR relative to the current price (typically the close price).
///
/// # Formula
/// text /// normalized_atr = ATR(n) / price ///
///
/// Where:
/// - ATR(n) is the Average True Range over a window of length n
/// - price is usually the closing price of the bar
///
/// # Description
/// Unlike the raw ATR, which is price-dependent, this normalized version allows:
/// - Comparing volatility across different instruments
/// - Using consistent thresholds in multi-asset strategies
/// - Improving robustness in position sizing and risk management
///
/// # Interpretation
/// - Higher values indicate higher relative volatility
/// - Lower values indicate more stable price movement
///
/// # Use Cases
/// - Volatility filtering (e.g., avoid trading in low volatility regimes)
/// - Dynamic stop-loss / take-profit scaling
/// - Cross-asset strategy normalization
///
/// # Notes
/// - If multiplied by 100, this can be interpreted as a percentage (ATRPercent)
/// - Also commonly referred to as:
/// - ATRRatio
/// - RelativeATR
///
/// # References
/// - J. Welles Wilder, New Concepts in Technical Trading Systems
///
/// # Warning
/// Ensure the denominator (price) is non-zero to avoid division errors.
#[repr(C)]
#[derive(Debug)]
pub struct ATRNormalized {
    pub period: usize,
    pub price_type: PriceType,
    pub value: f64,
    pub count: usize,
    pub initialized: bool,
    has_inputs: bool,
    atr: AverageTrueRange,
}

impl Display for ATRNormalized {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({})", self.name(), self.period)
    }
}

impl Indicator for ATRNormalized {
    fn name(&self) -> String {
        stringify!(ATRNormalized).to_string()
    }

    fn has_inputs(&self) -> bool {
        self.has_inputs
    }

    fn initialized(&self) -> bool {
        self.initialized
    }

    fn handle_bar(&mut self, bar: &Bar) {
        self.update_raw(bar);
    }

    fn reset(&mut self) {
        self.value = 0.0;
        self.atr.reset();
        self.count = 0;
        self.has_inputs = false;
        self.initialized = false;
    }
}
impl ATRNormalized {
    /// Creates a new [`ATRNormalized`] instance.
    ///
    /// # Panics
    ///
    /// Panics if `period` is not a positive integer (> 0).
    #[must_use]
    pub fn new(period: usize, price_type: Option<PriceType>) -> Self {
        assert!(
            period > 0,
            "ATRNormalized: period must be > 0 (received {period})"
        );

        let pt = price_type.unwrap_or(PriceType::Last);

        Self {
            period,
            price_type: pt,
            value: 0.0,
            count: 0,
            has_inputs: false,
            initialized: false,
            atr: AverageTrueRange::new(
                period, // 14-period ATR (可以根据实际情况调整为10~20周期)
                Some(MovingAverageType::Simple),
                Some(true),   // 使用前一收盘价
                Some(0.0001), // 设置一个最低的 ATR 值（比如0.0001）
            ),
        }
    }

    pub fn update_raw(&mut self, bar: &Bar) {
        // Update ATR using bar data (high, low, close)
        self.atr
            .update_raw(bar.high.as_f64(), bar.low.as_f64(), bar.close.as_f64());

        // Perform the normalization step using (open + close) / 2
        let open = bar.open.as_f64();
        let close = bar.close.as_f64();
        let open_close_avg = (open + close) / 2.0;

        if open_close_avg != 0.0 {
            // Normalize the ATR value by dividing by the average of open and close
            self.value = self.atr.value / open_close_avg;
        } else {
            // Avoid division by zero (equivalent to np.nan_to_num)
            self.value = 0.0;
        }

        // Increment count and check if initialized
        self.increment_count();
    }

    fn increment_count(&mut self) {
        self.count += 1;

        if !self.initialized {
            self.has_inputs = true;

            if self.count >= self.period {
                self.initialized = true;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use nautilus_core::UnixNanos;
    use nautilus_model::types::{Price, Quantity};
    use rstest::rstest;

    use super::*;

    #[rstest]
    fn test_name_returns_expected_string() {
        let atr = ATRNormalized::new(10, Some(PriceType::Last));
        assert_eq!(atr.name(), "ATRNormalized");
    }

    #[rstest]
    fn test_value_with_three_inputs() {
        let mut atr = ATRNormalized::new(10, Some(PriceType::Last));
        let bar = Bar {
            high: Price::new(100.0, 2),
            low: Price::new(90.0, 2),
            close: Price::new(95.0, 2),
            open: Price::new(92.0, 2),
            volume: Quantity::from("1000.0"),
            ..Default::default()
        };
        atr.update_raw(&bar);
        // assert!(approx_equal(atr.value, 0.050));
    }
}
