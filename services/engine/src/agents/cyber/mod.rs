//! Cyber Security Agents
//!
//! Security-focused agents for IAM hardening, secret management,
//! and Zero Trust compliance.
//!
//! ## Available Agents
//!
//! - [`ZeroTrustAgent`]: Scans for and proposes corrections to over-privileged
//!   IAM roles, stale secrets, and long-lived credentials.
//!
//! - [`Remediator`]: Generates GitHub PRs to apply IAM corrections via CDK8s
//!   TypeScript updates (GitOps-compliant).
//!
//! ## Example Usage
//!
//! ```ignore
//! use lornu_engine::agents::cyber::{ZeroTrustAgent, Remediator};
//!
//! // Initialize the Zero Trust agent
//! let agent = ZeroTrustAgent::new("my-project", "http://qdrant:6333", openai_key).await?;
//!
//! // Run a scan
//! let result = agent.scan().await?;
//!
//! // Create PRs for corrections
//! let remediator = Remediator::new("lornu-ai", "private-lornu-ai")?;
//! let pr_result = remediator.create_remediation_pr(&result.corrections).await?;
//! ```

pub mod remediator;
pub mod types;
pub mod zero_trust;

pub use remediator::Remediator;
pub use types::*;
pub use zero_trust::ZeroTrustAgent;
