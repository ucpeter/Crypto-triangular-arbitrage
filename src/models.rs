use serde::{Deserialize, Serialize};

/// Represents a single trading pair price
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairPrice {
    pub base: String,
    pub quote: String,
    pub price: f64,
    pub is_spot: bool,
}

/// Represents a triangular arbitrage result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriangularResult {
    pub triangle: String,           // e.g. USDT/BTC -> BTC/ETH -> ETH/USDT
    pub profit_before_fees: f64,    // % before fees
    pub trade_fees: f64,            // total % fees
    pub profit_after_fees: f64,     // % after fees
}

/// Request payload from UI
#[derive(Debug, Clone, Deserialize)]
pub struct ScanRequest {
    pub exchanges: Vec<String>, // selected exchanges
    pub min_profit: f64,        // minimum % profit
}

/// Response to UI
#[derive(Debug, Clone, Serialize)]
pub struct ScanResponse {
    pub count: usize,
    pub results: Vec<TriangularResult>,
}
