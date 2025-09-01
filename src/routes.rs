use axum::{extract::State, Json, response::Html};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::models::{AppState, ArbResult};
use crate::exchanges::{fetch_binance, fetch_kucoin, fetch_bybit, fetch_gateio, fetch_kraken};
use crate::logic::scan_all_exchanges;

#[derive(Deserialize)]
pub struct ScanParams {
    pub exchanges: Option<Vec<String>>,
    pub min_profit: Option<f64>,
}

#[derive(Serialize)]
pub struct ScanResponse {
    pub opportunities: Vec<ArbResult>,
}

pub async fn ui_handler() -> Html<&'static str> {
    Html(include_str!("../static/index.html"))
}

pub async fn scan_handler(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(params): Json<ScanParams>,
) -> Json<ScanResponse> {
    let exchanges = params.exchanges.clone().unwrap_or_else(|| vec![
        "binance".to_string(),
        "kucoin".to_string(),
        "bybit".to_string(),
        "gateio".to_string(),
        "kraken".to_string(),
    ]);

    let min_profit = params.min_profit.unwrap_or(0.0);

    let (b, k, bb, g, kr) = tokio::join!(
        fetch_binance(),
        fetch_kucoin(),
        fetch_bybit(),
        fetch_gateio(),
        fetch_kraken()
    );

    let mut bundle: Vec<(String, crate::models::PriceMap)> = Vec::new();
    if exchanges.iter().any(|e| e.eq_ignore_ascii_case("binance")) && !b.is_empty() { bundle.push(("binance".to_string(), b)); }
    if exchanges.iter().any(|e| e.eq_ignore_ascii_case("kucoin")) && !k.is_empty() { bundle.push(("kucoin".to_string(), k)); }
    if exchanges.iter().any(|e| e.eq_ignore_ascii_case("bybit")) && !bb.is_empty() { bundle.push(("bybit".to_string(), bb)); }
    if exchanges.iter().any(|e| e.eq_ignore_ascii_case("gateio") || exchanges.iter().any(|ex| ex.eq_ignore_ascii_case("gate"))) && !g.is_empty() { bundle.push(("gateio".to_string(), g)); }
    if exchanges.iter().any(|e| e.eq_ignore_ascii_case("kraken")) && !kr.is_empty() { bundle.push(("kraken".to_string(), kr)); }

    let results = scan_all_exchanges(bundle, min_profit);

    {
        let mut app = state.lock().await;
        app.last_results = results.clone();
    }

    Json(ScanResponse { opportunities: results })
        }
