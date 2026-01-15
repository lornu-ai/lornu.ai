//! Lornu AI Gateway
//!
//! API gateway for routing requests to backend services.

use anyhow::Result;
use axum::{
    body::Body,
    extract::{Path, State},
    http::{Request, StatusCode},
    response::{IntoResponse, Response},
    routing::{any, get},
    Json, Router,
};
use reqwest::Client;
use std::collections::HashMap;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Clone)]
struct AppState {
    http_client: Client,
    routes: Arc<HashMap<String, String>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .json()
        .init();

    info!("Starting Lornu AI Gateway");

    let http_client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let mut routes = HashMap::new();
    routes.insert("engine".to_string(),
        env::var("ENGINE_URL").unwrap_or_else(|_| "http://engine:8080".to_string()));
    routes.insert("worker".to_string(),
        env::var("WORKER_URL").unwrap_or_else(|_| "http://agent-worker:8082".to_string()));

    let state = AppState {
        http_client,
        routes: Arc::new(routes),
    };

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/*path", any(proxy_request))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8081));
    info!("Gateway listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "lornu-gateway",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

async fn proxy_request(
    State(state): State<AppState>,
    Path(path): Path<String>,
    request: Request<Body>,
) -> Response {
    let parts: Vec<&str> = path.splitn(2, '/').collect();
    let service = parts.first().unwrap_or(&"");
    let remaining = parts.get(1).unwrap_or(&"");

    let target_url = match state.routes.get(*service) {
        Some(url) => format!("{}/{}", url, remaining),
        None => return (StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "service_not_found"}))).into_response(),
    };

    info!("Proxying to: {}", target_url);

    let method = request.method().clone();
    let headers = request.headers().clone();
    let mut req_builder = state.http_client.request(method, &target_url);

    for (name, value) in headers.iter() {
        if name != "host" {
            req_builder = req_builder.header(name.clone(), value.clone());
        }
    }

    let body_bytes = match axum::body::to_bytes(request.into_body(), usize::MAX).await {
        Ok(bytes) => bytes,
        Err(e) => return (StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    };

    if !body_bytes.is_empty() {
        req_builder = req_builder.body(body_bytes.to_vec());
    }

    match req_builder.send().await {
        Ok(resp) => {
            let status = resp.status();
            let body = resp.bytes().await.unwrap_or_default();
            Response::builder()
                .status(status)
                .body(Body::from(body))
                .unwrap_or_else(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Error").into_response())
        }
        Err(e) => (StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({"error": e.to_string()}))).into_response(),
    }
}
