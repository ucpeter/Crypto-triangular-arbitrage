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

/// Root endpoint: confirms API status
pub async fn ui_handler() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "message": "Triangular Arbitrage Scanner API is running",
            "usage": "POST /scan with { exchanges: [], min_profit: number }"
        })),
    )
}

/// Main handler for scanning arbitrage opportunities
pub async fn scan_handler(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(payload): Json<ScanRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let mut shared_state = state.lock().await;

    // Ensure both match arms return the same type
    let response: (StatusCode, Json<serde_json::Value>) = match scan_all_exchanges(&payload.exchanges, payload.min_profit) {
        Ok(results) => {
            // Save the results to shared state for later access
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
        Err(e) => {
            eprintln!("Scan error: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "status": "error",
                    "message": format!("Failed to scan: {}", e)
                })),
            )
        }
    };

    response
                }
