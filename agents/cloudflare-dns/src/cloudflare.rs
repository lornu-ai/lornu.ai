//! Cloudflare API Client
//!
//! Type-safe Cloudflare DNS API wrapper using the v4 REST API.
//! Handles zone lookup, record CRUD, and error responses.

use anyhow::{bail, Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::debug;

const CLOUDFLARE_API: &str = "https://api.cloudflare.com/client/v4";

/// Cloudflare API client
pub struct CloudflareClient {
    client: Client,
    api_token: String,
}

// ============================================================
// API Response Types
// ============================================================

#[derive(Debug, Deserialize)]
struct ApiResponse<T> {
    success: bool,
    errors: Vec<ApiError>,
    result: Option<T>,
}

#[derive(Debug, Deserialize)]
struct ApiError {
    code: i32,
    message: String,
}

#[derive(Debug, Deserialize)]
struct Zone {
    id: String,
    name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsRecord {
    pub id: String,
    #[serde(rename = "type")]
    pub record_type: String,
    pub name: String,
    pub content: String,
    pub ttl: u32,
    pub proxied: bool,
}

#[derive(Debug, Serialize)]
struct CreateRecordRequest {
    #[serde(rename = "type")]
    record_type: String,
    name: String,
    content: String,
    ttl: u32,
    proxied: bool,
}

#[derive(Debug, Serialize)]
struct UpdateRecordRequest {
    content: String,
}

// ============================================================
// Client Implementation
// ============================================================

impl CloudflareClient {
    /// Create a new Cloudflare client with API token
    pub fn new(api_token: String) -> Self {
        let client = Client::builder()
            .user_agent("lornu-cloudflare-dns/0.1.0")
            .build()
            .expect("Failed to build HTTP client");

        Self { client, api_token }
    }

    /// Get authorization headers
    fn auth_headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Bearer {}", self.api_token).parse().unwrap(),
        );
        headers.insert("Content-Type", "application/json".parse().unwrap());
        headers
    }

    /// Look up zone ID by name
    async fn get_zone_id(&self, zone_name: &str) -> Result<String> {
        debug!("Looking up zone ID for: {}", zone_name);

        let url = format!("{}/zones?name={}", CLOUDFLARE_API, zone_name);

        let response: ApiResponse<Vec<Zone>> = self
            .client
            .get(&url)
            .headers(self.auth_headers())
            .send()
            .await
            .context("Failed to query zones")?
            .json()
            .await
            .context("Failed to parse zones response")?;

        if !response.success {
            let errors: Vec<String> = response.errors.iter().map(|e| e.message.clone()).collect();
            bail!("Cloudflare API error: {}", errors.join(", "));
        }

        let zones = response.result.unwrap_or_default();
        let zone = zones
            .first()
            .context(format!("Zone not found: {}", zone_name))?;

        debug!("Found zone ID: {}", zone.id);
        Ok(zone.id.clone())
    }

    /// List all DNS records for a zone
    pub async fn list_records(&self, zone_name: &str) -> Result<Vec<DnsRecord>> {
        let zone_id = self.get_zone_id(zone_name).await?;

        let url = format!("{}/zones/{}/dns_records", CLOUDFLARE_API, zone_id);

        let response: ApiResponse<Vec<DnsRecord>> = self
            .client
            .get(&url)
            .headers(self.auth_headers())
            .send()
            .await
            .context("Failed to list DNS records")?
            .json()
            .await
            .context("Failed to parse DNS records response")?;

        if !response.success {
            let errors: Vec<String> = response.errors.iter().map(|e| e.message.clone()).collect();
            bail!("Cloudflare API error: {}", errors.join(", "));
        }

        Ok(response.result.unwrap_or_default())
    }

    /// Create a new DNS record
    pub async fn create_record(
        &self,
        zone_name: &str,
        name: &str,
        record_type: &str,
        content: &str,
        ttl: u32,
        proxied: bool,
    ) -> Result<DnsRecord> {
        let zone_id = self.get_zone_id(zone_name).await?;

        let url = format!("{}/zones/{}/dns_records", CLOUDFLARE_API, zone_id);

        let request = CreateRecordRequest {
            record_type: record_type.to_uppercase(),
            name: name.to_string(),
            content: content.to_string(),
            ttl,
            proxied,
        };

        let response: ApiResponse<DnsRecord> = self
            .client
            .post(&url)
            .headers(self.auth_headers())
            .json(&request)
            .send()
            .await
            .context("Failed to create DNS record")?
            .json()
            .await
            .context("Failed to parse create response")?;

        if !response.success {
            let errors: Vec<String> = response.errors.iter().map(|e| e.message.clone()).collect();
            bail!("Cloudflare API error: {}", errors.join(", "));
        }

        response.result.context("No record in response")
    }

    /// Delete a DNS record
    pub async fn delete_record(&self, zone_name: &str, record_id: &str) -> Result<()> {
        let zone_id = self.get_zone_id(zone_name).await?;

        let url = format!(
            "{}/zones/{}/dns_records/{}",
            CLOUDFLARE_API, zone_id, record_id
        );

        let response: ApiResponse<serde_json::Value> = self
            .client
            .delete(&url)
            .headers(self.auth_headers())
            .send()
            .await
            .context("Failed to delete DNS record")?
            .json()
            .await
            .context("Failed to parse delete response")?;

        if !response.success {
            let errors: Vec<String> = response.errors.iter().map(|e| e.message.clone()).collect();
            bail!("Cloudflare API error: {}", errors.join(", "));
        }

        Ok(())
    }

    /// Update an existing DNS record
    pub async fn update_record(
        &self,
        zone_name: &str,
        record_id: &str,
        content: &str,
    ) -> Result<()> {
        let zone_id = self.get_zone_id(zone_name).await?;

        let url = format!(
            "{}/zones/{}/dns_records/{}",
            CLOUDFLARE_API, zone_id, record_id
        );

        let request = UpdateRecordRequest {
            content: content.to_string(),
        };

        let response: ApiResponse<DnsRecord> = self
            .client
            .patch(&url)
            .headers(self.auth_headers())
            .json(&request)
            .send()
            .await
            .context("Failed to update DNS record")?
            .json()
            .await
            .context("Failed to parse update response")?;

        if !response.success {
            let errors: Vec<String> = response.errors.iter().map(|e| e.message.clone()).collect();
            bail!("Cloudflare API error: {}", errors.join(", "));
        }

        Ok(())
    }
}
