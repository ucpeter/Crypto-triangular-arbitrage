use axum::{
    extract::{State, Json},
    response::IntoResponse,
    http::StatusCode,
};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::models::{AppState, ScanRequest, TriangularResult, PairPrice};
use crate::exchanges;
use crate::logic;
use futures::future::join_all;

pub async fn ui_handler() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({
            "message": "Triangular Arbitrage Scanner API is running",
            "usage": "POST /scan with { exchanges: [\"binance\",\"kucoin\"], min_profit: 0.3 }"
        })),
    )
}

pub async fn scan_handler(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(payload): Json<ScanRequest>,
) -> impl IntoResponse {
    tracing::info!(?payload, "received /scan request");

    // fetch all exchanges concurrently via helper
    let requested = payload.exchanges.clone();
    if requested.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "no exchanges selected"})));
    }

    // spawn fetch tasks and wait
    let fetches = exchanges::fetch_many(&requested).await; // returns Vec<(name, Vec<PairPrice>)>

    // merge pairs
    let mut all_pairs: Vec<PairPrice> = Vec::new();
    for (name, pairs) in fetches.into_iter() {
        tracing::info!("{} pairs from {}", pairs.len(), name);
        all_pairs.extend(pairs);
    }

    // Run heavy scan on blocking thread so async reactor not blocked
    let min_profit = payload.min_profit;
    let scan_result: Vec<TriangularResult> = tokio::task::spawn_blocking(move || {
        logic::scan_triangles(&all_pairs, min_profit, 0.1)
    }).await.map_err(|e| {
        tracing::error!("scan task join error: {:?}", e);
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "scan task failed"})))
    })?;

    tracing::info!("scan finished, found {}", scan_result.len());

    // store results into state briefly
    {
        let mut s = state.lock().await;
        s.last_results = Some(scan_result.clone());
    }

    // prepare JSON response
    let results_json: Vec<_> = scan_result.into_iter().map(|r| {
        json!({
            "triangle": r.triangle,
            "pairs": r.pairs,
            "profit_before": r.profit_before_fees,
            "fees": r.trade_fees,
            "profit_after": r.profit_after_fees,
        })
    }).collect();

    (StatusCode::OK, Json(json!({"status":"success","count": results_json.len(), "results": results_json})))
}
