//! Multi-Cloud Service Discovery Agent
//!
//! Issue #119: Building on the Lornu AI "last mile" objectives, this agent
//! is a Global Control Plane that synchronizes cross-cloud state into a
//! unified entry point.
//!
//! ## Architecture
//!
//! - **Federated Identity**: Workload Identity across AWS, Azure, GCP
//! - **Discovery Engine**: Trait-based factory pattern for cloud-agnostic discovery
//! - **Cross-Cloud Reconciler**: Learns and self-heals across providers
//! - **99.9% SLA Target**: Predictive multi-cloud switching
//!
//! ## Security (Zero-Trust)
//!
//! Uses Workload Identity Federation - no static keys:
//! - AWS: EKS OIDC -> IAM Role via STS AssumeRoleWithWebIdentity
//! - GCP: Workload Identity Pools
//! - Azure: Microsoft Entra ID with Federated Identity Credentials

mod identity;
mod discovery;
mod reconciler;
mod experience;

pub use identity::*;
pub use discovery::*;
pub use reconciler::*;
pub use experience::*;
