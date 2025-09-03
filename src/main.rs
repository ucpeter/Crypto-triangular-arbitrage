mod models;
mod exchanges;
mod logic;
mod routes;

use axum::{routing::post, Router};
use std::{net::SocketAddr, sync::Arc};
use tokio::{net::TcpListener, sync::Mutex};
use tower_http::{
    cors::{Any, CorsLayer},
    services::{ServeDir, ServeFile},
};

use crate::models::AppState;
use crate::routes::scan_handler;

#[tokio::main]
async fn main() {
    let shared_state = Arc::new(Mutex::new(AppState::default()));

    // CORS: allow your static UI to call /scan
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Serve the UI from ./static with index.html fallback
    let static_svc = ServeDir::new("static")
        .not_found_service(ServeFile::new("static/index.html"));

    // Build the app: static UI + API
    let app = Router::new()
        .route("/scan", post(scan_handler))
        .nest_service("/", static_svc)
        .layer(cors)
        .with_state(shared_state);

    // Bind + serve
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr: SocketAddr = format!("0.0.0.0:{port}").parse().expect("invalid addr");
    println!("▶️  UI available at http://0.0.0.0:{port}");

    let listener = TcpListener::bind(addr).await.expect("bind failed");
    axum::serve(listener, app).await.expect("server error");
                           }
