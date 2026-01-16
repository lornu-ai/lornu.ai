//! Agent modules for the Lornu AI Engine
//!
//! This module contains various agent implementations including:
//! - `cherry_pick`: Context-aware cherry-pick with learning from past conflicts
//! - `cyber`: Security agents (Zero Trust IAM hardening)
//! - `dns_sync`: Multi-cloud DNS orchestration (Issue #118)
//! - `executor`: Task execution and orchestration
//! - `lifecycle`: Secret lifecycle management and cleanup
//! - `service_discovery`: Multi-cloud service discovery with federated identity (Issue #119)

pub mod cherry_pick;
pub mod cyber;
pub mod dns_sync;
pub mod executor;
pub mod lifecycle;
pub mod service_discovery;

pub use cherry_pick::CherryPickAgent;
pub use cyber::ZeroTrustAgent;
pub use dns_sync::MultiCloudDnsSyncAgent;
pub use service_discovery::{CrossCloudReconciler, FederatedIdentityManager, MultiCloudDiscovery};
pub use lifecycle::{TempFileGuard, cleanup_sensitive_files, exec_with_secret_env, exec_with_secret_stdin};
