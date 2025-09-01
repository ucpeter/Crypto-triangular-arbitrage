mod models;
mod routes;
mod logic;
mod exchanges;

use axum::{
    routing::{get, post},
    Router,
    serve,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::net::TcpListener;

use crate::models::AppState;
use crate::routes::{ui_handler, scan_handler};

#[tokio::main]
async fn main() {
    // Shared state
    let state = Arc::new(Mutex::new(AppState::default()));

    // Router
    let app = Router::new()
        .route("/", get(ui_handler))
        .route("/scan", post(scan_handler))
        .with_state(state);

    // Read port (default 8080)
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());

    // Create a persistent String first to avoid dropped temporary
    let addr_str = format!("0.0.0.0:{}", port);
    let addr: SocketAddr = addr_str
        .parse()
        .expect("Invalid socket address");

    // Bind listener
    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind address");

    println!("▶️ Triangular arbitrage server running on http://0.0.0.0:{}", port);

    serve(listener, app)
        .await
        .expect("Server failed");
}
