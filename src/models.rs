use serde::{Serialize, Deserialize};

#[derive(Default)]
pub struct AppState {
    pub last_results: Option<Vec<ArbResult>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScanRequest {
    pub exchanges: Vec<String>,
    pub min_profit: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ArbResult {
    pub exchange: String,
    pub route: String,   // triangle path like A → B → C → A
    pub pairs: String,   // "A/B | B/C | C/A"
    pub profit_before: f64,
    pub fee: f64,
    pub profit_after: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScanResponse {
    pub status: String,
    pub count: usize,
    pub results: Vec<ArbResult>,
}
