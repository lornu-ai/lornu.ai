//! DNS Sync Agent - Standalone Binary
//!
//! Syncs K8s Ingress IPs to Cloudflare DNS records.
//! Replaces the Python cloudflare-dns-agent with a high-performance Rust implementation.

use anyhow::{Context, Result};
use clap::Parser;
use kube::{Api, Client};
use k8s_openapi::api::networking::v1::Ingress;
use std::env;
use std::time::Duration;
use tokio::time::interval;
use tracing::{info, warn, error, Level};
use tracing_subscriber::FmtSubscriber;

use lornu_engine::agents::dns_sync::{CloudflareDnsClient, DnsRecordType, IngressDnsMapping};

/// DNS Sync Agent - Syncs K8s Ingress IPs to Cloudflare DNS
#[derive(Parser, Debug)]
#[command(name = "dns-sync-agent", version, about)]
struct Args {
    /// Cloudflare Zone ID
    #[arg(long, env = "CLOUDFLARE_ZONE_ID")]
    zone_id: String,

    /// Sync interval in seconds
    #[arg(long, default_value = "60", env = "SYNC_INTERVAL")]
    interval: u64,

    /// Run once and exit (for CronJob mode)
    #[arg(long, default_value = "false")]
    once: bool,

    /// Dry run - don't actually update DNS
    #[arg(long, default_value = "false")]
    dry_run: bool,

    /// Label selector for ingresses to sync (e.g., "lornu.ai/dns-sync=enabled")
    #[arg(long, default_value = "lornu.ai/dns-sync=enabled", env = "INGRESS_LABEL_SELECTOR")]
    label_selector: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .json()
        .init();

    let args = Args::parse();

    info!(
        zone_id = %args.zone_id,
        interval = args.interval,
        label_selector = %args.label_selector,
        "Starting DNS Sync Agent (Rust)"
    );

    // Get Cloudflare API token from environment or Secret Manager
    let api_token = get_cloudflare_token().await?;

    // Initialize Cloudflare client
    let cf_client = CloudflareDnsClient::new(api_token, args.zone_id.clone())?;

    // Initialize K8s client
    let k8s_client = Client::try_default()
        .await
        .context("Failed to create K8s client")?;

    if args.once {
        // Run once and exit
        run_sync(&cf_client, &k8s_client, &args).await?;
    } else {
        // Run in loop
        let mut ticker = interval(Duration::from_secs(args.interval));

        loop {
            ticker.tick().await;

            if let Err(e) = run_sync(&cf_client, &k8s_client, &args).await {
                error!(error = %e, "Sync cycle failed");
            }
        }
    }

    Ok(())
}

