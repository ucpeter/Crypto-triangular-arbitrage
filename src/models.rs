use serde::Deserialize;

#[derive(Clone, Debug)]
pub struct ArbitrageResult {
    pub route: String,
    pub profit_before: f64,
    pub fee: f64,
    pub profit_after: f64,
    pub spread: f64,
}

#[derive(Clone, Debug)]
pub struct MarketPrice {
    pub symbol: String,
    pub price: f64,
    pub exchange: String,
}

#[derive(Deserialize)]
pub struct BinanceTicker {
    pub symbol: String,
    pub price: String,
}
