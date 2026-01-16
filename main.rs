use anyhow::Result;
use axum::{extract::State, http::StatusCode, routing::{get, post}, Json, Router};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use surrealdb::engine::remote::ws::Ws;
use surrealdb::{kvs::Datastore, sql::Thing, Surreal};
use tracing::info;

type Db = Surreal<Datastore>;

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
    let db_url = std::env::var("DB_URL").unwrap_or_else(|_| "ws://localhost:8000".to_string());
    let db: Surreal<Db> = Surreal::new::<Ws>(db_url).await?;
    db.use_ns("lornu").use_db("orchestrator").await?;
    info!("✅ Database connection established.");

    // Configure Axum routes
    let app = Router::new()
        .route("/", get(health_check))
        .route("/api/v1/goals", post(create_goal))
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

#[derive(Debug, Serialize, Deserialize)]
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
    State(db): State<Surreal<Db>>,
    Json(input): Json<CreateGoal>,
) -> (StatusCode, Json<Goal>) {
    let goal: Goal = db
        .create("goals")
        .content(Goal { id: None, description: input.description, status: "pending".to_string() })
        .await.unwrap().remove(0);

    (StatusCode::CREATED, Json(goal))
}