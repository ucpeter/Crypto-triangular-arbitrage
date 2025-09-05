use serde::{Deserialize, Serialize};

/// Shared application state (stores last scan results)
#[derive(Default)]
pub struct AppState {
    pub last_results: Option<Vec<TriangularResult>>,
}

/// Single pair price (spot)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairPrice {
    pub base: String,
    pub quote: String,
    pub price: f64,
    pub is_spot: bool,
}

/// Triangular result passed to UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriangularResult {
    pub triangle: String,           // e.g. "USDT → BTC → ETH → USDT"
    pub pairs: String,              // e.g. "USDT/BTC | BTC/ETH | ETH/USDT"
    pub profit_before_fees: f64,
    pub trade_fees: f64,
    pub profit_after_fees: f64,
}

/// Request body from UI
#[derive(Debug, Clone, Deserialize)]
pub struct ScanRequest {
    pub exchanges: Vec<String>,
    pub min_profit: f64,
    }
