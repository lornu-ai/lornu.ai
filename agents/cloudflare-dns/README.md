# Cloudflare DNS Agent

A Rust agent for managing Cloudflare DNS records with secrets fetched from Google Secret Manager at runtime.

**Zero hardcoded credentials. Pure runtime injection.**

## Features

- List, create, update, and delete DNS records
- Secrets fetched from Google Secret Manager (GSM)
- Works with Workload Identity (GKE) or Application Default Credentials (local)
- Type-safe Cloudflare API wrapper

## Prerequisites

### Local Development

```bash
# Authenticate with GCP
gcloud auth application-default login

# Set project
export GCP_PROJECT_ID=gcp-lornu-ai
```

### GKE / Cloud Run

Configure Workload Identity to grant the service account access to GSM.

### Secret Setup

Create the Cloudflare API token in GSM:

```bash
echo -n "your-cloudflare-api-token" | \
  gcloud secrets create cloudflare-api-token \
    --project=gcp-lornu-ai \
    --data-file=-
```

## Usage

```bash
# Build
cargo build --release

# List DNS records
./target/release/cloudflare-dns \
  --project gcp-lornu-ai \
  list --zone lornu.ai

# Create A record
./target/release/cloudflare-dns \
  --project gcp-lornu-ai \
  create --zone lornu.ai --name api --type A --content 1.2.3.4 --proxied

# Update record
./target/release/cloudflare-dns \
  --project gcp-lornu-ai \
  update --zone lornu.ai --record-id abc123 --content 5.6.7.8

# Delete record
./target/release/cloudflare-dns \
  --project gcp-lornu-ai \
  delete --zone lornu.ai --record-id abc123
```

## Environment Variables

| Variable | Description | Required |
|----------|-------------|----------|
| `GCP_PROJECT_ID` | GCP project for Secret Manager | Yes |
| `GOOGLE_APPLICATION_CREDENTIALS` | Path to service account key (local only) | No |

## Architecture

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   CLI (clap)    │────▶│  Secret Manager │────▶│  Cloudflare API │
└─────────────────┘     │     (GSM)       │     │     (v4)        │
                        └─────────────────┘     └─────────────────┘
                               │                       │
                               ▼                       ▼
                        ┌─────────────────┐     ┌─────────────────┐
                        │  ADC / Workload │     │  Bearer Token   │
                        │    Identity     │     │  (from GSM)     │
                        └─────────────────┘     └─────────────────┘
```

## Security

- No credentials in code or config files
- API tokens fetched at runtime from GSM
- Works with GCP's Workload Identity for zero-secret deployments
- All API calls use TLS (rustls)
