use serde::{Deserialize, Serialize};

/// Shared app state
#[derive(Default)]
pub struct AppState {
    pub last_results: Option<Vec<TriangularResult>>,
}

/// Input payload when user hits "Scan"
#[derive(Debug, Deserialize)]
pub struct ScanRequest {
    pub exchanges: Vec<String>,
    pub min_profit: f64,
}

/// Output payload for UI
#[derive(Debug, Serialize)]
pub struct ScanResponse {
    pub status: String,
    pub count: usize,
    pub results: Vec<TriangularResult>,
}

/// Individual spot trading pair price
#[derive(Debug, Clone)]
pub struct PairPrice {
    pub base: String,
    pub quote: String,
    pub price: f64,
    pub is_spot: bool,
    pub liquidity: f64, // ✅ new: reported 24h quote volume
}

/// Single triangular arbitrage opportunity
#[derive(Debug, Clone, Serialize)]
pub struct TriangularResult {
    /// Triangle path like `BTC → ETH → USDT → BTC`
    pub triangle: String,
    /// The actual tradable pairs in that path
    pub pairs: String,
    /// Profit margin before fees
    pub profit_before_fees: f64,
    /// Total trade fees considered
    pub trade_fees: f64,
    /// Net profit margin after fees
    pub profit_after_fees: f64,
    /// ✅ minimum liquidity among the 3 legs (quote currency volume)
    pub min_liquidity: f64,
}
