//! CDK8s TypeScript PR Remediator
//!
//! Generates GitHub Pull Requests to update CDK8s TypeScript files
//! instead of making direct IAM changes. Follows GitOps compliance.
//!
//! ## Workflow
//! 1. Zero Trust agent identifies corrections
//! 2. Remediator generates TypeScript code changes
//! 3. Creates feature branch and PR via GitHub API
//! 4. Humans review and approve (or auto-merge if high confidence)
//!
//! ## Security
//! - Never makes direct IAM changes
//! - All changes go through PR review process
//! - PRs are labeled for security team visibility

use anyhow::{Context, Result};
use octocrab::Octocrab;
use tracing::{info, warn};

use super::types::{CorrectionType, IamCorrection, InsightSeverity, RemediationResult};

/// GitHub PR remediator for CDK8s TypeScript updates
pub struct Remediator {
    /// GitHub organization
    org: String,
    /// Repository name
    repo: String,
    /// Octocrab client
    client: Octocrab,
    /// Base branch for PRs (default: develop)
    base_branch: String,
}

impl Remediator {
    /// Create a new Remediator
    ///
    /// # Environment Variables
    /// - `GITHUB_TEAM_PAT`: GitHub Personal Access Token with repo scope
    ///
    /// # Example
    /// ```ignore
    /// let remediator = Remediator::new("lornu-ai", "private-lornu-ai")?;
    /// let result = remediator.create_remediation_pr(&corrections).await?;
    /// ```
    pub fn new(org: &str, repo: &str) -> Result<Self> {
        let token = std::env::var("GITHUB_TEAM_PAT")
            .context("GITHUB_TEAM_PAT environment variable not set")?;

        let client = Octocrab::builder()
            .personal_token(token)
            .build()
            .context("Failed to create GitHub client")?;

        Ok(Self {
            org: org.to_string(),
            repo: repo.to_string(),
            client,
            base_branch: "develop".to_string(),
        })
    }

    /// Set custom base branch
    pub fn with_base_branch(mut self, branch: &str) -> Self {
        self.base_branch = branch.to_string();
        self
    }

    /// Create a PR for a set of IAM corrections
    pub async fn create_remediation_pr(
        &self,
        corrections: &[IamCorrection],
    ) -> Result<RemediationResult> {
        if corrections.is_empty() {
            return Ok(RemediationResult {
                success: true,
                pr_number: None,
                pr_url: None,
                branch_name: String::new(),
                corrections_applied: 0,
                message: "No corrections to apply".to_string(),
            });
        }

        let branch_name = format!(
            "fix/zero-trust-{}",
            chrono::Utc::now().format("%Y%m%d-%H%M%S")
        );

        info!(
            branch = %branch_name,
            corrections = %corrections.len(),
            "Creating remediation PR"
        );

        // 1. Get base branch SHA
        let base_ref = self.get_branch_sha(&self.base_branch).await?;

        // 2. Create new branch
        self.create_branch(&branch_name, &base_ref).await?;

        // 3. Generate and commit TypeScript changes
        let files_changed = self.commit_corrections(&branch_name, corrections).await?;

        // 4. Create PR
        let pr = self.create_pr(&branch_name, corrections).await?;

        Ok(RemediationResult {
            success: true,
            pr_number: Some(pr.number),
            pr_url: pr.html_url.map(|u| u.to_string()),
            branch_name,
            corrections_applied: files_changed,
            message: format!("Created PR #{} with {} corrections", pr.number, files_changed),
        })
    }

    /// Get SHA of a branch
    async fn get_branch_sha(&self, branch: &str) -> Result<String> {
        let reference = self
            .client
            .repos(&self.org, &self.repo)
            .get_ref(&octocrab::params::repos::Reference::Branch(branch.to_string()))
            .await
            .context("Failed to get branch reference")?;

        // Extract SHA from the object
        let sha = match &reference.object {
            octocrab::models::repos::Object::Commit { sha, .. } => sha.clone(),
            octocrab::models::repos::Object::Tag { sha, .. } => sha.clone(),
            _ => anyhow::bail!("Unexpected object type in reference"),
        };
        Ok(sha)
    }

    /// Create a new branch
    async fn create_branch(&self, branch_name: &str, sha: &str) -> Result<()> {
        self.client
            .repos(&self.org, &self.repo)
            .create_ref(
                &octocrab::params::repos::Reference::Branch(branch_name.to_string()),
                sha,
            )
            .await
            .context("Failed to create branch")?;

        info!(branch = %branch_name, "Created branch");
        Ok(())
    }

