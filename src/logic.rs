use crate::models::ArbResult;
use crate::exchanges::PriceMap;
use std::collections::{HashMap, HashSet};

/// Build directional rates from a price map.
/// For pair BASE/QUOTE at price p (1 BASE = p QUOTE), we add:
///   BASE -> QUOTE with rate p
///   QUOTE -> BASE with rate 1/p
fn build_graph(prices: &PriceMap) -> HashMap<String, HashMap<String, f64>> {
    let mut g: HashMap<String, HashMap<String, f64>> = HashMap::new();
    for (pair, price) in prices {
        if *price <= 0.0 { continue; }
        let parts: Vec<&str> = pair.split('/').collect();
        if parts.len() != 2 { continue; }
        let base = parts[0].to_string();
        let quote = parts[1].to_string();
        let p = *price;

        g.entry(base.clone()).or_default().insert(quote.clone(), p);
        g.entry(quote.clone()).or_default().insert(base.clone(), 1.0 / p);
    }
    g
}

/// Triangular arbitrage within a single exchange, returning all profitable cycles
/// above `min_profit_after` threshold.
/// `fee_per_trade_pct` is applied on each hop (three hops).
pub fn tri_arb_single_exchange(
    exchange: &str,
    prices: &PriceMap,
    min_profit_after: f64,
    fee_per_trade_pct: f64,
) -> Vec<ArbResult> {
    let g = build_graph(prices);
    let assets: Vec<String> = g.keys().cloned().collect();

    // small safety: if too many assets, limit to reduce O(N^3)
    let assets = if assets.len() > 250 { assets.into_iter().take(250).collect() } else { assets };

    let mut results = Vec::new();
    let fee_factor = (1.0 - fee_per_trade_pct / 100.0).powf(3.0);

    // A -> B -> C -> A
    for a in &assets {
        if let Some(map_ab) = g.get(a) {
            for (b, r_ab) in map_ab {
                if a == b { continue; }
                if let Some(map_bc) = g.get(b) {
                    for (c, r_bc) in map_bc {
                        if c == a || c == b { continue; }
                        if let Some(r_ca) = g.get(c).and_then(|m| m.get(a)) {
                            let cycle = r_ab * r_bc * r_ca;
                            let profit_before = (cycle - 1.0) * 100.0;
                            let profit_after = (cycle * fee_factor - 1.0) * 100.0;

                            if profit_after >= min_profit_after {
                                results.push(ArbResult {
                                    exchange: exchange.to_string(),
                                    route: format!("[{}] {} → {} → {} → {}", exchange.to_uppercase(), a, b, c, a),
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

    // Deduplicate by route text (optional defense)
    let mut seen = HashSet::new();
    results.retain(|r| seen.insert(r.route.clone()));
    results.sort_by(|x, y| y.profit_after.partial_cmp(&x.profit_after).unwrap_or(std::cmp::Ordering::Equal));
    results
}

/// Scan multiple exchanges independently and collect all profitable cycles.
pub fn scan_exchanges(
    all: Vec<(String, PriceMap)>,
    min_profit_after: f64,
) -> Vec<ArbResult> {
    // Typical taker fee default assumption (0.10% per hop) unless we customize per-exchange.
    let default_fee = 0.10;
    let mut all_results = Vec::new();
    for (ex, pm) in all {
        let mut v = tri_arb_single_exchange(&ex, &pm, min_profit_after, default_fee);
        all_results.append(&mut v);
    }
    all_results
        }
