//! Agent modules for the Lornu AI Engine
//!
//! This module contains various agent implementations including:
//! - `cherry_pick`: Context-aware cherry-pick with learning from past conflicts
//! - `executor`: Task execution and orchestration

pub mod cherry_pick;
pub mod executor;

pub use cherry_pick::CherryPickAgent;
