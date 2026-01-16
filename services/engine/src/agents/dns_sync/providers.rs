//! Cloud Provider Adapters
//!
//! Trait-based abstractions for discovering and querying endpoints
//! across AWS, Azure, and GCP.

use anyhow::{Context, Result};
use async_trait::async_trait;
use kube::{Api, Client};
use serde::de::DeserializeOwned;
use tracing::{info, warn};

use super::types::{CloudEndpoint, CloudProvider, HealthStatus};

/// Trait for cloud provider endpoint discovery
#[async_trait]
pub trait CloudProviderAdapter: Send + Sync {
    /// Get the provider type
    fn provider(&self) -> CloudProvider;

    /// Discover all endpoints from this provider
    async fn discover_endpoints(&self) -> Result<Vec<CloudEndpoint>>;

    /// Check health of a specific endpoint
    async fn check_health(&self, endpoint: &CloudEndpoint) -> Result<HealthStatus>;
}

/// AWS Provider - discovers ALB/NLB endpoints via Crossplane
pub struct AwsProvider {
    k8s_client: Client,
    namespace: String,
}

impl AwsProvider {
    pub async fn new(namespace: &str) -> Result<Self> {
        let k8s_client = Client::try_default()
            .await
            .context("Failed to create K8s client for AWS provider")?;

        Ok(Self {
            k8s_client,
            namespace: namespace.to_string(),
        })
    }

    /// Query Crossplane AWS managed resources
    async fn query_crossplane_resources<T: DeserializeOwned + Clone>(
        &self,
        group: &str,
        version: &str,
        plural: &str,
    ) -> Result<Vec<T>> {
        let api: Api<kube::core::DynamicObject> = Api::namespaced_with(
            self.k8s_client.clone(),
            &self.namespace,
            &kube::discovery::ApiResource {
                group: group.to_string(),
                version: version.to_string(),
                api_version: format!("{}/{}", group, version),
                kind: String::new(),
                plural: plural.to_string(),
            },
        );

        let list = api.list(&Default::default()).await?;
        let results: Vec<T> = list
            .items
            .into_iter()
            .filter_map(|obj| serde_json::from_value(serde_json::to_value(obj).ok()?).ok())
            .collect();

        Ok(results)
    }
}

#[async_trait]
impl CloudProviderAdapter for AwsProvider {
    fn provider(&self) -> CloudProvider {
        CloudProvider::Aws
    }

    async fn discover_endpoints(&self) -> Result<Vec<CloudEndpoint>> {
        let mut endpoints = Vec::new();

        // Query Crossplane AWS LB resources
        // These are typically created via Upbound provider-aws
        let lbs: Vec<serde_json::Value> = self
            .query_crossplane_resources("elbv2.aws.upbound.io", "v1beta1", "lbs")
            .await
            .unwrap_or_default();

        for lb in lbs {
            if let Some(status) = lb.get("status").and_then(|s| s.get("atProvider")) {
                if let Some(dns_name) = status.get("dnsName").and_then(|d| d.as_str()) {
                    let region = lb
                        .get("spec")
                        .and_then(|s| s.get("forProvider"))
                        .and_then(|f| f.get("region"))
                        .and_then(|r| r.as_str())
                        .unwrap_or("us-east-1");

                    let name = lb
                        .get("metadata")
                        .and_then(|m| m.get("name"))
                        .and_then(|n| n.as_str())
                        .unwrap_or("unknown");

                    info!("Discovered AWS ALB: {} -> {}", name, dns_name);
                    endpoints.push(CloudEndpoint::aws_alb(dns_name, region, 33));
                }
            }
        }

        Ok(endpoints)
    }

    async fn check_health(&self, endpoint: &CloudEndpoint) -> Result<HealthStatus> {
        // Query AWS ALB target group health via Crossplane status
        // For now, return Unknown and let Cloudflare health checks determine actual status
        Ok(HealthStatus::Unknown)
    }
}

/// Azure Provider - discovers Front Door/Public IP endpoints via Crossplane
pub struct AzureProvider {
    k8s_client: Client,
    namespace: String,
}

impl AzureProvider {
    pub async fn new(namespace: &str) -> Result<Self> {
        let k8s_client = Client::try_default()
            .await
            .context("Failed to create K8s client for Azure provider")?;

        Ok(Self {
            k8s_client,
            namespace: namespace.to_string(),
        })
    }
}

