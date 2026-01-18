# ai-agent-pr-comment

Foundational Rust setup for a PR comment agent that listens to GitHub webhooks.

## Features

- Axum webhook listener on `POST /webhook`
- Optional GitHub signature verification (`GITHUB_WEBHOOK_SECRET`)
- Handles `pull_request` events (`opened`, `synchronize`) with a stub analyzer

## Quick Start

```bash
cd ai-agents/ai-agent-pr-comment

# Optional: verify GitHub signatures
export GITHUB_WEBHOOK_SECRET="your-webhook-secret"

# Optional: enable GitHub App comment posting
export GITHUB_APP_ID="12345"
export GITHUB_INSTALLATION_ID="67890"
export GITHUB_APP_PRIVATE_KEY="-----BEGIN RSA PRIVATE KEY-----\n...\n-----END RSA PRIVATE KEY-----"
export GITHUB_POST_COMMENTS=true

# Optional: override port (default 3000)
export PORT=3000

cargo run
```

## Configuration

- `GITHUB_WEBHOOK_SECRET`: HMAC secret for webhook signature verification.
- `GITHUB_APP_ID`: GitHub App ID for posting comments.
- `GITHUB_INSTALLATION_ID`: Installation ID for the GitHub App.
- `GITHUB_APP_PRIVATE_KEY` or `GITHUB_APP_PRIVATE_KEY_B64`: App private key.
- `GITHUB_API_URL`: Override GitHub API base (default `https://api.github.com`).
- `GITHUB_POST_COMMENTS`: Enable comment posting when set to `true` or `1`.

## Next Steps

- Implement `analyze_and_comment` to fetch diffs and post comments.
- Add GitHub App authentication for API access.
- Wire in a learning loop for accepted vs. rejected comments.

## Health Check

- `GET /healthz` returns `{ "status": "ok" }`.
