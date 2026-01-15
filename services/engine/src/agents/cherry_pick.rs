//! Context-Aware Cherry-Pick Agent
//!
//! An intelligent agent that learns from merge conflicts and self-corrects
//! by storing resolution patterns in a Vector DB.
//!
//! Features:
//! - Analyzes diffs and dependency graphs
//! - Attempts cherry-picks with conflict detection
//! - Learns from resolutions by storing patterns in Qdrant
//! - Self-corrects by looking up similar past conflicts

use anyhow::{Context, Result};
use async_openai::config::OpenAIConfig;
use chrono::{DateTime, Utc};
use git2::{CherrypickOptions, Index, Repository, Signature};
use qdrant_client::qdrant::{
    CreateCollectionBuilder, Distance, PointStruct, SearchPointsBuilder, UpsertPointsBuilder,
    Value, VectorParamsBuilder,
};
use qdrant_client::Qdrant;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tracing::{error, info, warn};
use uuid::Uuid;

/// Collection name for storing conflict resolutions
const COLLECTION_NAME: &str = "cherry_pick_resolutions";

/// Embedding dimension (OpenAI text-embedding-3-small)
const EMBEDDING_DIM: u64 = 1536;

/// Minimum similarity score to consider a resolution match
const MIN_SIMILARITY_SCORE: f32 = 0.85;

/// Minimum success rate threshold to automatically apply a resolution
const MIN_SUCCESS_RATE_THRESHOLD: f32 = 0.7;

/// A stored conflict resolution pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionPattern {
    /// Unique identifier
    pub id: Uuid,
    /// The conflict signature (the <<<< HEAD block content)
    pub conflict_signature: String,
    /// File path where conflict occurred
    pub file_path: String,
    /// The resolved code
    pub resolution: String,
    /// Number of times this resolution was successfully used
    pub success_count: u32,
    /// Number of times this resolution failed
    pub failure_count: u32,
    /// When this pattern was first learned
    pub created_at: DateTime<Utc>,
    /// When this pattern was last used
    pub last_used_at: DateTime<Utc>,
    /// Source commit hash that was cherry-picked
    pub source_commit: String,
    /// Target branch where conflict occurred
    pub target_branch: String,
}

impl ResolutionPattern {
    /// Calculate success rate
    pub fn success_rate(&self) -> f32 {
        let total = self.success_count + self.failure_count;
        if total == 0 {
            0.0
        } else {
            self.success_count as f32 / total as f32
        }
    }
}

/// Result of a cherry-pick operation
#[derive(Debug, Serialize)]
pub struct CherryPickResult {
    pub success: bool,
    pub commit_hash: String,
    pub target_branch: String,
    pub conflicts: Vec<ConflictInfo>,
    pub resolutions_applied: u32,
    pub new_commit_sha: Option<String>,
    pub message: String,
}

/// Information about a single conflict
#[derive(Debug, Serialize)]
pub struct ConflictInfo {
    pub file_path: String,
    pub conflict_text: String,
    pub resolution_found: bool,
    pub resolution_applied: bool,
}

/// The Cherry-Pick Agent with learning capabilities
pub struct CherryPickAgent {
    /// Git repository
    repo: Repository,
    /// Qdrant client for vector storage
    qdrant: Qdrant,
    /// OpenAI client for embeddings (reused across calls)
    openai_client: async_openai::Client<async_openai::config::OpenAIConfig>,
}

impl CherryPickAgent {
    /// Create a new CherryPickAgent
    ///
    /// # Arguments
    /// * `repo_path` - Path to the git repository
    /// * `qdrant_url` - URL of the Qdrant server
    /// * `openai_api_key` - OpenAI API key for embeddings
    pub async fn new(
        repo_path: &Path,
        qdrant_url: &str,
        openai_api_key: String,
    ) -> Result<Self> {
        use async_openai::Client;

        let repo = Repository::open(repo_path)
            .with_context(|| format!("Failed to open repository at {:?}", repo_path))?;

        let qdrant = Qdrant::from_url(qdrant_url)
            .build()
            .with_context(|| format!("Failed to connect to Qdrant at {}", qdrant_url))?;

        // Create OpenAI client once for reuse
        let config = OpenAIConfig::new().with_api_key(&openai_api_key);
        let openai_client = Client::with_config(config);

        let agent = Self {
            repo,
            qdrant,
            openai_client,
        };

        // Ensure collection exists
        agent.ensure_collection().await?;

        Ok(agent)
    }

