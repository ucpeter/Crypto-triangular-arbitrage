use axum::{
    extract::State,
    response::Json,
    http::StatusCode,
};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::logic::scan_all_exchanges;
use crate::models::ScanRequest;

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
    State(_): State<Arc<Mutex<()>>>,
    Json(payload): Json<ScanRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    match scan_all_exchanges(&payload.exchanges, payload.min_profit).await {
        Ok(results) => {
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
        Err(e) => {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "status": "error",
                    "message": format!("Failed to scan: {}", e)
                })),
            )
        }
    }
}
