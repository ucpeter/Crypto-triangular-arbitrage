mod models;
mod exchanges;
mod logic;
mod routes;

use axum::{routing::{get, post}, Router};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::net::TcpListener;
use tower_http::cors::{CorsLayer, Any};

use crate::models::AppState;
use crate::routes::{ui_handler, scan_handler};

#[tokio::main]
async fn main() {
    let shared_state = Arc::new(Mutex::new(AppState::default()));

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(ui_handler))
        .route("/scan", post(scan_handler))
        // apply CORS layer *before* attaching state
        .layer(cors)
        .with_state(shared_state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse().expect("invalid addr");

    println!("▶️  Starting server on http://0.0.0.0:{}", port);

    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind address");

    axum::serve(listener, app)
        .await
        .expect("server error");
}
