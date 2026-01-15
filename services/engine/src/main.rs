//! Lornu AI Engine
//!
//! The core orchestration engine for Lornu AI agents.
//! Manages agent lifecycles, provisions resources via Crossplane, and handles execution.

use anyhow::Result;
use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod agents;

use agents::executor::CrossplaneExecutor;

#[derive(Clone)]
struct AppState {
    executor: Arc<CrossplaneExecutor>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .json()
        .init();

    info!("Starting Lornu AI Engine");

    // Initialize Crossplane executor
    let executor = Arc::new(CrossplaneExecutor::new().await?);

    let state = AppState { executor };

    // Build router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/provision/memory", post(provision_memory))
        .route("/api/provision/worker", post(provision_worker))
        .route("/api/agents/status", get(agent_status))
        .layer(CorsLayer::permissive())
        .with_state(state);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    info!("Engine listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "lornu-engine",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

#[derive(serde::Deserialize)]
struct ProvisionMemoryRequest {
    user: String,
    agent: String,
    #[serde(default = "default_provider")]
    provider: String,
    #[serde(default = "default_memory_type")]
    memory_type: String,
    #[serde(default = "default_size")]
    size: String,
}

fn default_provider() -> String { "gcp".to_string() }
fn default_memory_type() -> String { "postgres".to_string() }
fn default_size() -> String { "10Gi".to_string() }

async fn provision_memory(
    State(state): State<AppState>,
    Json(req): Json<ProvisionMemoryRequest>,
) -> Json<serde_json::Value> {
    match state.executor.provision_agent_memory(
        &req.user,
        &req.agent,
        &req.provider,
        &req.memory_type,
        &req.size,
    ).await {
        Ok(name) => Json(serde_json::json!({
            "status": "provisioned",
            "claim_name": name
        })),
        Err(e) => Json(serde_json::json!({
            "status": "error",
            "message": e.to_string()
        })),
    }
}

#[derive(serde::Deserialize)]
struct ProvisionWorkerRequest {
    user: String,
    agent: String,
    #[serde(default)]
    gpu: bool,
    gpu_type: Option<String>,
    #[serde(default = "default_replicas")]
    replicas: i32,
}

fn default_replicas() -> i32 { 1 }

async fn provision_worker(
    State(state): State<AppState>,
    Json(req): Json<ProvisionWorkerRequest>,
) -> Json<serde_json::Value> {
    match state.executor.provision_agent_worker(
        &req.user,
        &req.agent,
        req.gpu,
        req.gpu_type.as_deref(),
        req.replicas,
    ).await {
        Ok(name) => Json(serde_json::json!({
            "status": "provisioned",
            "claim_name": name
        })),
        Err(e) => Json(serde_json::json!({
            "status": "error",
            "message": e.to_string()
        })),
    }
}

async fn agent_status(State(state): State<AppState>) -> Json<serde_json::Value> {
    match state.executor.list_agent_resources().await {
        Ok(resources) => Json(serde_json::json!({
            "status": "ok",
            "resources": resources
        })),
        Err(e) => Json(serde_json::json!({
            "status": "error",
            "message": e.to_string()
        })),
    }
}