    /// Ensure the Qdrant collection exists
    async fn ensure_collection(&self) -> Result<()> {
        let collections = self.qdrant.list_collections().await?;
        let exists = collections
            .collections
            .iter()
            .any(|c| c.name == COLLECTION_NAME);

        if !exists {
            info!("Creating collection: {}", COLLECTION_NAME);
            self.qdrant
                .create_collection(
                    CreateCollectionBuilder::new(COLLECTION_NAME)
                        .vectors_config(VectorParamsBuilder::new(EMBEDDING_DIM, Distance::Cosine)),
                )
                .await
                .context("Failed to create Qdrant collection")?;
        }

        Ok(())
    }

    /// Execute a cherry-pick with learning
    ///
    /// # Arguments
    /// * `commit_hash` - The commit SHA to cherry-pick
    /// * `target_branch` - The branch to cherry-pick onto
    pub async fn execute_and_learn(
        &self,
        commit_hash: &str,
        target_branch: &str,
    ) -> Result<CherryPickResult> {
        info!(
            commit = %commit_hash,
            target = %target_branch,
            "Attempting context-aware cherry-pick"
        );

        // Checkout target branch
        self.checkout_branch(target_branch)?;

        // Find the commit
        let commit = self
            .repo
            .revparse_single(commit_hash)?
            .peel_to_commit()
            .context("Failed to find commit")?;

        let mut opts = CherrypickOptions::new();

        // Attempt the cherry-pick
        match self.repo.cherrypick(&commit, Some(&mut opts)) {
            Ok(_) => {
                let index = self.repo.index()?;

                if index.has_conflicts() {
                    self.handle_conflicts(commit_hash, target_branch, index)
                        .await
                } else {
                    self.finalize_success(commit_hash, target_branch, &commit)
                        .await
                }
            }
            Err(e) => {
                error!(error = %e, "Git cherry-pick failed");
                Ok(CherryPickResult {
                    success: false,
                    commit_hash: commit_hash.to_string(),
                    target_branch: target_branch.to_string(),
                    conflicts: vec![],
                    resolutions_applied: 0,
                    new_commit_sha: None,
                    message: format!("Git error: {}", e),
                })
            }
        }
    }

    /// Checkout a branch
    fn checkout_branch(&self, branch_name: &str) -> Result<()> {
        let branch = self
            .repo
            .find_branch(branch_name, git2::BranchType::Local)
            .with_context(|| format!("Branch '{}' not found", branch_name))?;

        let reference = branch.into_reference();
        let tree = reference.peel_to_tree()?;

        self.repo
            .checkout_tree(tree.as_object(), None)
            .context("Failed to checkout tree")?;

        self.repo
            .set_head(reference.name().context("Reference name is not valid UTF-8")?)
            .context("Failed to set HEAD")?;

        info!(branch = %branch_name, "Checked out branch");
        Ok(())
    }

    /// Handle conflicts during cherry-pick
    async fn handle_conflicts(
        &self,
        commit_hash: &str,
        target_branch: &str,
        index: Index,
    ) -> Result<CherryPickResult> {
        let mut conflicts_info = Vec::new();
        let mut resolutions_applied = 0;

        // Get all conflicts
        let conflicts: Vec<_> = index.conflicts()?.collect();

        for conflict in conflicts {
            let conflict = conflict?;

            // Extract conflict information
            let file_path = conflict
                .our
                .as_ref()
                .or(conflict.their.as_ref())
                .map(|e| String::from_utf8_lossy(&e.path).to_string())
                .unwrap_or_else(|| "unknown".to_string());

            let conflict_text = self.extract_conflict_text(&conflict)?;

            info!(file = %file_path, "Processing conflict");

            // Query knowledge base for similar resolutions
            let resolution = self.query_resolution(&conflict_text).await?;

            let (resolution_found, resolution_applied) = if let Some(pattern) = resolution {
                info!(
                    file = %file_path,
                    success_rate = %pattern.success_rate(),
                    "Found similar resolution in knowledge base"
                );

                // Apply the resolution if success rate is high enough
                if pattern.success_rate() >= MIN_SUCCESS_RATE_THRESHOLD {
                    match self.apply_resolution(&file_path, &pattern.resolution).await {
                        Ok(_) => {
                            resolutions_applied += 1;
                            (true, true)
                        }
                        Err(e) => {
                            warn!(error = %e, "Failed to apply resolution");
                            (true, false)
                        }
                    }
                } else {
                    warn!(
                        success_rate = %pattern.success_rate(),
                        "Resolution success rate too low, skipping"
                    );
                    (true, false)
                }
            } else {
                info!(file = %file_path, "No similar resolution found, requires human review");
                (false, false)
            };

            conflicts_info.push(ConflictInfo {
                file_path,
                conflict_text,
                resolution_found,
                resolution_applied,
            });
        }

        let all_resolved = conflicts_info.iter().all(|c| c.resolution_applied);

        Ok(CherryPickResult {
            success: all_resolved,
            commit_hash: commit_hash.to_string(),
            target_branch: target_branch.to_string(),
            conflicts: conflicts_info,
            resolutions_applied,
            new_commit_sha: None,
            message: if all_resolved {
                "All conflicts resolved using learned patterns".to_string()
            } else {
                "Some conflicts require human review".to_string()
            },
        })
    }

