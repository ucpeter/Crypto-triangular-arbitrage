use axum::{
    extract::State,
    response::Json,
    http::StatusCode,
};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, error};

use crate::models::{AppState, ScanRequest, TriangularResult};
use crate::exchanges::fetch_many;
use crate::logic::scan_triangles;

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
    info!("received /scan request payload={:?}", payload);

    let mut shared_state = state.lock().await;

    // fetch selected exchanges
    let data = fetch_many(payload.exchanges.clone()).await;
    let mut results: Vec<TriangularResult> = Vec::new();

    for (ex, pairs) in data {
        info!("fetched pairs exchange={} count={}", ex, pairs.len());

        let mut r = scan_triangles(&pairs, payload.min_profit, 0.1); // 0.1% per trade
        results.append(&mut r);
    }

    info!("scan completed found={}", results.len());

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
