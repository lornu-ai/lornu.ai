//! GitHub Bot Tools Library
//!
//! Rust utilities for GitHub App authentication and PR operations.
//!
//! ## Binaries
//!
//! - `get-token`: Generate GitHub App installation access tokens
//! - `approve-pr`: Approve pull requests using a bot token
//!
//! ## Usage
//!
//! These tools are designed to work together in a no-shell automation pipeline:
//!
//! 1. Use TypeScript `create-manifest.ts` to generate the GitHub App registration URL
//! 2. Use TypeScript `upload-secrets.ts` to store credentials in GCP Secret Manager
//! 3. Use Rust `get-token` to generate short-lived installation tokens
//! 4. Use Rust `approve-pr` to approve PRs with the generated token
//!
//! ## Example Pipeline
//!
//! ```bash
//! # Generate installation token
//! TOKEN=$(get-token \
//!   --app-id $GITHUB_APP_ID \
//!   --private-key-path /path/to/key.pem \
//!   --installation-id $INSTALLATION_ID)
//!
//! # Approve PR
//! approve-pr \
//!   --repo lornu-ai/lornu.ai \
//!   --token $TOKEN \
//!   --pr-number 123
//! ```

pub mod auth;
pub mod pr;
