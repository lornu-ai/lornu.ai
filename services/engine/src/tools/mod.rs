//! Agent Tools
//!
//! Secure tools that agents can use without accessing sensitive credentials.
//! All tools use ADC (Application Default Credentials) or K8s Secrets for authentication.

pub mod cloudflare;
pub mod github;

pub use cloudflare::CloudflareTool;
