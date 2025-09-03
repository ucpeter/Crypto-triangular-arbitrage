use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::models::{AppState, ArbResult, ScanRequest};
use crate::logic::scan_all_exchanges;

/// Root endpoint: API status check
pub async fn ui_handler() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "message": "Triangular Arbitrage Scanner API is running",
            "usage": "POST /scan with { exchanges: [], min_profit: number }"
        })),
    )
}

/// Main handler to process arbitrage scan requests
pub async fn scan_handler(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(payload): Json<ScanRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let mut shared_state = state.lock().await;

    // Explicit type to avoid mismatched types error
    let results: Vec<ArbResult> = scan_all_exchanges(&payload.exchanges, payload.min_profit);

    // Store results in shared state
    shared_state.last_results = Some(results.clone());

    // Return response
    (
        StatusCode::OK,
        Json(json!({
            "status": "success",
            "count": results.len(),
            "results": results
        })),
    )
}
