mod models;
mod exchanges;
mod logic;
mod routes;
mod utils;

use axum::{routing::{get, post}, Router};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::net::TcpListener;
use tower_http::cors::{CorsLayer, Any};
use tower_http::services::ServeDir;

use crate::routes::{ui_handler, scan_handler};

#[tokio::main]
async fn main() {
    // shared state (empty since we don’t need AppState anymore)
    let shared_state = Arc::new(Mutex::new(()));

    // allow CORS for frontend UI
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // define routes
    let app = Router::new()
        .route("/api", get(ui_handler))
        .route("/scan", post(scan_handler))
        .with_state(shared_state)
        .layer(cors)
        // serve static files (UI) from "static" folder
        .nest_service("/", ServeDir::new("static"));

    // read port from env or fallback to 8080
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse().expect("invalid addr");

    println!("🚀 Starting scanner on http://0.0.0.0:{}", port);

    let listener = TcpListener::bind(addr)
        .await
        .expect("❌ Failed to bind address");

    axum::serve(listener, app)
        .await
        .expect("❌ Server error");
}