#[async_trait]
impl CloudProviderAdapter for AzureProvider {
    fn provider(&self) -> CloudProvider {
        CloudProvider::Azure
    }

    async fn discover_endpoints(&self) -> Result<Vec<CloudEndpoint>> {
        let mut endpoints = Vec::new();

        // Query Crossplane Azure Front Door resources
        let api: Api<kube::core::DynamicObject> = Api::all_with(
            self.k8s_client.clone(),
            &kube::discovery::ApiResource {
                group: "cdn.azure.upbound.io".to_string(),
                version: "v1beta1".to_string(),
                api_version: "cdn.azure.upbound.io/v1beta1".to_string(),
                kind: "FrontdoorEndpoint".to_string(),
                plural: "frontdoorendpoints".to_string(),
            },
        );

        if let Ok(list) = api.list(&Default::default()).await {
            for fd in list.items {
                if let Some(status) = fd.data.get("status").and_then(|s| s.get("atProvider")) {
                    if let Some(hostname) = status.get("hostName").and_then(|h| h.as_str()) {
                        let name = fd
                            .metadata
                            .name
                            .as_deref()
                            .unwrap_or("unknown");

                        info!("Discovered Azure Front Door: {} -> {}", name, hostname);
                        endpoints.push(CloudEndpoint::azure_front_door(hostname, 33));
                    }
                }
            }
        }

        // Also check for Public IPs
        let public_ip_api: Api<kube::core::DynamicObject> = Api::all_with(
            self.k8s_client.clone(),
            &kube::discovery::ApiResource {
                group: "network.azure.upbound.io".to_string(),
                version: "v1beta1".to_string(),
                api_version: "network.azure.upbound.io/v1beta1".to_string(),
                kind: "PublicIP".to_string(),
                plural: "publicips".to_string(),
            },
        );

        if let Ok(list) = public_ip_api.list(&Default::default()).await {
            for pip in list.items {
                if let Some(status) = pip.data.get("status").and_then(|s| s.get("atProvider")) {
                    if let Some(ip) = status.get("ipAddress").and_then(|i| i.as_str()) {
                        let name = pip.metadata.name.as_deref().unwrap_or("unknown");
                        info!("Discovered Azure Public IP: {} -> {}", name, ip);
                        
                        endpoints.push(CloudEndpoint {
                            provider: CloudProvider::Azure,
                            address: ip.to_string(),
                            weight: 33,
                            enabled: true,
                            region: status
                                .get("location")
                                .and_then(|l| l.as_str())
                                .map(|s| s.to_string()),
                            health: HealthStatus::Unknown,
                            endpoint_type: "public_ip".to_string(),
                        });
                    }
                }
            }
        }

        Ok(endpoints)
    }

    async fn check_health(&self, endpoint: &CloudEndpoint) -> Result<HealthStatus> {
        Ok(HealthStatus::Unknown)
    }
}

/// GCP Provider - discovers Global Load Balancer endpoints via Crossplane
pub struct GcpProvider {
    k8s_client: Client,
    namespace: String,
}

impl GcpProvider {
    pub async fn new(namespace: &str) -> Result<Self> {
        let k8s_client = Client::try_default()
            .await
            .context("Failed to create K8s client for GCP provider")?;

        Ok(Self {
            k8s_client,
            namespace: namespace.to_string(),
        })
    }
}

#[async_trait]
impl CloudProviderAdapter for GcpProvider {
    fn provider(&self) -> CloudProvider {
        CloudProvider::Gcp
    }

