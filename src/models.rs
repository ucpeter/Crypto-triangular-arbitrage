use std::collections::HashMap;
use serde::{Deserialize, Serialize};

pub type PriceMap = HashMap<String, f64>; // <--- Add this line

#[derive(Default)]
pub struct AppState {
    pub last_results: Option<Vec<ArbResult>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ArbResult {
    pub exchange: String,
    pub route: String,
    pub profit_before: f64,
    pub fee: f64,
    pub profit_after: f64,
    pub spread: f64,
}

#[derive(Deserialize)]
pub struct ScanRequest {
    pub exchanges: Vec<String>,
    pub min_profit: f64,
}

#[derive(Serialize)]
pub struct ScanResponse {
    pub status: String,
    pub count: usize,
    pub results: Vec<ArbResult>,
}
