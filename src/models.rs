use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairPrice {
    pub base: String,
    pub quote: String,
    pub price: f64,
    pub is_spot: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriangularResult {
    pub triangle: String,            // e.g. "BTC/USDT -> ETH/BTC -> ETH/USDT"
    pub pairs: String,               // e.g. "BTC/USDT | ETH/BTC | ETH/USDT"
    pub profit_before_fees: f64,     // % profit before fees
    pub trade_fees: f64,             // % total fees for 3 trades
    pub profit_after_fees: f64,      // % profit after fees
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScanRequest {
    pub exchanges: Vec<String>,
    pub min_profit: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScanResponse {
    pub status: String,
    pub count: usize,
    pub results: Vec<TriangularResult>,
}
