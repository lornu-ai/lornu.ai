//! Example: Accessing cross-project secrets from Rust
//!
//! This example demonstrates how to access secrets stored in one GCP project
//! (e.g., lornu-legacy) from workloads running in another project (e.g., lornu-v2).

use anyhow::Result;
use google_cloud_secretmanager::Client;
use google_cloud_secretmanager::protos::AccessSecretVersionRequest;

/// Access a secret from a cross-project location
///
/// # Arguments
/// * `project_id` - The GCP project ID where the secret is stored (e.g., "lornu-legacy")
/// * `secret_name` - The name of the secret in Secret Manager
/// * `version` - The version to access (defaults to "latest")
///
/// # Returns
/// The secret value as a String
async fn get_cross_project_secret(
    project_id: &str,
    secret_name: &str,
    version: Option<&str>,
) -> Result<String> {
    // Create client (uses default credentials from environment)
    // The service account must have been granted access via IAM binding
    let client = Client::default().await?;
    
    // Build full resource name for cross-project access
    let version_str = version.unwrap_or("latest");
    let secret_path = format!(
        "projects/{}/secrets/{}/versions/{}",
        project_id, secret_name, version_str
    );
    
    println!("Accessing secret: {}", secret_path);
    
    // Access the secret
    let request = AccessSecretVersionRequest {
        name: secret_path,
        ..Default::default()
    };
    
    let response = client.access_secret_version(request).await?;
    let payload = response.payload.ok_or_else(|| anyhow::anyhow!("No payload in response"))?;
    let secret_value = String::from_utf8(payload.data)?;
    
    Ok(secret_value)
}

/// Example: Access OpenAI API key from legacy project
#[tokio::main]
async fn main() -> Result<()> {
    // Example: Access secret from lornu-legacy project
    // The service account running this code must be in lornu-v2
    // and have been granted access via IAM binding
    
    let secret_project = "lornu-legacy";
    let secret_name = "OPENAI_KEY";
    
    match get_cross_project_secret(secret_project, secret_name, None).await {
        Ok(value) => {
            println!("✅ Successfully retrieved secret: {}", secret_name);
            println!("   Value length: {} characters", value.len());
            // In production, don't print the actual value!
        }
        Err(e) => {
            eprintln!("❌ Failed to retrieve secret: {}", e);
            eprintln!("\nTroubleshooting:");
            eprintln!("1. Verify IAM binding exists:");
            eprintln!("   gcloud secrets get-iam-policy {} --project={}", secret_name, secret_project);
            eprintln!("2. Verify service account has access:");
            eprintln!("   gcloud secrets get-iam-policy {} --project={} --flatten='bindings[].members' --filter='bindings.members:serviceAccount:*'", secret_name, secret_project);
            eprintln!("3. Verify using full resource name: projects/{}/secrets/{}/versions/latest", secret_project, secret_name);
            return Err(e);
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    #[ignore] // Requires GCP credentials and actual secrets
    async fn test_cross_project_secret_access() {
        // This test requires:
        // 1. GCP credentials configured
        // 2. Service account with cross-project access
        // 3. Actual secret in the project
        
        let result = get_cross_project_secret("lornu-legacy", "OPENAI_KEY", None).await;
        assert!(result.is_ok());
    }
}
