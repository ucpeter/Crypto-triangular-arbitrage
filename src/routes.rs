use axum::{
    extract::State,
    response::Json,
    http::StatusCode,
};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::models::{AppState, ScanRequest};
use crate::logic::scan_all_exchanges;

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
    let _shared_state = state.lock().await;

    // Directly run the scan — no `.await` needed
    let results = scan_all_exchanges(&payload.exchanges, payload.min_profit);

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
