//! Cloudflare DNS Agent
//!
//! A Rust agent that manages Cloudflare DNS records using secrets from
//! Google Secret Manager. Zero hardcoded credentials.
//!
//! # Usage
//! ```bash
//! # List DNS records
//! cloudflare-dns list --zone lornu.ai
//!
//! # Create A record
//! cloudflare-dns create --zone lornu.ai --name api --type A --content 1.2.3.4
//!
//! # Delete record
//! cloudflare-dns delete --zone lornu.ai --record-id abc123
//! ```

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use tracing::{info, warn, Level};
use tracing_subscriber::FmtSubscriber;

mod secrets;
mod cloudflare;

use secrets::SecretManager;
use cloudflare::CloudflareClient;

// ============================================================
// CLI Definition
// ============================================================

#[derive(Parser)]
#[command(name = "cloudflare-dns")]
#[command(about = "Lornu AI Cloudflare DNS Agent", long_about = None)]
#[command(version)]
struct Cli {
    /// GCP Project ID for Secret Manager
    #[arg(long, env = "GCP_PROJECT_ID")]
    project: String,

    /// Secret name for Cloudflare API token
    #[arg(long, default_value = "cloudflare-api-token")]
    secret_name: String,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List DNS records for a zone
    List {
        /// Zone name (e.g., lornu.ai)
        #[arg(long)]
        zone: String,
    },

    /// Create a new DNS record
    Create {
        /// Zone name
        #[arg(long)]
        zone: String,

        /// Record name (e.g., api, www)
        #[arg(long)]
        name: String,

        /// Record type (A, AAAA, CNAME, TXT, MX)
        #[arg(long, name = "type")]
        record_type: String,

        /// Record content (IP address, hostname, or text)
        #[arg(long)]
        content: String,

        /// TTL in seconds (default: auto)
        #[arg(long, default_value = "1")]
        ttl: u32,

        /// Enable Cloudflare proxy
        #[arg(long)]
        proxied: bool,
    },

    /// Delete a DNS record
    Delete {
        /// Zone name
        #[arg(long)]
        zone: String,

        /// Record ID to delete
        #[arg(long)]
        record_id: String,
    },

    /// Update an existing DNS record
    Update {
        /// Zone name
        #[arg(long)]
        zone: String,

        /// Record ID to update
        #[arg(long)]
        record_id: String,

        /// New content
        #[arg(long)]
        content: String,
    },
}

// ============================================================
// Main Entry Point
// ============================================================

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let level = if cli.verbose { Level::DEBUG } else { Level::INFO };
    let subscriber = FmtSubscriber::builder()
        .with_max_level(level)
        .with_target(false)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("🚀 Cloudflare DNS Agent starting...");

    // Fetch API token from Google Secret Manager
    info!("🔐 Fetching credentials from GSM (project: {})", cli.project);
    let secret_manager = SecretManager::new(&cli.project).await?;
    let api_token = secret_manager
        .get_secret(&cli.secret_name)
        .await
        .context("Failed to fetch Cloudflare API token from GSM")?;

    info!("✅ Credentials loaded successfully");

    // Initialize Cloudflare client
    let cf = CloudflareClient::new(api_token);

    // Execute command
    match cli.command {
        Commands::List { zone } => {
            info!("📋 Listing DNS records for zone: {}", zone);
            let records = cf.list_records(&zone).await?;

            println!("\n{:<36} {:<6} {:<20} {:<40}", "ID", "TYPE", "NAME", "CONTENT");
            println!("{}", "-".repeat(100));

            let count = records.len();
            for record in &records {
                println!(
                    "{:<36} {:<6} {:<20} {:<40}",
                    record.id,
                    record.record_type,
                    record.name,
                    truncate(&record.content, 40)
                );
            }

            info!("✅ Listed {} records", count);
        }

        Commands::Create { zone, name, record_type, content, ttl, proxied } => {
            info!("➕ Creating DNS record: {}.{} -> {}", name, zone, content);

            let record = cf.create_record(&zone, &name, &record_type, &content, ttl, proxied).await?;

            println!("✅ Created record: {}", record.id);
            info!("Record created successfully");
        }

        Commands::Delete { zone, record_id } => {
            warn!("🗑️  Deleting DNS record: {}", record_id);

            cf.delete_record(&zone, &record_id).await?;

            println!("✅ Deleted record: {}", record_id);
            info!("Record deleted successfully");
        }

        Commands::Update { zone, record_id, content } => {
            info!("✏️  Updating DNS record: {} -> {}", record_id, content);

            cf.update_record(&zone, &record_id, &content).await?;

            println!("✅ Updated record: {}", record_id);
            info!("Record updated successfully");
        }
    }

    Ok(())
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() > max {
        format!("{}...", &s[..max - 3])
    } else {
        s.to_string()
    }
}
