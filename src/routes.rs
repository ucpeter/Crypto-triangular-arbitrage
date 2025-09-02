use axum::{
    extract::State,
    response::Json,
    http::StatusCode,
};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::models::{AppState, ScanRequest};
use crate::logic::{scan_all_exchanges, fetch_prices_for_exchange};

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
    State(_state): State<Arc<Mutex<AppState>>>,
    Json(payload): Json<ScanRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let mut bundles = Vec::new();

    // Fetch market data for each selected exchange
    for ex in &payload.exchanges {
        match fetch_prices_for_exchange(ex).await {
            Ok(price_map) => {
                bundles.push((ex.clone(), price_map));
            }
            Err(err) => {
                eprintln!("⚠️ Failed to fetch data for {}: {}", ex, err);
            }
        }
    }

    // Now scan for opportunities
    let results = scan_all_exchanges(bundles, payload.min_profit);

    if results.is_empty() {
        return (
            StatusCode::OK,
            Json(json!({
                "status": "no_opportunities",
                "message": "No arbitrage opportunities found for the selected exchanges."
            })),
        );
    }

    (
        StatusCode::OK,
        Json(json!({
            "status": "success",
            "count": results.len(),
            "results": results
        })),
    )
                       }