    /// Extract conflict text from an index conflict entry
    fn extract_conflict_text(
        &self,
        conflict: &git2::IndexConflict,
    ) -> Result<String> {
        let mut parts = Vec::new();

        if let Some(ancestor) = &conflict.ancestor {
            let blob = self.repo.find_blob(ancestor.id)?;
            parts.push(format!(
                "<<<<<<< ANCESTOR\n{}\n",
                String::from_utf8_lossy(blob.content())
            ));
        }

        if let Some(our) = &conflict.our {
            let blob = self.repo.find_blob(our.id)?;
            parts.push(format!(
                "<<<<<<< OURS\n{}\n",
                String::from_utf8_lossy(blob.content())
            ));
        }

        if let Some(their) = &conflict.their {
            let blob = self.repo.find_blob(their.id)?;
            parts.push(format!(
                ">>>>>>> THEIRS\n{}\n",
                String::from_utf8_lossy(blob.content())
            ));
        }

        Ok(parts.join("=======\n"))
    }

    /// Query the knowledge base for similar resolutions
    async fn query_resolution(&self, conflict_text: &str) -> Result<Option<ResolutionPattern>> {
        // Generate embedding for the conflict
        let embedding = self.generate_embedding(conflict_text).await?;

        // Search for similar conflicts
        let results = self
            .qdrant
            .search_points(
                SearchPointsBuilder::new(COLLECTION_NAME, embedding, 1)
                    .with_payload(true)
                    .score_threshold(MIN_SIMILARITY_SCORE),
            )
            .await?;

        if let Some(point) = results.result.first() {
            let payload = &point.payload;

            // Deserialize the pattern from payload with proper error handling
            let id_str = payload
                .get("id")
                .and_then(|v| v.as_str())
                .context("id field missing or invalid from payload")?;
            let id = Uuid::parse_str(id_str)
                .with_context(|| format!("id field '{}' is not a valid UUID", id_str))?;

            let conflict_signature = payload
                .get("conflict_signature")
                .and_then(|v| v.as_str())
                .context("conflict_signature field missing or invalid from payload")?
                .to_string();

            let file_path = payload
                .get("file_path")
                .and_then(|v| v.as_str())
                .context("file_path field missing or invalid from payload")?
                .to_string();

            let resolution = payload
                .get("resolution")
                .and_then(|v| v.as_str())
                .context("resolution field missing or invalid from payload")?
                .to_string();

            let success_count = payload
                .get("success_count")
                .and_then(|v| v.as_integer())
                .context("success_count field missing or invalid from payload")? as u32;

            let failure_count = payload
                .get("failure_count")
                .and_then(|v| v.as_integer())
                .context("failure_count field missing or invalid from payload")? as u32;

            let created_at_str = payload
                .get("created_at")
                .and_then(|v| v.as_str())
                .context("created_at field missing or invalid from payload")?;
            let created_at = DateTime::parse_from_rfc3339(created_at_str)
                .with_context(|| format!("created_at '{}' is not valid RFC3339", created_at_str))?
                .with_timezone(&Utc);

            let last_used_at_str = payload
                .get("last_used_at")
                .and_then(|v| v.as_str())
                .context("last_used_at field missing or invalid from payload")?;
            let last_used_at = DateTime::parse_from_rfc3339(last_used_at_str)
                .with_context(|| format!("last_used_at '{}' is not valid RFC3339", last_used_at_str))?
                .with_timezone(&Utc);

            let source_commit = payload
                .get("source_commit")
                .and_then(|v| v.as_str())
                .context("source_commit field missing or invalid from payload")?
                .to_string();

            let target_branch = payload
                .get("target_branch")
                .and_then(|v| v.as_str())
                .context("target_branch field missing or invalid from payload")?
                .to_string();

            let pattern = ResolutionPattern {
                id,
                conflict_signature,
                file_path,
                resolution,
                success_count,
                failure_count,
                created_at,
                last_used_at,
                source_commit,
                target_branch,
            };

            info!(
                similarity = %point.score,
                success_rate = %pattern.success_rate(),
                "Found matching resolution pattern"
            );

            return Ok(Some(pattern));
        }

        Ok(None)
    }

