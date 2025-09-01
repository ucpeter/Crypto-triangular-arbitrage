mod models;
mod exchanges;
mod logic;
mod routes;

use axum::{routing::{get, post}, Router, Server};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};

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
        .with_state(shared_state)
        .layer(cors);

    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse().expect("invalid addr");

    println!("▶️  Starting server on http://0.0.0.0:{}", port);

    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("server error");
}
