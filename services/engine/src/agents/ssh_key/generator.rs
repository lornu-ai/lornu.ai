//! SSH Key Generation
//!
//! Generates Ed25519 SSH key pairs using the `ssh-key` crate.

use crate::agents::ssh_key::types::{KeyAlgorithm, SshKeyError};
use rand::rngs::OsRng;
use ssh_key::{private::Ed25519Keypair, HashAlg, LineEnding, PrivateKey};
use tracing::debug;

/// Generated SSH key pair
pub struct GeneratedKeyPair {
    /// Private key in OpenSSH PEM format
    pub private_key_pem: String,
    /// Public key in OpenSSH format (e.g., "ssh-ed25519 AAAA... comment")
    pub public_key_openssh: String,
    /// SHA256 fingerprint
    pub fingerprint: String,
}

/// Generate an Ed25519 SSH key pair
///
/// # Arguments
/// * `comment` - Comment to embed in the public key (e.g., "deploy@lornu.ai")
///
/// # Returns
/// A `GeneratedKeyPair` containing the private key PEM, public key, and fingerprint.
pub fn generate_ed25519_keypair(comment: &str) -> Result<GeneratedKeyPair, SshKeyError> {
    debug!(algorithm = %KeyAlgorithm::Ed25519, comment = %comment, "Generating SSH key pair");

    // Generate Ed25519 keypair using OS random number generator
    let ed25519_keypair = Ed25519Keypair::random(&mut OsRng);
    let private_key = PrivateKey::from(ed25519_keypair);

    // Convert private key to OpenSSH PEM format
    let private_key_pem = private_key
        .to_openssh(LineEnding::LF)
        .map_err(|e| SshKeyError::KeyGenerationFailed(format!("Failed to encode private key: {e}")))?
        .to_string();

    // Get public key with comment
    let public_key = private_key.public_key();
    let public_key_openssh = format!("{} {}", public_key.to_openssh().map_err(|e| {
        SshKeyError::KeyGenerationFailed(format!("Failed to encode public key: {e}"))
    })?, comment);

    // Calculate SHA256 fingerprint
    let fingerprint = public_key.fingerprint(HashAlg::Sha256).to_string();

    debug!(fingerprint = %fingerprint, "Key pair generated successfully");

    Ok(GeneratedKeyPair {
        private_key_pem,
        public_key_openssh,
        fingerprint,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_ed25519_keypair() {
        let result = generate_ed25519_keypair("test@lornu.ai").unwrap();

        // Verify private key format
        assert!(result.private_key_pem.contains("-----BEGIN OPENSSH PRIVATE KEY-----"));
        assert!(result.private_key_pem.contains("-----END OPENSSH PRIVATE KEY-----"));

        // Verify public key format
        assert!(result.public_key_openssh.starts_with("ssh-ed25519 "));
        assert!(result.public_key_openssh.ends_with("test@lornu.ai"));

        // Verify fingerprint format (SHA256:base64)
        assert!(result.fingerprint.starts_with("SHA256:"));
    }

    #[test]
    fn test_keypairs_are_unique() {
        let key1 = generate_ed25519_keypair("test1@lornu.ai").unwrap();
        let key2 = generate_ed25519_keypair("test2@lornu.ai").unwrap();

        // Each generation should produce a unique key
        assert_ne!(key1.fingerprint, key2.fingerprint);
        assert_ne!(key1.private_key_pem, key2.private_key_pem);
    }
}
