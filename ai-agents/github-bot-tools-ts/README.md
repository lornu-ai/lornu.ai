# GitHub Bot Tools (TypeScript/Bun)

TypeScript/Bun versions of GitHub bot tools for generating App tokens and approving PRs.

## Tools

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

## Installation

```bash
cd ai-agents/github-bot-tools-ts
bun install
```

## Dependencies

- `@octokit/rest` - GitHub API client
- `jsonwebtoken` - JWT generation for GitHub App auth
- `bun` - Runtime (TypeScript/Bun)

## Related

- Rust version: `ai-agents/github-bot-tools/` (in `private-lornu-ai`)
- Issue: #24

## License

Apache-2.0
