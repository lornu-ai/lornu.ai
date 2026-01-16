//! Multi-Cloud DNS Sync Agent
//!
//! Issue #118: Evolve ai-agent-dns-sync into a Global Control Plane.
//! Using an AWS Hub command center, it orchestrates endpoints across AWS, Azure,
//! and GCP via Cloudflare's intelligent global entry point.
//!
//! ## Architecture
//!
//! - **Cloud Providers**: AWS (ALB), Azure (Front Door/Public IP), GCP (Global LB)
//! - **Global Entry Point**: Cloudflare Load Balancer Pool
//! - **Control Plane**: K8s-based orchestration via Crossplane
//!
//! ## Security
//!
//! - Zero-Trust: All credentials fetched from Secret Manager via ADC
//! - Workload Identity Federation across all clouds

mod types;
mod providers;
mod orchestrator;
pub mod cloudflare;
pub mod cloudflare_permissions;

#[allow(unused_imports)]
pub use orchestrator::MultiCloudDnsSyncAgent;
#[allow(unused_imports)]
pub use cloudflare::{CloudflareDnsClient, DnsRecordType, DnsRecordSyncResult, IngressDnsMapping};
#[allow(unused_imports)]
pub use cloudflare_permissions::{CloudflareConfig, TokenPolicy, permission_groups};
