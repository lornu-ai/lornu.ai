//! Cloudflare DNS API Client
//!
//! High-performance Rust client for Cloudflare DNS record management.

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{info, debug};

const CLOUDFLARE_API_BASE: &str = "https://api.cloudflare.com/client/v4";

/// Cloudflare DNS record type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum DnsRecordType {
    A,
    #[allow(clippy::upper_case_acronyms)]
    AAAA,
    #[allow(clippy::upper_case_acronyms)]
    CNAME,
    #[allow(clippy::upper_case_acronyms)]
    TXT,
    MX,
    NS,
}

impl std::fmt::Display for DnsRecordType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DnsRecordType::A => write!(f, "A"),
            DnsRecordType::AAAA => write!(f, "AAAA"),
            DnsRecordType::CNAME => write!(f, "CNAME"),
            DnsRecordType::TXT => write!(f, "TXT"),
            DnsRecordType::MX => write!(f, "MX"),
            DnsRecordType::NS => write!(f, "NS"),
        }
    }
}

/// A DNS record from Cloudflare
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsRecord {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub record_type: DnsRecordType,
    pub content: String,
    pub ttl: u32,
    pub proxied: bool,
    #[serde(default)]
    pub comment: Option<String>,
}

/// Request to create/update a DNS record
#[derive(Debug, Clone, Serialize)]
pub struct DnsRecordRequest {
    #[serde(rename = "type")]
    pub record_type: DnsRecordType,
    pub name: String,
    pub content: String,
    pub ttl: u32,
    pub proxied: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

/// Cloudflare API response wrapper
#[derive(Debug, Deserialize)]
struct CloudflareResponse<T> {
    success: bool,
    errors: Vec<CloudflareError>,
    result: Option<T>,
}

#[derive(Debug, Deserialize)]
struct CloudflareError {
    _code: i32,
    message: String,
}

/// Result of a DNS sync operation
#[derive(Debug, Clone, Serialize)]
pub struct DnsRecordSyncResult {
    pub record_name: String,
    pub action: DnsAction,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DnsAction {
    Created,
    Updated,
    Unchanged,
    Deleted,
    Error,
}

/// Cloudflare DNS Client
pub struct CloudflareDnsClient {
    http_client: Client,
    api_token: String,
    zone_id: String,
}

impl CloudflareDnsClient {
    /// Create a new Cloudflare DNS client
    pub fn new(api_token: String, zone_id: String) -> Result<Self> {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            http_client,
            api_token,
            zone_id,
        })
    }

    /// List all DNS records in the zone
    pub async fn list_dns_records(&self) -> Result<Vec<DnsRecord>> {
        let url = format!(
            "{}/zones/{}/dns_records?per_page=1000",
            CLOUDFLARE_API_BASE, self.zone_id
        );

        let response = self
            .http_client
            .get(&url)
            .bearer_auth(&self.api_token)
            .send()
            .await
            .context("Failed to call Cloudflare API")?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            anyhow::bail!("Cloudflare API error {}: {}", status, text);
        }

        let result: CloudflareResponse<Vec<DnsRecord>> = response.json().await?;

        if !result.success {
            let errors: Vec<String> = result.errors.iter().map(|e| e.message.clone()).collect();
            anyhow::bail!("Cloudflare errors: {}", errors.join(", "));
        }

