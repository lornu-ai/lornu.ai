//! SSH Key Agent - Standalone Binary
//!
//! Generates Ed25519 SSH key pairs, stores private keys in GCP Secret Manager,
//! and outputs public keys for GitHub Deploy Keys.
//!
//! ## Usage
//!
//! ```bash
//! # Generate a key with custom name
//! ssh-key-agent --project-id gcp-lornu-ai --secret-name my-deploy-key --comment "deploy@lornu.ai"
//!
//! # Generate a GitHub Deploy Key (with standard labels)
//! ssh-key-agent --project-id gcp-lornu-ai --deploy-key lornu-ai/repo --environment production
//!
//! # Output as JSON
//! ssh-key-agent --project-id gcp-lornu-ai --secret-name my-key --comment "test" --output json
//! ```

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use lornu_engine::agents::ssh_key::{KeyGenerationOptions, SshKeyAgent};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

/// SSH Key Agent - Generate SSH keys and store in GCP Secret Manager
#[derive(Parser, Debug)]
#[command(name = "ssh-key-agent", version, about)]
struct Args {
    /// GCP Project ID
    #[arg(long, env = "GCP_PROJECT_ID")]
    project_id: String,

    /// Secret name in GCP Secret Manager
    #[arg(long, required_unless_present = "deploy_key")]
    secret_name: Option<String>,

    /// Comment to embed in the public key
    #[arg(long, required_unless_present = "deploy_key")]
    comment: Option<String>,

    /// Generate a GitHub Deploy Key for this repository (e.g., "lornu-ai/repo")
    #[arg(long, conflicts_with_all = ["secret_name", "comment"])]
    deploy_key: Option<String>,

    /// Environment for deploy key (used with --deploy-key)
    #[arg(long, default_value = "production", requires = "deploy_key")]
    environment: String,

    /// Output format
    #[arg(long, value_enum, default_value = "text")]
    output: OutputFormat,

    /// Additional labels (key=value format, can be repeated)
    #[arg(long, short = 'l')]
    label: Vec<String>,
}

#[derive(Debug, Clone, ValueEnum)]
enum OutputFormat {
    /// Human-readable text output
    Text,
    /// JSON output
    Json,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .with_writer(std::io::stderr)
        .init();

    let args = Args::parse();

    info!(
        project_id = %args.project_id,
        "Starting SSH Key Agent"
    );

    // Initialize the agent
    let agent = SshKeyAgent::new(&args.project_id)
        .await
        .context("Failed to initialize SSH Key Agent")?;

    // Generate the key
    let result = if let Some(repo) = &args.deploy_key {
        // Deploy key mode
        info!(
            repository = %repo,
            environment = %args.environment,
            "Generating GitHub Deploy Key"
        );
        agent.generate_deploy_key(repo, &args.environment).await
    } else {
        // Custom key mode
        let secret_name = args.secret_name.as_ref().unwrap();
        let comment = args.comment.as_ref().unwrap();

        info!(
            secret_name = %secret_name,
            comment = %comment,
            "Generating SSH key"
        );

        // Build options with any additional labels
        let mut options = KeyGenerationOptions::new(comment).with_standard_labels();

        for label in &args.label {
            if let Some((key, value)) = label.split_once('=') {
                options = options.label(key, value);
            } else {
                anyhow::bail!("Invalid label format: {}. Use key=value format.", label);
            }
        }

        agent.generate(secret_name, options).await
    }
    .context("Failed to generate SSH key")?;

    // Output the result
    match args.output {
        OutputFormat::Text => {
            println!("{}", result.display());
        }
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&result).context("Failed to serialize result")?
            );
        }
    }

    info!(
        fingerprint = %result.fingerprint,
        secret_name = %result.secret_name,
        "SSH key generated successfully"
    );

    Ok(())
}
