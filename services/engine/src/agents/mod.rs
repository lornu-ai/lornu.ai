//! Agent modules for the Lornu AI Engine
//!
//! This module contains various agent implementations including:
//! - `cherry_pick`: Context-aware cherry-pick with learning from past conflicts
//! - `executor`: Task execution and orchestration
//! - `lifecycle`: Secret lifecycle management and cleanup

pub mod cherry_pick;
pub mod executor;
pub mod lifecycle;

pub use cherry_pick::CherryPickAgent;
pub use lifecycle::{TempFileGuard, cleanup_sensitive_files, exec_with_secret_env, exec_with_secret_stdin};
