//! GitHub App Installation Token Generator
//!
//! Generates a short-lived installation access token from GitHub App credentials.
//! Uses RS256 JWT signing to authenticate as the GitHub App, then exchanges
//! the JWT for an installation token.
//!
//! ## Usage
//! ```bash
//! # With command line arguments
//! get-token \
//!   --app-id 123456 \
//!   --private-key-path ./key.pem \
//!   --installation-id 78901234
//!
//! # With environment variables
//! GITHUB_APP_ID=123456 \
//! GITHUB_PRIVATE_KEY_PATH=./key.pem \
//! GITHUB_INSTALLATION_ID=78901234 \
//! get-token
//! ```

use anyhow::{Context, Result};
use clap::Parser;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

/// GitHub App Installation Token Generator
#[derive(Parser, Debug)]
#[command(name = "get-token")]
#[command(about = "Generate GitHub App installation access tokens")]
#[command(version)]
struct Args {
    /// GitHub App ID
    #[arg(long, env = "GITHUB_APP_ID")]
    app_id: String,

    /// Path to the private key PEM file
    #[arg(long, env = "GITHUB_PRIVATE_KEY_PATH")]
    private_key_path: String,

    /// GitHub App Installation ID
    #[arg(long, env = "GITHUB_INSTALLATION_ID")]
    installation_id: u64,

    /// Output file path (optional, prints to stdout if not specified)
    #[arg(long, short)]
    output: Option<String>,

    /// Output format: token (default), json
    #[arg(long, default_value = "token")]
    format: String,
}

/// JWT claims for GitHub App authentication
#[derive(Debug, Serialize)]
struct Claims {
    /// Issued at time (Unix timestamp)
    iat: u64,
    /// Expiration time (Unix timestamp)
    exp: u64,
    /// Issuer (GitHub App ID)
    iss: String,
}

/// Response from GitHub installation token endpoint
#[derive(Debug, Deserialize)]
struct InstallationToken {
    token: String,
    expires_at: String,
}

/// Generate a JWT for GitHub App authentication
fn generate_jwt(app_id: &str, private_key_path: &str) -> Result<String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("Failed to get current time")?
        .as_secs();

    let claims = Claims {
        iat: now.saturating_sub(60), // 60 seconds ago to account for clock skew
        exp: now + 600,              // Expires in 10 minutes
        iss: app_id.to_string(),
    };

    let key_data = fs::read(private_key_path)
        .with_context(|| format!("Failed to read private key: {}", private_key_path))?;

    let encoding_key =
        EncodingKey::from_rsa_pem(&key_data).context("Failed to parse private key as RSA PEM")?;

    let header = Header::new(Algorithm::RS256);

    encode(&header, &claims, &encoding_key).context("Failed to encode JWT")
}

/// Exchange JWT for an installation access token using the GitHub REST API
async fn get_installation_token(jwt: &str, installation_id: u64) -> Result<InstallationToken> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://api.github.com/app/installations/{}/access_tokens",
        installation_id
    );

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", jwt))
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "lornu-ai-github-bot")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await
        .context("Failed to send request to GitHub API")?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("GitHub API error ({}): {}", status, body);
    }

    response
        .json::<InstallationToken>()
        .await
        .context("Failed to parse installation token response")
}

#[derive(Serialize)]
struct TokenOutput {
    token: String,
    installation_id: u64,
    expires_at: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    eprintln!("🔐 Generating JWT for GitHub App {}...", args.app_id);
    let jwt = generate_jwt(&args.app_id, &args.private_key_path)?;

    eprintln!(
        "🔑 Exchanging JWT for installation token (installation: {})...",
        args.installation_id
    );
    let token_response = get_installation_token(&jwt, args.installation_id).await?;

    let output = match args.format.as_str() {
        "json" => {
            let output = TokenOutput {
                token: token_response.token.clone(),
                installation_id: args.installation_id,
                expires_at: token_response.expires_at.clone(),
            };
            serde_json::to_string_pretty(&output)?
        }
        _ => token_response.token.clone(),
    };

    if let Some(output_path) = args.output {
        fs::write(&output_path, &output)
            .with_context(|| format!("Failed to write token to {}", output_path))?;
        // Set restrictive permissions on the token file
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&output_path, fs::Permissions::from_mode(0o600))?;
        }
        eprintln!("✅ Token saved to {}", output_path);
    } else {
        println!("{}", output);
        eprintln!(
            "✅ Token generated successfully (expires: {})",
            token_response.expires_at
        );
    }

    Ok(())
}
