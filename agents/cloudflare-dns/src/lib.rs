//! Cloudflare DNS Agent Library
//!
//! Provides type-safe access to Cloudflare DNS management
//! with Google Secret Manager integration.

pub mod cloudflare;
pub mod secrets;

pub use cloudflare::CloudflareClient;
pub use secrets::SecretManager;
