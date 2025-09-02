use axum::{extract::State, response::Html, Json};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;
use reqwest::Client;

use crate::{
    models::{AppState, ScanRequest, ScanResult},
    exchanges::*,
    logic::scan_all_exchanges,
};

/// Serve the UI
pub async fn ui_handler() -> Html<&'static str> {
    Html(include_str!("../static/index.html"))
}

/// Handle scan requests
pub async fn scan_handler(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(payload): Json<ScanRequest>,
) -> Json<Vec<ScanResult>> {
    let client = Client::new();
    let mut results = Vec::new();

    let exchanges = payload.exchanges.clone();
    let min_profit = payload.min_profit;

    // Build bundles: (exchange name, prices, spot pairs)
    let mut bundle = Vec::new();

    for ex in exchanges {
        let (prices, spot_pairs) = match ex.as_str() {
            "binance" => fetch_binance(&client).await,
            "kucoin" => fetch_kucoin(&client).await,
            "bybit" => fetch_bybit(&client).await,
            "gateio" => fetch_gateio(&client).await,
            "kraken" => fetch_kraken(&client).await,
            _ => (Default::default(), Default::default()),
        };

        if !prices.is_empty() {
            bundle.push((ex.clone(), prices, spot_pairs));
        }
    }

    let arb_results = scan_all_exchanges(bundle, min_profit);

    // Map to ScanResult for JSON response
    results = arb_results
        .into_iter()
        .map(|r| ScanResult {
            exchange: r.exchange,
            route: r.route,
            profit_before: r.profit_before,
            fee: r.fee,
            profit_after: r.profit_after,
        })
        .collect();

    Json(results)
           }
