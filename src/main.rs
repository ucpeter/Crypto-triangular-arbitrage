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
    // Shared state across handlers
    let shared_state = Arc::new(Mutex::new(AppState::default()));

    // CORS settings
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Router
    let app = Router::new()
        .route("/", get(ui_handler))
        .route("/scan", post(scan_handler))
        .layer(cors)
        .with_state(shared_state);

    // Get PORT from environment or fallback to 8080
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse().expect("invalid addr");

    println!("▶️  Starting server on http://0.0.0.0:{}", port);

    // Bind and serve
    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind address");

    axum::serve(listener, app)
        .await
        .expect("server error");
                                                    }
