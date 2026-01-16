//! Workload Identity Federation
//!
//! Multi-cloud federated identity using OIDC tokens from Kubernetes
//! ServiceAccounts to assume native cloud identities without static keys.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::env;
use std::time::{Duration, Instant};
use tracing::info;

/// Cloud identity credentials with automatic refresh
#[derive(Debug, Clone)]
pub struct CloudCredentials {
    pub value: CredentialValue,
    pub expires_at: Instant,
    pub provider: IdentityProvider,
}

#[derive(Debug, Clone)]
pub enum CredentialValue {
    Token(String),
    AwsKeys {
        access_key_id: String,
        secret_access_key: String,
        session_token: String,
    },
}

impl CloudCredentials {
    pub fn access_token(&self) -> Option<&str> {
        match &self.value {
            CredentialValue::Token(t) => Some(t),
            _ => None,
        }
    }
}

impl CloudCredentials {
    pub fn is_expired(&self) -> bool {
        Instant::now() >= self.expires_at
    }

    pub fn time_until_expiry(&self) -> Duration {
        self.expires_at
            .checked_duration_since(Instant::now())
            .unwrap_or(Duration::ZERO)
    }
}

/// Identity provider type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentityProvider {
    Aws,
    Azure,
    Gcp,
}

impl std::fmt::Display for IdentityProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IdentityProvider::Aws => write!(f, "aws"),
            IdentityProvider::Azure => write!(f, "azure"),
            IdentityProvider::Gcp => write!(f, "gcp"),
        }
    }
}

/// AWS STS AssumeRoleWithWebIdentity response
#[derive(Debug, Deserialize)]
struct AwsStsResponse {
    #[serde(rename = "AssumeRoleWithWebIdentityResponse")]
    response: AwsStsResult,
}

#[derive(Debug, Deserialize)]
struct AwsStsResult {
    #[serde(rename = "AssumeRoleWithWebIdentityResult")]
    result: AwsCredentialsResult,
}

#[derive(Debug, Deserialize)]
struct AwsCredentialsResult {
    #[serde(rename = "Credentials")]
    credentials: AwsCredentials,
}

#[derive(Debug, Deserialize)]
struct AwsCredentials {
    #[serde(rename = "AccessKeyId")]
    access_key_id: String,
    #[serde(rename = "SecretAccessKey")]
    secret_access_key: String,
    #[serde(rename = "SessionToken")]
    session_token: String,
}

/// Workload Identity Federation Manager
///
/// Handles credential exchange across all cloud providers using
/// Kubernetes ServiceAccount OIDC tokens.
pub struct FederatedIdentityManager {
    http_client: reqwest::Client,
    /// K8s ServiceAccount token path
    sa_token_path: String,
    /// AWS configuration
    aws_role_arn: Option<String>,
    aws_region: String,
    /// GCP configuration
    gcp_project_number: Option<String>,
    gcp_pool_id: Option<String>,
    gcp_provider_id: Option<String>,
    /// Azure configuration
    azure_tenant_id: Option<String>,
    azure_client_id: Option<String>,
}

impl FederatedIdentityManager {
    /// Create a new FederatedIdentityManager
    pub fn new() -> Result<Self> {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        // Standard K8s projected token path
        let sa_token_path = env::var("KUBERNETES_SERVICE_ACCOUNT_TOKEN_PATH")
            .unwrap_or_else(|_| "/var/run/secrets/kubernetes.io/serviceaccount/token".to_string());

        Ok(Self {
            http_client,
            sa_token_path,
            // AWS IRSA config
            aws_role_arn: env::var("AWS_ROLE_ARN").ok(),
            aws_region: env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
            // GCP Workload Identity config
            gcp_project_number: env::var("GCP_PROJECT_NUMBER").ok(),
            gcp_pool_id: env::var("GCP_WORKLOAD_IDENTITY_POOL_ID").ok(),
            gcp_provider_id: env::var("GCP_WORKLOAD_IDENTITY_PROVIDER_ID").ok(),
            // Azure Workload Identity config
            azure_tenant_id: env::var("AZURE_TENANT_ID").ok(),
            azure_client_id: env::var("AZURE_CLIENT_ID").ok(),
        })
    }

