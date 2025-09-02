use axum::{
    routing::{get, post},
    Router,
};
use axum_server::Server;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{CorsLayer, Any};

mod routes;
mod logic;
mod models;
mod exchanges;

use routes::{ui_handler, scan_handler};
use models::AppState;

#[tokio::main]
async fn main() {
    // Shared state for the scanner
    let shared_state = Arc::new(AppState::default());

    // Enable CORS for browser access
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build the Axum app with routes and layers
    let app = Router::new()
        .route("/", get(ui_handler))
        .route("/scan", post(scan_handler))
        .with_state(shared_state)
        .layer(cors.clone());

    // Start the server on port 8080
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("Server running at http://{}", addr);

    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
