use serde::{Serialize, Deserialize};

#[derive(Debug, Clone)]
pub struct PairPrice {
    pub base: String,
    pub quote: String,
    pub price: f64,
    pub is_spot: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TriangularResult {
    pub triangle: String,           // "A/B -> B/C -> C/A"
    pub profit_before_fees: f64,    // %
    pub trade_fees: f64,            // total fee in %
    pub profit_after_fees: f64,     // %
}
