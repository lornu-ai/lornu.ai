//! Minimal Rust observability microservice
use axum::{routing::get, Router, response::IntoResponse};
use std::net::SocketAddr;

async fn healthz() -> impl IntoResponse {
    "ok"
}

async fn metrics() -> impl IntoResponse {
    // Example static metrics
    "# HELP requests_total Total HTTP requests\nrequests_total 1\n"
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/metrics", get(metrics));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8082));
    println!("Rust observability microservice running on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
