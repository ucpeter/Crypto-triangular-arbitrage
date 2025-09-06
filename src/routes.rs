use axum::{
    extract::State,
    response::Json,
    http::StatusCode,
};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::models::{AppState, ScanRequest, ScanResponse, TriangularResult};
use crate::exchanges::fetch_exchange_data;
use crate::logic::scan_triangles;

/// Root endpoint (simple status check)
pub async fn ui_handler() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "message": "Triangular Arbitrage Scanner API is running",
            "usage": "POST /scan with { exchanges: [], min_profit: number }"
        })),
    )
}

/// Main scan endpoint
pub async fn scan_handler(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(payload): Json<ScanRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let mut all_pairs = Vec::new();

    // Fetch spot pairs from all selected exchanges
    for ex in &payload.exchanges {
        match fetch_exchange_data(ex).await {
            Ok(mut pairs) => {
                tracing::info!("✅ {} returned {} spot pairs", ex, pairs.len());
                all_pairs.append(&mut pairs);
            }
            Err(e) => {
                tracing::error!("❌ Error fetching {}: {:?}", ex, e);
            }
        }
    }

    // Run arbitrage scan
    let results: Vec<TriangularResult> =
        scan_triangles(&all_pairs, payload.min_profit, 0.10);

    // Save results to state
    let mut shared_state = state.lock().await;
    shared_state.last_results = Some(results.clone());

    (
        StatusCode::OK,
        Json(json!(ScanResponse {
            status: "success".to_string(),
            count: results.len(),
            results,
        })),
    )
}
