# GitHub Bot Account Setup Guide

This guide helps you set up a GitHub bot account for automated PR approvals and other GitHub operations.

## Option 1: GitHub App (Recommended)

GitHub Apps are the recommended approach for bot accounts because they:
- Have fine-grained permissions
- Are more secure than Personal Access Tokens
- Can be installed on specific repositories or organizations
- Support JWT authentication

### Step 1: Create a GitHub App

1. Go to your GitHub organization settings (or personal account settings)
2. Navigate to **Developer settings** → **GitHub Apps**
3. Click **New GitHub App**
4. Fill in the app details:
   - **GitHub App name**: `lornu-ai-bot` (or your preferred name)
   - **Homepage URL**: `https://lornu.ai` (or your org URL)
   - **Webhook**: Leave unchecked (unless you need webhooks)
   - **Callback URL**: Leave empty
   - **Expire user authorization tokens**: Unchecked

### Step 2: Set Permissions

Under **Permissions**, configure:

**Repository permissions:**
- **Pull requests**: `Read and write` (required for approvals)
- **Contents**: `Read` (optional, for reading PR diffs)
- **Metadata**: `Read-only` (always enabled)

**Organization permissions:**
- Leave as default unless you need organization-level access

### Step 3: Install the App

1. After creating the app, click **Install App**
2. Choose **Only select repositories** or **All repositories**
3. Select the repositories where the bot should operate:
   - `lornu-ai/lornu.ai`
   - `lornu-ai/private-lornu-ai`
   - Any other repos you need

### Step 4: Generate Private Key

1. In your GitHub App settings, scroll to **Private keys**
2. Click **Generate a private key**
3. **Save the `.pem` file securely** - you can only download it once!
4. Store it in a secure location (e.g., GCP Secret Manager, AWS Secrets Manager)

### Step 5: Get App Credentials

You'll need:
- **App ID**: Found in the app's "About" section
- **Installation ID**: Found after installing the app (in the installation URL or via API)
- **Private Key**: The `.pem` file you downloaded

### Step 6: Test the Setup

```bash
# Generate an installation token
cd ai-agents/github-bot-tools-ts

export GITHUB_APP_ID="123456"  # Your app ID
export GITHUB_APP_PRIVATE_KEY_PATH="./private-key.pem"  # Path to your .pem file
export GITHUB_APP_INSTALLATION_ID="789012"  # Your installation ID

bun src/bin/generate-github-app-token.ts --output /tmp/github-bot-token

# Test approving a PR
export GITHUB_TOKEN="$(cat /tmp/github-bot-token)"
export GITHUB_REPOSITORY="lornu-ai/lornu.ai"

bun src/bin/approve-prs-with-bot.ts --pr-number 29
```

## Option 2: Personal Access Token (Classic)

If you prefer a simpler setup without GitHub Apps:

### Step 1: Create a Bot User Account

1. Create a new GitHub account (e.g., `lornu-ai-bot`)
2. Add it to your organization as a member
3. Grant appropriate permissions

### Step 2: Generate a Classic PAT

1. Log in as the bot user
2. Go to **Settings** → **Developer settings** → **Personal access tokens** → **Tokens (classic)**
3. Click **Generate new token (classic)**
4. Set expiration (or no expiration for bots)
5. Select scopes:
   - ✅ `repo` (Full control of private repositories)
   - ✅ `write:org` (if needed for organization operations)

### Step 3: Store the Token Securely

```bash
# Store in GCP Secret Manager
gcloud secrets create github-bot-token \
  --data-file=- \
  --project=YOUR_PROJECT

# Or store in environment variable (for local testing)
export GITHUB_TOKEN="ghp_..."
```

### Step 4: Test the Setup

```bash
cd ai-agents/github-bot-tools-ts

export GITHUB_TOKEN="ghp_..."  # Bot user's PAT
export GITHUB_REPOSITORY="lornu-ai/lornu.ai"

bun src/bin/approve-prs-with-bot.ts --pr-number 29
```

