use crate::models::{ArbResult, PriceMap};

fn build_graph(prices: &PriceMap) -> std::collections::HashMap<String, std::collections::HashMap<String, f64>> {
    let mut g = std::collections::HashMap::new();
    for (pair, price) in prices {
        if *price <= 0.0 { continue; }
        let parts: Vec<&str> = pair.split('/').collect();
        if parts.len() != 2 { continue; }
        let base = parts[0].to_string();
        let quote = parts[1].to_string();
        g.entry(base.clone()).or_default().insert(quote.clone(), *price);
        g.entry(quote.clone()).or_default().insert(base.clone(), 1.0 / *price);
    }
    g
}

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
                                let route = format!("[{}] {} → {} → {} → {}", exchange_name.to_uppercase(), a, b, c, a);
                                results.push(ArbResult {
                                    exchange: exchange_name.to_string(),
                                    route,
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

    results.sort_by(|a,b| b.profit_after.partial_cmp(&a.profit_after).unwrap_or(std::cmp::Ordering::Equal));
    results
}

pub fn scan_all_exchanges(bundle: Vec<(String, PriceMap)>, min_profit_after: f64) -> Vec<ArbResult> {
    let default_fee = 0.10;
    let mut out = Vec::new();
    for (ex, pm) in bundle {
        let mut v = tri_arb_single_exchange(&ex, &pm, min_profit_after, default_fee);
        out.append(&mut v);
    }
    out
}
