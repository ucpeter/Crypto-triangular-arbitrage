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
    // Create the shared app state as an owned Arc<Mutex<...>>
    let state: Arc<Mutex<AppState>> = Arc::new(Mutex::new(AppState::default()));

    // Build the router with cloned owned state
    let app = {
        let shared_state = state.clone();
        Router::new()
            .route("/", get(ui_handler))
            .route("/scan", post(scan_handler))
            .with_state(shared_state)
    };

    // Create a persistent owned string for address
    let port_env = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr_string = format!("0.0.0.0:{}", port_env);
    let addr: SocketAddr = addr_string.parse().expect("Invalid socket address");

    // Bind listener to the address
    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind address");

    println!(
        "▶️  Triangular arbitrage server running at http://0.0.0.0:{}",
        port_env
    );

    // Start the server using owned listener and app
    serve(listener, app).await.expect("Server failed");
}