## Option 3: Fine-Grained Personal Access Token

GitHub also supports fine-grained PATs (newer, more secure):

1. Go to **Settings** → **Developer settings** → **Personal access tokens** → **Fine-grained tokens**
2. Click **Generate new token**
3. Configure:
   - **Token name**: `lornu-ai-bot`
   - **Expiration**: Set as needed
   - **Repository access**: Select specific repos
   - **Permissions**: 
     - Repository permissions: `Pull requests: Write`
     - Account permissions: None (unless needed)

## Storing Credentials Securely

### GCP Secret Manager (Recommended)

```bash
# Store GitHub App credentials
gcloud secrets create github-app-id --data-file=- --project=YOUR_PROJECT <<< "123456"
gcloud secrets create github-app-installation-id --data-file=- --project=YOUR_PROJECT <<< "789012"
gcloud secrets create github-app-private-key --data-file=- --project=YOUR_PROJECT < private-key.pem

# Or store PAT
gcloud secrets create github-bot-token --data-file=- --project=YOUR_PROJECT <<< "ghp_..."
```

### Environment Variables (Local Development)

Create a `.env` file (add to `.gitignore`):

```bash
# GitHub App
GITHUB_APP_ID=123456
GITHUB_APP_PRIVATE_KEY_PATH=/path/to/private-key.pem
GITHUB_APP_INSTALLATION_ID=789012

# Or Personal Access Token
GITHUB_TOKEN=ghp_...
```

## Finding Your Installation ID

If you installed a GitHub App, find the installation ID:

```bash
# Using GitHub CLI
gh api /app/installations | jq '.[] | {id: .id, account: .account.login}'

# Or via API
curl -H "Authorization: Bearer YOUR_JWT" \
  https://api.github.com/app/installations
```

The installation ID is in the response JSON.

## Troubleshooting

### "Cannot approve your own pull request"

- Make sure you're using a **different** GitHub account than the PR author
- For GitHub Apps, ensure the app is installed and you're using the installation token
- Check that the token has `pull_requests: write` permission

### "Resource not accessible by integration"

- The GitHub App needs to be installed on the repository
- Check installation settings in the app's "Install App" section

### "Bad credentials"

- Verify your App ID, Installation ID, and private key are correct
- Ensure the private key file is readable and in PEM format
- Check that the JWT is being generated correctly

### "Installation not found"

- Verify the installation ID matches the installation in your organization
- Ensure the app is installed on the repository you're trying to access

## Quick Setup Script

Save this as `setup-github-bot.sh`:

```bash
#!/bin/bash
set -euo pipefail

echo "🔧 Setting up GitHub Bot Account"
echo ""

# Check if using GitHub App or PAT
if [ -z "${GITHUB_APP_ID:-}" ]; then
  echo "Using Personal Access Token method"
  echo "Set GITHUB_TOKEN environment variable"
else
  echo "Using GitHub App method"
  echo "App ID: ${GITHUB_APP_ID}"
  echo "Installation ID: ${GITHUB_APP_INSTALLATION_ID:-<not set>}"
  echo "Private Key: ${GITHUB_APP_PRIVATE_KEY_PATH:-<not set>}"
fi

echo ""
echo "✅ Setup complete!"
echo ""
echo "Test with:"
echo "  bun src/bin/approve-prs-with-bot.ts --pr-number <PR_NUMBER>"
```

## Next Steps

1. Set up the bot account using one of the methods above
2. Store credentials securely (GCP Secret Manager recommended)
3. Test with a PR approval
4. Integrate into your CI/CD pipeline

## Related

- [GitHub Apps Documentation](https://docs.github.com/en/apps)
- [Personal Access Tokens Documentation](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/creating-a-personal-access-token)
- `ai-agents/github-bot-tools-ts/` - TypeScript/Bun bot tools
- `ai-agents/github-bot-tools/` - Rust bot tools (in private-lornu-ai)
