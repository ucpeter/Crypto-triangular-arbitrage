use serde::Serialize;
use std::collections::HashMap;

/// Shared application state
#[derive(Default)]
pub struct AppState {
    /// A simple cache for storing market prices or other shared data
    pub cache: HashMap<String, f64>,
}

/// Represents one arbitrage opportunity found in the scan
#[derive(Serialize)]
pub struct ScanResult {
    /// The triangle pair, e.g., "BTC/USDT -> ETH/USDT -> BTC/ETH"
    pub pair: String,
    /// Profit percentage before exchange fees are deducted
    pub profit_before_fee: f64,
    /// Estimated fees percentage
    pub fee: f64,
    /// Profit percentage after exchange fees
    pub profit_after_fee: f64,
}
