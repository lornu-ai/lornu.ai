use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use base64::{engine::general_purpose, Engine as _};
use hmac::{Hmac, Mac};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use reqwest::Client;
use serde::Deserialize;
use sha2::Sha256;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{error, info, warn};

#[derive(Clone)]
struct AppState {
    webhook_secret: Option<Vec<u8>>,
    github_app_id: Option<String>,
    github_installation_id: Option<String>,
    github_private_key: Option<Vec<u8>>,
    github_api_url: String,
    post_comments: bool,
    http: Client,
}

#[derive(Debug, Deserialize)]
struct PullRequestEvent {
    action: String,
    pull_request: PullRequestPayload,
    repository: RepositoryPayload,
}

#[derive(Debug, Deserialize)]
struct PullRequestPayload {
    number: u64,
    title: String,
    diff_url: String,
    comments_url: String,
    html_url: String,
}

#[derive(Debug, Deserialize)]
struct RepositoryPayload {
    full_name: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let webhook_secret = env::var("GITHUB_WEBHOOK_SECRET")
        .ok()
        .map(|value| value.into_bytes());
    let github_app_id = env::var("GITHUB_APP_ID").ok();
    let github_installation_id = env::var("GITHUB_INSTALLATION_ID").ok();
    let github_private_key = load_private_key();
    let github_api_url =
        env::var("GITHUB_API_URL").unwrap_or_else(|_| "https://api.github.com".to_string());
    let post_comments = env::var("GITHUB_POST_COMMENTS")
        .map(|value| value == "true" || value == "1")
        .unwrap_or(false);

    if webhook_secret.is_none() {
        warn!("GITHUB_WEBHOOK_SECRET not set; signature verification disabled.");
    }
    if github_app_id.is_none()
        || github_installation_id.is_none()
        || github_private_key.is_none()
    {
        warn!("GitHub App credentials missing; comment posting disabled.");
    }

    let state = AppState {
        webhook_secret,
        github_app_id,
        github_installation_id,
        github_private_key,
        github_api_url,
        post_comments,
        http: Client::new(),
    };

    let app = Router::new()
        .route("/webhook", post(handle_webhook))
        .route("/healthz", post(healthz).get(healthz))
        .with_state(state);

    let address = format!("0.0.0.0:{port}");
    info!("ai-agent-pr-comment listening on {address}");
    let listener = tokio::net::TcpListener::bind(address).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn healthz() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({"status": "ok"})))
}

async fn handle_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    if let Some(secret) = &state.webhook_secret {
        let signature = headers
            .get("X-Hub-Signature-256")
            .and_then(|value| value.to_str().ok());
        match signature {
            Some(signature) if verify_signature(secret, &body, signature) => {}
            _ => {
                warn!("Webhook signature verification failed.");
                return StatusCode::UNAUTHORIZED;
            }
        }
    }

    let event = headers
        .get("X-GitHub-Event")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("unknown");

    match event {
        "ping" => {
            info!("Received ping event.");
            StatusCode::OK
        }
        "pull_request" => match serde_json::from_slice::<PullRequestEvent>(&body) {
            Ok(payload) => {
                if matches!(payload.action.as_str(), "opened" | "synchronize") {
                    if let Err(err) =
                        analyze_and_comment(&state, &payload.pull_request, &payload.repository)
                            .await
                    {
                        error!("Failed to process PR: {err:#}");
                    }
                }
                StatusCode::OK
            }
            Err(err) => {
                error!("Failed to parse pull_request payload: {err:#}");
                StatusCode::BAD_REQUEST
            }
        },
        other => {
            info!("Ignoring GitHub event: {other}");
            StatusCode::OK
        }
    }
}

fn verify_signature(secret: &[u8], body: &[u8], signature: &str) -> bool {
    let signature = signature.strip_prefix("sha256=");
    let Some(signature_hex) = signature else {
        return false;
    };

    let Ok(expected) = hex::decode(signature_hex) else {
        return false;
    };

    let mut mac = Hmac::<Sha256>::new_from_slice(secret).expect("HMAC can take key of any size");
    mac.update(body);
    mac.verify_slice(&expected).is_ok()
}

async fn analyze_and_comment(
    state: &AppState,
    pr: &PullRequestPayload,
    repo: &RepositoryPayload,
) -> anyhow::Result<()> {
    info!(
        pr_number = pr.number,
        pr_title = %pr.title,
        diff_url = %pr.diff_url,
        repo = %repo.full_name,
        "Queued PR analysis (stub)."
    );
    if !state.post_comments {
        return Ok(());
    }

    let Some(token) = get_installation_token(state).await? else {
        return Ok(());
    };

    let body = serde_json::json!({
        "body": format!(
            "ai-agent-pr-comment: received PR #{} ({})",
            pr.number, pr.title
        )
    });

    state
        .http
        .post(&pr.comments_url)
        .bearer_auth(token)
        .header("User-Agent", "ai-agent-pr-comment")
        .header("Accept", "application/vnd.github+json")
        .json(&body)
        .send()
        .await?
        .error_for_status()?;

    info!(pr_url = %pr.html_url, "Posted PR comment.");
    Ok(())
}

fn load_private_key() -> Option<Vec<u8>> {
    if let Ok(value) = env::var("GITHUB_APP_PRIVATE_KEY") {
        return Some(value.into_bytes());
    }
    if let Ok(encoded) = env::var("GITHUB_APP_PRIVATE_KEY_B64") {
        if let Ok(decoded) = general_purpose::STANDARD.decode(encoded) {
            return Some(decoded);
        }
    }
    None
}

async fn get_installation_token(state: &AppState) -> anyhow::Result<Option<String>> {
    let (Some(app_id), Some(installation_id), Some(private_key)) = (
        state.github_app_id.as_ref(),
        state.github_installation_id.as_ref(),
        state.github_private_key.as_ref(),
    ) else {
        return Ok(None);
    };

    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;
    let claims = serde_json::json!({
        "iat": now - 60,
        "exp": now + 600,
        "iss": app_id,
    });

    let header = Header::new(Algorithm::RS256);
    let token = encode(
        &header,
        &claims,
        &EncodingKey::from_rsa_pem(private_key)?,
    )?;

    let url = format!(
        "{}/app/installations/{}/access_tokens",
        state.github_api_url, installation_id
    );
    let response = state
        .http
        .post(url)
        .bearer_auth(token)
        .header("User-Agent", "ai-agent-pr-comment")
        .header("Accept", "application/vnd.github+json")
        .send()
        .await?
        .error_for_status()?;

    let payload: serde_json::Value = response.json().await?;
    let access_token = payload
        .get("token")
        .and_then(|value| value.as_str())
        .map(|value| value.to_string());
    Ok(access_token)
}
