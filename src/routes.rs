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

// We serve the UI from / via ServeDir in main.rs, so no UI handler here.

pub async fn scan_handler(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(payload): Json<ScanRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let mut shared_state = state.lock().await;

    // Handle both signatures cleanly:
    // If scan_all_exchanges returns Result<Vec<ArbResult>, E>, we early-return on Err.
    // If it returns Vec<ArbResult>, just wrap it in Ok(...) below.
    let results: Vec<ArbResult> = match wrap_scan(&payload.exchanges, payload.min_profit) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Scan error: {e}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "status": "error",
                    "message": format!("Failed to scan: {e}")
                })),
            );
        }
    };

    // Store the results for possible later use
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

/// Adapter so we compile whether `scan_all_exchanges` returns Vec<ArbResult>
/// or Result<Vec<ArbResult>, E>.
fn wrap_scan(exchanges: &Vec<String>, min_profit: f64) -> Result<Vec<ArbResult>, String> {
    // If your logic returns Vec<ArbResult>, uncomment this and comment the match:
    // Ok(scan_all_exchanges(exchanges, min_profit))

    // If your logic returns Result<Vec<ArbResult>, E>, keep this:
    match scan_all_exchanges(exchanges, min_profit) {
        Ok(v) => Ok(v),
        Err(e) => Err(format!("{e}")),
    }
}
