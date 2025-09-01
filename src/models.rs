use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type PriceMap = HashMap<String, f64>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArbResult {
    pub exchange: String,
    pub route: String,
    pub profit_before: f64,
    pub fee: f64,
    pub profit_after: f64,
    pub spread: f64,
}

impl Default for ArbResult {
    fn default() -> Self {
        Self {
            exchange: String::new(),
            route: String::new(),
            profit_before: 0.0,
            fee: 0.0,
            profit_after: 0.0,
            spread: 0.0,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct AppState {
    pub last_results: Vec<ArbResult>,
}
