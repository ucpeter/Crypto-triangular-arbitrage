// src/logic.rs
use crate::models::{PairPrice, ArbitrageOpportunity};

/// Compute arbitrage opportunities given a set of pair prices
pub fn find_triangular_arbitrage(pairs: &[PairPrice], fee_rate: f64) -> Vec<ArbitrageOpportunity> {
    let mut results = Vec::new();

    // build a lookup for quick price access
    let mut map = std::collections::HashMap::new();
    for p in pairs {
        map.insert((p.base.clone(), p.quote.clone()), p);
    }

    let mut seen = std::collections::HashSet::new();

    // brute force triangles
    for a in pairs {
        for b in pairs {
            if a.quote != b.base {
                continue;
            }
            for c in pairs {
                if b.quote != c.base || c.quote != a.base {
                    continue;
                }

                let triangle_key = format!("{}-{}-{}", a.base, a.quote, b.quote);
                if seen.contains(&triangle_key) {
                    continue;
                }
                seen.insert(triangle_key.clone());

                let rate1 = a.price;
                let rate2 = b.price;
                let rate3 = c.price;

                let mut amount = 1.0f64;
                amount /= rate1;
                amount /= rate2;
                amount /= rate3;

                let profit_before_fees = (amount - 1.0) * 100.0;
                let total_fees = 3.0 * fee_rate * 100.0;
                let profit_after_fees = profit_before_fees - total_fees;

                // --- NEW: compute liquidity ---
                // pick quote_volume as "effective liquidity" (since that's in stable quote terms)
                let leg_liqs = vec![
                    a.quote_volume.max(a.base_volume * a.price),
                    b.quote_volume.max(b.base_volume * b.price),
                    c.quote_volume.max(c.base_volume * c.price),
                ];
                let min_liq = leg_liqs.iter().cloned().fold(f64::INFINITY, f64::min);

                results.push(ArbitrageOpportunity {
                    triangle: triangle_key,
                    pairs: vec![
                        format!("{}/{}", a.base, a.quote),
                        format!("{}/{}", b.base, b.quote),
                        format!("{}/{}", c.base, c.quote),
                    ],
                    profit_before_fees,
                    trade_fees: total_fees,
                    profit_after_fees,
                    min_liquidity: min_liq,
                    leg_liquidities: leg_liqs,
                });
            }
        }
    }

    results
            }                              }
