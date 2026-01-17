//! GCP Secret Manager Write Operations
//!
//! Handles storing SSH private keys in Google Secret Manager using
//! Application Default Credentials (ADC).

use crate::agents::ssh_key::types::SshKeyError;
use anyhow::{Context, Result};
use gcloud_sdk::google::cloud::secretmanager::v1::secret_manager_service_client::SecretManagerServiceClient;
use gcloud_sdk::google::cloud::secretmanager::v1::{
    replication, AddSecretVersionRequest, CreateSecretRequest, GetSecretRequest, Replication,
    Secret,
};
use gcloud_sdk::proto_ext::secretmanager::SecretPayload;
use gcloud_sdk::{GoogleApi, GoogleAuthMiddleware};
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// GCP Secret Manager client for write operations
pub struct SecretManagerWriter {
    client: GoogleApi<SecretManagerServiceClient<GoogleAuthMiddleware>>,
    project_id: String,
}

impl SecretManagerWriter {
    /// Create a new Secret Manager writer client
    ///
    /// Uses Application Default Credentials (ADC) for authentication.
    pub async fn new(project_id: &str) -> Result<Self> {
        debug!("Initializing GSM writer client for project: {}", project_id);

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

    /// Check if a secret exists
    pub async fn secret_exists(&self, secret_name: &str) -> Result<bool> {
        let name = format!("projects/{}/secrets/{}", self.project_id, secret_name);

        debug!("Checking if secret exists: {}", name);

        let request = GetSecretRequest { name };

        match self.client.get().get_secret(request).await {
            Ok(_) => {
                debug!("Secret exists");
                Ok(true)
            }
            Err(status) => {
                // Check if the error is a NOT_FOUND gRPC status
                let status_str = status.to_string();
                if status_str.contains("NOT_FOUND") || status_str.contains("not found") {
                    debug!("Secret does not exist");
                    Ok(false)
                } else {
                    Err(anyhow::anyhow!("Failed to check secret: {}", status))
                }
            }
        }
    }

    /// Create a new secret (without a version)
    pub async fn create_secret(
        &self,
        secret_name: &str,
        labels: HashMap<String, String>,
    ) -> Result<()> {
        let parent = format!("projects/{}", self.project_id);

        debug!("Creating secret: {}/{}", parent, secret_name);

        let request = CreateSecretRequest {
            parent,
            secret_id: secret_name.to_string(),
            secret: Some(Secret {
                name: String::new(), // Set by server
                replication: Some(Replication {
                    replication: Some(replication::Replication::Automatic(
                        replication::Automatic {
                            customer_managed_encryption: None,
                        },
                    )),
                }),
                labels,
                ..Default::default()
            }),
        };

        self.client
            .get()
            .create_secret(request)
            .await
            .context("Failed to create secret")?;

        info!("Secret created: {}", secret_name);
        Ok(())
    }

    /// Add a new version to an existing secret
    pub async fn add_secret_version(&self, secret_name: &str, payload: &[u8]) -> Result<String> {
        let parent = format!("projects/{}/secrets/{}", self.project_id, secret_name);

        debug!("Adding version to secret: {}", parent);

        let request = AddSecretVersionRequest {
            parent,
            payload: Some(SecretPayload {
                data: payload.to_vec().into(),
                ..Default::default()
            }),
        };

        let response = self
            .client
            .get()
            .add_secret_version(request)
            .await
            .context("Failed to add secret version")?;

        let version_name = response.into_inner().name;
        // Extract version number from full path
        let version = version_name
            .rsplit('/')
            .next()
            .unwrap_or("unknown")
            .to_string();

        info!(
            secret = %secret_name,
            version = %version,
            "Secret version added"
        );

        Ok(version)
    }

    /// Store a private key in Secret Manager
    ///
    /// Creates the secret if it doesn't exist, then adds a new version.
    /// Returns the version identifier.
    pub async fn store_private_key(
        &self,
        secret_name: &str,
        private_key_pem: &str,
        labels: HashMap<String, String>,
    ) -> Result<String, SshKeyError> {
        // Validate secret name
        if secret_name.is_empty()
            || secret_name.len() > 255
            || !secret_name
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(SshKeyError::InvalidSecretName(format!(
                "Secret name must be 1-255 alphanumeric characters, hyphens, or underscores: {}",
                secret_name
            )));
        }

        // Check if secret exists
        let exists = self
            .secret_exists(secret_name)
            .await
            .map_err(|e| SshKeyError::SecretManagerError(e.to_string()))?;

        if !exists {
            // Create the secret
            self.create_secret(secret_name, labels)
                .await
                .map_err(|e| SshKeyError::SecretManagerError(e.to_string()))?;
        } else {
            warn!(
                secret = %secret_name,
                "Secret already exists, adding new version"
            );
        }

        // Add the private key as a new version
        let version = self
            .add_secret_version(secret_name, private_key_pem.as_bytes())
            .await
            .map_err(|e| SshKeyError::SecretManagerError(e.to_string()))?;

        Ok(version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_secret_name() {
        // Valid names
        assert!(validate_secret_name("my-ssh-key").is_ok());
        assert!(validate_secret_name("deploy_key_123").is_ok());
        assert!(validate_secret_name("MyKey").is_ok());

        // Invalid names
        assert!(validate_secret_name("").is_err());
        assert!(validate_secret_name("key.with.dots").is_err());
        assert!(validate_secret_name("key with spaces").is_err());
    }

    fn validate_secret_name(name: &str) -> Result<(), SshKeyError> {
        if name.is_empty()
            || name.len() > 255
            || !name
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            Err(SshKeyError::InvalidSecretName(name.to_string()))
        } else {
            Ok(())
        }
    }
}
