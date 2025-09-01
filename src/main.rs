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
    // Shared app state
    let state = Arc::new(Mutex::new(AppState::default()));

    // Build router
    let app = Router::new()
        .route("/", get(ui_handler))
        .route("/scan", post(scan_handler))
        .with_state(state);

    // Get port from environment (Render sets $PORT), default to 8080 locally
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr: SocketAddr = format!("0.0.0.0:{}", port)
        .parse()
        .expect("Invalid socket address");

    // Bind and serve
    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind address");

    println!("▶️  Triangular arbitrage server running on http://0.0.0.0:{}", port);

    serve(listener, app).await.expect("Server failed");
        }
