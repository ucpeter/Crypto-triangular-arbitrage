use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type PriceMap = HashMap<String, f64>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArbResult {
    pub exchange: String,
    pub route: String,
    pub pairs: Option<String>,     // NEW → explicit pairs like A/B | B/C | C/A
    pub profit_before: f64,
    pub fee: f64,
    pub profit_after: f64,
    pub spread: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScanRequest {
    pub exchanges: Vec<String>,
    pub min_profit: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScanResponse {
    pub status: String,
    pub count: usize,
    pub results: Vec<ArbResult>,
}

#[derive(Clone, Debug, Default)]
pub struct AppState {
    pub last_results: Option<Vec<ArbResult>>,
}
