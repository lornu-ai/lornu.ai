//! Zero Trust IAM Hardening Agent
//!
//! An intelligent agent that scans for and remediates IAM security issues:
//! - Unused IAM roles (90-day inactivity threshold)
//! - Stale secrets in Secret Manager
//! - Long-lived credentials that should be ephemeral
//!
//! The agent learns from successful shrink operations by storing patterns
//! in Qdrant, improving future recommendations.
//!
//! ## GCP API Integration
//! - IAM Admin API: List service accounts and keys
//! - IAM Recommender API: Permission insights
//! - Secret Manager API: Secret age and rotation
//!
//! ## Security Model
//! - Uses ADC (Application Default Credentials) - no hardcoded secrets
//! - Never makes direct changes - generates PRs for human review
//! - Confidence thresholds prevent auto-applying low-confidence fixes

use anyhow::{Context, Result};
use async_openai::config::OpenAIConfig;
use async_openai::types::{CreateEmbeddingRequestArgs, EmbeddingInput};
use async_openai::Client as OpenAIClient;
use chrono::Utc;
use qdrant_client::qdrant::{
    CreateCollectionBuilder, Distance, PointStruct, SearchPointsBuilder, UpsertPointsBuilder,
    Value, VectorParamsBuilder,
};
use qdrant_client::Qdrant;
use reqwest::Client;
use serde_json::json;
use std::collections::HashMap;
use tracing::{info, warn};
use uuid::Uuid;

use super::types::*;

/// Collection name for storing shrink patterns
const SHRINK_PATTERNS_COLLECTION: &str = "zero_trust_shrink_patterns";

/// Embedding dimension (OpenAI text-embedding-3-small)
const EMBEDDING_DIM: u64 = 1536;

/// Minimum confidence to auto-apply a shrink pattern
const MIN_CONFIDENCE_THRESHOLD: f32 = 0.85;

/// Default inactivity threshold in days
const DEFAULT_INACTIVITY_DAYS: u32 = 90;

/// Default secret age threshold in days
const DEFAULT_SECRET_AGE_DAYS: u32 = 90;

/// Zero Trust IAM Hardening Agent
pub struct ZeroTrustAgent {
    /// GCP project ID
    project_id: String,
    /// HTTP client for GCP API calls
    http_client: Client,
    /// Qdrant client for learning storage
    qdrant: Qdrant,
    /// OpenAI client for embeddings
    openai_client: OpenAIClient<OpenAIConfig>,
    /// Inactivity threshold in days
    inactivity_days: u32,
    /// Secret age threshold in days
    secret_age_days: u32,
}

impl ZeroTrustAgent {
    /// Create a new ZeroTrustAgent
    ///
    /// # Arguments
    /// * `project_id` - GCP project ID
    /// * `qdrant_url` - Qdrant server URL
    /// * `openai_api_key` - OpenAI API key for embeddings
    ///
    /// # Example
    /// ```ignore
    /// let agent = ZeroTrustAgent::new("my-project", "http://localhost:6333", "sk-...").await?;
    /// let result = agent.scan().await?;
    /// ```
    pub async fn new(
        project_id: &str,
        qdrant_url: &str,
        openai_api_key: String,
    ) -> Result<Self> {
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .context("Failed to create HTTP client")?;

        let qdrant = Qdrant::from_url(qdrant_url)
            .build()
            .context("Failed to connect to Qdrant")?;

        let config = OpenAIConfig::new().with_api_key(&openai_api_key);
        let openai_client = OpenAIClient::with_config(config);

        let agent = Self {
            project_id: project_id.to_string(),
            http_client,
            qdrant,
            openai_client,
            inactivity_days: DEFAULT_INACTIVITY_DAYS,
            secret_age_days: DEFAULT_SECRET_AGE_DAYS,
        };

        agent.ensure_collection().await?;

        info!(project = %project_id, "ZeroTrustAgent initialized");
        Ok(agent)
    }

    /// Set custom inactivity threshold
    pub fn with_inactivity_days(mut self, days: u32) -> Self {
        self.inactivity_days = days;
        self
    }

    /// Set custom secret age threshold
    pub fn with_secret_age_days(mut self, days: u32) -> Self {
        self.secret_age_days = days;
        self
    }

