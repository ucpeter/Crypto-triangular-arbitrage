use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::models::{AppState, ScanRequest};
use crate::logic::scan_all_exchanges;

/// UI handler to confirm service is running
pub async fn ui_handler() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "message": "Triangular Arbitrage Scanner API is running",
            "usage": "POST /scan with { exchanges: [], min_profit: number }"
        })),
    )
}

/// Scan handler to process arbitrage scans
pub async fn scan_handler(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(payload): Json<ScanRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let mut shared_state = state.lock().await;

    // Directly get the results (no .await)
    let results = scan_all_exchanges(&payload.exchanges, payload.min_profit);

    // Save results to state for later retrieval
    shared_state.last_results = Some(results.clone());

    (
        StatusCode::OK,
        Json(json!({
            "status": "success",
            "count": results.len(),
            "results": results
        })),
    )
}
