use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type PriceMap = HashMap<String, f64>;

#[derive(Clone, Default)]
pub struct AppState {
    pub last_scan: Vec<ArbResult>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScanRequest {
    pub exchanges: Vec<String>,
    pub min_profit: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScanResult {
    pub exchange: String,
    pub route: String,
    pub profit_before: f64,
    pub fee: f64,
    pub profit_after: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ArbResult {
    pub exchange: String,
    pub route: String,
    pub profit_before: f64,
    pub fee: f64,
    pub profit_after: f64,
    pub spread: f64,
}
