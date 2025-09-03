use serde::{Deserialize, Serialize};

#[derive(Default)]
pub struct AppState {
    /// Stores the latest scan results for reuse or API UI retrieval
    pub last_results: Option<Vec<ArbResult>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ArbResult {
    pub exchange: String,       // Exchange name
    pub route: String,          // The triangular route like A → B → C → A
    pub profit_before: f64,     // Profit before fees
    pub fee: f64,               // Total fees in percentage
    pub profit_after: f64,      // Profit after fees
    pub spread: f64,            // Spread percentage
}

#[derive(Deserialize)]
pub struct ScanRequest {
    pub exchanges: Vec<String>, // List of exchanges to scan
    pub min_profit: f64,        // Minimum profit filter
}

#[derive(Serialize)]
pub struct ScanResponse {
    pub status: String,
    pub count: usize,
    pub results: Vec<ArbResult>,
}
