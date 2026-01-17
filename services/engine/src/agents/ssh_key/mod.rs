//! SSH Key Generation Agent
//!
//! Generates Ed25519 SSH key pairs, stores private keys in GCP Secret Manager,
//! and returns public keys for use as GitHub Deploy Keys.
//!
//! ## Features
//!
//! - Ed25519 key generation (fast, secure, small keys)
//! - Automatic storage in GCP Secret Manager with labels
//! - ADC-based authentication (no hardcoded credentials)
//! - Convenience methods for GitHub Deploy Keys
//!
//! ## Example Usage
//!
//! ```ignore
//! use lornu_engine::agents::ssh_key::{SshKeyAgent, KeyGenerationOptions};
//!
//! // Initialize the agent
//! let agent = SshKeyAgent::new("gcp-lornu-ai").await?;
//!
//! // Generate a key with custom options
//! let options = KeyGenerationOptions::new("deploy@lornu.ai")
//!     .for_deploy_key("lornu-ai/repo", "production");
//! let result = agent.generate("my-deploy-key", options).await?;
//!
//! // Or use the convenience method for deploy keys
//! let result = agent.generate_deploy_key("lornu-ai/repo", "production").await?;
//!
//! println!("Public key: {}", result.public_key);
//! ```

pub mod generator;
pub mod secret_manager;
pub mod types;

#[allow(unused_imports)]
pub use types::{KeyAlgorithm, KeyGenerationOptions, SshKeyError, SshKeyResult};

use generator::generate_ed25519_keypair;
use secret_manager::SecretManagerWriter;
use tracing::{debug, info};

/// SSH Key Generation Agent
///
/// Generates SSH key pairs and stores them securely in GCP Secret Manager.
pub struct SshKeyAgent {
    secret_manager: SecretManagerWriter,
    project_id: String,
}

impl SshKeyAgent {
    /// Create a new SSH Key Agent
    ///
    /// Uses Application Default Credentials (ADC) for authentication.
    pub async fn new(project_id: &str) -> Result<Self, SshKeyError> {
        debug!("Initializing SSH Key Agent for project: {}", project_id);

        let secret_manager = SecretManagerWriter::new(project_id)
            .await
            .map_err(|e| SshKeyError::AuthenticationFailed(e.to_string()))?;

        Ok(Self {
            secret_manager,
            project_id: project_id.to_string(),
        })
    }

    /// Generate an SSH key pair and store it in Secret Manager
    ///
    /// # Arguments
    /// * `secret_name` - Name for the secret in GCP Secret Manager
    /// * `options` - Key generation options (algorithm, comment, labels)
    ///
    /// # Returns
    /// An `SshKeyResult` containing the public key, fingerprint, and secret metadata.
    pub async fn generate(
        &self,
        secret_name: &str,
        options: KeyGenerationOptions,
    ) -> Result<SshKeyResult, SshKeyError> {
        info!(
            secret_name = %secret_name,
            algorithm = %options.algorithm,
            comment = %options.comment,
            "Generating SSH key pair"
        );

        // Generate the key pair
        let keypair = generate_ed25519_keypair(&options.comment)?;

        // Store the private key in Secret Manager (never log the private key!)
        let version = self
            .secret_manager
            .store_private_key(secret_name, &keypair.private_key_pem, options.labels)
            .await?;

        info!(
            fingerprint = %keypair.fingerprint,
            secret_name = %secret_name,
            version = %version,
            "SSH key stored in Secret Manager"
        );

        Ok(SshKeyResult {
            public_key: keypair.public_key_openssh,
            fingerprint: keypair.fingerprint,
            secret_name: secret_name.to_string(),
            secret_version: version,
            algorithm: options.algorithm,
        })
    }

    /// Generate a GitHub Deploy Key
    ///
    /// Convenience method that generates a key with standard Deploy Key labels.
    ///
    /// # Arguments
    /// * `repository` - GitHub repository (e.g., "lornu-ai/repo")
    /// * `environment` - Environment name (e.g., "production", "staging")
    ///
    /// # Returns
    /// An `SshKeyResult` with the public key ready to be added to GitHub.
    pub async fn generate_deploy_key(
        &self,
        repository: &str,
        environment: &str,
    ) -> Result<SshKeyResult, SshKeyError> {
        // Generate secret name from repository and environment
        let secret_name = format!(
            "github-deploy-key-{}-{}",
            repository.replace('/', "-"),
            environment
        );

        // Create options with deploy key labels
        let comment = format!("deploy-{}@{}", environment, repository);
        let options = KeyGenerationOptions::new(comment).for_deploy_key(repository, environment);

        self.generate(&secret_name, options).await
    }

    /// Get the project ID this agent is configured for
    pub fn project_id(&self) -> &str {
        &self.project_id
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_deploy_key_secret_name_generation() {
        let repo = "lornu-ai/private-lornu-ai";
        let env = "production";
        let expected = "github-deploy-key-lornu-ai-private-lornu-ai-production";

        let actual = format!("github-deploy-key-{}-{}", repo.replace('/', "-"), env);
        assert_eq!(actual, expected);
    }
}
