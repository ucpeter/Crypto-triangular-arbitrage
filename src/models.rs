use serde::{Deserialize, Serialize};

#[derive(Clone, Default)]
pub struct AppState {
    pub last_results: Vec<ArbResult>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ArbResult {
    pub exchange: String,
    pub route: String,           // e.g. "[BINANCE] BTC → ETH → USDT → BTC"
    pub profit_before: f64,      // %
    pub fee: f64,                // total fee % assumed for 3 hops
    pub profit_after: f64,       // %
}