        Ok(result.result.unwrap_or_default())
    }

    /// Get a specific DNS record by name and type
    pub async fn get_record(&self, name: &str, record_type: DnsRecordType) -> Result<Option<DnsRecord>> {
        let url = format!(
            "{}/zones/{}/dns_records?name={}&type={}",
            CLOUDFLARE_API_BASE, self.zone_id, name, record_type
        );

        let response = self
            .http_client
            .get(&url)
            .bearer_auth(&self.api_token)
            .send()
            .await?;

        let result: CloudflareResponse<Vec<DnsRecord>> = response.json().await?;

        if !result.success {
            let errors: Vec<String> = result.errors.iter().map(|e| e.message.clone()).collect();
            anyhow::bail!("Cloudflare errors: {}", errors.join(", "));
        }

        Ok(result.result.and_then(|r| r.into_iter().next()))
    }

    /// Create a new DNS record
    pub async fn create_record(&self, request: &DnsRecordRequest) -> Result<DnsRecord> {
        let url = format!(
            "{}/zones/{}/dns_records",
            CLOUDFLARE_API_BASE, self.zone_id
        );

        let response = self
            .http_client
            .post(&url)
            .bearer_auth(&self.api_token)
            .json(request)
            .send()
            .await?;

        let status = response.status();
        let result: CloudflareResponse<DnsRecord> = response.json().await?;

        if !result.success {
            let errors: Vec<String> = result.errors.iter().map(|e| e.message.clone()).collect();
            anyhow::bail!("Failed to create record ({}): {}", status, errors.join(", "));
        }

        result.result.context("No record in response")
    }

    /// Update an existing DNS record
    pub async fn update_record(&self, record_id: &str, request: &DnsRecordRequest) -> Result<DnsRecord> {
        let url = format!(
            "{}/zones/{}/dns_records/{}",
            CLOUDFLARE_API_BASE, self.zone_id, record_id
        );

        let response = self
            .http_client
            .patch(&url)
            .bearer_auth(&self.api_token)
            .json(request)
            .send()
            .await?;

        let result: CloudflareResponse<DnsRecord> = response.json().await?;

        if !result.success {
            let errors: Vec<String> = result.errors.iter().map(|e| e.message.clone()).collect();
            anyhow::bail!("Failed to update record: {}", errors.join(", "));
        }

        result.result.context("No record in response")
    }

    /// Delete a DNS record
    pub async fn delete_record(&self, record_id: &str) -> Result<()> {
        let url = format!(
            "{}/zones/{}/dns_records/{}",
            CLOUDFLARE_API_BASE, self.zone_id, record_id
        );

        let response = self
            .http_client
            .delete(&url)
            .bearer_auth(&self.api_token)
            .send()
            .await?;

        let result: CloudflareResponse<serde_json::Value> = response.json().await?;

        if !result.success {
            let errors: Vec<String> = result.errors.iter().map(|e| e.message.clone()).collect();
            anyhow::bail!("Failed to delete record: {}", errors.join(", "));
        }

        Ok(())
    }

    /// Sync a DNS record - create if not exists, update if changed
    pub async fn sync_record(
        &self,
        name: &str,
        record_type: DnsRecordType,
        content: &str,
        ttl: u32,
        proxied: bool,
    ) -> Result<DnsRecordSyncResult> {
        let comment = Some("Managed by Lornu DNS Sync Agent".to_string());

        // Check if record exists
        let existing = self.get_record(name, record_type).await?;

        let request = DnsRecordRequest {
            record_type,
            name: name.to_string(),
            content: content.to_string(),
            ttl,
            proxied,
            comment: comment.clone(),
        };

        match existing {
            Some(record) => {
                // Check if update needed
                if record.content == content && record.ttl == ttl && record.proxied == proxied {
                    debug!("DNS record {} unchanged", name);
                    return Ok(DnsRecordSyncResult {
                        record_name: name.to_string(),
                        action: DnsAction::Unchanged,
                        success: true,
                        error: None,
                    });
                }

                // Update record
                info!("Updating DNS record {} -> {}", name, content);
                match self.update_record(&record.id, &request).await {
                    Ok(_) => Ok(DnsRecordSyncResult {
                        record_name: name.to_string(),
                        action: DnsAction::Updated,
                        success: true,
                        error: None,
                    }),
                    Err(e) => Ok(DnsRecordSyncResult {
                        record_name: name.to_string(),
                        action: DnsAction::Error,
                        success: false,
                        error: Some(e.to_string()),
                    }),
                }
            }
            None => {
                // Create new record
                info!("Creating DNS record {} -> {}", name, content);
                match self.create_record(&request).await {
                    Ok(_) => Ok(DnsRecordSyncResult {
                        record_name: name.to_string(),
                        action: DnsAction::Created,
                        success: true,
                        error: None,
                    }),
                    Err(e) => Ok(DnsRecordSyncResult {
                        record_name: name.to_string(),
                        action: DnsAction::Error,
                        success: false,
                        error: Some(e.to_string()),
                    }),
                }
            }
        }
    }

    /// Sync multiple DNS records from K8s ingresses
    pub async fn sync_from_ingresses(
        &self,
        ingresses: &[IngressDnsMapping],
    ) -> Result<Vec<DnsRecordSyncResult>> {
        let mut results = Vec::new();

        for mapping in ingresses {
            let result = self
                .sync_record(
                    &mapping.hostname,
                    DnsRecordType::A,
                    &mapping.ip_address,
                    1, // Auto TTL
                    false, // Don't proxy - need direct IP for GCP cert validation
                )
                .await?;

            results.push(result);
        }

        Ok(results)
    }
}

/// Mapping from K8s Ingress to DNS record
#[derive(Debug, Clone)]
pub struct IngressDnsMapping {
    pub hostname: String,
    pub ip_address: String,
    pub namespace: String,
    pub ingress_name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dns_record_request_serialization() {
        let request = DnsRecordRequest {
            record_type: DnsRecordType::A,
            name: "test.lornu.ai".to_string(),
            content: "1.2.3.4".to_string(),
            ttl: 300,
            proxied: false,
            comment: Some("Test".to_string()),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"type\":\"A\""));
        assert!(json.contains("\"name\":\"test.lornu.ai\""));
    }
}