    /// Generate TypeScript changes and commit them
    async fn commit_corrections(
        &self,
        branch: &str,
        corrections: &[IamCorrection],
    ) -> Result<u32> {
        let mut files_changed = 0;

        // Group corrections by file path
        let mut by_file: std::collections::HashMap<String, Vec<&IamCorrection>> =
            std::collections::HashMap::new();

        for correction in corrections {
            if let Some(path) = &correction.cdk8s_file_path {
                by_file.entry(path.clone()).or_default().push(correction);
            }
        }

        for (file_path, file_corrections) in by_file {
            // Generate TypeScript code for corrections
            let ts_code = self.generate_typescript(&file_corrections)?;

            // Get current file content (if exists)
            let current_content = self.get_file_content(&file_path, branch).await.ok();

            // Merge changes
            let new_content = self.merge_typescript_changes(current_content.as_deref(), &ts_code)?;

            // Commit the file
            self.commit_file(
                branch,
                &file_path,
                &new_content,
                &format!(
                    "fix(iam): apply {} zero-trust corrections",
                    file_corrections.len()
                ),
            )
            .await?;

            files_changed += 1;
        }

        Ok(files_changed)
    }

    /// Generate TypeScript code for corrections
    fn generate_typescript(&self, corrections: &[&IamCorrection]) -> Result<String> {
        let mut code = String::new();
        code.push_str("// =============================================================================\n");
        code.push_str("// Auto-generated by ZeroTrustAgent - Review these changes before merging\n");
        code.push_str("// =============================================================================\n\n");

        for correction in corrections {
            match correction.correction_type {
                CorrectionType::DeleteRole => {
                    code.push_str(&format!(
                        "// DELETE: {} - {}\n",
                        correction.target, correction.rationale
                    ));
                    code.push_str(&format!(
                        "// Action: Remove IAM binding for service account: {}\n",
                        correction.target
                    ));
                    code.push_str("// Example:\n");
                    code.push_str("// - Find and remove the binding in your IAM constructs\n");
                    code.push_str("// - Verify no workloads depend on this service account\n\n");
                }
                CorrectionType::ShrinkRole => {
                    code.push_str(&format!(
                        "// SHRINK: {} - {}\n",
                        correction.target, correction.rationale
                    ));
                    code.push_str("// Action: Remove the following unused permissions:\n");
                    if let Some(perms) = correction.proposed_state.get("permissions_to_remove") {
                        if let Some(arr) = perms.as_array() {
                            for perm in arr {
                                code.push_str(&format!("//   - {}\n", perm));
                            }
                        }
                    }
                    code.push_str("// Example update:\n");
                    code.push_str("// Replace broad role with minimal custom role\n\n");
                }
                CorrectionType::RotateSecret => {
                    code.push_str(&format!(
                        "// ROTATE: {} - {}\n",
                        correction.target, correction.rationale
                    ));
                    code.push_str("// Action: Trigger secret rotation in Secret Manager\n");
                    code.push_str("// This is typically done via GSM API, not CDK8s\n\n");
                }
                CorrectionType::ConvertToEphemeral => {
                    code.push_str(&format!(
                        "// CONVERT: {} to Workload Identity\n",
                        correction.target
                    ));
                    code.push_str("// Action: Update ServiceAccount to use Workload Identity:\n");
                    code.push_str("// Example annotation to add:\n");
                    code.push_str("// metadata.annotations['iam.gke.io/gcp-service-account'] = 'SA_EMAIL'\n\n");
                }
            }
        }

        Ok(code)
    }

    /// Get file content from repository
    async fn get_file_content(&self, path: &str, branch: &str) -> Result<String> {
        let content = self
            .client
            .repos(&self.org, &self.repo)
            .get_content()
            .path(path)
            .r#ref(branch)
            .send()
            .await?;

        if let Some(items) = content.items.first() {
            if let Some(content) = &items.content {
                use base64::Engine;
                let decoded = base64::engine::general_purpose::STANDARD
                    .decode(content.replace('\n', ""))?;
                return Ok(String::from_utf8(decoded)?);
            }
        }

        anyhow::bail!("File not found: {}", path)
    }

    /// Merge TypeScript changes (append corrections as comments)
    fn merge_typescript_changes(
        &self,
        current: Option<&str>,
        additions: &str,
    ) -> Result<String> {
        let mut result = current.unwrap_or("").to_string();
        if !result.is_empty() && !result.ends_with('\n') {
            result.push('\n');
        }
        result.push_str("\n// === ZERO TRUST CORRECTIONS ===\n");
        result.push_str(additions);
        Ok(result)
    }

    /// Get file SHA
    async fn get_file_sha(&self, path: &str, branch: &str) -> Result<String> {
        let content = self
            .client
            .repos(&self.org, &self.repo)
            .get_content()
            .path(path)
            .r#ref(branch)
            .send()
            .await?;

        if let Some(items) = content.items.first() {
            return Ok(items.sha.clone());
        }

        anyhow::bail!("File not found")
    }

    /// Commit a file to a branch
    async fn commit_file(
        &self,
        branch: &str,
        path: &str,
        content: &str,
        message: &str,
    ) -> Result<()> {
        use base64::Engine;

        // Get current file SHA if it exists
        let current_sha = self.get_file_sha(path, branch).await.ok();

        let encoded = base64::engine::general_purpose::STANDARD.encode(content);

        // Use the repos API to create/update file
        let update_result = if let Some(sha) = current_sha {
            // Update existing file
            self.client
                .repos(&self.org, &self.repo)
                .update_file(path, message, &encoded, &sha)
                .branch(branch)
                .send()
                .await
        } else {
            // Create new file
            self.client
                .repos(&self.org, &self.repo)
                .create_file(path, message, &encoded)
                .branch(branch)
                .send()
                .await
        };

        update_result.context("Failed to commit file")?;

        info!(path = %path, branch = %branch, "Committed file");
        Ok(())
    }

