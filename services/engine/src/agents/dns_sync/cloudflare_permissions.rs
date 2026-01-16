//! Cloudflare API Permission Group IDs
//!
//! These IDs are PUBLIC and the same for all Cloudflare accounts.
//! They define what permissions a token has, NOT what resources it can access.
//!
//! SENSITIVE values (account ID, zone ID, tokens) must NEVER be hardcoded.
//! They should be loaded from:
//! - GCP Secret Manager (via `CloudflareConfig::from_secret_manager()`)
//! - Environment variables (via `CloudflareConfig::from_env()`)
//! - Kubernetes Secrets (injected as env vars)

use serde::{Deserialize, Serialize};
use std::env;
use anyhow::{Context, Result};

/// Cloudflare Permission Group IDs (PUBLIC - same for all accounts)
pub mod permission_groups {
    // ============================================================
    // Zone Permissions
    // ============================================================
    
    /// Read zone details
    pub const ZONE_READ: &str = "c8fed203ed3043cba015a93ad1616f1f";
    /// Write zone settings
    pub const ZONE_WRITE: &str = "e6d2666161e84845a636613608cee8d5";
    /// Read zone settings
    pub const ZONE_SETTINGS_READ: &str = "517b21aee92c4d89936c976ba6e4be55";
    /// Write zone settings
    pub const ZONE_SETTINGS_WRITE: &str = "3030687196b94b638145a3953da2b699";
    
    // ============================================================
    // DNS Permissions
    // ============================================================
    
    /// Create/update/delete DNS records
    pub const DNS_WRITE: &str = "4755a26eedb94da69e1066d98aa820be";
    /// Read DNS firewall settings
    pub const DNS_FIREWALL_READ: &str = "5f48a472240a4b489a21d43bd19a06e1";
    /// Write DNS firewall settings
    pub const DNS_FIREWALL_WRITE: &str = "da6d2d6f2ec8442eaadda60d13f42bca";
    /// Write account-level DNS settings
    pub const ACCOUNT_DNS_SETTINGS_WRITE: &str = "dc44f27f48ab405392a5f69fe822bd01";
    
    // ============================================================
    // Load Balancer Permissions
    // ============================================================
    
    /// Read load balancer configurations
    pub const LOAD_BALANCERS_READ: &str = "e9a975f628014f1d85b723993116f7d5";
    /// Write load balancer configurations
    pub const LOAD_BALANCERS_WRITE: &str = "6d7f2f5f5b1d4a0e9081fdc98d432fd1";
    /// Read load balancer monitors and pools
    pub const LB_MONITORS_POOLS_READ: &str = "9d24387c6e8544e2bc4024a03991339f";
    /// Write load balancer monitors and pools
    pub const LB_MONITORS_POOLS_WRITE: &str = "d2a1802cc9a34e30852f8b33869b2f3c";
    
    // ============================================================
    // Account Permissions
    // ============================================================
    
    /// Create/manage API tokens (MASTER TOKEN ONLY)
    pub const ACCOUNT_API_TOKENS_WRITE: &str = "5bc3f8b21c554832afc660159ab75fa4";
    /// Read account settings
    pub const ACCOUNT_SETTINGS_READ: &str = "c1fde68c7bcc44588cbb6ddbc16d6480";
    /// Write SSL certificates
    pub const ACCOUNT_SSL_CERTS_WRITE: &str = "db37e5f1cb1a4e1aabaef8deaea43575";
    
    // ============================================================
    // Cache & Workers Permissions
    // ============================================================
    
    /// Purge cache
    pub const CACHE_PURGE: &str = "e17beae8b8cb423a99b1730f21238bed";
    /// Write cache settings
    pub const CACHE_SETTINGS_WRITE: &str = "9ff81cbbe65c400b97d92c3c1033cab6";
    /// Write Workers routes
    pub const WORKERS_ROUTES_WRITE: &str = "28f4b596e7d643029c524985477ae49a";
}

/// Cloudflare configuration loaded from secure sources
/// 
/// NEVER hardcode these values. Always load from:
/// - GCP Secret Manager
/// - AWS Secrets Manager  
/// - Environment variables (in K8s, from ExternalSecrets)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudflareConfig {
    /// Cloudflare Account ID (SENSITIVE)
    pub account_id: String,
    /// Cloudflare Zone ID for lornu.ai (SENSITIVE)
    pub zone_id: String,
    /// API Token with DNS/LB permissions (SENSITIVE)
    pub api_token: String,
    /// Master token for creating new tokens (SENSITIVE, optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub master_token: Option<String>,
}