    async fn discover_endpoints(&self) -> Result<Vec<CloudEndpoint>> {
        let mut endpoints = Vec::new();

        // Query Crossplane GCP Global Forwarding Rules
        let api: Api<kube::core::DynamicObject> = Api::all_with(
            self.k8s_client.clone(),
            &kube::discovery::ApiResource {
                group: "compute.gcp.upbound.io".to_string(),
                version: "v1beta1".to_string(),
                api_version: "compute.gcp.upbound.io/v1beta1".to_string(),
                kind: "GlobalForwardingRule".to_string(),
                plural: "globalforwardingrules".to_string(),
            },
        );

        if let Ok(list) = api.list(&Default::default()).await {
            for gfr in list.items {
                if let Some(status) = gfr.data.get("status").and_then(|s| s.get("atProvider")) {
                    if let Some(ip) = status.get("ipAddress").and_then(|i| i.as_str()) {
                        let name = gfr.metadata.name.as_deref().unwrap_or("unknown");
                        info!("Discovered GCP Global LB: {} -> {}", name, ip);
                        endpoints.push(CloudEndpoint::gcp_global_lb(ip, 34));
                    }
                }
            }
        }

        // Also check GKE Ingress resources for GCP-managed LBs
        let ingress_api: Api<k8s_openapi::api::networking::v1::Ingress> =
            Api::all(self.k8s_client.clone());

        if let Ok(list) = ingress_api.list(&Default::default()).await {
            for ingress in list.items {
                // Only process GCE-class ingresses
                let class = ingress
                    .spec
                    .as_ref()
                    .and_then(|s| s.ingress_class_name.as_deref());

                if class == Some("gce") || class == Some("gce-internal") {
                    if let Some(status) = ingress.status.as_ref() {
                        if let Some(lb) = status.load_balancer.as_ref() {
                            for ing in lb.ingress.iter().flatten() {
                                if let Some(ip) = &ing.ip {
                                    let name = ingress
                                        .metadata
                                        .name
                                        .as_deref()
                                        .unwrap_or("unknown");

                                    info!("Discovered GKE Ingress: {} -> {}", name, ip);
                                    endpoints.push(CloudEndpoint::gcp_global_lb(ip, 34));
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(endpoints)
    }

    async fn check_health(&self, endpoint: &CloudEndpoint) -> Result<HealthStatus> {
        Ok(HealthStatus::Unknown)
    }
}

/// Multi-cloud provider aggregator
pub struct MultiCloudProviders {
    providers: Vec<Box<dyn CloudProviderAdapter>>,
}

impl MultiCloudProviders {
    /// Create with all providers enabled
    pub async fn new(namespace: &str) -> Result<Self> {
        let mut providers: Vec<Box<dyn CloudProviderAdapter>> = Vec::new();

        // Try to initialize each provider, log warnings if any fail
        match AwsProvider::new(namespace).await {
            Ok(p) => providers.push(Box::new(p)),
            Err(e) => warn!("Failed to initialize AWS provider: {}", e),
        }

        match AzureProvider::new(namespace).await {
            Ok(p) => providers.push(Box::new(p)),
            Err(e) => warn!("Failed to initialize Azure provider: {}", e),
        }

        match GcpProvider::new(namespace).await {
            Ok(p) => providers.push(Box::new(p)),
            Err(e) => warn!("Failed to initialize GCP provider: {}", e),
        }

        if providers.is_empty() {
            anyhow::bail!("No cloud providers could be initialized");
        }

        info!("Initialized {} cloud providers", providers.len());
        Ok(Self { providers })
    }

    /// Discover endpoints from all providers
    pub async fn discover_all_endpoints(&self) -> Result<Vec<CloudEndpoint>> {
        let mut all_endpoints = Vec::new();

        for provider in &self.providers {
            match provider.discover_endpoints().await {
                Ok(endpoints) => {
                    info!(
                        "Discovered {} endpoints from {}",
                        endpoints.len(),
                        provider.provider()
                    );
                    all_endpoints.extend(endpoints);
                }
                Err(e) => {
                    warn!(
                        "Failed to discover endpoints from {}: {}",
                        provider.provider(),
                        e
                    );
                }
            }
        }

        Ok(all_endpoints)
    }

    /// Check health of all endpoints
    pub async fn check_all_health(
        &self,
        endpoints: &mut [CloudEndpoint],
    ) -> Result<()> {
        for endpoint in endpoints.iter_mut() {
            let provider = self
                .providers
                .iter()
                .find(|p| p.provider() == endpoint.provider);

            if let Some(provider) = provider {
                match provider.check_health(endpoint).await {
                    Ok(status) => endpoint.health = status,
                    Err(e) => {
                        warn!(
                            "Failed to check health for {}: {}",
                            endpoint.address, e
                        );
                        endpoint.health = HealthStatus::Unknown;
                    }
                }
            }
        }

        Ok(())
    }
}
