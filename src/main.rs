// src/main.rs
mod models;
mod routes;
mod logic;
mod exchanges;

use axum::{routing::{get, post}, Router};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::models::AppState;
use crate::routes::{ui_handler, scan_handler};

#[tokio::main]
async fn main() {
    // Shared application state
    let state = Arc::new(Mutex::new(AppState::default()));

    // Router: serve UI at GET / and scanning at POST /scan
    let app = Router::new()
        .route("/", get(ui_handler))
        .route("/scan", post(scan_handler))
        .with_state(state);

    // Bind to 0.0.0.0:8080 (Render-friendly)
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("▶️  Triangular arbitrage server running on http://{}", addr);

    // Start server
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("server failed");
}
