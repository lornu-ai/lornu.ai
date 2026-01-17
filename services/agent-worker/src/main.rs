//! Lornu AI Agent Worker
//!
//! Executes agent tasks by processing messages from a task queue.

use anyhow::Result;
use async_channel::{bounded, Receiver, Sender};
use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    task_tx: Sender<(String, TaskRequest)>,
    task_rx: Receiver<(String, TaskRequest)>,
    active_tasks: Arc<RwLock<Vec<TaskStatus>>>,
    http_client: Client,
    llm_endpoint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRequest {
    pub agent_id: String,
    pub prompt: String,
    #[serde(default)]
    pub context: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStatus {
    pub task_id: String,
    pub agent_id: String,
    pub status: String,
    pub result: Option<serde_json::Value>,
}

#[tokio::main]
async fn main() -> Result<()> {
    FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .json()
        .init();

    info!("Starting Lornu AI Agent Worker");

    let (task_tx, task_rx) = bounded(1000);
    let llm_endpoint = env::var("LLM_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:11434/api/generate".to_string());

    let state = AppState {
        task_tx,
        task_rx: task_rx.clone(),
        active_tasks: Arc::new(RwLock::new(Vec::new())),
        http_client: Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()?,
        llm_endpoint,
    };

    // Start background processor
    let processor_state = state.clone();
    tokio::spawn(async move {
        loop {
            if let Ok((task_id, req)) = processor_state.task_rx.recv().await {
                info!("Processing task: {}", task_id);

                // Update to running
                {
                    let mut tasks = processor_state.active_tasks.write().await;
                    if let Some(t) = tasks.iter_mut().find(|t| t.task_id == task_id) {
                        t.status = "running".to_string();
                    }
                }

                // Execute
                let result = execute_task(&processor_state, &req).await;

                // Update result
                {
                    let mut tasks = processor_state.active_tasks.write().await;
                    if let Some(t) = tasks.iter_mut().find(|t| t.task_id == task_id) {
                        match result {
                            Ok(output) => {
                                t.status = "completed".to_string();
                                t.result = Some(output);
                            }
                            Err(e) => {
                                t.status = "failed".to_string();
                                t.result = Some(serde_json::json!({"error": e.to_string()}));
                            }
                        }
                    }
                }
            }
        }
    });

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/tasks", post(submit_task))
        .route("/api/tasks", get(list_tasks))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8082));
    info!("Agent Worker listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "lornu-agent-worker",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

async fn submit_task(
    State(state): State<AppState>,
    Json(req): Json<TaskRequest>,
) -> Json<serde_json::Value> {
    let task_id = Uuid::new_v4().to_string();

    let status = TaskStatus {
        task_id: task_id.clone(),
        agent_id: req.agent_id.clone(),
        status: "queued".to_string(),
        result: None,
    };

    state.active_tasks.write().await.push(status);
    let _ = state.task_tx.send((task_id.clone(), req)).await;

    Json(serde_json::json!({"task_id": task_id, "status": "queued"}))
}

async fn list_tasks(State(state): State<AppState>) -> Json<serde_json::Value> {
    let tasks = state.active_tasks.read().await;
    Json(serde_json::json!({"tasks": tasks.clone(), "count": tasks.len()}))
}

async fn execute_task(state: &AppState, req: &TaskRequest) -> Result<serde_json::Value> {
    let model = match req.agent_id.split('-').next().unwrap_or("") {
        "summarizer" => "llama3.1:8b",
        "coder" => "codellama:13b",
        _ => "llama3.1:8b",
    };

    let llm_req = serde_json::json!({
        "model": model,
        "prompt": req.prompt,
        "stream": false,
        "context": req.context
    });

    let resp = state
        .http_client
        .post(&state.llm_endpoint)
        .json(&llm_req)
        .send()
        .await?;

    let result: serde_json::Value = resp.json().await?;

    Ok(serde_json::json!({
        "agent_id": req.agent_id,
        "response": result.get("response"),
        "model": model
    }))
}
