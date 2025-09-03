// src/logic.rs
use crate::models::{ArbResult, PriceMap};
use crate::exchanges;
use std::collections::{HashMap, HashSet};
use futures::future::join_all;

/// Build graph for triangular paths from a PriceMap
fn build_graph(prices: &PriceMap) -> HashMap<String, HashMap<String, f64>> {
    let mut g: HashMap<String, HashMap<String, f64>> = HashMap::new();

    for (pair, price) in prices.iter() {
        if !price.is_finite() || *price <= 0.0 {
            continue;
        }
        let parts: Vec<&str> = pair.split('/').collect();
        if parts.len() != 2 {
            continue;
        }
        let base = parts[0].trim().to_uppercase();
        let quote = parts[1].trim().to_uppercase();

        g.entry(base.clone()).or_default().insert(quote.clone(), *price);
        // insert inverse so graph holds both directions
        g.entry(quote.clone()).or_default().insert(base.clone(), 1.0 / *price);
    }

    g
}

/// Triangular arbitrage finder for a single exchange.
/// We ensure triangles only use pairs present in the original PriceMap
pub fn tri_arb_single_exchange(
    exchange_name: &str,
    prices: &PriceMap,
    min_profit_after: f64,
    fee_per_trade_pct: f64,
) -> Vec<ArbResult> {
    let g = build_graph(prices);
    let mut assets: Vec<String> = g.keys().cloned().collect();
    if assets.len() > 250 {
        assets.truncate(250);
    }

    let mut results: Vec<ArbResult> = Vec::new();
    let fee_factor = (1.0 - fee_per_trade_pct / 100.0).powf(3.0);
    let mut seen_routes: HashSet<String> = HashSet::new();

    for a in &assets {
        if let Some(map_ab) = g.get(a) {
            for (b, r_ab) in map_ab {
                if a == b { continue; }
                if let Some(map_bc) = g.get(b) {
                    for (c, r_bc) in map_bc {
                        if c == a || c == b { continue; }

                        // Validate that the three spot legs exist in the original price map.
                        // A triangle is valid only if A/B, B/C and C/A exist in the PriceMap (direct or reverse).
                        let leg1 = format!("{}/{}", a, b);
                        let leg2 = format!("{}/{}", b, c);
                        let leg3 = format!("{}/{}", c, a);

                        // price keys in PriceMap are normalized; we check for either direct key or its inverse
                        let leg_exists = |pmap: &PriceMap, leg: &str| -> bool {
                            if pmap.contains_key(leg) { return true; }
                            // check inverse
                            if let Some((x,y)) = leg.split_once('/') {
                                return pmap.contains_key(&format!("{}/{}", y, x));
                            }
                            false
                        };

                        if !(leg_exists(prices, &leg1) && leg_exists(prices, &leg2) && leg_exists(prices, &leg3)) {
                            continue; // not a valid spot triangle
                        }

                        if let Some(r_ca) = g.get(c).and_then(|m| m.get(a)) {
                            let cycle = r_ab * r_bc * r_ca;
                            if !cycle.is_finite() { continue; }

                            let profit_before = (cycle - 1.0) * 100.0;
                            let profit_after = (cycle * fee_factor - 1.0) * 100.0;

                            // dedupe
                            let route_key = format!("{}-{}-{}", a, b, c);
                            if seen_routes.contains(&route_key) { continue; }
                            seen_routes.insert(route_key);

                            // sanity clamp (ignore absurd outliers)
                            if !profit_after.is_finite() || profit_after.is_nan() || profit_after < -99.0 || profit_after > 100.0 {
                                continue;
                            }

                            if profit_after >= min_profit_after {
                                results.push(ArbResult {
                                    exchange: exchange_name.to_string(),
                                    route: format!("[{}] {} → {} → {} → {}", exchange_name.to_uppercase(), a, b, c, a),
                                    profit_before,
                                    fee: 3.0 * fee_per_trade_pct,
                                    profit_after,
                                    spread: (cycle - 1.0) * 100.0,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    // sort descending by profit_after
    results.sort_by(|a, b| b.profit_after.partial_cmp(&a.profit_after).unwrap_or(std::cmp::Ordering::Equal));
    results
}

/// New, single async entrypoint used by the API.
///
/// - `exchanges`: names like "binance", "kucoin", "bybit", "gateio", "kraken"
/// - fetches prices concurrently, runs tri_arb_single_exchange for each exchange,
/// - returns Result<Vec<ArbResult>, String> so the routes layer can be simple.
pub async fn scan_exchanges(exchanges: Vec<String>, min_profit_after: f64) -> Result<Vec<ArbResult>, String> {
    // build an async list of fetch futures
    let mut futures_vec = Vec::with_capacity(exchanges.len());
    for ex in exchanges.iter() {
        let ex_name = ex.clone();
        let fut = async move {
            let res = match ex_name.to_lowercase().as_str() {
                "binance" => exchanges::fetch_binance().await,
                "kucoin"  => exchanges::fetch_kucoin().await,
                "bybit"   => exchanges::fetch_bybit().await,
                "gateio"  => exchanges::fetch_gateio().await,
                "kraken"  => exchanges::fetch_kraken().await,
                other => Err(format!("unsupported exchange '{}'", other)),
            };
            (ex_name, res)
        };
        futures_vec.push(fut);
    }

    // concurrently run all fetches
    let fetched = join_all(futures_vec).await;

    // collect bundles and log fetch failures
    let mut out_results: Vec<ArbResult> = Vec::new();
    for (ex_name, fetch_res) in fetched.into_iter() {
        match fetch_res {
            Ok(price_map) => {
                // run triangular scan for this exchange
                let mut r = tri_arb_single_exchange(&ex_name, &price_map, min_profit_after, 0.10);
                out_results.append(&mut r);
            }
            Err(e) => {
                // don't fail entire scan because one exchange failed; log and continue
                eprintln!("fetch error for {}: {}", ex_name, e);
            }
        }
    }

    Ok(out_results)
            }
