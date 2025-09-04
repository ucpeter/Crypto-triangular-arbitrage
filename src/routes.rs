use axum::{
    extract::State,
    response::Json,
    http::StatusCode,
};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::models::{ScanRequest, ScanResponse, PairPrice};
use crate::logic::scan_triangles;
use crate::exchanges::{
    fetch_binance, fetch_kucoin, fetch_gateio, fetch_kraken, fetch_bybit,
};

type SharedState = Arc<Mutex<()>>; // no AppState anymore

pub async fn ui_handler() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "message": "Triangular Arbitrage Scanner API is running",
            "usage": "POST /scan with { exchanges: [], min_profit: number }"
        })),
    )
}

pub async fn scan_handler(
    State(_): State<SharedState>,
    Json(payload): Json<ScanRequest>,
) -> (StatusCode, Json<ScanResponse>) {
    let mut all_pairs: Vec<PairPrice> = Vec::new();

    for ex in &payload.exchanges {
        let data: Vec<PairPrice> = match ex.as_str() {
            "binance" => fetch_binance().await.unwrap_or_default(),
            "kucoin" => fetch_kucoin().await.unwrap_or_default(),
            "gateio" => fetch_gateio().await.unwrap_or_default(),
            "kraken" => fetch_kraken().await.unwrap_or_default(),
            "bybit" => fetch_bybit().await.unwrap_or_default(),
            _ => Vec::new(),
        };
        all_pairs.extend(data);
    }

    let results = scan_triangles(&all_pairs, payload.min_profit, 0.1);

    (
        StatusCode::OK,
        Json(ScanResponse {
            count: results.len(),
            results,
        }),
    )
        }
