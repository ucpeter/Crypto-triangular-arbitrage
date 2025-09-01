use axum::{extract::State, http::StatusCode, response::Html, Json};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{exchanges::fetch_prices, logic::find_triangular_arbitrage, models::{AppState, ArbResult}};

pub async fn ui_handler() -> Html<&'static str> {
    Html(include_str!("../static/index.html"))
}

pub async fn scan_handler(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let exchanges = payload["exchanges"].as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|v| v.as_str())
        .collect::<Vec<&str>>();
    let min_profit = payload["min_profit"].as_f64().unwrap_or(0.0);

    let mut all_results: Vec<ArbResult> = Vec::new();

    for &exchange in &exchanges {
        let prices = fetch_prices(exchange).await;
        let mut results = find_triangular_arbitrage(&prices, exchange, min_profit);
        all_results.append(&mut results);
    }

    {
        let mut st = state.lock().await;
        st.last_results = all_results.clone();
    }

    Ok(Json(json!({ "results": all_results })))
    }
