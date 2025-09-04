use std::collections::{HashMap, HashSet};
use crate::models::ArbResult;
use crate::exchanges::{fetch_binance, fetch_kucoin, fetch_gateio, fetch_kraken, fetch_bybit, PriceMap};

/// Build graph directly from available pairs (spot only, no fabricated reverse)
fn build_graph(prices: &PriceMap) -> HashMap<String, HashMap<String, f64>> {
    let mut g: HashMap<String, HashMap<String, f64>> = HashMap::new();

    // Build a set of real pairs
    let valid_pairs: HashSet<String> = prices.keys().cloned().collect();

    for (pair, &price) in prices {
        if price <= 0.0 {
            continue;
        }
        let parts: Vec<&str> = pair.split('/').collect();
        if parts.len() != 2 {
            continue;
        }
        let base = parts[0].to_string();
        let quote = parts[1].to_string();

        // forward (always safe)
        g.entry(base.clone()).or_default().insert(quote.clone(), price);

        // reverse (only if exchange really lists "QUOTE/BASE")
        let reverse = format!("{}/{}", quote, base);
        if valid_pairs.contains(&reverse) {
            g.entry(quote.clone()).or_default().insert(base.clone(), 1.0 / price);
        }
    }
    g
}

/// Find triangular arbitrage opportunities on one exchange
pub fn tri_arb_single_exchange(
    exchange_name: &str,
    prices: &PriceMap,
    min_profit_after: f64,
    fee_per_trade_pct: f64,
) -> Vec<ArbResult> {
    let g = build_graph(prices);
    let assets: Vec<String> = g.keys().cloned().collect();

    let mut results: Vec<ArbResult> = Vec::new();
    let fee_factor = (1.0 - fee_per_trade_pct / 100.0).powf(3.0);

    let mut seen: HashSet<String> = HashSet::new();

    for a in &assets {
        if let Some(map_ab) = g.get(a) {
            for (b, r_ab) in map_ab {
                if a == b {
                    continue;
                }
                if let Some(map_bc) = g.get(b) {
                    for (c, r_bc) in map_bc {
                        if c == a || c == b {
                            continue;
                        }
                        if let Some(r_ca) = g.get(c).and_then(|m| m.get(a)) {
                            let cycle = r_ab * r_bc * r_ca;
                            let profit_before = (cycle - 1.0) * 100.0;
                            let profit_after = (cycle * fee_factor - 1.0) * 100.0;

                            if profit_after >= min_profit_after {
                                let route = format!("{} → {} → {} → {}", a, b, c, a);
                                if seen.insert(route.clone()) {
                                    results.push(ArbResult {
                                        exchange: exchange_name.to_string(),
                                        route: route.clone(),
                                        pairs: format!("{}/{} | {}/{} | {}/{}", a, b, b, c, c, a),
                                        profit_before,
                                        fee: 3.0 * fee_per_trade_pct,
                                        profit_after,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    results.sort_by(|a, b| b.profit_after.partial_cmp(&a.profit_after).unwrap_or(std::cmp::Ordering::Equal));
    results
}

/// Run scan across all exchanges
pub async fn scan_all_exchanges(selected: &[String], min_profit_after: f64) -> Vec<ArbResult> {
    let mut out = Vec::new();
    let default_fee = 0.10;

    for ex in selected {
        let prices: Option<PriceMap> = match ex.as_str() {
            "binance" => fetch_binance().await.ok(),
            "kucoin" => fetch_kucoin().await.ok(),
            "gateio" => fetch_gateio().await.ok(),
            "kraken" => fetch_kraken().await.ok(),
            "bybit" => fetch_bybit().await.ok(),
            _ => None,
        };

        if let Some(pm) = prices {
            let mut v = tri_arb_single_exchange(ex, &pm, min_profit_after, default_fee);
            out.append(&mut v);
        } else {
            eprintln!("❌ Failed to fetch prices for {}", ex);
        }
    }

    out
    }
