use crate::models::{PairPrice, TriangularResult};
use crate::utils::round2;
use std::collections::{HashMap, HashSet};

/// Scan triangles using given pair prices (spot only).
/// ✅ Now uses 24h high liquidity values provided in PairPrice
pub fn scan_triangles(
    prices: &[PairPrice],
    min_profit: f64,
    fee_per_leg: f64,
) -> Vec<TriangularResult> {
    let mut rate: HashMap<(String, String), (f64, f64)> = HashMap::new(); // (price, liquidity)
    let mut neighbors: HashMap<String, HashSet<String>> = HashMap::new();

    for p in prices {
        if !p.is_spot || !p.price.is_finite() || p.price <= 0.0 {
            continue;
        }

        let a = p.base.to_uppercase();
        let b = p.quote.to_uppercase();

        // direct
        rate.insert((a.clone(), b.clone()), (p.price, p.liquidity));
        neighbors.entry(a.clone()).or_default().insert(b.clone());

        // inverse (approx liquidity same as reported high liquidity)
        rate.insert((b.clone(), a.clone()), (1.0 / p.price, p.liquidity));
        neighbors.entry(b.clone()).or_default().insert(a.clone());
    }

    let mut seen: HashSet<(String, String, String)> = HashSet::new();
    let mut out: Vec<TriangularResult> = Vec::new();
    let fee_mult_one = 1.0 - (fee_per_leg / 100.0);
    let total_fee_percent = 3.0 * fee_per_leg;

    for (a, bs) in &neighbors {
        for b in bs {
            if a == b {
                continue;
            }
            if let Some(cs) = neighbors.get(b) {
                for c in cs {
                    if c == a || c == b {
                        continue;
                    }
                    if !neighbors.get(c).map_or(false, |s| s.contains(a)) {
                        continue;
                    }

                    // lookup rates + high liquidity
                    let (r1, l1) = match rate.get(&(a.clone(), b.clone())) {
                        Some(v) => *v,
                        None => continue,
                    };
                    let (r2, l2) = match rate.get(&(b.clone(), c.clone())) {
                        Some(v) => *v,
                        None => continue,
                    };
                    let (r3, l3) = match rate.get(&(c.clone(), a.clone())) {
                        Some(v) => *v,
                        None => continue,
                    };

                    let gross = r1 * r2 * r3;
                    let profit_before = (gross - 1.0) * 100.0;
                    if !profit_before.is_finite() || profit_before < min_profit {
                        continue;
                    }

                    let net = (r1 * fee_mult_one) * (r2 * fee_mult_one) * (r3 * fee_mult_one);
                    let profit_after = (net - 1.0) * 100.0;

                    let reps = vec![
                        (a.clone(), b.clone(), c.clone()),
                        (b.clone(), c.clone(), a.clone()),
                        (c.clone(), a.clone(), b.clone()),
                    ];
                    let key = reps.iter().min().unwrap().clone();
                    if !seen.insert(key) {
                        continue;
                    }

                    // ✅ store leg liquidities and take minimum of high values
                    let leg_liqs = [l1, l2, l3];
                    let min_liq = leg_liqs.iter().cloned().fold(f64::INFINITY, f64::min);

                    out.push(TriangularResult {
                        triangle: format!("{} → {} → {} → {}", a, b, c, a),
                        pairs: format!("{}/{} | {}/{} | {}/{}", a, b, b, c, c, a),
                        profit_before_fees: round2(profit_before),
                        trade_fees: round2(total_fee_percent),
                        profit_after_fees: round2(profit_after),
                        leg_liquidities: leg_liqs,
                        min_liquidity: min_liq, // ✅ min of 24h high liquidity
                    });
                }
            }
        }
    }

    out.sort_by(|x, y| {
        y.profit_after_fees
            .partial_cmp(&x.profit_after_fees)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    out
            }  
            
