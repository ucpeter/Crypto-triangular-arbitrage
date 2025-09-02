use axum::{extract::State, response::Html, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use reqwest::Client;
use std::collections::HashSet;

use crate::{
    models::{AppState, ScanRequest, ScanResult, PriceMap},
    exchanges::{fetch_binance, fetch_kucoin, fetch_bybit, fetch_gateio, fetch_kraken},
    logic::scan_all_exchanges,
};

pub async fn ui_handler() -> Html<&'static str> {
    Html(include_str!("../static/index.html"))
}

pub async fn scan_handler(
    State(_state): State<Arc<Mutex<AppState>>>,
    Json(payload): Json<ScanRequest>,
) -> Json<Vec<ScanResult>> {
    let client = Client::new();
    let exchanges = payload.exchanges.clone();
    let min_profit = payload.min_profit;

    let mut bundle: Vec<(String, PriceMap, HashSet<String>)> = Vec::new();

    for ex in exchanges.into_iter() {
        let (prices, spot_pairs) = match ex.as_str() {
            "binance" => fetch_binance(&client).await,
            "kucoin" => fetch_kucoin(&client).await,
            "bybit" => fetch_bybit(&client).await,
            "gateio" => fetch_gateio(&client).await,
            "kraken" => fetch_kraken(&client).await,
            other => {
                // unknown exchange - skip
                (Default::default(), Default::default())
            }
        };

        if !prices.is_empty() {
            bundle.push((ex.clone(), prices, spot_pairs));
        }
    }

    let arb_results = scan_all_exchanges(bundle, min_profit);

    let results: Vec<ScanResult> = arb_results
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
