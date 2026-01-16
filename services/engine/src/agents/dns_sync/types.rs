//! Multi-Cloud DNS Sync Types
//!
//! Core types for representing cloud endpoints and sync operations.

use serde::{Deserialize, Serialize};

/// Cloud provider identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CloudProvider {
    Aws,
    Azure,
    Gcp,
}

impl std::fmt::Display for CloudProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CloudProvider::Aws => write!(f, "aws"),
            CloudProvider::Azure => write!(f, "azure"),
            CloudProvider::Gcp => write!(f, "gcp"),
        }
    }
}

/// Health status of a cloud endpoint
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// A cloud endpoint representing a load balancer or public IP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudEndpoint {
    /// Cloud provider (aws, azure, gcp)
    pub provider: CloudProvider,
    /// Hostname or IP address of the endpoint
    pub address: String,
    /// Traffic weight for load balancing (0-100)
    pub weight: u32,
    /// Whether this endpoint is enabled
    pub enabled: bool,
    /// Region/location of the endpoint
    pub region: Option<String>,
    /// Current health status
    pub health: HealthStatus,
    /// Endpoint type (alb, nlb, front_door, global_lb, etc.)
    pub endpoint_type: String,
}

impl CloudEndpoint {
    /// Create a new AWS ALB endpoint
    pub fn aws_alb(hostname: &str, region: &str, weight: u32) -> Self {
        Self {
            provider: CloudProvider::Aws,
            address: hostname.to_string(),
            weight,
            enabled: true,
            region: Some(region.to_string()),
            health: HealthStatus::Unknown,
            endpoint_type: "alb".to_string(),
        }
    }

    /// Create a new Azure Front Door endpoint
    pub fn azure_front_door(hostname: &str, weight: u32) -> Self {
        Self {
            provider: CloudProvider::Azure,
            address: hostname.to_string(),
            weight,
            enabled: true,
            region: None, // Front Door is global
            health: HealthStatus::Unknown,
            endpoint_type: "front_door".to_string(),
        }
    }

    /// Create a new GCP Global Load Balancer endpoint
    pub fn gcp_global_lb(ip: &str, weight: u32) -> Self {
        Self {
            provider: CloudProvider::Gcp,
            address: ip.to_string(),
            weight,
            enabled: true,
            region: None, // Global LB is global
            health: HealthStatus::Unknown,
            endpoint_type: "global_lb".to_string(),
        }
    }
}

/// Cloudflare Load Balancer Pool origin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolOrigin {
    /// Origin name (e.g., "aws-origin", "gcp-origin")
    pub name: String,
    /// Origin address (hostname or IP)
    pub address: String,
    /// Traffic weight (0.0 to 1.0)
    pub weight: f64,
    /// Whether origin is enabled
    pub enabled: bool,
    /// Optional headers to add
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<serde_json::Value>,
}

impl From<&CloudEndpoint> for PoolOrigin {
    fn from(endpoint: &CloudEndpoint) -> Self {
        Self {
            name: format!("{}-origin", endpoint.provider),
            address: endpoint.address.clone(),
            weight: endpoint.weight as f64 / 100.0,
            enabled: endpoint.enabled && endpoint.health != HealthStatus::Unhealthy,
            header: None,
        }
    }
}

/// Cloudflare Load Balancer Pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancerPool {
    /// Pool name
    pub name: String,
    /// Pool description
    pub description: Option<String>,
    /// Origins in this pool
    pub origins: Vec<PoolOrigin>,
    /// Health monitor ID
    pub monitor: Option<String>,
    /// Minimum number of healthy origins
    pub minimum_origins: u32,
    /// Check regions for health monitoring
    pub check_regions: Vec<String>,
    /// Notification email
    pub notification_email: Option<String>,
}

impl LoadBalancerPool {
    /// Create a new multi-cloud load balancer pool
    pub fn new(name: &str, endpoints: &[CloudEndpoint]) -> Self {
        Self {
            name: name.to_string(),
            description: Some("Multi-cloud load balancer pool managed by Lornu AI".to_string()),
            origins: endpoints.iter().map(PoolOrigin::from).collect(),
            monitor: None,
            minimum_origins: 1,
            check_regions: vec![
                "WNAM".to_string(), // Western North America
                "ENAM".to_string(), // Eastern North America
                "WEU".to_string(),  // Western Europe
            ],
            notification_email: None,
        }
    }
}

/// Result of a DNS sync operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsSyncResult {
    /// Whether the sync was successful
    pub success: bool,
    /// Pool ID if created/updated
    pub pool_id: Option<String>,
    /// Number of origins synced
    pub origins_synced: usize,
    /// Any errors encountered
    pub errors: Vec<String>,
    /// Timestamp of the sync
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Configuration for multi-cloud DNS sync
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiCloudConfig {
    /// Cloudflare Zone ID
    pub cloudflare_zone_id: String,
    /// Pool name prefix
    pub pool_name_prefix: String,
    /// Health check configuration
    pub health_check: HealthCheckConfig,
    /// Failover strategy
    pub failover_strategy: FailoverStrategy,
}

/// Health check configuration for the pool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Health check path (e.g., "/healthz")
    pub path: String,
    /// Expected status codes
    pub expected_codes: String,
    /// Check interval in seconds
    pub interval: u32,
    /// Timeout in seconds
    pub timeout: u32,
    /// Retries before marking unhealthy
    pub retries: u32,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            path: "/healthz".to_string(),
            expected_codes: "200".to_string(),
            interval: 60,
            timeout: 5,
            retries: 2,
        }
    }
}

/// Failover strategy for multi-cloud routing
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailoverStrategy {
    /// Route all traffic to highest priority healthy origin
    Failover,
    /// Distribute traffic based on weights
    WeightedRoundRobin,
    /// Route to geographically closest origin
    GeoProximity,
    /// Route based on latency measurements
    LatencyBased,
}

impl Default for FailoverStrategy {
    fn default() -> Self {
        Self::Failover
    }
}
