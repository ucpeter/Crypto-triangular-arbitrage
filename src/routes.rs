use axum::{
    extract::{Query, State},
    response::Html,
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::logic::find_arbitrage_opportunities;
use crate::models::{AppState, ScanResult};

#[derive(Deserialize)]
pub struct ScanQuery {
    exchanges: Option<String>,
    min_profit: Option<f64>,
}

pub async fn ui_handler() -> Html<String> {
    // Serve the static index.html file for the UI
    let html = std::fs::read_to_string("static/index.html")
        .unwrap_or_else(|_| "<h1>UI not found</h1>".to_string());
    Html(html)
}

pub async fn scan_handler(
    Query(params): Query<ScanQuery>,
    State(state): State<Arc<Mutex<AppState>>>,
) -> Json<Vec<ScanResult>> {
    // Split the exchanges provided in the query, fallback to empty vector if none provided
    let exchanges_vec: Vec<&str> = params
        .exchanges
        .as_deref()
        .map(|v| v.split(',').collect::<Vec<&str>>())
        .unwrap_or_else(Vec::new);

    // Get the minimum profit filter (default = 0.0)
    let min_profit = params.min_profit.unwrap_or(0.0);

    // Lock the state for scanning
    let mut app_state = state.lock().await;

    // Perform the arbitrage scan
    let results = find_arbitrage_opportunities(&exchanges_vec, min_profit, &mut app_state).await;

    Json(results)
}
