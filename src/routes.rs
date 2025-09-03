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

/// Root endpoint to confirm API status
pub async fn ui_handler() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "message": "Triangular Arbitrage Scanner API is running",
            "usage": "POST /scan with { exchanges: [], min_profit: number }"
        })),
    )
}

/// Handles the scan request and returns results
pub async fn scan_handler(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(payload): Json<ScanRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let mut shared_state = state.lock().await;

    match scan_all_exchanges(&payload.exchanges, payload.min_profit) {
        Ok(results) => {
            // Store results for later reference
            shared_state.last_results = Some(results.clone());

            // Return a success response
            (
                StatusCode::OK,
                Json(json!({
                    "status": "success",
                    "count": results.len(),
                    "results": results
                })),
            )
        }
        Err(err) => {
            eprintln!("Scan error: {:?}", err);

            // Return a clean error response with the same type
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "status": "error",
                    "message": format!("Failed to scan: {}", err)
                })),
            )
        }
    }
            }
