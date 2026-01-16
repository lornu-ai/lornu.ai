//! GitHub App Authentication
//!
//! Utilities for authenticating as a GitHub App using JWT and installation tokens.

use anyhow::{Context, Result};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

/// JWT claims for GitHub App authentication
#[derive(Debug, Serialize)]
pub struct GitHubAppClaims {
    /// Issued at time (Unix timestamp)
    pub iat: u64,
    /// Expiration time (Unix timestamp)
    pub exp: u64,
    /// Issuer (GitHub App ID)
    pub iss: String,
}

/// Generate a JWT for GitHub App authentication
///
/// # Arguments
/// * `app_id` - The GitHub App ID
/// * `private_key_pem` - The private key in PEM format
///
/// # Returns
/// A JWT string valid for 10 minutes
pub fn generate_jwt(app_id: &str, private_key_pem: &[u8]) -> Result<String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("Failed to get current time")?
        .as_secs();

    let claims = GitHubAppClaims {
        iat: now.saturating_sub(60), // 60 seconds ago to account for clock skew
        exp: now + 600,              // Expires in 10 minutes
        iss: app_id.to_string(),
    };

    let encoding_key =
        EncodingKey::from_rsa_pem(private_key_pem).context("Failed to parse private key")?;

    let header = Header::new(Algorithm::RS256);

    encode(&header, &claims, &encoding_key).context("Failed to encode JWT")
}
