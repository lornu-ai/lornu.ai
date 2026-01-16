# GitHub Bot Account Setup Guide

This guide helps you set up a GitHub bot account for automated PR approvals and other GitHub operations.

## Quick Start (Automated Setup)

Use the automated setup script for a guided experience:

```bash
# Run the interactive setup wizard
bun scripts/automated-github-bot-setup.ts --org lornu-ai --name lornu-ai-bot

# Or with GCP Secret Manager integration
bun scripts/automated-github-bot-setup.ts \
  --org lornu-ai \
  --name lornu-ai-bot \
  --project YOUR_GCP_PROJECT_ID
```

The wizard will:
1. Generate a manifest URL for one-click app registration
2. Guide you through the browser-based setup
3. Upload credentials to GCP Secret Manager (optional)
4. Test the configuration

## Manual Setup

### Step 1: Generate GitHub App Manifest URL

Use `create-manifest.ts` to generate a registration URL with exact permissions:

```bash
cd ai-agents/github-bot-tools-ts
bun install

# Generate manifest URL
bun src/bin/create-manifest.ts --org lornu-ai --name lornu-ai-bot
```

This outputs a URL that creates a GitHub App with:
- `pull_requests: write` - Approve/review PRs
- `contents: read` - Read repository files
- `metadata: read` - Read repository metadata
- `issues: write` - Comment on issues
- `checks: write` - Create check runs
- `statuses: write` - Set commit statuses

### Step 2: Register the App

1. Open the generated URL in your browser
2. Review the app name and permissions
3. Click **Create GitHub App**
4. **Download the private key** (.pem file) - you can only download it once!
5. Note the **App ID** from the app's About section

### Step 3: Install the App

1. In your GitHub App settings, click **Install App**
2. Choose **lornu-ai** organization
3. Select repositories:
   - `lornu-ai/lornu.ai`
   - `lornu-ai/private-lornu-ai`
4. Note the **Installation ID** from the URL (e.g., `/settings/installations/12345678`)

### Step 4: Store Credentials in GCP Secret Manager

Use `upload-secrets.ts` to securely store your credentials:

```bash
bun src/bin/upload-secrets.ts \
  --project YOUR_GCP_PROJECT_ID \
  --app-id 123456 \
  --installation-id 78901234 \
  --private-key-file ./lornu-ai-bot.pem
```

This creates three secrets:
- `lornu-github-app-id`
- `lornu-github-app-installation-id`
- `lornu-github-app-private-key`

### Step 5: Test the Setup

Generate an installation token:

```bash
# TypeScript/Bun
TOKEN=$(bun src/bin/generate-github-app-token.ts \
  --app-id 123456 \
  --private-key ./lornu-ai-bot.pem \
  --installation-id 78901234)

# Or Rust (faster)
TOKEN=$(cargo run --bin get-token -p github-bot -- \
  --app-id 123456 \
  --private-key-path ./lornu-ai-bot.pem \
  --installation-id 78901234)
```

Approve a PR:

```bash
# TypeScript/Bun
bun src/bin/approve-prs-with-bot.ts \
  --repo lornu-ai/lornu.ai \
  --token "$TOKEN" \
  --pr-number 29

# Or Rust
cargo run --bin approve-pr -p github-bot -- \
  --repo lornu-ai/lornu.ai \
  --token "$TOKEN" \
  --pr-number 29
```

## Tools Reference

### TypeScript/Bun Tools (`ai-agents/github-bot-tools-ts/`)

| Tool | Purpose |
|------|---------|
| `create-manifest.ts` | Generate GitHub App registration URL |
| `upload-secrets.ts` | Upload credentials to GCP Secret Manager |
| `generate-github-app-token.ts` | Generate installation access token |
| `approve-prs-with-bot.ts` | Approve PRs with bot account |

### Rust Tools (`services/github-bot/`)

| Binary | Purpose |
|--------|---------|
| `get-token` | Generate installation access token (faster) |
| `approve-pr` | Approve PRs with bot account |

### Environment Variables

```bash
# GitHub App credentials
GITHUB_APP_ID=123456
GITHUB_APP_PRIVATE_KEY_PATH=./private-key.pem
GITHUB_APP_INSTALLATION_ID=78901234

# GCP (for secret management)
GCP_PROJECT_ID=your-project-id

# For PR operations
GITHUB_REPOSITORY=lornu-ai/lornu.ai
GITHUB_TOKEN=<generated-token>
```

## CI/CD Integration

### GitHub Actions

```yaml
jobs:
  approve-pr:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Bun
        uses: oven-sh/setup-bun@v1

      - name: Generate Bot Token
        id: token
        run: |
          cd ai-agents/github-bot-tools-ts
          bun install
          TOKEN=$(bun src/bin/generate-github-app-token.ts \
            --app-id ${{ secrets.GITHUB_APP_ID }} \
            --private-key ${{ secrets.GITHUB_APP_PRIVATE_KEY }} \
            --installation-id ${{ secrets.GITHUB_APP_INSTALLATION_ID }})
          echo "token=$TOKEN" >> $GITHUB_OUTPUT

      - name: Approve PR
        run: |
          cd ai-agents/github-bot-tools-ts
          bun src/bin/approve-prs-with-bot.ts \
            --repo ${{ github.repository }} \
            --token ${{ steps.token.outputs.token }} \
            --pr-number ${{ github.event.pull_request.number }}
```

### Flux/GitOps Integration

For GitOps workflows, store secrets using External Secrets Operator (ESO):

```yaml
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: github-bot-credentials
  namespace: flux-system
spec:
  secretStoreRef:
    kind: ClusterSecretStore
    name: gcp-secrets-manager
  target:
    name: github-bot-credentials
  data:
    - secretKey: app-id
      remoteRef:
        key: lornu-github-app-id
    - secretKey: installation-id
      remoteRef:
        key: lornu-github-app-installation-id
    - secretKey: private-key
      remoteRef:
        key: lornu-github-app-private-key
```

## Alternative: Personal Access Token

If you prefer a simpler setup without GitHub Apps:

1. Create a bot user account (e.g., `lornu-ai-bot`)
2. Generate a Personal Access Token with `repo` scope
3. Use the token directly with `approve-prs-with-bot.ts`:

```bash
bun src/bin/approve-prs-with-bot.ts \
  --repo lornu-ai/lornu.ai \
  --token ghp_your_token_here \
  --pr-number 29
```

Note: GitHub Apps are recommended for better security and fine-grained permissions.

## Troubleshooting

### "Cannot approve your own pull request"

- Ensure you're using the bot's token, not your personal token
- The GitHub App must be a different identity than the PR author
- For GitHub Apps, ensure the app is installed and you're using the installation token
- Check that the token has `pull_requests: write` permission

### "Resource not accessible by integration"

- Install the GitHub App on the repository
- Check installation settings in the app's "Install App" section

### "Bad credentials"

- Verify App ID, Installation ID, and private key are correct
- Ensure the private key is in PEM format
- JWT tokens expire after 10 minutes - generate a fresh token

### "Installation not found"

- Verify the installation ID matches your organization
- Ensure the app is installed on the target repository

## Related

- [GitHub Apps Documentation](https://docs.github.com/en/apps)
- [Personal Access Tokens Documentation](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/creating-a-personal-access-token)
- Issues: #24, #60, #61
- PR: #29, #63
