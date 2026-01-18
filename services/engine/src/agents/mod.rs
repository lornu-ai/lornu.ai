//! Agent modules for the Lornu AI Engine
//!
//! This module contains various agent implementations including:
//! - `cherry_pick`: Context-aware cherry-pick with learning from past conflicts
//! - `cyber`: Security agents (Zero Trust IAM hardening)
//! - `dns_sync`: Multi-cloud DNS orchestration (Issue #118)
//! - `executor`: Task execution and orchestration
//! - `lifecycle`: Secret lifecycle management and cleanup
//! - `service_discovery`: Multi-cloud service discovery with federated identity (Issue #119)
//! - `ssh_key`: SSH key generation and GCP Secret Manager storage (Issue #176)

pub mod cherry_pick;
pub mod cyber;
pub mod dns_sync;
pub mod executor;
pub mod lifecycle;
pub mod service_discovery;
#[cfg(feature = "ssh-key-gen")]
pub mod ssh_key;

