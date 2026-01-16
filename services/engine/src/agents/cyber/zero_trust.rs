use crate::agents::cyber::remediator::Remediator;
use crate::agents::cyber::types::{IamCorrection, IamInsight, ServiceAccountInfo};
use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::Value;
use std::env;
use tracing::info;

pub struct ZeroTrustAgent {
    project_id: String,
    http_client: Client,
    threshold_days: u32,
    remediator: Remediator,
}

impl ZeroTrustAgent {
    pub fn new(threshold_days: u32) -> Result<Self> {
        let project_id = env::var("LORNU_GCP_PROJECT").context("LORNU_GCP_PROJECT must be set")?;

        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        info!("ZeroTrustAgent initialized (project: {})", project_id);

        Ok(Self {
            project_id,
            http_client,
            threshold_days,
            remediator: Remediator::new(),
        })
    }
    // ...
    pub async fn run_hardening_pass(&self) -> Result<Vec<IamCorrection>> {
        info!(
            "🛡️ Initiating IAM Hardening Pass (Threshold: {} days)...",
            self.threshold_days
        );
        let token = self.get_access_token().await?;
        let service_accounts = self.fetch_active_service_accounts(&token).await?;
        let mut corrections = Vec::new();

        for sa in service_accounts {
            let insights = self.get_iam_insights(&token, &sa.email).await?;
            if let Some(lean_policy) = self.calculate_lean_policy(insights) {
                let correction = IamCorrection {
                    sa_email: sa.email,
                    old_role: sa.current_role,
                    new_role: lean_policy,
                };
                self.remediator.propose_correction(&correction).await?;
                corrections.push(correction);
            }
        }
        Ok(corrections)
    }

    async fn fetch_active_service_accounts(&self, token: &str) -> Result<Vec<ServiceAccountInfo>> {
        let url = format!(
            "https://iam.googleapis.com/v1/projects/{}/serviceAccounts",
            self.project_id
        );
        let resp = self.http_client.get(&url).bearer_auth(token).send().await?;

        if !resp.status().is_success() {
            anyhow::bail!("Failed to fetch service accounts: {}", resp.status());
        }

        let json: Value = resp.json().await?;
        let accounts = json["accounts"]
            .as_array()
            .map(|v| v.clone())
            .unwrap_or_default();

        let mut result = Vec::new();
        for account in accounts {
            let email = account["email"].as_str().unwrap_or_default().to_string();
            // Placeholder for role fetching logic
            result.push(ServiceAccountInfo {
                email,
                current_role: "roles/editor".to_string(),
                last_authenticated_at: None,
            });
        }
        Ok(result)
    }

    async fn get_iam_insights(&self, _token: &str, email: &str) -> Result<IamInsight> {
        // Mocking the Recommender API call
        let mut unused = Vec::new();
        // Simulate finding unused permissions for demo purposes
        if email.contains("deploy") {
            unused.push("compute.instances.delete".to_string());
        }

        Ok(IamInsight {
            email: email.to_string(),
            description: "Usage analysis".to_string(),
            recommended_role: "roles/storage.objectViewer".to_string(),
            unused_permissions: unused,
        })
    }

    fn calculate_lean_policy(&self, insights: IamInsight) -> Option<String> {
        if insights.has_unused_permissions() {
            Some(insights.recommended_role)
        } else {
            None
        }
    }
}
