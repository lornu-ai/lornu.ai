//! GitHub Team Management Tool
//!
//! Manages GitHub organization teams using the octocrab library.
//! Authentication: Uses GITHUB_TEAM_PAT environment variable (synced from GSM via K8s Secret).

use octocrab::Octocrab;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

/// GitHub Team Management Tool
///
/// Provides team management capabilities for GitHub organizations.
/// The PAT must have "Members: Read/Write" permission on the organization.
pub struct GitHubTeamTool {
    /// GitHub organization name
    pub org: String,
    /// Octocrab client instance
    client: Octocrab,
}

/// Team member role
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TeamRole {
    Member,
    Maintainer,
}

impl Default for TeamRole {
    fn default() -> Self {
        Self::Member
    }
}

/// Result of a team operation
#[derive(Debug, Serialize)]
pub struct TeamOperationResult {
    pub success: bool,
    pub message: String,
    pub team_slug: Option<String>,
    pub username: Option<String>,
}

impl GitHubTeamTool {
    /// Create a new GitHubTeamTool instance.
    ///
    /// # Environment Variables
    /// - `GITHUB_TEAM_PAT`: GitHub Personal Access Token with org:admin scope
    ///
    /// # Errors
    /// Returns an error if GITHUB_TEAM_PAT is not set or client creation fails.
    pub fn new(org: impl Into<String>) -> anyhow::Result<Self> {
        // Pull token from K8s Secret (synced from GSM)
        let token = std::env::var("GITHUB_TEAM_PAT")
            .map_err(|_| anyhow::anyhow!("GITHUB_TEAM_PAT environment variable not set"))?;

        let client = Octocrab::builder()
            .personal_token(token)
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to create GitHub client: {}", e))?;

        Ok(Self {
            org: org.into(),
            client,
        })
    }

    /// Add or update a member in a team.
    ///
    /// # Arguments
    /// * `team_slug` - The slug of the team (e.g., "engineering")
    /// * `username` - The GitHub username to add
    /// * `role` - The role to assign (member or maintainer)
    pub async fn add_member_to_team(
        &self,
        team_slug: &str,
        username: &str,
        role: TeamRole,
    ) -> anyhow::Result<TeamOperationResult> {
        info!(
            org = %self.org,
            team = %team_slug,
            user = %username,
            role = ?role,
            "Adding member to team"
        );

        let role_str = match role {
            TeamRole::Member => "member",
            TeamRole::Maintainer => "maintainer",
        };

        // Use the Teams API to add/update membership
        let response: Result<serde_json::Value, _> = self
            .client
            .put(
                format!(
                    "/orgs/{}/teams/{}/memberships/{}",
                    self.org, team_slug, username
                ),
                Some(&serde_json::json!({ "role": role_str })),
            )
            .await;

        match response {
            Ok(_) => Ok(TeamOperationResult {
                success: true,
                message: format!("Added {} to team {} as {}", username, team_slug, role_str),
                team_slug: Some(team_slug.to_string()),
                username: Some(username.to_string()),
            }),
            Err(e) => {
                warn!(error = %e, "Failed to add member to team");
                Ok(TeamOperationResult {
                    success: false,
                    message: format!("Failed to add {} to team {}: {}", username, team_slug, e),
                    team_slug: Some(team_slug.to_string()),
                    username: Some(username.to_string()),
                })
            }
        }
    }

    /// Remove a member from a team.
    ///
    /// # Arguments
    /// * `team_slug` - The slug of the team
    /// * `username` - The GitHub username to remove
    pub async fn remove_member_from_team(
        &self,
        team_slug: &str,
        username: &str,
    ) -> anyhow::Result<TeamOperationResult> {
        info!(
            org = %self.org,
            team = %team_slug,
            user = %username,
            "Removing member from team"
        );

        let response: Result<serde_json::Value, _> = self
            .client
            .delete(
                format!(
                    "/orgs/{}/teams/{}/memberships/{}",
                    self.org, team_slug, username
                ),
                None::<&()>,
            )
            .await;

        match response {
            Ok(_) => Ok(TeamOperationResult {
                success: true,
                message: format!("Removed {} from team {}", username, team_slug),
                team_slug: Some(team_slug.to_string()),
                username: Some(username.to_string()),
            }),
            Err(e) => {
                warn!(error = %e, "Failed to remove member from team");
                Ok(TeamOperationResult {
                    success: false,
                    message: format!(
                        "Failed to remove {} from team {}: {}",
                        username, team_slug, e
                    ),
                    team_slug: Some(team_slug.to_string()),
                    username: Some(username.to_string()),
                })
            }
        }
    }

    /// List all teams in the organization.
    pub async fn list_teams(&self) -> anyhow::Result<Vec<String>> {
        info!(org = %self.org, "Listing teams");

        let teams = self
            .client
            .teams(&self.org)
            .list()
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to list teams: {}", e))?;

        let team_slugs: Vec<String> = teams.items.iter().map(|t| t.slug.clone()).collect();

        Ok(team_slugs)
    }

    /// List members of a specific team.
    pub async fn list_team_members(&self, team_slug: &str) -> anyhow::Result<Vec<String>> {
        info!(org = %self.org, team = %team_slug, "Listing team members");

        let members = self
            .client
            .teams(&self.org)
            .members(team_slug)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to list team members: {}", e))?;

        let usernames: Vec<String> = members.items.iter().map(|m| m.login.clone()).collect();

        Ok(usernames)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_team_role_default() {
        let role = TeamRole::default();
        assert!(matches!(role, TeamRole::Member));
    }

    #[test]
    fn test_operation_result() {
        let result = TeamOperationResult {
            success: true,
            message: "Test".to_string(),
            team_slug: Some("eng".to_string()),
            username: Some("testuser".to_string()),
        };
        assert!(result.success);
    }
}