    /// Get K8s ServiceAccount OIDC token
    fn get_k8s_token(&self) -> Result<String> {
        std::fs::read_to_string(&self.sa_token_path)
            .with_context(|| format!("Failed to read SA token from {}", self.sa_token_path))
    }

    /// Get AWS credentials using IRSA (IAM Roles for Service Accounts)
    ///
    /// Uses STS AssumeRoleWithWebIdentity to exchange K8s OIDC token
    /// for AWS temporary credentials.
    pub async fn get_aws_credentials(&self) -> Result<CloudCredentials> {
        let role_arn = self
            .aws_role_arn
            .as_ref()
            .context("AWS_ROLE_ARN not configured")?;

        let k8s_token = self.get_k8s_token()?;

        // Call STS AssumeRoleWithWebIdentity
        let sts_url = format!(
            "https://sts.{}.amazonaws.com/?Action=AssumeRoleWithWebIdentity&Version=2011-06-15&RoleArn={}&RoleSessionName=lornu-service-discovery&WebIdentityToken={}",
            self.aws_region,
            urlencoding::encode(role_arn),
            urlencoding::encode(&k8s_token)
        );

        let response = self
            .http_client
            .get(&sts_url)
            .send()
            .await
            .context("Failed to call AWS STS")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("AWS STS failed with {}: {}", status, body);
        }

        // Parse XML response (STS returns XML by default)
        let body = response.text().await?;
        
        let sts_response: AwsStsResponse = quick_xml::de::from_str(&body)
            .context("Failed to parse AWS STS XML response")?;
        
        let creds = sts_response.response.result.credentials;

        info!("AWS credentials obtained via IRSA for role: {}", role_arn);

