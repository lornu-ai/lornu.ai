use anyhow::Result;
use axum::{extract::{Path, State}, http::StatusCode, response::{IntoResponse, Response}, routing::{get, post}, Json, Router};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use surrealdb::engine::remote::ws::Ws;
use surrealdb::{sql::Thing, Surreal};
use tracing::info;

// --- Application Modules ---
mod llm_provider;

type DbConnection = Surreal<surrealdb::engine::remote::ws::Client>;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing (logging)
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("🚀 Starting Orchestrator Engine...");

    // Connect to SurrealDB
    // This will read the DB_URL from the environment, which our Dagger pipeline provides.
    info!("💾 Connecting to database...");
    let db_url = std::env::var("DB_URL").unwrap_or_else(|_| "ws://127.0.0.1:8000".to_string());
    let db: Surreal<DbConnection> = Surreal::new::<Ws>(db_url).await?;
    db.use_ns("lornu").use_db("orchestrator").await?;
    info!("✅ Database connection established.");

    // Configure Axum routes
    let app = Router::new()
        .route("/", get(health_check))
        .route("/api/v1/goals", post(create_goal))
        .route("/api/v1/goals/:id/context", post(get_goal_context))
        .with_state(db.clone());

    // Start the server
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    info!("📡 Listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "Orchestrator is running"
}

// --- Custom Error Type for Axum Handlers ---

struct AppError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` in handlers
// that return `Result<_, AppError>`.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

// We'll use this as the return type for our handlers
type AppResult<T> = Result<T, AppError>;
// And a specific version for JSON responses
type JsonResult<T> = AppResult<(StatusCode, Json<T>)>;


// --- Goal Management ---

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Goal {
    id: Option<Thing>,
    description: String,
    status: String,
}

#[derive(Debug, Deserialize)]
struct CreateGoal {
    description: String,
}

async fn create_goal(
    State(db): State<Surreal<DbConnection>>,
    Json(input): Json<CreateGoal>,
) -> JsonResult<Goal> {
    // In a real implementation, we would also generate and store the embedding for the description.
    let mut created_goals: Vec<Goal> = db
        .create("goals")
        .content(Goal { id: None, description: input.description, status: "pending".to_string() })
        .await?;

    let goal = created_goals.pop().ok_or_else(|| anyhow::anyhow!("Failed to create goal: No record returned from DB"))?;

    Ok((StatusCode::CREATED, Json(goal)))
}

// --- Hybrid Context Retrieval Implementation ---

#[derive(Debug, Deserialize)]
struct HybridContext {
    recent: Vec<Message>,
    memory: Vec<Message>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Message {
    id: Option<Thing>,
    content: String,
    role: String,
    // In a real scenario, this would be a vector of f32
    embedding: Vec<f32>,
}

/// Executes a hybrid retrieval query to get recent and semantically relevant messages.
async fn fetch_agent_context(
    db: &Surreal<DbConnection>,
    session_id: &str,
    embedding: Vec<f32>,
) -> surrealdb::Result<HybridContext> {
    let mut response = db.query(
        "BEGIN;
         LET $recent = (SELECT * FROM message WHERE session = $session ORDER BY created_at DESC LIMIT 10);
         LET $memory = (SELECT * FROM message WHERE embedding <|5|> $vector AND session != $session LIMIT 5);
         RETURN { recent: $recent, memory: $memory };
         COMMIT;"
    )
    .bind(("session", session_id))
    .bind(("vector", embedding))
    .await?;

    // The result of the `RETURN` statement is the 3rd one in the block (index 2)
    let context: Option<HybridContext> = response.take(2)?;

    Ok(context.unwrap_or(HybridContext { recent: vec![], memory: vec![] }))
}

#[derive(Debug, Deserialize)]
struct GetContextRequest {
    query: String,
}

async fn get_goal_context(
    State(db): State<Surreal<DbConnection>>,
    Path(goal_id): Path<String>,
    Json(_input): Json<GetContextRequest>,
) -> JsonResult<HybridContext> {
    // For demonstration, we generate a dummy embedding. In a real app, you'd use an embedding model.
    let dummy_embedding = vec![0.1, 0.2, 0.3, 0.4, 0.5];
    let context = fetch_agent_context(&db, &goal_id, dummy_embedding).await?;
    Ok((StatusCode::OK, Json(context)))
}