//! Pull Request Operations
//!
//! Types and utilities for managing GitHub pull requests.

/// Result of a PR approval operation
#[derive(Debug, Clone)]
pub struct ApprovalResult {
    /// PR number
    pub pr_number: u64,
    /// Whether the approval succeeded
    pub success: bool,
    /// Review ID if successful
    pub review_id: Option<u64>,
    /// Error message if failed
    pub error: Option<String>,
}

impl ApprovalResult {
    /// Create a successful approval result
    pub fn success(pr_number: u64, review_id: u64) -> Self {
        Self {
            pr_number,
            success: true,
            review_id: Some(review_id),
            error: None,
        }
    }

    /// Create a failed approval result
    pub fn failure(pr_number: u64, error: impl Into<String>) -> Self {
        Self {
            pr_number,
            success: false,
            review_id: None,
            error: Some(error.into()),
        }
    }
}
