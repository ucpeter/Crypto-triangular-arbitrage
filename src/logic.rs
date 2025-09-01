use crate::models::{ArbitrageResult, MarketPrice};
use std::collections::HashMap;

pub fn find_triangular_arbitrage(prices: Vec<MarketPrice>) -> Vec<ArbitrageResult> {
    let mut results = Vec::new();
    let mut price_map: HashMap<String, Vec<&MarketPrice>> = HashMap::new();

    // Group prices by symbol for easy lookup
    for price in &prices {
        price_map.entry(price.symbol.clone()).or_default().push(price);
    }

    // Naive example: find simple triangles with USDT, BTC, and ETH
    let bases = vec!["USDT", "BTC", "ETH"];
    for base in &bases {
        for quote in &bases {
            if base != quote {
                let pair = format!("{}{}", base, quote);
                if let Some(markets) = price_map.get(&pair) {
                    let min_price = markets.iter().map(|m| m.price).fold(f64::INFINITY, f64::min);
                    let max_price = markets.iter().map(|m| m.price).fold(0.0, f64::max);

                    if max_price > min_price {
                        let profit_before = ((max_price - min_price) / min_price) * 100.0;
                        let fee = 0.2;
                        let profit_after = profit_before - fee;
                        let spread = max_price - min_price;

                        results.push(ArbitrageResult {
                            route: format!("{}/{}", base, quote),
                            profit_before,
                            fee,
                            profit_after,
                            spread,
                        });
                    }
                }
            }
        }
    }

    results
}