    /// Ensure Qdrant collection exists for storing patterns
    async fn ensure_collection(&self) -> Result<()> {
        let collections = self.qdrant.list_collections().await?;
        let exists = collections
            .collections
            .iter()
            .any(|c| c.name == SHRINK_PATTERNS_COLLECTION);

        if !exists {
            info!("Creating collection: {}", SHRINK_PATTERNS_COLLECTION);
            self.qdrant
                .create_collection(
                    CreateCollectionBuilder::new(SHRINK_PATTERNS_COLLECTION)
                        .vectors_config(VectorParamsBuilder::new(EMBEDDING_DIM, Distance::Cosine)),
                )
                .await
                .context("Failed to create Qdrant collection")?;
        }

        Ok(())
    }

    /// Get ADC access token for GCP API calls
    async fn get_access_token(&self) -> Result<String> {
        // Try GCE metadata server first (Workload Identity in GKE)
        let metadata_url = "http://metadata.google.internal/computeMetadata/v1/instance/service-accounts/default/token";

        match self
            .http_client
            .get(metadata_url)
            .header("Metadata-Flavor", "Google")
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => {
                let token_response: serde_json::Value = resp.json().await?;
                Ok(token_response["access_token"]
                    .as_str()
                    .context("Invalid token response")?
                    .to_string())
            }
            _ => {
                // Fall back to gcloud CLI (local development)
                let output = tokio::process::Command::new("gcloud")
                    .args(["auth", "application-default", "print-access-token"])
                    .output()
                    .await
                    .context("gcloud CLI not available")?;

                if !output.status.success() {
                    anyhow::bail!("gcloud auth failed - run 'gcloud auth application-default login'");
                }

                Ok(String::from_utf8(output.stdout)?.trim().to_string())
            }
        }
    }

    // =========================================================================
    // CORE SCANNING OPERATIONS
    // =========================================================================

    /// Execute a full Zero Trust scan
    ///
    /// Scans for:
    /// 1. Unused IAM roles (based on Recommender API insights)
    /// 2. Stale secrets in Secret Manager
    /// 3. Long-lived credentials that should be ephemeral
    pub async fn scan(&self) -> Result<ZeroTrustScanResult> {
        let start = std::time::Instant::now();
        info!(project = %self.project_id, "Starting Zero Trust scan");

        // 1. List all service accounts
        let service_accounts = self.list_service_accounts().await?;
        let accounts_scanned = service_accounts.len() as u32;

        // 2. Get IAM insights from Recommender API
        let insights = self.get_iam_insights().await?;

        // 3. Identify stale secrets
        let secrets_to_rotate = self.scan_stale_secrets().await?;

        // 4. Identify long-lived credentials
        let ephemeral_conversions = self
            .scan_long_lived_credentials(&service_accounts)
            .await?;

        // 5. Generate corrections based on findings
        let corrections = self
            .generate_corrections(&insights, &secrets_to_rotate)
            .await?;

        let duration = start.elapsed();
        info!(
            accounts = %accounts_scanned,
            insights = %insights.len(),
            corrections = %corrections.len(),
            secrets = %secrets_to_rotate.len(),
            conversions = %ephemeral_conversions.len(),
            duration_ms = %duration.as_millis(),
            "Zero Trust scan complete"
        );

        Ok(ZeroTrustScanResult {
            accounts_scanned,
            insights,
            corrections,
            secrets_to_rotate,
            ephemeral_conversions,
            scan_duration_ms: duration.as_millis() as u64,
            completed_at: Utc::now(),
        })
    }

    /// List all service accounts in the project
    async fn list_service_accounts(&self) -> Result<Vec<serde_json::Value>> {
        let token = self.get_access_token().await?;
        let url = format!(
            "https://iam.googleapis.com/v1/projects/{}/serviceAccounts",
            self.project_id
        );

        let response = self
            .http_client
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await
            .context("Failed to list service accounts")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("IAM API returned {}: {}", status, body);
        }

        let data: serde_json::Value = response.json().await?;
        let accounts = data["accounts"].as_array().cloned().unwrap_or_default();

        info!(count = %accounts.len(), "Listed service accounts");
        Ok(accounts)
    }

    /// Get IAM insights from Recommender API
    async fn get_iam_insights(&self) -> Result<Vec<IamInsight>> {
        let token = self.get_access_token().await?;

        // Query IAM policy insights
        let url = format!(
            "https://recommender.googleapis.com/v1/projects/{}/locations/-/insightTypes/google.iam.policy.Insight/insights",
            self.project_id
        );

        let response = self
            .http_client
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await
            .context("Failed to get IAM insights")?;

        if !response.status().is_success() {
            warn!(
                status = %response.status(),
                "IAM Recommender API unavailable, returning empty insights"
            );
            return Ok(vec![]);
        }

        let data: serde_json::Value = response.json().await?;
        let raw_insights = data["insights"].as_array().cloned().unwrap_or_default();

        let insights: Vec<IamInsight> = raw_insights
            .iter()
            .filter_map(|i| self.parse_insight(i).ok())
            .filter(|i| i.days_inactive >= self.inactivity_days)
            .collect();

        info!(count = %insights.len(), "Retrieved IAM insights");
        Ok(insights)
    }

    /// Parse raw insight JSON into IamInsight struct
    fn parse_insight(&self, raw: &serde_json::Value) -> Result<IamInsight> {
        let content = &raw["content"];

        Ok(IamInsight {
            insight_id: raw["name"].as_str().unwrap_or("").to_string(),
            service_account: content["member"].as_str().unwrap_or("").to_string(),
            days_inactive: content["exercisedPermissions"]
                .as_array()
                .map(|_| 0) // If exercised, 0 days inactive
                .unwrap_or(self.inactivity_days + 30),
            unused_permissions: content["inferredPermissions"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|p| p["permission"].as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default(),
            severity: self.calculate_severity(content),
            generated_at: Utc::now(),
        })
    }

    /// Calculate severity based on insight content
    fn calculate_severity(&self, content: &serde_json::Value) -> InsightSeverity {
        let unused_count = content["inferredPermissions"]
            .as_array()
            .map(|a| a.len())
            .unwrap_or(0);

        match unused_count {
            0..=5 => InsightSeverity::Low,
            6..=15 => InsightSeverity::Medium,
            16..=30 => InsightSeverity::High,
            _ => InsightSeverity::Critical,
        }
    }

    /// Scan for stale secrets in Secret Manager
    async fn scan_stale_secrets(&self) -> Result<Vec<SecretRotationRequest>> {
        let token = self.get_access_token().await?;
        let url = format!(
            "https://secretmanager.googleapis.com/v1/projects/{}/secrets",
            self.project_id
        );

        let response = self
            .http_client
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await
            .context("Failed to list secrets")?;

        if !response.status().is_success() {
            warn!("Secret Manager API unavailable");
            return Ok(vec![]);
        }

        let data: serde_json::Value = response.json().await?;
        let secrets = data["secrets"].as_array().cloned().unwrap_or_default();

        let mut stale_secrets = Vec::new();

        for secret in secrets {
            if let Some(name) = secret["name"].as_str() {
                // Get latest version to check age
                if let Ok(age_days) = self.get_secret_age(name, &token).await {
                    if age_days >= self.secret_age_days {
                        stale_secrets.push(SecretRotationRequest {
                            secret_id: name.to_string(),
                            project_id: self.project_id.clone(),
                            age_days,
                            rotation_triggered: false,
                            requested_at: Utc::now(),
                        });
                    }
                }
            }
        }

        info!(count = %stale_secrets.len(), "Found stale secrets");
        Ok(stale_secrets)
    }

    /// Get age of a secret's latest version
    async fn get_secret_age(&self, secret_name: &str, token: &str) -> Result<u32> {
        let url = format!(
            "https://secretmanager.googleapis.com/v1/{}/versions/latest",
            secret_name
        );

        let response = self.http_client.get(&url).bearer_auth(token).send().await?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to get secret version");
        }

        let data: serde_json::Value = response.json().await?;
        let create_time = data["createTime"].as_str().unwrap_or("");

        let created = chrono::DateTime::parse_from_rfc3339(create_time)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        let age = Utc::now().signed_duration_since(created);
        Ok(age.num_days().max(0) as u32)
    }

    /// Scan for long-lived credentials that should be converted to ephemeral tokens
    async fn scan_long_lived_credentials(
        &self,
        service_accounts: &[serde_json::Value],
    ) -> Result<Vec<EphemeralTokenRequest>> {
        let token = self.get_access_token().await?;
        let mut conversions = Vec::new();

        for sa in service_accounts {
            let email = sa["email"].as_str().unwrap_or("");
            if email.is_empty() {
                continue;
            }

            // List keys for this service account
            let url = format!(
                "https://iam.googleapis.com/v1/projects/{}/serviceAccounts/{}/keys",
                self.project_id, email
            );

            let response = self.http_client.get(&url).bearer_auth(&token).send().await;

            if let Ok(resp) = response {
                if resp.status().is_success() {
                    let data: serde_json::Value = resp.json().await.unwrap_or_default();
                    let keys = data["keys"].as_array().cloned().unwrap_or_default();

                    // Check for user-managed keys (should be converted to Workload Identity)
                    for key in keys {
                        if key["keyType"].as_str() == Some("USER_MANAGED") {
                            conversions.push(EphemeralTokenRequest {
                                service_account: email.to_string(),
                                current_credential_type: "json_key".to_string(),
                                target_token_type: "workload_identity".to_string(),
                                token_ttl_seconds: 3600, // 1 hour
                            });
                            break; // One conversion per SA
                        }
                    }
                }
            }
        }

        info!(count = %conversions.len(), "Found long-lived credentials");
        Ok(conversions)
    }

    /// Generate corrections based on insights and secret findings
    async fn generate_corrections(
        &self,
        insights: &[IamInsight],
        secrets: &[SecretRotationRequest],
    ) -> Result<Vec<IamCorrection>> {
        let mut corrections = Vec::new();

        // Generate corrections from IAM insights
        for insight in insights {
            let correction_type = if insight.days_inactive > 180 {
                CorrectionType::DeleteRole
            } else {
                CorrectionType::ShrinkRole
            };

            // Check if we have a learned pattern for this type of shrink
            let similar_pattern = self.find_similar_shrink_pattern(insight).await?;

            let (proposed_state, rationale) = if let Some(pattern) = similar_pattern {
                if pattern.confidence_score() >= MIN_CONFIDENCE_THRESHOLD {
                    (
                        json!({
                            "permissions_to_remove": pattern.removed_permissions,
                            "auto_apply": true,
                            "confidence": pattern.confidence_score()
                        }),
                        format!(
                            "Based on {} successful similar corrections ({}% confidence)",
                            pattern.success_count,
                            (pattern.confidence_score() * 100.0) as u32
                        ),
                    )
                } else {
                    (
                        json!({ "permissions_to_remove": insight.unused_permissions }),
                        "Low confidence pattern - requires human review".to_string(),
                    )
                }
            } else {
                (
                    json!({ "permissions_to_remove": insight.unused_permissions }),
                    format!(
                        "Service account {} has {} unused permissions after {} days",
                        insight.service_account,
                        insight.unused_permissions.len(),
                        insight.days_inactive
                    ),
                )
            };

            corrections.push(IamCorrection {
                id: Uuid::new_v4(),
                correction_type,
                target: insight.service_account.clone(),
                current_state: json!({
                    "permissions": insight.unused_permissions,
                    "days_inactive": insight.days_inactive
                }),
                proposed_state,
                rationale,
                risk_level: insight.severity.clone(),
                cdk8s_file_path: Some("infra/constructs/iam-bindings.ts".to_string()),
                created_at: Utc::now(),
            });
        }

        // Generate corrections for stale secrets
        for secret in secrets {
            corrections.push(IamCorrection {
                id: Uuid::new_v4(),
                correction_type: CorrectionType::RotateSecret,
                target: secret.secret_id.clone(),
                current_state: json!({ "age_days": secret.age_days }),
                proposed_state: json!({ "trigger_rotation": true }),
                rationale: format!(
                    "Secret {} is {} days old (threshold: {})",
                    secret.secret_id, secret.age_days, self.secret_age_days
                ),
                risk_level: InsightSeverity::High,
                cdk8s_file_path: None, // Secrets rotated via GSM API
                created_at: Utc::now(),
            });
        }

        Ok(corrections)
    }

    // =========================================================================
    // LEARNING OPERATIONS (Qdrant)
    // =========================================================================

    /// Find similar shrink patterns from past successful operations
    async fn find_similar_shrink_pattern(
        &self,
        insight: &IamInsight,
    ) -> Result<Option<ShrinkPattern>> {
        let embedding = self.generate_embedding(&insight.service_account).await?;

        let results = self
            .qdrant
            .search_points(
                SearchPointsBuilder::new(SHRINK_PATTERNS_COLLECTION, embedding, 1)
                    .with_payload(true)
                    .score_threshold(0.85),
            )
            .await?;

        if let Some(point) = results.result.first() {
            // Deserialize pattern from payload
            let pattern = self.deserialize_shrink_pattern(&point.payload)?;
            return Ok(Some(pattern));
        }

        Ok(None)
    }

    /// Learn from a successful shrink operation
    ///
    /// Call this after a shrink PR is merged and verified to work correctly.
    pub async fn learn_shrink_success(
        &self,
        service_account: &str,
        service_type: &str,
        removed_permissions: Vec<String>,
    ) -> Result<()> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        let embedding = self.generate_embedding(service_account).await?;

        let mut payload: HashMap<String, Value> = HashMap::new();
        payload.insert("id".to_string(), Value::from(id.to_string()));
        payload.insert(
            "permission_signature".to_string(),
            Value::from(service_account.to_string()),
        );
        payload.insert(
            "service_type".to_string(),
            Value::from(service_type.to_string()),
        );
        payload.insert(
            "removed_permissions".to_string(),
            Value::from(serde_json::to_string(&removed_permissions).unwrap_or_default()),
        );
        payload.insert("success_count".to_string(), Value::from(1i64));
        payload.insert("rollback_count".to_string(), Value::from(0i64));
        payload.insert("created_at".to_string(), Value::from(now.to_rfc3339()));
        payload.insert("last_used_at".to_string(), Value::from(now.to_rfc3339()));

        self.qdrant
            .upsert_points(
                UpsertPointsBuilder::new(
                    SHRINK_PATTERNS_COLLECTION,
                    vec![PointStruct::new(id.to_string(), embedding, payload)],
                )
                .wait(true),
            )
            .await?;

        info!(
            service_account = %service_account,
            permissions_removed = %removed_permissions.len(),
            "Learned successful shrink pattern"
        );

        Ok(())
    }

    /// Record a rollback (shrink caused issues)
    ///
    /// Call this if a shrink PR caused problems and was reverted.
    pub async fn learn_shrink_rollback(&self, pattern_id: &str) -> Result<()> {
        // This would fetch, increment rollback_count, and upsert
        // For now, just log the rollback
        info!(pattern_id = %pattern_id, "Recording shrink rollback");
        warn!("Rollback learning not yet implemented - pattern confidence unchanged");
        Ok(())
    }

    /// Generate embedding using OpenAI API
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let request = CreateEmbeddingRequestArgs::default()
            .model("text-embedding-3-small")
            .input(EmbeddingInput::String(text.to_string()))
            .build()?;

        let response = self.openai_client.embeddings().create(request).await?;
        Ok(response.data[0].embedding.clone())
    }

    /// Deserialize ShrinkPattern from Qdrant payload
    fn deserialize_shrink_pattern(
        &self,
        payload: &HashMap<String, Value>,
    ) -> Result<ShrinkPattern> {
        let id_str = payload
            .get("id")
            .and_then(|v| v.as_str())
            .context("id field missing")?;
        let id = Uuid::parse_str(id_str)?;

        let removed_str = payload
            .get("removed_permissions")
            .and_then(|v| v.as_str())
            .map(|s| s.as_str())
            .unwrap_or("[]");
        let removed_permissions: Vec<String> = serde_json::from_str(removed_str)?;

        Ok(ShrinkPattern {
            id,
            permission_signature: payload
                .get("permission_signature")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_default(),
            service_type: payload
                .get("service_type")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_default(),
            removed_permissions,
            success_count: payload
                .get("success_count")
                .and_then(|v| v.as_integer())
                .unwrap_or(0) as u32,
            rollback_count: payload
                .get("rollback_count")
                .and_then(|v| v.as_integer())
                .unwrap_or(0) as u32,
            created_at: Utc::now(),
            last_used_at: Utc::now(),
        })
    }

    // =========================================================================
    // SECRET ROTATION
    // =========================================================================

    /// Trigger rotation for a secret in Google Secret Manager
    ///
    /// Note: Actual rotation depends on the secret type and rotation policy.
    /// This method logs the request for audit purposes.
    pub async fn trigger_secret_rotation(&self, secret_id: &str) -> Result<()> {
        info!(secret = %secret_id, "Rotation triggered via GSM");
        // In a full implementation, this would:
        // 1. Generate new credentials based on secret type
        // 2. Add new version to GSM
        // 3. Optionally disable old version after grace period
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_calculation() {
        let agent = ZeroTrustAgent {
            project_id: "test".to_string(),
            http_client: Client::new(),
            qdrant: Qdrant::from_url("http://localhost:6333")
                .build()
                .unwrap(),
            openai_client: OpenAIClient::new(),
            inactivity_days: 90,
            secret_age_days: 90,
        };

        // Test low severity (5 or fewer unused permissions)
        let content = json!({
            "inferredPermissions": [{"permission": "a"}, {"permission": "b"}]
        });
        assert_eq!(agent.calculate_severity(&content), InsightSeverity::Low);

        // Test medium severity (6-15)
        let content = json!({
            "inferredPermissions": (0..10).map(|i| json!({"permission": format!("p{}", i)})).collect::<Vec<_>>()
        });
        assert_eq!(agent.calculate_severity(&content), InsightSeverity::Medium);

        // Test critical severity (31+)
        let content = json!({
            "inferredPermissions": (0..50).map(|i| json!({"permission": format!("p{}", i)})).collect::<Vec<_>>()
        });
        assert_eq!(agent.calculate_severity(&content), InsightSeverity::Critical);
    }
}