        // AWS STS tokens typically expire in 1 hour
        Ok(CloudCredentials {
            value: CredentialValue::AwsKeys {
                access_key_id: creds.access_key_id,
                secret_access_key: creds.secret_access_key,
                session_token: creds.session_token,
            },
            expires_at: Instant::now() + Duration::from_secs(3600),
            provider: IdentityProvider::Aws,
        })
    }

    /// Get GCP credentials using Workload Identity Federation
    ///
    /// Exchanges K8s OIDC token for GCP access token via STS.
    pub async fn get_gcp_credentials(&self) -> Result<CloudCredentials> {
        let project_number = self
            .gcp_project_number
            .as_ref()
            .context("GCP_PROJECT_NUMBER not configured")?;
        let pool_id = self
            .gcp_pool_id
            .as_ref()
            .context("GCP_WORKLOAD_IDENTITY_POOL_ID not configured")?;
        let provider_id = self
            .gcp_provider_id
            .as_ref()
            .context("GCP_WORKLOAD_IDENTITY_PROVIDER_ID not configured")?;

        let k8s_token = self.get_k8s_token()?;

        // Step 1: Exchange K8s token for federated token
        let sts_url = "https://sts.googleapis.com/v1/token";
        let audience = format!(
            "//iam.googleapis.com/projects/{}/locations/global/workloadIdentityPools/{}/providers/{}",
            project_number, pool_id, provider_id
        );

        let sts_request = serde_json::json!({
            "grant_type": "urn:ietf:params:oauth:grant-type:token-exchange",
            "subject_token_type": "urn:ietf:params:oauth:token-type:jwt",
            "requested_token_type": "urn:ietf:params:oauth:token-type:access_token",
            "audience": audience,
            "subject_token": k8s_token,
            "scope": "https://www.googleapis.com/auth/cloud-platform"
        });

        let response = self
            .http_client
            .post(sts_url)
            .json(&sts_request)
            .send()
            .await
            .context("Failed to call GCP STS")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("GCP STS failed with {}: {}", status, body);
        }

        let token_response: serde_json::Value = response.json().await?;
        let access_token = token_response["access_token"]
            .as_str()
            .context("Missing access_token in GCP response")?
            .to_string();

        let expires_in = token_response["expires_in"]
            .as_u64()
            .unwrap_or(3600);

        info!("GCP credentials obtained via Workload Identity Federation");

        Ok(CloudCredentials {
            value: CredentialValue::Token(access_token),
            expires_at: Instant::now() + Duration::from_secs(expires_in),
            provider: IdentityProvider::Gcp,
        })
    }

    /// Get Azure credentials using Workload Identity
    ///
    /// Uses Microsoft Entra ID Federated Identity Credentials.
    pub async fn get_azure_credentials(&self) -> Result<CloudCredentials> {
        let tenant_id = self
            .azure_tenant_id
            .as_ref()
            .context("AZURE_TENANT_ID not configured")?;
        let client_id = self
            .azure_client_id
            .as_ref()
            .context("AZURE_CLIENT_ID not configured")?;

        let k8s_token = self.get_k8s_token()?;

        // Exchange K8s token for Azure AD token
        let token_url = format!(
            "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
            tenant_id
        );

        let form_data = [
            ("grant_type", "client_credentials"),
            ("client_id", client_id),
            (
                "client_assertion_type",
                "urn:ietf:params:oauth:client-assertion-type:jwt-bearer",
            ),
            ("client_assertion", &k8s_token),
            ("scope", "https://management.azure.com/.default"),
        ];

        let response = self
            .http_client
            .post(&token_url)
            .form(&form_data)
            .send()
            .await
            .context("Failed to call Azure AD")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Azure AD failed with {}: {}", status, body);
        }

        let token_response: serde_json::Value = response.json().await?;
        let access_token = token_response["access_token"]
            .as_str()
            .context("Missing access_token in Azure response")?
            .to_string();

        let expires_in = token_response["expires_in"]
            .as_u64()
            .unwrap_or(3600);

        info!("Azure credentials obtained via Workload Identity");

        Ok(CloudCredentials {
            value: CredentialValue::Token(access_token),
            expires_at: Instant::now() + Duration::from_secs(expires_in),
            provider: IdentityProvider::Azure,
        })
    }

    /// Check which providers are configured
    pub fn configured_providers(&self) -> Vec<IdentityProvider> {
        let mut providers = Vec::new();

        if self.aws_role_arn.is_some() {
            providers.push(IdentityProvider::Aws);
        }
        if self.gcp_project_number.is_some()
            && self.gcp_pool_id.is_some()
            && self.gcp_provider_id.is_some()
        {
            providers.push(IdentityProvider::Gcp);
        }
        if self.azure_tenant_id.is_some() && self.azure_client_id.is_some() {
            providers.push(IdentityProvider::Azure);
        }

        providers
    }
}

/// No longer used internally, replaced by quick-xml
#[deprecated(note = "Use quick-xml for robust parsing")]
fn _extract_xml_value(xml: &str, tag: &str) -> Option<String> {
    let start_tag = format!("<{}>", tag);
    let end_tag = format!("</{}>", tag);

    let start = xml.find(&start_tag)? + start_tag.len();
    let end = xml[start..].find(&end_tag)? + start;

    Some(xml[start..end].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_xml_value() {
        let xml = r#"<Response><AccessKeyId>AKIA123</AccessKeyId><SecretAccessKey>secret</SecretAccessKey></Response>"#;

        assert_eq!(
            extract_xml_value(xml, "AccessKeyId"),
            Some("AKIA123".to_string())
        );
        assert_eq!(
            extract_xml_value(xml, "SecretAccessKey"),
            Some("secret".to_string())
        );
        assert_eq!(extract_xml_value(xml, "NotFound"), None);
    }

    #[test]
    fn test_credentials_expiry() {
        let creds = CloudCredentials {
            value: CredentialValue::Token("test".to_string()),
            expires_at: Instant::now() + Duration::from_secs(3600),
            provider: IdentityProvider::Gcp,
        };

        assert!(!creds.is_expired());
        assert!(creds.time_until_expiry() > Duration::from_secs(3500));
    }
}
