# GitHub Bot Tools (TypeScript/Bun)

TypeScript/Bun tools for GitHub App registration, token generation, secret management, and PR approval.

## Tools

### `create-manifest.ts` (NEW - Issue #60)

Generates a GitHub App manifest URL for one-click app creation.

**Usage:**

```bash
bun src/bin/create-manifest.ts [options]

Options:
  --org <ORG>           GitHub organization name (default: lornu-ai)
  --name <NAME>         App name (default: lornu-ai-bot)
  --url <URL>           App homepage URL (default: https://lornu.ai)
  --description <DESC>  App description
  --public              Make the app public (default: private)
```

**Example:**

```bash
bun src/bin/create-manifest.ts --org lornu-ai --name my-bot
# Opens URL in browser to register the app with exact permissions
```

### `upload-secrets.ts` (NEW - Issue #60)

Uploads GitHub App credentials to GCP Secret Manager.

**Usage:**

```bash
bun src/bin/upload-secrets.ts \
  --project <GCP_PROJECT_ID> \
  --app-id <GITHUB_APP_ID> \
  --installation-id <INSTALLATION_ID> \
  --private-key-file <PATH_TO_PEM>
```

**Environment Variables:**

- `GCP_PROJECT_ID` - Google Cloud project ID
- `GITHUB_APP_ID` - GitHub App ID
- `GITHUB_APP_INSTALLATION_ID` - Installation ID
- `GITHUB_APP_PRIVATE_KEY_PATH` - Path to .pem file

**Example:**

```bash
bun src/bin/upload-secrets.ts \
  --project my-gcp-project \
  --app-id 123456 \
  --installation-id 78901234 \
  --private-key-file ./key.pem
```

### `generate-github-app-token.ts`

Generates GitHub App installation access tokens using JWT authentication.

**Usage:**

```bash
bun src/bin/generate-github-app-token.ts \
  --app-id <APP_ID> \
  --private-key <PRIVATE_KEY_PATH> \
  --installation-id <INSTALLATION_ID> \
  --output <TOKEN_FILE>  # Optional: save token to file
```

**Environment Variables:**

- `GITHUB_APP_ID` - GitHub App ID
- `GITHUB_APP_PRIVATE_KEY_PATH` - Path to private key file
- `GITHUB_APP_INSTALLATION_ID` - Installation ID

**Example:**

```bash
export GITHUB_APP_ID="123456"
export GITHUB_APP_PRIVATE_KEY_PATH="./private-key.pem"
export GITHUB_APP_INSTALLATION_ID="789012"

bun src/bin/generate-github-app-token.ts --output /tmp/github-token
```

### `approve-prs-with-bot.ts`

Approves PRs using a bot token (bypasses self-approval restriction).

**Usage:**

```bash
# Approve single PR
bun src/bin/approve-prs-with-bot.ts \
  --repo <OWNER/REPO> \
  --token <BOT_TOKEN> \
  --pr-number <PR_NUMBER> \
  --message "Approved by bot"  # Optional

# Approve multiple PRs
bun src/bin/approve-prs-with-bot.ts \
  --repo <OWNER/REPO> \
  --token <BOT_TOKEN> \
  --pr-numbers 1019,1049,1018
```

**Environment Variables:**

- `GITHUB_REPOSITORY` - Repository in format `owner/repo`
- `GITHUB_TOKEN` - Bot token or installation token

**Example:**

```bash
export GITHUB_REPOSITORY="lornu-ai/lornu.ai"
export GITHUB_TOKEN="$(cat /tmp/github-token)"

bun src/bin/approve-prs-with-bot.ts --pr-number 24
```

## The "No-Bash" Pipeline

This toolset enables a complete GitHub App automation workflow without shell scripts:

| Phase | Tool | Implementation |
|-------|------|----------------|
| **Registration** | TypeScript | `create-manifest.ts` generates manifest URL |
| **Secret Sync** | TypeScript | `upload-secrets.ts` uploads to GCP Secret Manager |
| **Token Gen** | TypeScript/Rust | `generate-github-app-token.ts` or Rust `get-token` |
| **PR Approval** | TypeScript/Rust | `approve-prs-with-bot.ts` or Rust `approve-pr` |

## Installation

```bash
cd ai-agents/github-bot-tools-ts
bun install
```

## npm Scripts

```bash
bun run create-manifest     # Generate app manifest URL
bun run upload-secrets      # Upload secrets to GCP
bun run generate-token      # Generate installation token
bun run approve-prs         # Approve PRs
```

## Dependencies

- `@octokit/rest` - GitHub API client
- `@google-cloud/secret-manager` - GCP Secret Manager SDK
- `jsonwebtoken` - JWT generation for GitHub App auth
- `bun` - Runtime (TypeScript/Bun)

## Related

- Rust version: `services/github-bot/`
- Issues: #24, #60, #61
- PR: #29

## License

Apache-2.0
