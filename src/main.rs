mod models;
mod exchanges;
mod logic;
mod routes;
mod utils;

use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::net::TcpListener;
use tower_http::cors::{CorsLayer, Any};
use tower_http::services::ServeDir;

use crate::routes::{ui_handler, scan_handler};
use crate::models::AppState;

#[tokio::main]
async fn main() {
    // init tracing for logs
    tracing_subscriber::fmt::init();

    let shared_state = Arc::new(Mutex::new(AppState::default()));

    // CORS to allow the static frontend to call /scan
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api", get(ui_handler))
        .route("/scan", post(scan_handler))
        .nest_service("/", ServeDir::new("static"))
        .layer(cors)
        .with_state(shared_state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse().expect("invalid addr");
    tracing::info!(%port, "starting server");

    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind address");

    axum::serve(listener, app)
        .await
        .expect("server error");
}
