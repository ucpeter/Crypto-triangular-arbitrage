use serde::{Deserialize, Serialize};

/// Shared application state (keeps last results)
#[derive(Default)]
pub struct AppState {
    pub last_results: Option<Vec<TriangularResult>>,
}

/// Represents a single trading pair price
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairPrice {
    pub base: String,
    pub quote: String,
    pub price: f64,
    pub is_spot: bool,
}

/// A single triangular arbitrage result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriangularResult {
    /// e.g., "USDT → BTC → ETH → USDT"
    pub triangle: String,

    /// e.g., "USDT/BTC | BTC/ETH | ETH/USDT"
    pub pairs: String,

    /// Profit before fees (%)
    pub profit_before_fees: f64,

    /// Total trade fees applied (%)
    pub trade_fees: f64,

    /// Profit after fees (%)
    pub profit_after_fees: f64,
}

/// Request body for POST /scan
#[derive(Debug, Clone, Deserialize)]
pub struct ScanRequest {
    /// Exchanges to scan, e.g., ["binance", "kraken"]
    pub exchanges: Vec<String>,

    /// Minimum profit margin (before fees) to include
    pub min_profit: f64,
}

/// Response wrapper (if you want strong typing, but currently JSON is built in routes)
#[derive(Debug, Clone, Serialize)]
pub struct ScanResponse {
    pub status: String,
    pub count: usize,
    pub results: Vec<TriangularResult>,
}
