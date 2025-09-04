use axum::{
    extract::State,
    response::Json,
    http::StatusCode,
};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::models::{AppState, ScanRequest, PairPrice};
use crate::logic::scan_triangles;
use crate::exchanges::fetch_exchange_data;

pub async fn ui_handler() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "message": "Triangular Arbitrage Scanner is running",
            "usage": "POST /scan with { exchanges: [], min_profit: number }"
        })),
    )
}

pub async fn scan_handler(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(payload): Json<ScanRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let mut shared_state = state.lock().await;

    // Collect prices across selected exchanges
    let mut all_prices: Vec<PairPrice> = Vec::new();
    for ex in &payload.exchanges {
        match fetch_exchange_data(ex).await {
            Ok(mut prices) => all_prices.append(&mut prices),
            Err(e) => {
                eprintln!("❌ Error fetching {}: {}", ex, e);
            }
        }
    }

    // Run the triangle scanner
    let results = scan_triangles(&all_prices, payload.min_profit, 0.1);

    // Save last results in state
    shared_state.last_results = Some(results.clone());

    (
        StatusCode::OK,
        Json(json!({
            "status": "success",
            "count": results.len(),
            "results": results.iter().map(|r| {
                json!({
                    "triangle": r.triangle,
                    "pairs": r.pairs,
                    "profit_before": r.profit_before_fees,
                    "fees": r.trade_fees,
                    "profit_after": r.profit_after_fees,
                })
            }).collect::<Vec<_>>()
        })),
    )
                   }
