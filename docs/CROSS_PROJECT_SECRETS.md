# Cross-Project Google Secret Manager Access

This guide explains how to access secrets stored in one GCP project from workloads running in another project. This is useful for the **lornu.ai** remake where legacy secrets remain in `lornu-legacy` while new Rust Engine/Agents run in `lornu-v2`.

## Architecture

```
┌─────────────────────────────────┐
│  lornu-legacy (Secret Project)  │
│  ┌───────────────────────────┐  │
│  │  GCP Secret Manager       │  │
│  │  - OPENAI_KEY             │  │
│  │  - CLOUDFLARE_TOKEN        │  │
│  │  - DATABASE_PASSWORD       │  │
│  └───────────────────────────┘  │
└─────────────────────────────────┘
              │
              │ IAM Binding
              │ (roles/secretmanager.secretAccessor)
              ▼
┌─────────────────────────────────┐
│  lornu-v2 (Workload Project)     │
│  ┌───────────────────────────┐  │
│  │  Service Account          │  │
│  │  engine-sa@lornu-v2       │  │
│  └───────────────────────────┘  │
│  ┌───────────────────────────┐  │
│  │  Rust Engine / Agents     │  │
│  │  (Accesses secrets)        │  │
│  └───────────────────────────┘  │
└─────────────────────────────────┘
```

## Quick Start

### 1. Setup IAM Binding

Grant the Service Account in your workload project access to secrets in the secret project:

```bash
# Grant access to a specific secret
gcloud secrets add-iam-policy-binding "OPENAI_KEY" \
    --project="lornu-legacy" \
    --role="roles/secretmanager.secretAccessor" \
    --member="serviceAccount:engine-sa@lornu-v2.iam.gserviceaccount.com"
```

### 2. Use Full Resource Name in Code

When accessing secrets, use the **full resource name** instead of just the secret ID:

**❌ Wrong:**
```rust
let secret_path = "secrets/OPENAI_KEY/versions/latest";
```

**✅ Correct:**
```rust
let secret_path = "projects/lornu-legacy/secrets/OPENAI_KEY/versions/latest";
```

## Using the Sync Script

### Single Secret

```bash
# Sync a secret to lornu-legacy
bun ci/sync_secrets.ts \
  --project-id lornu-legacy \
  --secret-name OPENAI_KEY

# Sync with cross-project access
bun ci/sync_secrets.ts \
  --project-id lornu-legacy \
  --secret-name OPENAI_KEY \
  --accessor-project-id lornu-v2 \
  --accessor-service-account engine-sa@lornu-v2.iam.gserviceaccount.com
```

### Multiple Secrets (Config File)

Create `secrets.json`:

```json
{
  "defaultProjectId": "lornu-legacy",
  "secrets": [
    {
      "name": "OPENAI_KEY",
      "projectId": "lornu-legacy",
      "accessorProjectId": "lornu-v2",
      "accessorServiceAccount": "engine-sa@lornu-v2.iam.gserviceaccount.com",
      "envVar": "OPENAI_KEY"
    },
    {
      "name": "CLOUDFLARE_TOKEN",
      "projectId": "lornu-legacy",
      "accessorProjectId": "lornu-v2",
      "accessorServiceAccount": "engine-sa@lornu-v2.iam.gserviceaccount.com",
      "envVar": "CLOUDFLARE_TOKEN"
    },
    {
      "name": "DATABASE_URL",
      "projectId": "lornu-v2",
      "envVar": "DATABASE_URL"
    }
  ]
}
```

Sync all secrets:

```bash
bun ci/sync_secrets.ts --config secrets.json
```

## Rust Example

### Using `google-cloud-secretmanager`

Add to `Cargo.toml`:

```toml
[dependencies]
google-cloud-secretmanager = "3.0"
tokio = { version = "1.35", features = ["full"] }
```

Example code:

```rust
use google_cloud_secretmanager::Client;
use google_cloud_secretmanager::protos::AccessSecretVersionRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client (uses default credentials from environment)
    let client = Client::default().await?;
    
    // Access secret from cross-project (lornu-legacy)
    let secret_path = "projects/lornu-legacy/secrets/OPENAI_KEY/versions/latest";
    
    let request = AccessSecretVersionRequest {
        name: secret_path.to_string(),
        ..Default::default()
    };
    
    let response = client.access_secret_version(request).await?;
    let secret_value = String::from_utf8(response.payload.unwrap().data)?;
    
    println!("Secret value: {}", secret_value);
    
    Ok(())
}
```

### Using `gcp-secret-manager` (Alternative)

