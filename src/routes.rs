use axum::{
    extract::State,
    response::{Html, Json, IntoResponse},
    http::StatusCode,
};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::fs;

use crate::models::{AppState, ScanRequest};
use crate::logic::scan_all_exchanges;

pub async fn ui_handler() -> impl IntoResponse {
    match fs::read_to_string("static/index.html") {
        Ok(content) => Html(content).into_response(),
        Err(_) => Html("<h1>UI not found. Please redeploy with static/index.html.</h1>").into_response(),
    }
}

pub async fn scan_handler(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(payload): Json<ScanRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let mut shared_state = state.lock().await;

    match scan_all_exchanges(&payload.exchanges, payload.min_profit).await {
        Ok(results) => {
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
    }
    }
