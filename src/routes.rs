use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::models::{AppState, ArbResult};
use crate::logic::scan_exchanges;
use crate::exchanges::{
    fetch_prices_binance, fetch_prices_kucoin, fetch_prices_bybit,
    fetch_prices_gateio, fetch_prices_kraken, PriceMap,
};

pub async fn ui_handler() -> &'static str {
    include_str!("../static/index.html")
}

#[derive(Deserialize)]
pub struct ScanParams {
    pub min_profit: f64,
}

#[derive(Serialize)]
pub struct ScanResponse {
    pub opportunities: Vec<ArbResult>,
}

pub async fn scan_handler(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(params): Json<ScanParams>,
) -> Json<ScanResponse> {
    // Fetch concurrently
    let (b, k, bb, g, kr) = tokio::join!(
        fetch_prices_binance(),
        fetch_prices_kucoin(),
        fetch_prices_bybit(),
        fetch_prices_gateio(),
        fetch_prices_kraken(),
    );

    // Each exchange stays isolated to avoid mixing pairs across venues
    let mut bundle: Vec<(String, PriceMap)> = Vec::new();
    if !b.is_empty()  { bundle.push(("binance".to_string(), b)); }
    if !k.is_empty()  { bundle.push(("kucoin".to_string(), k)); }
    if !bb.is_empty() { bundle.push(("bybit".to_string(), bb)); }
    if !g.is_empty()  { bundle.push(("gateio".to_string(), g)); }
    if !kr.is_empty() { bundle.push(("kraken".to_string(), kr)); }

    let mut results = scan_exchanges(bundle, params.min_profit);

    // keep top 200 for UI
    if results.len() > 200 { results.truncate(200); }

    {
        let mut lock = state.lock().await;
        lock.last_results = results.clone();
    }

    Json(ScanResponse { opportunities: results })
}
