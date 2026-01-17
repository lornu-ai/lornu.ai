//! Types for SSH key generation and management
//!
//! This module contains types used by the SSH Key Agent for generating
//! Ed25519 SSH key pairs and storing them in GCP Secret Manager.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Supported SSH key algorithms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum KeyAlgorithm {
    /// Ed25519 (recommended - fast, secure, small keys)
    #[default]
    Ed25519,
}

impl std::fmt::Display for KeyAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeyAlgorithm::Ed25519 => write!(f, "ed25519"),
        }
    }
}

/// Options for SSH key generation
#[derive(Debug, Clone)]
pub struct KeyGenerationOptions {
    /// Key algorithm (default: Ed25519)
    pub algorithm: KeyAlgorithm,
    /// Comment to embed in the public key (e.g., "deploy@lornu.ai")
    pub comment: String,
    /// Labels to apply to the secret in GCP Secret Manager
    pub labels: HashMap<String, String>,
}

impl KeyGenerationOptions {
    /// Create new options with the given comment
    pub fn new(comment: impl Into<String>) -> Self {
        Self {
            algorithm: KeyAlgorithm::Ed25519,
            comment: comment.into(),
            labels: HashMap::new(),
        }
    }

    /// Set the key algorithm
    pub fn algorithm(mut self, algorithm: KeyAlgorithm) -> Self {
        self.algorithm = algorithm;
        self
    }

    /// Add a label to the secret
    pub fn label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.labels.insert(key.into(), value.into());
        self
    }

    /// Add standard labels for managed keys
    pub fn with_standard_labels(self) -> Self {
        self.label("managed-by", "ssh-key-agent")
    }

    /// Add labels for a GitHub Deploy Key
    pub fn for_deploy_key(self, repository: &str, environment: &str) -> Self {
        self.label("purpose", "github-deploy-key")
            .label("repository", repository)
            .label("environment", environment)
            .with_standard_labels()
    }
}

/// Result of SSH key generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshKeyResult {
    /// The public key in OpenSSH format (safe to share)
    pub public_key: String,
    /// SHA256 fingerprint of the key
    pub fingerprint: String,
    /// Name of the secret in GCP Secret Manager
    pub secret_name: String,
    /// Version of the secret that was created
    pub secret_version: String,
    /// Key algorithm used
    pub algorithm: KeyAlgorithm,
}

impl SshKeyResult {
    /// Format the result for display
    pub fn display(&self) -> String {
        format!(
            "SSH Key Generated:\n  Algorithm: {}\n  Fingerprint: {}\n  Secret: {}\n  Version: {}\n  Public Key:\n{}",
            self.algorithm, self.fingerprint, self.secret_name, self.secret_version, self.public_key
        )
    }
}

/// Errors that can occur during SSH key operations
#[derive(Debug, Error)]
pub enum SshKeyError {
    /// Key generation failed
    #[error("Key generation failed: {0}")]
    KeyGenerationFailed(String),

    /// Secret Manager operation failed
    #[error("Secret Manager error: {0}")]
    SecretManagerError(String),

    /// Invalid secret name
    #[error("Invalid secret name: {0}")]
    InvalidSecretName(String),

    /// Authentication failed
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    /// Secret already exists
    #[error("Secret already exists: {0}")]
    SecretAlreadyExists(String),
}

impl From<anyhow::Error> for SshKeyError {
    fn from(err: anyhow::Error) -> Self {
        SshKeyError::SecretManagerError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_algorithm_display() {
        assert_eq!(KeyAlgorithm::Ed25519.to_string(), "ed25519");
    }

    #[test]
    fn test_key_generation_options_builder() {
        let opts = KeyGenerationOptions::new("test@example.com")
            .algorithm(KeyAlgorithm::Ed25519)
            .label("env", "test")
            .with_standard_labels();

        assert_eq!(opts.comment, "test@example.com");
        assert_eq!(opts.algorithm, KeyAlgorithm::Ed25519);
        assert_eq!(opts.labels.get("env"), Some(&"test".to_string()));
        assert_eq!(
            opts.labels.get("managed-by"),
            Some(&"ssh-key-agent".to_string())
        );
    }

    #[test]
    fn test_deploy_key_options() {
        let opts = KeyGenerationOptions::new("deploy@lornu.ai")
            .for_deploy_key("lornu-ai/repo", "production");

        assert_eq!(
            opts.labels.get("purpose"),
            Some(&"github-deploy-key".to_string())
        );
        assert_eq!(
            opts.labels.get("repository"),
            Some(&"lornu-ai/repo".to_string())
        );
        assert_eq!(
            opts.labels.get("environment"),
            Some(&"production".to_string())
        );
    }
}
