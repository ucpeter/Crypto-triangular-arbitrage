use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct Market {
    pub symbol: String,
    pub base: String,
    pub quote: String,
    pub active: bool,
    pub spot: bool,
}

#[derive(Debug, Serialize)]
pub struct ArbitrageResult {
    pub exchange: String,
    pub triangle: String,
    pub profit_percent: f64,
}
