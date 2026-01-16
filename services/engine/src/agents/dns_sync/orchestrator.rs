//! Multi-Cloud DNS Sync Orchestrator
//!
//! The main agent that coordinates endpoint discovery and Cloudflare updates.

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use tracing::{info, warn, error};

use super::providers::MultiCloudProviders;
use super::types::{
    DnsSyncResult, LoadBalancerPool, 
    MultiCloudConfig,
};

/// Multi-Cloud DNS Sync Agent
///
/// Issue #118: Global Control Plane that orchestrates endpoints across
/// AWS, Azure, and GCP via Cloudflare's intelligent global entry point.
pub struct MultiCloudDnsSyncAgent {
    /// Cloud provider adapters
    providers: MultiCloudProviders,
    /// HTTP client for Cloudflare API
    http_client: Client,
    /// GCP project ID for Secret Manager
    gcp_project_id: String,
    /// Cloudflare Secret ID in Secret Manager
    cloudflare_secret_id: String,
    /// Configuration
    config: MultiCloudConfig,
}

#[derive(Debug, Serialize, Deserialize)]
struct CloudflarePoolResponse {
    success: bool,
    errors: Vec<CloudflareError>,
    result: Option<CloudflarePool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CloudflarePoolListResponse {
    success: bool,
    errors: Vec<CloudflareError>,
    result: Option<Vec<CloudflarePool>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CloudflarePool {
    id: String,
    name: String,
    origins: Vec<CloudflareOrigin>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CloudflareOrigin {
    name: String,
    address: String,
    weight: f64,
    enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct CloudflareError {
    code: i32,
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CloudflareMonitorResponse {
    success: bool,
    errors: Vec<CloudflareError>,
    result: Option<CloudflareMonitor>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CloudflareMonitor {
    id: String,
}

impl MultiCloudDnsSyncAgent {
    /// Create a new Multi-Cloud DNS Sync Agent
    pub async fn new(config: MultiCloudConfig) -> Result<Self> {
        let providers = MultiCloudProviders::new("crossplane-system").await?;

        let gcp_project_id = env::var("LORNU_GCP_PROJECT")
            .context("LORNU_GCP_PROJECT must be set")?;

        let cloudflare_secret_id = env::var("CLOUDFLARE_SECRET_ID")
            .unwrap_or_else(|_| "CLOUDFLARE_API_TOKEN".to_string());

        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        info!(
            "MultiCloudDnsSyncAgent initialized (zone: {})",
            config.cloudflare_zone_id
        );

        Ok(Self {
            providers,
            http_client,
            gcp_project_id,
            cloudflare_secret_id,
            config,
        })
    }

    /// Fetch Cloudflare API token from Secret Manager
    async fn get_cloudflare_token(&self) -> Result<String> {
        // Try GCE metadata server first (GKE with Workload Identity)
        let metadata_url = "http://metadata.google.internal/computeMetadata/v1/instance/service-accounts/default/token";

        let gcp_token = match self
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
                // Fall back to gcloud CLI for local development
                let output = tokio::process::Command::new("gcloud")
                    .args(["auth", "application-default", "print-access-token"])
                    .output()
                    .await
                    .context("Failed to run gcloud CLI")?;

                if !output.status.success() {
                    anyhow::bail!("gcloud auth failed");
                }

                String::from_utf8(output.stdout)?.trim().to_string()
            }
        };

        // Fetch Cloudflare token from Secret Manager
        let secret_url = format!(
            "https://secretmanager.googleapis.com/v1/projects/{}/secrets/{}/versions/latest:access",
            self.gcp_project_id, self.cloudflare_secret_id
        );

        let response = self
            .http_client
            .get(&secret_url)
            .bearer_auth(&gcp_token)
            .send()
            .await
            .context("Failed to call Secret Manager")?;

        if !response.status().is_success() {
            anyhow::bail!("Secret Manager returned {}", response.status());
        }

        let secret_response: serde_json::Value = response.json().await?;
        let payload_base64 = secret_response["payload"]["data"]
            .as_str()
            .context("Secret payload not found")?;

        let payload_bytes = base64::Engine::decode(
            &base64::engine::general_purpose::STANDARD,
            payload_base64,
        )?;

        Ok(String::from_utf8(payload_bytes)?.trim().to_string())
    }

    /// Sync multi-cloud endpoints to Cloudflare Load Balancer Pool
    pub async fn sync(&self) -> Result<DnsSyncResult> {
        let timestamp = chrono::Utc::now();
        let mut errors = Vec::new();

        info!("Starting multi-cloud DNS sync");

        // 1. Discover endpoints from all clouds
        let mut endpoints = match self.providers.discover_all_endpoints().await {
            Ok(e) => e,
            Err(e) => {
                error!("Failed to discover endpoints: {}", e);
                return Ok(DnsSyncResult {
                    success: false,
                    pool_id: None,
                    origins_synced: 0,
                    errors: vec![e.to_string()],
                    timestamp,
                });
            }
        };

        if endpoints.is_empty() {
            warn!("No cloud endpoints discovered");
            return Ok(DnsSyncResult {
                success: true,
                pool_id: None,
                origins_synced: 0,
                errors: vec!["No endpoints discovered".to_string()],
                timestamp,
            });
        }

        info!("Discovered {} total endpoints", endpoints.len());

        // 2. Check health of all endpoints
        if let Err(e) = self.providers.check_all_health(&mut endpoints).await {
            warn!("Failed to check endpoint health: {}", e);
            errors.push(format!("Health check warning: {}", e));
        }

        // 3. Get Cloudflare token
        let cf_token = match self.get_cloudflare_token().await {
            Ok(t) => t,
            Err(e) => {
                error!("Failed to get Cloudflare token: {}", e);
                return Ok(DnsSyncResult {
                    success: false,
                    pool_id: None,
                    origins_synced: 0,
                    errors: vec![format!("Cloudflare auth failed: {}", e)],
                    timestamp,
                });
            }
        };

        // 4. Create or update health monitor
        let monitor_id = match self.ensure_health_monitor(&cf_token).await {
            Ok(id) => Some(id),
            Err(e) => {
                warn!("Failed to create health monitor: {}", e);
                errors.push(format!("Monitor warning: {}", e));
                None
            }
        };

        // 5. Create or update Load Balancer Pool
        let pool_name = format!("{}-multi-cloud", self.config.pool_name_prefix);
        let mut pool = LoadBalancerPool::new(&pool_name, &endpoints);
        pool.monitor = monitor_id;

        let pool_id = match self.upsert_pool(&cf_token, &pool).await {
            Ok(id) => id,
            Err(e) => {
                error!("Failed to upsert pool: {}", e);
                return Ok(DnsSyncResult {
                    success: false,
                    pool_id: None,
                    origins_synced: 0,
                    errors: vec![format!("Pool update failed: {}", e)],
                    timestamp,
                });
            }
        };

        info!(
            "Successfully synced {} origins to pool {}",
            endpoints.len(),
            pool_id
        );

        Ok(DnsSyncResult {
            success: true,
            pool_id: Some(pool_id),
            origins_synced: endpoints.len(),
            errors,
            timestamp,
        })
    }

    /// Ensure a health monitor exists for the pool
    async fn ensure_health_monitor(&self, token: &str) -> Result<String> {
        let url = format!(
            "https://api.cloudflare.com/client/v4/accounts/{}/load_balancers/monitors",
            self.get_account_id(token).await?
        );

        // Check if monitor already exists
        let list_response = self
            .http_client
            .get(&url)
            .bearer_auth(token)
            .send()
            .await?;

        let list_result: serde_json::Value = list_response.json().await?;
        
        if let Some(monitors) = list_result["result"].as_array() {
            for monitor in monitors {
                if monitor["description"]
                    .as_str()
                    .map(|d| d.contains("lornu-multi-cloud"))
                    .unwrap_or(false)
                {
                    if let Some(id) = monitor["id"].as_str() {
                        info!("Found existing health monitor: {}", id);
                        return Ok(id.to_string());
                    }
                }
            }
        }

        // Create new monitor
        let hc = &self.config.health_check;
        let monitor_config = serde_json::json!({
            "type": "http",
            "description": "lornu-multi-cloud-health-monitor",
            "method": "GET",
            "path": hc.path,
            "expected_codes": hc.expected_codes,
            "interval": hc.interval,
            "timeout": hc.timeout,
            "retries": hc.retries,
            "follow_redirects": true,
            "allow_insecure": false
        });

        let response = self
            .http_client
            .post(&url)
            .bearer_auth(token)
            .json(&monitor_config)
            .send()
            .await?;

        let result: CloudflareMonitorResponse = response.json().await?;

        if !result.success {
            let errors: Vec<String> = result.errors.iter().map(|e| e.message.clone()).collect();
            anyhow::bail!("Failed to create monitor: {}", errors.join(", "));
        }

        let monitor_id = result
            .result
            .context("No monitor result")?
            .id;

        info!("Created health monitor: {}", monitor_id);
        Ok(monitor_id)
    }

    /// Get Cloudflare account ID
    async fn get_account_id(&self, token: &str) -> Result<String> {
        let response = self
            .http_client
            .get("https://api.cloudflare.com/client/v4/accounts")
            .bearer_auth(token)
            .send()
            .await?;

        let result: serde_json::Value = response.json().await?;

        result["result"]
            .as_array()
            .and_then(|accounts| accounts.first())
            .and_then(|account| account["id"].as_str())
            .map(|s| s.to_string())
            .context("No account found")
    }

    /// Create or update a Load Balancer Pool
    async fn upsert_pool(&self, token: &str, pool: &LoadBalancerPool) -> Result<String> {
        let account_id = self.get_account_id(token).await?;
        let base_url = format!(
            "https://api.cloudflare.com/client/v4/accounts/{}/load_balancers/pools",
            account_id
        );

        // Check if pool exists
        let list_response = self
            .http_client
            .get(&base_url)
            .bearer_auth(token)
            .send()
            .await?;

        let list_result: CloudflarePoolListResponse = list_response.json().await?;
        
        let existing_pool = list_result
            .result
            .as_ref()
            .and_then(|pools| pools.iter().find(|p| p.name == pool.name));

        let origins: Vec<CloudflareOrigin> = pool
            .origins
            .iter()
            .map(|o| CloudflareOrigin {
                name: o.name.clone(),
                address: o.address.clone(),
                weight: o.weight,
                enabled: o.enabled,
            })
            .collect();

        let pool_config = serde_json::json!({
            "name": pool.name,
            "description": pool.description,
            "origins": origins,
            "monitor": pool.monitor,
            "minimum_origins": pool.minimum_origins,
            "check_regions": pool.check_regions,
            "notification_email": pool.notification_email
        });

        let response = if let Some(existing) = existing_pool {
            // Update existing pool
            info!("Updating existing pool: {}", existing.id);
            self.http_client
                .put(format!("{}/{}", base_url, existing.id))
                .bearer_auth(token)
                .json(&pool_config)
                .send()
                .await?
        } else {
            // Create new pool
            info!("Creating new pool: {}", pool.name);
            self.http_client
                .post(&base_url)
                .bearer_auth(token)
                .json(&pool_config)
                .send()
                .await?
        };

        let result: CloudflarePoolResponse = response.json().await?;

        if !result.success {
            let errors: Vec<String> = result.errors.iter().map(|e| e.message.clone()).collect();
            anyhow::bail!("Cloudflare pool error: {}", errors.join(", "));
        }

        let pool_id = result.result.context("No pool result")?.id;
        Ok(pool_id)
    }

    /// Trigger immediate failover to a specific cloud
    pub async fn failover_to(&self, target_provider: super::types::CloudProvider) -> Result<DnsSyncResult> {
        let timestamp = chrono::Utc::now();

        info!("Triggering failover to {}", target_provider);

        // Discover all endpoints
        let mut endpoints = self.providers.discover_all_endpoints().await?;

        // Set all non-target endpoints to weight 0
        for endpoint in &mut endpoints {
            if endpoint.provider != target_provider {
                endpoint.weight = 0;
                endpoint.enabled = false;
            } else {
                endpoint.weight = 100;
                endpoint.enabled = true;
            }
        }

        // Get token and update pool
        let cf_token = self.get_cloudflare_token().await?;
        let pool_name = format!("{}-multi-cloud", self.config.pool_name_prefix);
        let pool = LoadBalancerPool::new(&pool_name, &endpoints);

        let pool_id = self.upsert_pool(&cf_token, &pool).await?;

        info!("Failover to {} complete (pool: {})", target_provider, pool_id);

        Ok(DnsSyncResult {
            success: true,
            pool_id: Some(pool_id),
            origins_synced: 1,
            errors: vec![],
            timestamp,
        })
    }

    /// Rebalance traffic across all healthy endpoints
    pub async fn rebalance(&self) -> Result<DnsSyncResult> {
        info!("Rebalancing traffic across all clouds");

        // Just run normal sync - it will set equal weights
        self.sync().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_origin_conversion() {
        let endpoint = CloudEndpoint::gcp_global_lb("34.111.65.194", 50);
        let origin = PoolOrigin::from(&endpoint);

        assert_eq!(origin.name, "gcp-origin");
        assert_eq!(origin.address, "34.111.65.194");
        assert!((origin.weight - 0.5).abs() < 0.01);
        assert!(origin.enabled);
    }

    #[test]
    fn test_load_balancer_pool_creation() {
        let endpoints = vec![
            CloudEndpoint::aws_alb("aws.elb.amazonaws.com", "us-east-1", 33),
            CloudEndpoint::azure_front_door("azure.azurefd.net", 33),
            CloudEndpoint::gcp_global_lb("34.111.65.194", 34),
        ];

        let pool = LoadBalancerPool::new("lornu-ai", &endpoints);

        assert_eq!(pool.name, "lornu-ai");
        assert_eq!(pool.origins.len(), 3);
        assert_eq!(pool.minimum_origins, 1);
    }
}