impl CloudflareConfig {
    /// Load configuration from environment variables
    /// 
    /// Expected env vars (injected by ExternalSecrets):
    /// - CLOUDFLARE_ACCOUNT_ID
    /// - CLOUDFLARE_ZONE_ID  
    /// - CLOUDFLARE_API_TOKEN
    /// - CLOUDFLARE_MASTER_TOKEN (optional)
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            account_id: env::var("CLOUDFLARE_ACCOUNT_ID")
                .context("CLOUDFLARE_ACCOUNT_ID not set")?,
            zone_id: env::var("CLOUDFLARE_ZONE_ID")
                .context("CLOUDFLARE_ZONE_ID not set")?,
            api_token: env::var("CLOUDFLARE_API_TOKEN")
                .context("CLOUDFLARE_API_TOKEN not set")?,
            master_token: env::var("CLOUDFLARE_MASTER_TOKEN").ok(),
        })
    }
    
    /// Load configuration from GCP Secret Manager
    /// 
    /// Uses the `gcloud` CLI or Google Cloud SDK to access secrets.
    /// In GKE with Workload Identity, this uses ADC automatically.
    /// 
    /// Secrets should be stored as:
    /// - projects/{project}/secrets/CLOUDFLARE_ACCOUNT_ID
    /// - projects/{project}/secrets/CLOUDFLARE_ZONE_ID
    /// - projects/{project}/secrets/CLOUDFLARE_API_TOKEN
    pub async fn from_secret_manager(project_id: &str) -> Result<Self> {
        use tokio::process::Command;
        
        async fn get_secret(project: &str, name: &str) -> Result<String> {
            let output = Command::new("gcloud")
                .args([
                    "secrets", "versions", "access", "latest",
                    "--secret", name,
                    "--project", project,
                ])
                .output()
                .await
                .context("Failed to run gcloud")?;
            
            if !output.status.success() {
                anyhow::bail!("Failed to access secret {}: {}", 
                    name, String::from_utf8_lossy(&output.stderr));
            }
            
            String::from_utf8(output.stdout)
                .context("Secret is not valid UTF-8")
                .map(|s| s.trim().to_string())
        }
        
        Ok(Self {
            account_id: get_secret(project_id, "CLOUDFLARE_ACCOUNT_ID").await?,
            zone_id: get_secret(project_id, "CLOUDFLARE_ZONE_ID").await?,
            api_token: get_secret(project_id, "CLOUDFLARE_API_TOKEN").await?,
            master_token: get_secret(project_id, "CLOUDFLARE_MASTER_TOKEN").await.ok(),
        })
    }
}

/// Token policy for creating scoped API tokens
#[derive(Debug, Serialize)]
pub struct TokenPolicy {
    pub effect: &'static str,
    pub resources: std::collections::HashMap<String, String>,
    pub permission_groups: Vec<PermissionGroup>,
}

#[derive(Debug, Serialize)]
pub struct PermissionGroup {
    pub id: &'static str,
}

impl TokenPolicy {
    /// Create a policy for DNS sync agent (read zone, write DNS)
    pub fn dns_sync(zone_id: &str) -> Self {
        let mut resources = std::collections::HashMap::new();
        resources.insert(
            format!("com.cloudflare.api.account.zone.{}", zone_id),
            "*".to_string(),
        );
        
        Self {
            effect: "allow",
            resources,
            permission_groups: vec![
                PermissionGroup { id: permission_groups::ZONE_READ },
                PermissionGroup { id: permission_groups::DNS_WRITE },
            ],
        }
    }
    
    /// Create a policy for load balancer management
    pub fn load_balancer(account_id: &str, zone_id: &str) -> Vec<Self> {
        let mut zone_resources = std::collections::HashMap::new();
        zone_resources.insert(
            format!("com.cloudflare.api.account.zone.{}", zone_id),
            "*".to_string(),
        );
        
        let mut account_resources = std::collections::HashMap::new();
        account_resources.insert(
            format!("com.cloudflare.api.account.{}", account_id),
            "*".to_string(),
        );
        
        vec![
            Self {
                effect: "allow",
                resources: zone_resources,
                permission_groups: vec![
                    PermissionGroup { id: permission_groups::ZONE_READ },
                    PermissionGroup { id: permission_groups::DNS_WRITE },
                    PermissionGroup { id: permission_groups::LOAD_BALANCERS_WRITE },
                ],
            },
            Self {
                effect: "allow",
                resources: account_resources,
                permission_groups: vec![
                    PermissionGroup { id: permission_groups::LB_MONITORS_POOLS_WRITE },
                ],
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_permission_ids_are_valid_hex() {
        // Permission IDs should be 32-char hex strings
        let ids = [
            permission_groups::ZONE_READ,
            permission_groups::DNS_WRITE,
            permission_groups::LB_MONITORS_POOLS_WRITE,
        ];
        
        for id in ids {
            assert_eq!(id.len(), 32, "Permission ID should be 32 chars");
            assert!(id.chars().all(|c| c.is_ascii_hexdigit()), 
                "Permission ID should be hex");
        }
    }
    
    #[test]
    fn test_dns_sync_policy() {
        let policy = TokenPolicy::dns_sync("test-zone-id");
        assert_eq!(policy.effect, "allow");
        assert_eq!(policy.permission_groups.len(), 2);
    }
}