```toml
[dependencies]
gcp-secret-manager = "0.1"
tokio = { version = "1.35", features = ["full"] }
```

```rust
use gcp_secret_manager::SecretManagerClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SecretManagerClient::new().await?;
    
    // Full resource name for cross-project access
    let secret_path = "projects/lornu-legacy/secrets/OPENAI_KEY/versions/latest";
    let secret_value = client.access_secret_version(secret_path).await?;
    
    println!("Secret value: {}", secret_value);
    
    Ok(())
}
```

## Bun/TypeScript Example

```typescript
import { SecretManagerServiceClient } from "@google-cloud/secret-manager";

const client = new SecretManagerServiceClient();

async function getSecret() {
  // Full resource name for cross-project access
  const name = "projects/lornu-legacy/secrets/OPENAI_KEY/versions/latest";
  
  const [version] = await client.accessSecretVersion({ name });
  const secretValue = version.payload?.data?.toString();
  
  return secretValue;
}
```

## IAM Setup Script

Create `scripts/setup-cross-project-secrets.sh`:

```bash
#!/bin/bash
# Setup cross-project secret access for lornu.ai

set -euo pipefail

SECRET_PROJECT="lornu-legacy"
WORKLOAD_PROJECT="lornu-v2"
SERVICE_ACCOUNT="engine-sa@${WORKLOAD_PROJECT}.iam.gserviceaccount.com"

# List of secrets to grant access to
SECRETS=(
  "OPENAI_KEY"
  "CLOUDFLARE_TOKEN"
  "DATABASE_PASSWORD"
)

echo "🔐 Setting up cross-project secret access..."
echo "   Secret Project: ${SECRET_PROJECT}"
echo "   Workload Project: ${WORKLOAD_PROJECT}"
echo "   Service Account: ${SERVICE_ACCOUNT}"
echo ""

for secret in "${SECRETS[@]}"; do
  echo "Granting access to ${secret}..."
  gcloud secrets add-iam-policy-binding "${secret}" \
    --project="${SECRET_PROJECT}" \
    --role="roles/secretmanager.secretAccessor" \
    --member="serviceAccount:${SERVICE_ACCOUNT}" || {
    echo "⚠️  Failed to grant access to ${secret}"
  }
done

echo ""
echo "✅ Cross-project access configured!"
```

## Pros & Cons

| Pros | Cons |
|------|------|
| **Centralization:** Manage all API keys in one secure vault | **Latency:** Minor increase in API response time (usually negligible) |
| **Easier Rotation:** Rotate once, all projects get update | **Dependency:** If secret project has IAM misconfiguration, all projects lose access |
| **Security:** Limits "Blast Radius" if workload project is compromised | **Quota:** Secret usage counts against the project where secret lives |
| **Clean Separation:** Legacy utilities stay in legacy project | **Complexity:** Requires IAM setup and full resource names |

## Best Practices

1. **Use Full Resource Names**: Always use `projects/{project-id}/secrets/{name}/versions/{version}` format
2. **Least Privilege**: Grant access only to specific secrets, not entire project
3. **Documentation**: Document which secrets are cross-project and why
4. **Monitoring**: Monitor secret access logs for unusual patterns
5. **Rotation**: Plan for secret rotation without breaking workloads

## Troubleshooting

### Permission Denied

```bash
# Check IAM binding
gcloud secrets get-iam-policy OPENAI_KEY --project=lornu-legacy

# Verify service account exists
gcloud iam service-accounts describe engine-sa@lornu-v2.iam.gserviceaccount.com
```

### Secret Not Found

```bash
# Verify secret exists in source project
gcloud secrets describe OPENAI_KEY --project=lornu-legacy

# Check if using full resource name in code
# Should be: projects/lornu-legacy/secrets/OPENAI_KEY/versions/latest
```

### Authentication Issues

```bash
# Verify workload identity is configured
gcloud iam service-accounts get-iam-policy engine-sa@lornu-v2.iam.gserviceaccount.com

# Check if service account has necessary permissions
gcloud projects get-iam-policy lornu-v2 \
  --flatten="bindings[].members" \
  --filter="bindings.members:serviceAccount:engine-sa@lornu-v2.iam.gserviceaccount.com"
```

## Related Documentation

- [Google Secret Manager IAM](https://cloud.google.com/secret-manager/docs/access-control)
- [Cross-Project Resource Access](https://cloud.google.com/iam/docs/cross-project-resource-access)
- [Secret Manager Best Practices](https://cloud.google.com/secret-manager/docs/best-practices)
