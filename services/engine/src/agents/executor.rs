//! Crossplane Executor
//!
//! Creates Crossplane Claims at runtime via the Kubernetes API.
//! This allows agents to dynamically provision infrastructure without YAML.

use anyhow::{Context, Result};
use kube::{
    api::{Api, DynamicObject, ListParams, PostParams},
    discovery::ApiResource,
    Client, ResourceExt,
};
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;
use tracing::info;

const NAMESPACE: &str = "lornu-ai";

/// Executor for creating Crossplane resources at runtime.
pub struct CrossplaneExecutor {
    client: Client,
}

impl CrossplaneExecutor {
    /// Create a new executor with in-cluster credentials.
    pub async fn new() -> Result<Self> {
        let client = Client::try_default()
            .await
            .context("Failed to create Kubernetes client")?;

        info!("CrossplaneExecutor initialized");
        Ok(Self { client })
    }

    /// Provision an AgentMemory claim for an agent.
    pub async fn provision_agent_memory(
        &self,
        user: &str,
        agent: &str,
        provider: &str,
        memory_type: &str,
        size: &str,
    ) -> Result<String> {
        let name = format!("mem-{}-{}", user, agent);
        info!("Provisioning AgentMemory: {}", name);

        let claim = json!({
            "apiVersion": "lornu.ai/v1alpha1",
            "kind": "AgentMemory",
            "metadata": {
                "name": &name,
                "namespace": NAMESPACE,
                "labels": {
                    "lornu.ai/user": user,
                    "lornu.ai/agent": agent,
                    "lornu.ai/managed-by": "engine"
                }
            },
            "spec": {
                "provider": provider,
                "type": memory_type,
                "size": size,
                "tier": "small"
            }
        });

        self.create_claim("agentmemories", &name, claim).await?;
        self.wait_for_ready("agentmemories", &name).await?;

        Ok(name)
    }

    /// Provision an AgentWorker claim for compute.
    pub async fn provision_agent_worker(
        &self,
        user: &str,
        agent: &str,
        gpu: bool,
        gpu_type: Option<&str>,
        replicas: i32,
    ) -> Result<String> {
        let name = format!("worker-{}-{}", user, agent);
        info!("Provisioning AgentWorker: {}", name);

        let mut spec = json!({
            "provider": "gcp",
            "gpu": gpu,
            "replicas": replicas,
            "timeout": "30m"
        });

        if let Some(gt) = gpu_type {
            spec["gpuType"] = json!(gt);
        }

        let claim = json!({
            "apiVersion": "lornu.ai/v1alpha1",
            "kind": "AgentWorker",
            "metadata": {
                "name": &name,
                "namespace": NAMESPACE,
                "labels": {
                    "lornu.ai/user": user,
                    "lornu.ai/agent": agent,
                    "lornu.ai/managed-by": "engine"
                }
            },
            "spec": spec
        });

        self.create_claim("agentworkers", &name, claim).await?;
        self.wait_for_ready("agentworkers", &name).await?;

        Ok(name)
    }

    /// Create a Crossplane claim using dynamic API.
    async fn create_claim(
        &self,
        resource: &str,
        name: &str,
        claim: serde_json::Value,
    ) -> Result<()> {
        let ar = ApiResource {
            group: "lornu.ai".to_string(),
            version: "v1alpha1".to_string(),
            api_version: "lornu.ai/v1alpha1".to_string(),
            kind: resource.to_string(),
            plural: resource.to_string(),
        };
        let api: Api<DynamicObject> = Api::namespaced_with(self.client.clone(), NAMESPACE, &ar);

        if api.get_opt(name).await?.is_some() {
            info!("Claim {} already exists, skipping creation", name);
            return Ok(());
        }

        let obj: DynamicObject = serde_json::from_value(claim)?;
        api.create(&PostParams::default(), &obj).await?;

        info!("Created claim: {}", name);
        Ok(())
    }

    /// Wait for a claim to become ready.
    async fn wait_for_ready(&self, resource: &str, name: &str) -> Result<()> {
        let ar = ApiResource {
            group: "lornu.ai".to_string(),
            version: "v1alpha1".to_string(),
            api_version: "lornu.ai/v1alpha1".to_string(),
            kind: resource.to_string(),
            plural: resource.to_string(),
        };
        let api: Api<DynamicObject> = Api::namespaced_with(self.client.clone(), NAMESPACE, &ar);

        for attempt in 0..60 {
            let obj = api.get(name).await?;

            if let Some(status) = obj.data.get("status") {
                if let Some(conditions) = status.get("conditions") {
                    if let Some(conds) = conditions.as_array() {
                        let ready = conds.iter().any(|c| {
                            c.get("type").and_then(|t| t.as_str()) == Some("Ready")
                                && c.get("status").and_then(|s| s.as_str()) == Some("True")
                        });

                        if ready {
                            info!("Claim {} is Ready", name);
                            return Ok(());
                        }
                    }
                }
            }

            if attempt % 10 == 0 {
                info!("Waiting for {} to be ready (attempt {})", name, attempt);
            }

            sleep(Duration::from_secs(5)).await;
        }

        anyhow::bail!("Timed out waiting for {} to be Ready", name);
    }

    /// List all agent resources.
    pub async fn list_agent_resources(&self) -> Result<Vec<serde_json::Value>> {
        let mut resources = Vec::new();

        for (kind, plural) in [("AgentMemory", "agentmemories"), ("AgentWorker", "agentworkers")] {
            let ar = ApiResource {
                group: "lornu.ai".to_string(),
                version: "v1alpha1".to_string(),
                api_version: "lornu.ai/v1alpha1".to_string(),
                kind: kind.to_string(),
                plural: plural.to_string(),
            };
            let api: Api<DynamicObject> = Api::namespaced_with(self.client.clone(), NAMESPACE, &ar);

            if let Ok(list) = api.list(&ListParams::default()).await {
                for item in list {
                    resources.push(json!({
                        "kind": kind,
                        "name": item.name_any(),
                        "status": item.data.get("status")
                    }));
                }
            }
        }

        Ok(resources)
    }

    /// Delete an agent resource claim.
    pub async fn delete_claim(&self, resource: &str, name: &str) -> Result<()> {
        let ar = ApiResource {
            group: "lornu.ai".to_string(),
            version: "v1alpha1".to_string(),
            api_version: "lornu.ai/v1alpha1".to_string(),
            kind: resource.to_string(),
            plural: resource.to_string(),
        };
        let api: Api<DynamicObject> = Api::namespaced_with(self.client.clone(), NAMESPACE, &ar);

        api.delete(name, &Default::default()).await?;
        info!("Deleted claim: {}", name);

        Ok(())
    }
}
