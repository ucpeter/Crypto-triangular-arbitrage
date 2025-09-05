use axum::{
    extract::{State, Json},
    http::StatusCode,
    response::IntoResponse,
};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::models::{AppState, ScanRequest, TriangularResult, PairPrice};
use crate::exchanges;
use crate::logic;

pub async fn ui_handler() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({
            "message": "Triangular Arbitrage Scanner API is running",
            "usage": "POST /scan with { exchanges: [\"binance\",\"kucoin\"], min_profit: 0.3 }"
        })),
    )
}

/// Scan handler returns Result so we can use `?` nicely
pub async fn scan_handler(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(payload): Json<ScanRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    tracing::info!(?payload, "received /scan request");

    if payload.exchanges.is_empty() {
        return Err((StatusCode::BAD_REQUEST, Json(json!({"error": "no exchanges selected"}))));
    }

    // 1) fetch pairs for requested exchanges concurrently
    let fetches = exchanges::fetch_many(&payload.exchanges).await;

    // merge pairs
    let mut all_pairs: Vec<PairPrice> = Vec::new();
    for (name, pairs) in fetches.into_iter() {
        tracing::info!(exchange = %name, count = pairs.len(), "fetched pairs");
        all_pairs.extend(pairs);
    }

    // 2) compute triangles on blocking thread
    let min_profit = payload.min_profit;
    let scan_result: Vec<TriangularResult> = tokio::task::spawn_blocking(move || {
        logic::scan_triangles(&all_pairs, min_profit, 0.1)
    })
    .await
    .map_err(|e| {
        tracing::error!("scan task failed to join: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "scan task failed"})))
    })?;

    tracing::info!(found = scan_result.len(), "scan completed");

    // 3) store a copy in shared state
    {
        let mut s = state.lock().await;
        s.last_results = Some(scan_result.clone());
    }

    // 4) prepare JSON result
    let results_json: Vec<_> = scan_result.into_iter().map(|r| {
        json!({
            "triangle": r.triangle,
            "pairs": r.pairs,
            "profit_before": r.profit_before_fees,
            "fees": r.trade_fees,
            "profit_after": r.profit_after_fees,
        })
    }).collect();

    Ok((StatusCode::OK, Json(json!({"status":"success","count": results_json.len(), "results": results_json}))))
}
