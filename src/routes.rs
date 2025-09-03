use axum::{
    extract::State,
    response::Json,
    http::StatusCode,
};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::models::{AppState, ScanRequest, ScanResponse, ArbResult};
use crate::logic::scan_all_exchanges;
use crate::exchanges::fetch_exchange_data;

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
    State(state): State<Arc<Mutex<AppState>>>,
    Json(payload): Json<ScanRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let mut shared_state = state.lock().await;

    // Collect data from selected exchanges
    let mut bundle: Vec<(String, crate::models::PriceMap)> = Vec::new();
    for ex in &payload.exchanges {
        match fetch_exchange_data(ex).await {
            Ok(prices) => bundle.push((ex.clone(), prices)),
            Err(e) => {
                eprintln!("❌ Error fetching data from {}: {}", ex, e);
            }
        }
    }

    // Run the arbitrage scan
    let results: Vec<ArbResult> = scan_all_exchanges(bundle, payload.min_profit);

    // Save last results in state
    shared_state.last_results = Some(results.clone());

    // Return structured response
    let response = ScanResponse {
        status: "success".to_string(),
        count: results.len(),
        results,
    };

    (
        StatusCode::OK,
        Json(serde_json::to_value(response).unwrap())
    )
}
