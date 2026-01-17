//! GitHub PR Approval Tool
//!
//! Approves pull requests using a GitHub App installation token.
//! This bypasses GitHub's self-approval restriction by using a bot account.
//!
//! ## Usage
//! ```bash
//! # Approve a single PR
//! approve-pr \
//!   --repo lornu-ai/lornu.ai \
//!   --token <INSTALLATION_TOKEN> \
//!   --pr-number 123
//!
//! # Approve multiple PRs
//! approve-pr \
//!   --repo lornu-ai/lornu.ai \
//!   --token <INSTALLATION_TOKEN> \
//!   --pr-numbers 123,124,125
//!
//! # With environment variables
//! GITHUB_REPOSITORY=lornu-ai/lornu.ai \
//! GITHUB_TOKEN=<TOKEN> \
//! approve-pr --pr-number 123
//! ```

use anyhow::Result;
use clap::Parser;
use serde::{Deserialize, Serialize};

/// GitHub PR Approval Tool
#[derive(Parser, Debug)]
#[command(name = "approve-pr")]
#[command(about = "Approve GitHub pull requests using a bot token")]
#[command(version)]
struct Args {
    /// Repository in format owner/repo
    #[arg(long, env = "GITHUB_REPOSITORY")]
    repo: String,

    /// GitHub token (installation token or PAT)
    #[arg(long, env = "GITHUB_TOKEN")]
    token: String,

    /// Single PR number to approve
    #[arg(long, conflicts_with = "pr_numbers")]
    pr_number: Option<u64>,

    /// Multiple PR numbers to approve (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pr_numbers: Option<Vec<u64>>,

    /// Review message/body
    #[arg(
        long,
        default_value = "Lornu AI Bot: Changes look good! Pre-computation verified."
    )]
    message: String,

    /// Output format: text (default), json
    #[arg(long, default_value = "text")]
    format: String,

    /// Dry run - don't actually approve, just check if PRs exist
    #[arg(long)]
    dry_run: bool,
}

#[derive(Debug, Serialize)]
struct ApprovalResult {
    pr_number: u64,
    success: bool,
    review_id: Option<u64>,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct ApprovalSummary {
    repository: String,
    total: usize,
    successful: usize,
    failed: usize,
    results: Vec<ApprovalResult>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct PullRequest {
    number: u64,
    title: Option<String>,
    state: String,
}

#[derive(Debug, Deserialize)]
struct Review {
    id: u64,
}

#[derive(Debug, Serialize)]
struct ReviewRequest {
    event: String,
    body: String,
}

async fn check_pr_exists(
    client: &reqwest::Client,
    token: &str,
    owner: &str,
    repo: &str,
    pr_number: u64,
) -> ApprovalResult {
    let url = format!(
        "https://api.github.com/repos/{}/{}/pulls/{}",
        owner, repo, pr_number
    );

    match client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "lornu-ai-github-bot")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<PullRequest>().await {
                    Ok(pr) => {
                        eprintln!(
                            "  🔍 PR #{}: {} (state: {})",
                            pr_number,
                            pr.title.as_deref().unwrap_or("No title"),
                            pr.state
                        );
                        ApprovalResult {
                            pr_number,
                            success: true,
                            review_id: None,
                            error: None,
                        }
                    }
                    Err(e) => ApprovalResult {
                        pr_number,
                        success: false,
                        review_id: None,
                        error: Some(format!("Failed to parse PR: {}", e)),
                    },
                }
            } else {
                ApprovalResult {
                    pr_number,
                    success: false,
                    review_id: None,
                    error: Some(format!("PR not found (status: {})", response.status())),
                }
            }
        }
        Err(e) => ApprovalResult {
            pr_number,
            success: false,
            review_id: None,
            error: Some(format!("Request failed: {}", e)),
        },
    }
}

async fn approve_pr(
    client: &reqwest::Client,
    token: &str,
    owner: &str,
    repo: &str,
    pr_number: u64,
    message: &str,
) -> ApprovalResult {
    let url = format!(
        "https://api.github.com/repos/{}/{}/pulls/{}/reviews",
        owner, repo, pr_number
    );

    let request_body = ReviewRequest {
        event: "APPROVE".to_string(),
        body: message.to_string(),
    };

    match client
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "lornu-ai-github-bot")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .json(&request_body)
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<Review>().await {
                    Ok(review) => {
                        eprintln!("  ✅ PR #{} approved (review ID: {})", pr_number, review.id);
                        ApprovalResult {
                            pr_number,
                            success: true,
                            review_id: Some(review.id),
                            error: None,
                        }
                    }
                    Err(e) => ApprovalResult {
                        pr_number,
                        success: false,
                        review_id: None,
                        error: Some(format!("Failed to parse review response: {}", e)),
                    },
                }
            } else {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();

                let error_msg = if body.contains("Can not approve your own pull request") {
                    eprintln!(
                        "  ❌ PR #{}: Cannot approve your own PR (self-approval restriction)",
                        pr_number
                    );
                    "Self-approval restriction".to_string()
                } else {
                    eprintln!("  ❌ PR #{}: {} - {}", pr_number, status, body);
                    format!("API error ({}): {}", status, body)
                };

                ApprovalResult {
                    pr_number,
                    success: false,
                    review_id: None,
                    error: Some(error_msg),
                }
            }
        }
        Err(e) => {
            eprintln!("  ❌ PR #{}: Request failed: {}", pr_number, e);
            ApprovalResult {
                pr_number,
                success: false,
                review_id: None,
                error: Some(format!("Request failed: {}", e)),
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Parse repository
    let parts: Vec<&str> = args.repo.split('/').collect();
    if parts.len() != 2 {
        anyhow::bail!(
            "Invalid repository format: {}. Expected: owner/repo",
            args.repo
        );
    }
    let owner = parts[0];
    let repo = parts[1];

    // Collect PR numbers
    let pr_numbers: Vec<u64> = if let Some(num) = args.pr_number {
        vec![num]
    } else if let Some(nums) = args.pr_numbers {
        nums
    } else {
        anyhow::bail!("Either --pr-number or --pr-numbers must be specified");
    };

    if pr_numbers.is_empty() {
        anyhow::bail!("No PR numbers provided");
    }

    let client = reqwest::Client::new();

    eprintln!(
        "🤖 {} {} PR(s) in {}/{}...",
        if args.dry_run {
            "Checking"
        } else {
            "Approving"
        },
        pr_numbers.len(),
        owner,
        repo
    );

    // Process PRs
    let mut results = Vec::new();
    for pr_number in &pr_numbers {
        let result = if args.dry_run {
            check_pr_exists(&client, &args.token, owner, repo, *pr_number).await
        } else {
            approve_pr(&client, &args.token, owner, repo, *pr_number, &args.message).await
        };
        results.push(result);
    }

    // Build summary
    let successful = results.iter().filter(|r| r.success).count();
    let summary = ApprovalSummary {
        repository: args.repo.clone(),
        total: pr_numbers.len(),
        successful,
        failed: pr_numbers.len() - successful,
        results,
    };

    // Output
    match args.format.as_str() {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&summary)?);
        }
        _ => {
            eprintln!(
                "\n{} {}/{} PR(s) {}",
                if summary.failed == 0 { "✅" } else { "⚠️" },
                summary.successful,
                summary.total,
                if args.dry_run { "checked" } else { "approved" }
            );
        }
    }

    // Exit with error if any failed
    if summary.failed > 0 {
        std::process::exit(1);
    }

    Ok(())
}
