use crate::models::Market;
use crate::utils::build_graph;
use crate::models::ArbitrageResult;
use std::collections::{HashMap, HashSet};

pub fn build_triangles(markets: &Vec<Market>) -> Vec<(String, String, String)> {
    let pairs: Vec<(String, String)> = markets
        .iter()
        .filter(|m| m.spot && m.active)
        .map(|m| (m.base.clone(), m.quote.clone()))
        .collect();

    let graph = build_graph(&pairs);
    let mut triangles = Vec::new();
    let mut seen = HashSet::new();

    for a in graph.keys() {
        for b in &graph[a] {
            for c in &graph[b] {
                if c == a {
                    continue;
                }
                if graph[c].contains(a) {
                    let mut key = vec![a.clone(), b.clone(), c.clone()];
                    key.sort();
                    if seen.insert(key.clone()) {
                        triangles.push((a.clone(), b.clone(), c.clone()));
                    }
                }
            }
        }
    }

    triangles
}

pub fn evaluate_triangles(
    exchange: &str,
    triangles: &Vec<(String, String, String)>,
    tickers: &HashMap<String, f64>,
) -> Vec<ArbitrageResult> {
    let mut results = Vec::new();

    for (a, b, c) in triangles {
        let pair1 = format!("{}{}", a, b);
        let pair2 = format!("{}{}", b, c);
        let pair3 = format!("{}{}", c, a);

        if let (Some(p1), Some(p2), Some(p3)) = (tickers.get(&pair1), tickers.get(&pair2), tickers.get(&pair3)) {
            if *p1 > 0.0 && *p2 > 0.0 && *p3 > 0.0 {
                let before = p1 * p2 * p3;
                let after = before * (1.0 - 0.001).powi(3);
                if after > 1.0 {
                    results.push(ArbitrageResult {
                        exchange: exchange.to_string(),
                        triangle: format!("{} → {} → {} → {}", a, b, c, a),
                        profit_percent: (after - 1.0) * 100.0,
                    });
                }
            }
        }
    }

    results
          }
