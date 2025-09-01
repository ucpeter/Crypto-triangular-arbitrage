use crate::models::ArbResult;
use std::collections::HashMap;

pub fn find_triangular_arbitrage(prices: &HashMap<String, f64>, exchange: &str, min_profit: f64) -> Vec<ArbResult> {
    let mut results = Vec::new();
    let pairs: Vec<(String, f64)> = prices.iter().map(|(k, v)| (k.clone(), *v)).collect();

    for (base1, price1) in &pairs {
        for (base2, price2) in &pairs {
            if base1 == base2 {
                continue;
            }
            for (base3, price3) in &pairs {
                if base3 == base1 || base3 == base2 {
                    continue;
                }

                let implied = (price2 / price1) * (price3 / price2);
                let profit_before = (implied - 1.0) * 100.0;
                let fee = 0.1 * 3.0;
                let profit_after = profit_before - fee;

                if profit_after >= min_profit {
                    results.push(ArbResult {
                        exchange: exchange.to_string(),
                        route: format!("{} → {} → {}", base1, base2, base3),
                        profit_before,
                        fee,
                        profit_after,
                        spread: implied - 1.0,
                    });
                }
            }
        }
    }

    results
                        }
