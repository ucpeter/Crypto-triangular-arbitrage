// src/routes.rs
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    Json as AxumJson,
};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::models::{AppState, ArbResult, ScanRequest};
use crate::logic::scan_exchanges;

/// Main handler: runs the async scan_exchanges function and returns JSON
pub async fn scan_handler(
    State(state): State<Arc<Mutex<AppState>>>,
    AxumJson(payload): AxumJson<ScanRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    // run the scan (concurrently fetching prices + scanning)
    let scanned = match scan_exchanges(payload.exchanges.clone(), payload.min_profit).await {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Scan failed: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "status": "error", "message": e })),
            );
        }
    };

    // store results once scanning is done
    {
        let mut lock = state.lock().await;
        lock.last_results = Some(scanned.clone());
    }

    (
        StatusCode::OK,
        Json(json!({
            "status": "success",
            "count": scanned.len(),
            "results": scanned
        })),
    )
                   }
