//! Cloudflare Tool
//!
//! Secure DNS management tool that fetches API tokens from Google Secret Manager.
//! Agents can only call public methods - they never see the API token.
//!
//! ## Security Model
//!
//! - **Zero-Trust**: No secrets in code, env files, or config
//! - **ADC Authentication**: Uses `gcloud auth application-default login` locally,
//!   Workload Identity in production
//! - **Token Scoping**: Agents call `create_dns_record()` but never see `get_api_token()`
//! - **Runtime Injection**: `LORNU_GCP_PROJECT` is injected by K8s, not hardcoded

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use tracing::{info, warn};

/// Cloudflare DNS management tool with secure credential handling.
///
/// This tool allows agents to manage DNS records without exposing API tokens.
/// Credentials are fetched at runtime from Google Secret Manager using ADC.
#[derive(Debug)]
pub struct CloudflareTool {
    /// GCP project ID (injected at runtime via LORNU_GCP_PROJECT)
    project_id: String,
    /// Secret ID in Google Secret Manager
    secret_id: String,
    /// HTTP client for API calls
    http_client: Client,
    /// Cloudflare Zone ID (optional, can be provided per-request)
    default_zone_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CloudflareDnsRecord {
    #[serde(rename = "type")]
    record_type: String,
    name: String,
    content: String,
    ttl: u32,
    proxied: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct CloudflareResponse<T> {
    success: bool,
    errors: Vec<CloudflareError>,
    result: Option<T>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CloudflareError {
    code: i32,
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct DnsRecordResult {
    id: String,
    name: String,
    content: String,
}

impl CloudflareTool {
    /// Create a new CloudflareTool.
    ///
    /// Reads `LORNU_GCP_PROJECT` from environment (injected by K8s at runtime).
    /// Does NOT store any secrets - they are fetched on-demand from Secret Manager.
    pub fn new() -> Result<Self> {
        let project_id = env::var("LORNU_GCP_PROJECT")
            .context("LORNU_GCP_PROJECT must be set - this should be injected by K8s")?;

        let secret_id =
            env::var("CLOUDFLARE_SECRET_ID").unwrap_or_else(|_| "CLOUDFLARE_API_TOKEN".to_string());

        let default_zone_id = env::var("CLOUDFLARE_ZONE_ID").ok();

        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        info!(
            "CloudflareTool initialized (project: {}, secret: {})",
            project_id, secret_id
        );

        Ok(Self {
            project_id,
            secret_id,
            http_client,
            default_zone_id,
        })
    }

    /// Privately fetches the Cloudflare API token from Google Secret Manager.
    ///
    /// This method is NOT exposed to agents. It uses ADC for authentication:
    /// - Locally: `gcloud auth application-default login`
    /// - In K8s: Workload Identity automatically provides credentials
    ///
    /// # Security
    /// - No JSON keys on disk
    /// - No hardcoded credentials
    /// - Token is fetched fresh each time (can be cached if needed)
    async fn get_api_token(&self) -> Result<String> {
        // Get ADC token for accessing Secret Manager
        let metadata_url = "http://metadata.google.internal/computeMetadata/v1/instance/service-accounts/default/token";

        // Try GCE metadata server first (works in GKE with Workload Identity)
        let token = match self
            .http_client
            .get(metadata_url)
            .header("Metadata-Flavor", "Google")
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => {
                let token_response: serde_json::Value = resp.json().await?;
                token_response["access_token"]
                    .as_str()
                    .context("Invalid token response")?
                    .to_string()
            }
            _ => {
                // Fall back to ADC from environment (local development)
                self.get_adc_token().await?
            }
        };

        // Fetch secret from Secret Manager
        let secret_url = format!(
            "https://secretmanager.googleapis.com/v1/projects/{}/secrets/{}/versions/latest:access",
            self.project_id, self.secret_id
        );

        let response = self
            .http_client
            .get(&secret_url)
            .bearer_auth(&token)
            .send()
            .await
            .context("Failed to call Secret Manager")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!(
                "Secret Manager returned {}: {}. Ensure IAM binding exists.",
                status,
                body
            );
        }

        let secret_response: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse Secret Manager response")?;

        // Secret Manager returns base64-encoded payload
        let payload_base64 = secret_response["payload"]["data"]
            .as_str()
            .context("Secret payload not found")?;

        let payload_bytes = base64_decode(payload_base64)?;
        let api_token = String::from_utf8(payload_bytes)
            .context("Secret is not valid UTF-8")?
            .trim()
            .to_string();

        Ok(api_token)
    }

    /// Get ADC token for local development
    async fn get_adc_token(&self) -> Result<String> {
        // Try to get token from gcloud CLI's cached credentials
        let output = tokio::process::Command::new("gcloud")
            .args(["auth", "application-default", "print-access-token"])
            .output()
            .await
            .context("Failed to run gcloud CLI - ensure you've run 'gcloud auth application-default login'")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!(
                "gcloud auth failed: {}. Run 'gcloud auth application-default login'",
                stderr
            );
        }

        let token = String::from_utf8(output.stdout)
            .context("Invalid token output")?
            .trim()
            .to_string();

        Ok(token)
    }

    // =========================================================================
    // PUBLIC API - These are the only methods agents can call
    // =========================================================================

    /// Create a DNS A record.
    ///
    /// # Arguments
    /// * `zone_id` - Cloudflare Zone ID (or uses default if set)
    /// * `name` - DNS record name (e.g., "api.lornu.ai")
    /// * `content` - IP address
    /// * `proxied` - Whether to proxy through Cloudflare
    ///
    /// # Example
    /// ```ignore
    /// tool.create_dns_record(None, "api.lornu.ai", "1.2.3.4", true).await?;
    /// ```
    pub async fn create_dns_record(
        &self,
        zone_id: Option<&str>,
        name: &str,
        content: &str,
        proxied: bool,
    ) -> Result<String> {
        let zone = zone_id
            .map(|s| s.to_string())
            .or_else(|| self.default_zone_id.clone())
            .context("Zone ID must be provided or set via CLOUDFLARE_ZONE_ID")?;

        info!(
            "Creating DNS record: {} -> {} (proxied: {})",
            name, content, proxied
        );

        let token = self.get_api_token().await?;

        let record = CloudflareDnsRecord {
            record_type: "A".to_string(),
            name: name.to_string(),
            content: content.to_string(),
            ttl: 1, // Auto
            proxied,
        };

        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
            zone
        );

        let response = self
            .http_client
            .post(&url)
            .bearer_auth(&token)
            .json(&record)
            .send()
            .await
            .context("Failed to call Cloudflare API")?;

        let cf_response: CloudflareResponse<DnsRecordResult> = response
            .json()
            .await
            .context("Failed to parse Cloudflare response")?;

        if !cf_response.success {
            let errors: Vec<String> = cf_response
                .errors
                .iter()
                .map(|e| e.message.clone())
                .collect();
            anyhow::bail!("Cloudflare error: {}", errors.join(", "));
        }

        let result = cf_response.result.context("No result in response")?;
        info!("DNS record created: {} (ID: {})", result.name, result.id);

        Ok(result.id)
    }