    /// Generate embedding using OpenAI API
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        use async_openai::types::{CreateEmbeddingRequestArgs, EmbeddingInput};

        let request = CreateEmbeddingRequestArgs::default()
            .model("text-embedding-3-small")
            .input(EmbeddingInput::String(text.to_string()))
            .build()?;

        let response = self.openai_client.embeddings().create(request).await?;

        Ok(response.data[0].embedding.clone())
    }

    /// Apply a resolution to a file
    async fn apply_resolution(&self, file_path: &str, resolution: &str) -> Result<()> {
        let workdir = self
            .repo
            .workdir()
            .context("Repository has no working directory")?;

        let full_path = workdir.join(file_path);
        std::fs::write(&full_path, resolution)
            .with_context(|| format!("Failed to write resolution to {:?}", full_path))?;

        // Stage the file
        let mut index = self.repo.index()?;
        index.add_path(Path::new(file_path))?;
        index.write()?;

        info!(file = %file_path, "Applied and staged resolution");
        Ok(())
    }

    /// Finalize a successful cherry-pick (no conflicts)
    async fn finalize_success(
        &self,
        commit_hash: &str,
        target_branch: &str,
        original_commit: &git2::Commit<'_>,
    ) -> Result<CherryPickResult> {
        // Create the commit
        let mut index = self.repo.index()?;
        let tree_id = index.write_tree()?;
        let tree = self.repo.find_tree(tree_id)?;

        let head = self.repo.head()?;
        let parent = head.peel_to_commit()?;

        let sig = Signature::now("CherryPickAgent", "agent@lornu.ai")?;

        let message = format!(
            "{}\n\n(cherry picked from commit {})\nCo-Authored-By: CherryPickAgent <agent@lornu.ai>",
            original_commit.message().unwrap_or(""),
            commit_hash
        );

        let new_commit = self.repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            &message,
            &tree,
            &[&parent],
        )?;

        // Clean up cherry-pick state
        self.repo.cleanup_state()?;

        info!(
            new_commit = %new_commit,
            "Cherry-pick completed successfully"
        );

        Ok(CherryPickResult {
            success: true,
            commit_hash: commit_hash.to_string(),
            target_branch: target_branch.to_string(),
            conflicts: vec![],
            resolutions_applied: 0,
            new_commit_sha: Some(new_commit.to_string()),
            message: "Cherry-pick completed successfully".to_string(),
        })
    }

    /// Learn from a human-provided resolution
    ///
    /// Call this after a human resolves a conflict to store the pattern
    pub async fn learn_resolution(
        &self,
        conflict_signature: &str,
        file_path: &str,
        resolution: &str,
        source_commit: &str,
        target_branch: &str,
    ) -> Result<()> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        // Generate embedding
        let embedding = self.generate_embedding(conflict_signature).await?;

        // Create payload with explicit Value type for Qdrant
        let mut payload: HashMap<String, Value> = HashMap::new();
        payload.insert("id".to_string(), Value::from(id.to_string()));
        payload.insert(
            "conflict_signature".to_string(),
            Value::from(conflict_signature.to_string()),
        );
        payload.insert("file_path".to_string(), Value::from(file_path.to_string()));
        payload.insert("resolution".to_string(), Value::from(resolution.to_string()));
        payload.insert("success_count".to_string(), Value::from(1i64));
        payload.insert("failure_count".to_string(), Value::from(0i64));
        payload.insert("source_commit".to_string(), Value::from(source_commit.to_string()));
        payload.insert("target_branch".to_string(), Value::from(target_branch.to_string()));
        payload.insert("created_at".to_string(), Value::from(now.to_rfc3339()));
        payload.insert("last_used_at".to_string(), Value::from(now.to_rfc3339()));

        // Upsert to Qdrant
        self.qdrant
            .upsert_points(
                UpsertPointsBuilder::new(
                    COLLECTION_NAME,
                    vec![PointStruct::new(
                        id.to_string(),
                        embedding,
                        payload,
                    )],
                )
                .wait(true),
            )
            .await?;

        info!(
            id = %id,
            file = %file_path,
            "Learned new resolution pattern"
        );

        Ok(())
    }

    /// Train the agent on historical PR merge resolutions
    pub async fn train_on_history(&self, depth: u32) -> Result<u32> {
        info!(depth = %depth, "Training on git history");

        let mut walk = self.repo.revwalk()?;
        walk.push_head()?;

        let mut patterns_learned = 0;

        for (i, oid_result) in walk.enumerate() {
            if i as u32 >= depth {
                break;
            }

            let oid = oid_result?;
            let commit = self.repo.find_commit(oid)?;

            // Look for merge commits (they have > 1 parent)
            if commit.parent_count() > 1 {
                // This is a merge commit - analyze the resolution
                if let Some(message) = commit.message() {
                    if message.contains("Merge") || message.contains("cherry") {
                        // Extract diff between parents
                        let parent1 = commit.parent(0)?;
                        let parent2 = commit.parent(1)?;

                        // Get diff between the two parents to find conflicts
                        let parent2_tree = parent2.tree()?;
                        let diff = self.repo.diff_tree_to_tree(
                            Some(&parent1.tree()?),
                            Some(&parent2_tree),
                            None,
                        )?;

                        // Extract conflicts and resolutions from the merge
                        let mut conflict_files = Vec::new();
                        diff.foreach(
                            &mut |delta, _| {
                                if let Some(path) = delta.new_file().path() {
                                    conflict_files.push(path.to_path_buf());
                                }
                                true
                            },
                            None,
                            None,
                            None,
                        )?;

                        // For each conflicted file, extract the resolution
                        let commit_tree = commit.tree()?;
                        let parent1_tree = parent1.tree()?;
                        
                        for file_path in conflict_files {
                            // Get the resolved version from the merge commit
                            if let Ok(blob_entry) = commit_tree.get_path(&file_path) {
                                if let Ok(resolved_blob) = self.repo.find_blob(blob_entry.id()) {
                                    let resolution = String::from_utf8_lossy(resolved_blob.content()).to_string();
                                    
                                    // Extract conflict signature by comparing parent versions
                                    let mut conflict_signature = String::new();
                                    
                                    // Get parent1 version
                                    if let Ok(blob1_entry) = parent1_tree.get_path(&file_path) {
                                        if let Ok(blob1) = self.repo.find_blob(blob1_entry.id()) {
                                            conflict_signature.push_str("<<<<<<< PARENT1\n");
                                            conflict_signature.push_str(&String::from_utf8_lossy(blob1.content()));
                                            conflict_signature.push_str("\n=======\n");
                                        }
                                    }
                                    
                                    // Get parent2 version
                                    if let Ok(blob2_entry) = parent2_tree.get_path(&file_path) {
                                        if let Ok(blob2) = self.repo.find_blob(blob2_entry.id()) {
                                            conflict_signature.push_str(&String::from_utf8_lossy(blob2.content()));
                                            conflict_signature.push_str("\n>>>>>>> PARENT2\n");
                                        }
                                    }
                                    
                                    if !conflict_signature.is_empty() && !resolution.is_empty() {
                                        // Learn this resolution pattern
                                        if let Err(e) = self.learn_resolution(
                                            &conflict_signature,
                                            &file_path.to_string_lossy(),
                                            &resolution,
                                            &oid.to_string(),
                                            "history",
                                        ).await {
                                            warn!(error = %e, file = %file_path.display(), "Failed to learn resolution from history");
                                        } else {
                                            patterns_learned += 1;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        info!(patterns = %patterns_learned, "Training complete");
        Ok(patterns_learned)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolution_pattern_success_rate() {
        let pattern = ResolutionPattern {
            id: Uuid::new_v4(),
            conflict_signature: "test".to_string(),
            file_path: "test.rs".to_string(),
            resolution: "resolved".to_string(),
            success_count: 8,
            failure_count: 2,
            created_at: Utc::now(),
            last_used_at: Utc::now(),
            source_commit: "abc123".to_string(),
            target_branch: "main".to_string(),
        };

        assert!((pattern.success_rate() - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_resolution_pattern_zero_usage() {
        let pattern = ResolutionPattern {
            id: Uuid::new_v4(),
            conflict_signature: "test".to_string(),
            file_path: "test.rs".to_string(),
            resolution: "resolved".to_string(),
            success_count: 0,
            failure_count: 0,
            created_at: Utc::now(),
            last_used_at: Utc::now(),
            source_commit: "abc123".to_string(),
            target_branch: "main".to_string(),
        };

        assert_eq!(pattern.success_rate(), 0.0);
    }
}