/// Run a single sync cycle
async fn run_sync(
    cf_client: &CloudflareDnsClient,
    k8s_client: &Client,
    args: &Args,
) -> Result<()> {
    info!("Starting DNS sync cycle");

    // Discover ingresses with the dns-sync label
    let ingress_api: Api<Ingress> = Api::all(k8s_client.clone());

    let label_selector = kube::api::ListParams::default()
        .labels(&args.label_selector);

    let ingresses = ingress_api
        .list(&label_selector)
        .await
        .context("Failed to list ingresses")?;

    info!(count = ingresses.items.len(), "Found labeled ingresses");

    // Build mappings from ingresses
    let mut mappings: Vec<IngressDnsMapping> = Vec::new();

    for ingress in ingresses.items {
        let namespace = ingress.metadata.namespace.as_deref().unwrap_or("default");
        let name = ingress.metadata.name.as_deref().unwrap_or("unknown");

        // Get the IP from ingress status
        let ip = ingress
            .status
            .as_ref()
            .and_then(|s| s.load_balancer.as_ref())
            .and_then(|lb| lb.ingress.as_ref())
            .and_then(|ing| ing.first())
            .and_then(|i| i.ip.clone());

        let Some(ip) = ip else {
            warn!(ingress = %name, namespace = %namespace, "Ingress has no IP assigned");
            continue;
        };

        // Get hostname from spec or annotation
        let hostname = ingress
            .metadata
            .annotations
            .as_ref()
            .and_then(|a| a.get("lornu.ai/dns-name"))
            .cloned()
            .or_else(|| {
                ingress
                    .spec
                    .as_ref()
                    .and_then(|s| s.rules.as_ref())
                    .and_then(|r| r.first())
                    .and_then(|rule| rule.host.clone())
            });

        let Some(hostname) = hostname else {
            warn!(ingress = %name, namespace = %namespace, "Ingress has no hostname");
            continue;
        };

        // Handle annotation shorthand (e.g., "preview" -> "preview.lornu.ai")
        let full_hostname = if hostname.contains('.') {
            hostname
        } else {
            format!("{}.lornu.ai", hostname)
        };

        info!(
            ingress = %name,
            namespace = %namespace,
            hostname = %full_hostname,
            ip = %ip,
            "Discovered DNS mapping"
        );

        mappings.push(IngressDnsMapping {
            hostname: full_hostname,
            ip_address: ip,
            namespace: namespace.to_string(),
            ingress_name: name.to_string(),
        });
    }

    if mappings.is_empty() {
        warn!("No ingress DNS mappings found");
        return Ok(());
    }

    // Sync to Cloudflare
    if args.dry_run {
        info!("DRY RUN - would sync {} records", mappings.len());
        for m in &mappings {
            info!(hostname = %m.hostname, ip = %m.ip_address, "Would sync");
        }
        return Ok(());
    }

    let results = cf_client.sync_from_ingresses(&mappings).await?;

    // Log results
    let mut created = 0;
    let mut updated = 0;
    let mut unchanged = 0;
    let mut errors = 0;

    for result in &results {
        match result.action {
            lornu_engine::agents::dns_sync::cloudflare::DnsAction::Created => created += 1,
            lornu_engine::agents::dns_sync::cloudflare::DnsAction::Updated => updated += 1,
            lornu_engine::agents::dns_sync::cloudflare::DnsAction::Unchanged => unchanged += 1,
            lornu_engine::agents::dns_sync::cloudflare::DnsAction::Error => {
                errors += 1;
                error!(
                    record = %result.record_name,
                    error = ?result.error,
                    "Failed to sync DNS record"
                );
            }
            _ => {}
        }
    }

    info!(
        created = created,
        updated = updated,
        unchanged = unchanged,
        errors = errors,
        "DNS sync cycle complete"
    );

    Ok(())
}

/// Get Cloudflare API token from environment or GCP Secret Manager
async fn get_cloudflare_token() -> Result<String> {
    // First try environment variable
    if let Ok(token) = env::var("CLOUDFLARE_API_TOKEN") {
        if !token.is_empty() {
            info!("Using Cloudflare token from environment");
            return Ok(token);
        }
    }

    // Try GCP Secret Manager
    let project_id = env::var("LORNU_GCP_PROJECT")
        .or_else(|_| env::var("GCP_PROJECT"))
        .context("No GCP project ID found in environment")?;

    info!(project = %project_id, "Fetching Cloudflare token from Secret Manager");

    // Get GCP access token via metadata server or gcloud
    let gcp_token = get_gcp_access_token().await?;

    // Fetch secret
    let secret_name = env::var("CLOUDFLARE_SECRET_ID")
        .unwrap_or_else(|_| "CLOUDFLARE_API_TOKEN".to_string());

    let url = format!(
        "https://secretmanager.googleapis.com/v1/projects/{}/secrets/{}/versions/latest:access",
        project_id, secret_name
    );

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .bearer_auth(&gcp_token)
        .send()
        .await
        .context("Failed to call Secret Manager")?;

    if !response.status().is_success() {
        anyhow::bail!("Secret Manager returned {}", response.status());
    }

    let data: serde_json::Value = response.json().await?;
    let payload_b64 = data["payload"]["data"]
        .as_str()
        .context("Secret payload not found")?;

    let payload = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        payload_b64,
    )?;

    Ok(String::from_utf8(payload)?.trim().to_string())
}

/// Get GCP access token from metadata server or gcloud CLI
async fn get_gcp_access_token() -> Result<String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()?;

    // Try metadata server first (GKE with Workload Identity)
    let metadata_url = "http://metadata.google.internal/computeMetadata/v1/instance/service-accounts/default/token";

    if let Ok(resp) = client
        .get(metadata_url)
        .header("Metadata-Flavor", "Google")
        .send()
        .await
    {
        if resp.status().is_success() {
            let data: serde_json::Value = resp.json().await?;
            if let Some(token) = data["access_token"].as_str() {
                return Ok(token.to_string());
            }
        }
    }

    // Fall back to gcloud CLI
    let output = tokio::process::Command::new("gcloud")
        .args(["auth", "application-default", "print-access-token"])
        .output()
        .await
        .context("Failed to run gcloud CLI")?;

    if !output.status.success() {
        anyhow::bail!("gcloud auth failed: {}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}
