// src/models.rs

/// PairPrice represents a tradable market pair on an exchange
#[derive(Debug, Clone)]
pub struct PairPrice {
    pub base: String,
    pub quote: String,
    pub price: f64,
    pub is_spot: bool,

    // New fields for liquidity
    pub base_volume: f64,   // 24h volume in base asset
    pub quote_volume: f64,  // 24h volume in quote asset
}

/// ArbitrageOpportunity is computed from a triangle of pairs
#[derive(Debug, Clone)]
pub struct ArbitrageOpportunity {
    pub triangle: String,
    pub pairs: Vec<String>,
    pub profit_before_fees: f64,
    pub trade_fees: f64,
    pub profit_after_fees: f64,

    // New fields for liquidity
    pub min_liquidity: f64,       // smallest liquidity across all legs (in quote terms)
    pub leg_liquidities: Vec<f64> // liquidity for each leg
}
