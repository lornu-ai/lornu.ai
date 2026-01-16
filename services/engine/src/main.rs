//! Lornu AI Engine
//!
//! Core orchestration engine with secure tool integrations.
//! Uses ADC (Application Default Credentials) - no secrets in code.

use anyhow::{Context, Result};
use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use clap::{Parser, Subcommand};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing::{info, warn, Level};
use tracing_subscriber::FmtSubscriber;

mod agents;
mod tools;

use agents::executor::CrossplaneExecutor;
use agents::cherry_pick::CherryPickAgent;
use tools::CloudflareTool;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the API server (default)
    Server,
    /// Train the cherry-pick agent
    TrainCherryPick {
        #[arg(long, default_value = "100")]
        depth: u32,
    },
    /// Run a context-aware cherry-pick
    CherryPick {
        #[arg(long)]
        commit: String,
        #[arg(long)]
        branch: String,
    },
}

#[derive(Clone)]
struct AppState {
    executor: Arc<CrossplaneExecutor>,
    cloudflare: Option<Arc<CloudflareTool>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .json()
        .init();

    let cli = Cli::parse();

    match cli.command.unwrap_or(Commands::Server) {
        Commands::Server => run_server().await,
        Commands::TrainCherryPick { depth } => run_train_cherry_pick(depth).await,
        Commands::CherryPick { commit, branch } => run_cherry_pick(commit, branch).await,
    }
}

async fn run_train_cherry_pick(depth: u32) -> Result<()> {
    info!("Starting CherryPickAgent training (depth: {})", depth);

    let agent = create_cherry_pick_agent().await?;
    let count = agent.train_on_history(depth).await?;

    info!("Training complete. Learned {} patterns.", count);
    Ok(())
}

async fn run_cherry_pick(commit: String, branch: String) -> Result<()> {
    info!("Running CherryPickAgent (commit: {}, branch: {})", commit, branch);

    let agent = create_cherry_pick_agent().await?;
    let result = agent.execute_and_learn(&commit, &branch).await?;

    info!("Cherry-pick result: {:?}", result);

    if !result.success {
        std::process::exit(1);
    }

    Ok(())
}

async fn create_cherry_pick_agent() -> Result<CherryPickAgent> {
    let repo_path = std::env::current_dir()?;
    let qdrant_url = std::env::var("QDRANT_URL").unwrap_or_else(|_| "http://localhost:6333".to_string());
    let openai_api_key = std::env::var("OPENAI_API_KEY")
        .context("OPENAI_API_KEY environment variable must be set")?;

    CherryPickAgent::new(&repo_path, &qdrant_url, openai_api_key).await
}

async fn run_server() -> Result<()> {
    info!("Starting Lornu AI Engine Server");

    // Initialize Crossplane executor
    let executor = Arc::new(CrossplaneExecutor::new().await?);

    // Initialize Cloudflare tool (optional - requires LORNU_GCP_PROJECT)
    let cloudflare = match CloudflareTool::new() {
        Ok(tool) => {
            info!("CloudflareTool initialized");
            Some(Arc::new(tool))
        }
        Err(e) => {
            warn!("CloudflareTool not available: {} (set LORNU_GCP_PROJECT to enable)", e);
            None
        }
    };

    let state = AppState { executor, cloudflare };

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/provision/memory", post(provision_memory))
        .route("/api/provision/worker", post(provision_worker))
        .route("/api/agents/status", get(agent_status))
        .route("/api/dns/create", post(create_dns_record))
        .route("/api/dns/list", get(list_dns_records))
        .layer(CorsLayer::permissive())
        .with_state(state);

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

// ============================================================================
// DNS Endpoints (CloudflareTool)
// ============================================================================

#[derive(serde::Deserialize)]
struct CreateDnsRequest {
    zone_id: Option<String>,
    name: String,
    content: String,
    #[serde(default = "default_proxied")]
    proxied: bool,
}

fn default_proxied() -> bool { true }

async fn create_dns_record(
    State(state): State<AppState>,
    Json(req): Json<CreateDnsRequest>,
) -> Json<serde_json::Value> {
    let cloudflare = match &state.cloudflare {
        Some(cf) => cf,
        None => return Json(serde_json::json!({
            "status": "error",
            "message": "CloudflareTool not configured (set LORNU_GCP_PROJECT)"
        })),
    };

    match cloudflare.create_dns_record(
        req.zone_id.as_deref(),
        &req.name,
        &req.content,
        req.proxied,
    ).await {
        Ok(record_id) => Json(serde_json::json!({
            "status": "created",
            "record_id": record_id,
            "name": req.name
        })),
        Err(e) => Json(serde_json::json!({
            "status": "error",
            "message": e.to_string()
        })),
    }
}

#[derive(serde::Deserialize)]
struct ListDnsRequest {
    zone_id: Option<String>,
}

async fn list_dns_records(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<ListDnsRequest>,
) -> Json<serde_json::Value> {
    let cloudflare = match &state.cloudflare {
        Some(cf) => cf,
        None => return Json(serde_json::json!({
            "status": "error",
            "message": "CloudflareTool not configured"
        })),
    };

    match cloudflare.list_dns_records(query.zone_id.as_deref()).await {
        Ok(records) => Json(serde_json::json!({
            "status": "ok",
            "records": records
        })),
        Err(e) => Json(serde_json::json!({
            "status": "error",
            "message": e.to_string()
        })),
    }
}