    /// Delete a DNS record by ID.
    pub async fn delete_dns_record(&self, zone_id: Option<&str>, record_id: &str) -> Result<()> {
        let zone = zone_id
            .map(|s| s.to_string())
            .or_else(|| self.default_zone_id.clone())
            .context("Zone ID must be provided")?;

        info!("Deleting DNS record: {}", record_id);

        let token = self.get_api_token().await?;

        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
            zone, record_id
        );

        let response = self
            .http_client
            .delete(&url)
            .bearer_auth(&token)
            .send()
            .await
            .context("Failed to call Cloudflare API")?;

        let cf_response: CloudflareResponse<serde_json::Value> = response
            .json()
            .await
            .context("Failed to parse Cloudflare response")?;

        if !cf_response.success {
            let errors: Vec<String> = cf_response
                .errors
                .iter()
                .map(|e| e.message.clone())
                .collect();
            anyhow::bail!("Cloudflare error: {}", errors.join(", "));
        }

        info!("DNS record deleted: {}", record_id);
        Ok(())
    }

    /// List DNS records in a zone.
    pub async fn list_dns_records(&self, zone_id: Option<&str>) -> Result<Vec<serde_json::Value>> {
        let zone = zone_id
            .map(|s| s.to_string())
            .or_else(|| self.default_zone_id.clone())
            .context("Zone ID must be provided")?;

        let token = self.get_api_token().await?;

        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
            zone
        );

        let response = self
            .http_client
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await
            .context("Failed to call Cloudflare API")?;

        let cf_response: CloudflareResponse<Vec<serde_json::Value>> = response
            .json()
            .await
            .context("Failed to parse Cloudflare response")?;

        if !cf_response.success {
            let errors: Vec<String> = cf_response
                .errors
                .iter()
                .map(|e| e.message.clone())
                .collect();
            anyhow::bail!("Cloudflare error: {}", errors.join(", "));
        }

        Ok(cf_response.result.unwrap_or_default())
    }
}

/// Decode base64 string
fn base64_decode(input: &str) -> Result<Vec<u8>> {
    use std::io::Read;
    let bytes = input.as_bytes();
    let mut decoder =
        base64::read::DecoderReader::new(bytes, &base64::engine::general_purpose::STANDARD);
    let mut output = Vec::new();
    decoder.read_to_end(&mut output)?;
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_requires_project_id() {
        // Ensure LORNU_GCP_PROJECT is not set for this test
        env::remove_var("LORNU_GCP_PROJECT");
        let result = CloudflareTool::new();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("LORNU_GCP_PROJECT"));
    }
}