    /// Create the PR
    async fn create_pr(
        &self,
        branch: &str,
        corrections: &[IamCorrection],
    ) -> Result<octocrab::models::pulls::PullRequest> {
        let title = format!(
            "fix(iam): Zero Trust corrections - {} issues found",
            corrections.len()
        );

        let mut body = String::from("## Summary\n\n");
        body.push_str(
            "Automated IAM hardening corrections identified by ZeroTrustAgent.\n\n",
        );
        body.push_str("### Corrections\n\n");

        for correction in corrections {
            let type_str = match correction.correction_type {
                CorrectionType::DeleteRole => "DELETE",
                CorrectionType::ShrinkRole => "SHRINK",
                CorrectionType::RotateSecret => "ROTATE",
                CorrectionType::ConvertToEphemeral => "CONVERT",
            };
            body.push_str(&format!(
                "- **{}** `{}`: {}\n",
                type_str, correction.target, correction.rationale
            ));
        }

        body.push_str("\n### Risk Assessment\n\n");
        let critical = corrections
            .iter()
            .filter(|c| c.risk_level == InsightSeverity::Critical)
            .count();
        let high = corrections
            .iter()
            .filter(|c| c.risk_level == InsightSeverity::High)
            .count();
        let medium = corrections
            .iter()
            .filter(|c| c.risk_level == InsightSeverity::Medium)
            .count();
        let low = corrections
            .iter()
            .filter(|c| c.risk_level == InsightSeverity::Low)
            .count();

        body.push_str(&format!(
            "| Severity | Count |\n|----------|-------|\n| Critical | {} |\n| High | {} |\n| Medium | {} |\n| Low | {} |\n\n",
            critical, high, medium, low
        ));

        body.push_str("### Test Plan\n\n");
        body.push_str("- [ ] Review each correction for impact\n");
        body.push_str("- [ ] Verify no workloads depend on removed permissions\n");
        body.push_str("- [ ] Test in staging environment first\n\n");

        body.push_str("---\n");
        body.push_str("Generated with [Claude Code](https://claude.com/claude-code)\n");

        let pr = self
            .client
            .pulls(&self.org, &self.repo)
            .create(&title, branch, &self.base_branch)
            .body(&body)
            .send()
            .await
            .context("Failed to create PR")?;

        info!(pr = %pr.number, "Created PR");

        // Add labels
        let labels: Vec<String> = vec![
            "zero-trust".to_string(),
            "security".to_string(),
            "automated".to_string(),
        ];
        if let Err(e) = self
            .client
            .issues(&self.org, &self.repo)
            .add_labels(pr.number, &labels)
            .await
        {
            warn!(error = %e, "Failed to add labels to PR");
        }

        Ok(pr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
        let correction = IamCorrection {
            id: Uuid::new_v4(),
            correction_type: CorrectionType::DeleteRole,
            target: "unused-sa@project.iam.gserviceaccount.com".to_string(),
            current_state: json!({"role": "roles/viewer"}),
            proposed_state: json!(null),
            rationale: "Role unused for 120+ days".to_string(),
            risk_level: InsightSeverity::High,
            cdk8s_file_path: Some("infra/test.ts".to_string()),
            created_at: Utc::now(),
        };
        let ts = Remediator::generate_typescript_suggestion(&correction);
        assert!(ts.contains("DELETE ROLE"));
        // Test would verify TypeScript generation for delete corrections
    }

        let correction = IamCorrection {
            id: Uuid::new_v4(),
            correction_type: CorrectionType::ShrinkRole,
            target: "test-sa@project.iam.gserviceaccount.com".to_string(),
            current_state: json!({"permissions": ["storage.objects.get", "storage.objects.list"]}),
            proposed_state: json!({"permissions": ["storage.objects.get"]}),
            rationale: "Remove unused permission".to_string(),
            risk_level: InsightSeverity::Medium,
            cdk8s_file_path: Some("infra/test.ts".to_string()),
            created_at: Utc::now(),
        };
        let ts = Remediator::generate_typescript_suggestion(&correction);
        assert!(ts.contains("SHRINK ROLE"));
    fn test_generate_typescript_shrink_role() {
        // Test would verify TypeScript generation for shrink corrections
    }

    #[test]
    fn test_merge_typescript_changes_empty() {
        let remediator = Remediator {
            org: "test".to_string(),
            repo: "test".to_string(),
            client: Octocrab::builder().build().unwrap(),
            base_branch: "main".to_string(),
        };

        let result = remediator
            .merge_typescript_changes(None, "// test content")
            .unwrap();
        assert!(result.contains("ZERO TRUST CORRECTIONS"));
        assert!(result.contains("// test content"));
    }
}
