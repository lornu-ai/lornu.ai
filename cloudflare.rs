/// Cloudflare API Tool
///
///
/// This tool provides functions to interact with the Cloudflare API,
/// specifically for managing DNS records.
use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::env;

/// Represents the parameters required to upsert a DNS record.
/// This structure is the "contract" expected from the Bun/TypeScript side.
#[derive(Debug, Serialize, Deserialize)]
pub struct UpsertDnsParams {
    pub zone_id: String,
    pub name: String, // The FQDN of the record, e.g., "test-agent.lornu.ai"
    pub ip: String,   // The IP address for the 'A' record
}

/// Represents the response from a successful Cloudflare API call.
/// Using a generic structure for the top-level response.
#[derive(Debug, Serialize, Deserialize)]
struct CloudflareResponse<T> {
    result: T,
    success: bool,
    errors: Vec<serde_json::Value>,
    messages: Vec<serde_json::Value>,
}

/// Represents the body for creating a DNS record.
#[derive(Debug, Serialize)]
struct CreateDnsRecordBody<'a> {
    #[serde(rename = "type")]
    record_type: &'a str,
    name: &'a str,
    content: &'a str,
    ttl: u32,
    proxied: bool,
}

/// Represents the body for updating a DNS record.
/// It's the same as creating, so we can reuse the struct.
type UpdateDnsRecordBody<'a> = CreateDnsRecordBody<'a>;

/// Represents a single DNS record from the Cloudflare API.
#[derive(Debug, Serialize, Deserialize)]
struct DnsRecord {
    id: String,
    #[serde(rename = "type")]
    record_type: String,
    name: String,
    content: String,
    proxied: bool,
    ttl: u32,
}

/// Fetches the Cloudflare API token from the environment.
/// In a real application, this might come from GCP Secret Manager.
fn get_cloudflare_token() -> Result<String> {
    env::var("CLOUDFLARE_API_TOKEN")
        .context("CLOUDFLARE_API_TOKEN environment variable not set")
}

/// Upserts a DNS 'A' record in Cloudflare.
///
/// This function is idempotent. It first tries to find an existing record
/// with the same name and updates it if the IP is different. If no record
/// exists, it creates a new one.
///
/// # Arguments
/// * `params` - The parameters for the DNS record.
pub async fn upsert_dns_record(params: &UpsertDnsParams) -> Result<()> {
    let token = get_cloudflare_token()?;
    let client = reqwest::Client::new();

    let api_url = format!(
        "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
        params.zone_id
    );

    // 1. Check if a record with this name already exists using the helper
    let list_response: CloudflareResponse<Vec<DnsRecord>> = execute_request(
        &client,
        reqwest::Method::GET,
        &format!("{}?name={}", api_url, params.name),
        None::<&()>, // No body for GET request
    )
    .await?;

    let existing_record = list_response.result.iter().find(|r| r.record_type == "A");

    let (method, url, body) = if let Some(record) = existing_record {
        if record.content == params.ip {
            println!("✅ DNS record for '{}' is already up to date.", params.name);
            return Ok(());
        }

        // Record exists, but IP is different - update it (PUT)
        println!("🔄 Updating existing DNS record for '{}'...", params.name);
        (
            reqwest::Method::PUT,
            format!("{}/{}", api_url, record.id),
            UpdateDnsRecordBody {
                record_type: "A",
                name: &params.name,
                content: &params.ip,
                ttl: 60, // 1 minute TTL for dynamic records
                proxied: false,
            },
        )
    } else {
        // Record does not exist - create it (POST)
        println!("✨ Creating new DNS record for '{}'...", params.name);
        (
            reqwest::Method::POST,
            api_url,
            CreateDnsRecordBody {
                record_type: "A",
                name: &params.name,
                content: &params.ip,
                ttl: 60,
                proxied: false,
            },
        )
    };

    // Execute the create or update request
    let _: CloudflareResponse<DnsRecord> =
        execute_request(&client, method, &url, Some(&body)).await?;

    println!("✅ Successfully upserted DNS record for '{}'.", params.name);
    Ok(())
}

/// A helper function to execute Cloudflare API requests.
/// It handles authentication, sending the request, and basic error checking.
async fn execute_request<T, B>(
    client: &reqwest::Client,
    method: reqwest::Method,
    url: &str,
    body: Option<&B>,
) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
    B: Serialize + ?Sized,
{
    let token = get_cloudflare_token()?;
    let mut request_builder = client.request(method, url).bearer_auth(token);

    if let Some(body_content) = body {
        request_builder = request_builder.json(body_content);
    }

    let response = request_builder.send().await.context("Failed to send request to Cloudflare API")?;
    let status = response.status(); // Keep status for error reporting
    let resp_json: CloudflareResponse<T> = response.json().await.context("Failed to deserialize Cloudflare response")?;

    if !resp_json.success {
        return Err(anyhow!("Cloudflare API Error (Status: {}): {:?}", status, resp_json.errors));
    }

    Ok(resp_json.result)
}