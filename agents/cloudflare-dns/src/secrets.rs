//! Google Secret Manager Integration
//!
//! Fetches secrets at runtime using Application Default Credentials (ADC).
//! No hardcoded credentials - authentication is handled by:
//! - Local: `gcloud auth application-default login`
//! - GKE: Workload Identity
//! - Cloud Run: Service account

use anyhow::{Context, Result};
use gcloud_sdk::google::cloud::secretmanager::v1::secret_manager_service_client::SecretManagerServiceClient;
use gcloud_sdk::google::cloud::secretmanager::v1::AccessSecretVersionRequest;
use gcloud_sdk::{GoogleApi, GoogleAuthMiddleware};
use tracing::{debug, info};

/// Google Secret Manager client wrapper
pub struct SecretManager {
    client: GoogleApi<SecretManagerServiceClient<GoogleAuthMiddleware>>,
    project_id: String,
}

impl SecretManager {
    /// Create a new Secret Manager client
    ///
    /// Uses Application Default Credentials (ADC) for authentication.
    pub async fn new(project_id: &str) -> Result<Self> {
        debug!("Initializing GSM client for project: {}", project_id);

        let client = GoogleApi::from_function(
            SecretManagerServiceClient::new,
            "https://secretmanager.googleapis.com",
            None,
        )
        .await
        .context("Failed to initialize GSM client")?;

        Ok(Self {
            client,
            project_id: project_id.to_string(),
        })
    }

    /// Fetch a secret value by name
    ///
    /// Retrieves the latest version of the secret.
    pub async fn get_secret(&self, secret_name: &str) -> Result<String> {
        let name = format!(
            "projects/{}/secrets/{}/versions/latest",
            self.project_id, secret_name
        );

        debug!("Fetching secret: {}", name);

        let request = AccessSecretVersionRequest { name };

        let response = self
            .client
            .get()
            .access_secret_version(request)
            .await
            .context("Failed to access secret version")?;

        let payload = response
            .into_inner()
            .payload
            .context("Secret has no payload")?;

        // Access sensitive string value
        let secret_value = payload.data.as_sensitive_str().to_string();

        info!("Secret retrieved successfully (length: {} bytes)", secret_value.len());

        Ok(secret_value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires GCP credentials
    async fn test_secret_manager() {
        let sm = SecretManager::new("gcp-lornu-ai").await.unwrap();
        let secret = sm.get_secret("cloudflare-api-token").await;
        assert!(secret.is_ok());
    }
}
