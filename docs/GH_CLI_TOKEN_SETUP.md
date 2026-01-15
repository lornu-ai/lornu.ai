# Using GitHub CLI for Token Setup

## Quick Setup with Required Scopes

GitHub CLI can help you authenticate with the correct scopes. Here's how:

### Option 1: Re-authenticate with Required Scopes

```bash
# Re-authenticate with all required scopes
gh auth login --scopes repo,admin:repo,admin:org

# Or refresh existing authentication
gh auth refresh --scopes repo,admin:repo,admin:org
```

### Option 2: Use GitHub CLI Token Directly

GitHub CLI stores your token and you can use it:

```bash
# Get your current token
gh auth token

# Use it with the script
GITHUB_TOKEN=$(gh auth token) bun scripts/setup-branch-protection.ts
```

### Option 3: Set Token as Environment Variable

```bash
# Export token for current session
export GITHUB_TOKEN=$(gh auth token)

# Run the script
bun scripts/setup-branch-protection.ts
```

## Verify Your Token Has Required Scopes

```bash
# Check what scopes your current token has
gh auth status

# Or check via API
gh api user -q '.login'
```

## If You Need to Create a New Token

If `gh auth refresh` doesn't work, you'll need to create a new token via web UI:

1. **Go to**: https://github.com/settings/tokens
2. **Generate new token (classic)**
3. **Select scopes**:
   - ✅ `repo`
   - ✅ `admin:repo`
   - ✅ `admin:org` (if using `--create-teams`)
4. **Copy token** and use it:
   ```bash
   export GITHUB_TOKEN=your_token_here
   bun scripts/setup-branch-protection.ts
   ```

## Using GitHub CLI Token in Scripts

The setup script can automatically use `gh auth token` if `GITHUB_TOKEN` is not set. You can modify the script to check for gh CLI:

```bash
# In your script or shell
GITHUB_TOKEN=${GITHUB_TOKEN:-$(gh auth token 2>/dev/null)} bun scripts/setup-branch-protection.ts
```

## Troubleshooting

### "gh: command not found"
Install GitHub CLI:
```bash
# macOS
brew install gh

# Then authenticate
gh auth login
```

### "Authentication required"
```bash
gh auth login
```

### "Insufficient scopes"
```bash
# Re-authenticate with required scopes
gh auth refresh --scopes repo,admin:repo,admin:org
```

## Related

- [GitHub CLI Documentation](https://cli.github.com/manual/)
- [GitHub Token Setup Guide](./GITHUB_TOKEN_SETUP.md)
