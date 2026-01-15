# GitHub Token Setup for Branch Protection Script

## ⚠️ Important: Must Use Classic Token

**You MUST use a Personal Access Token (classic)**, not a fine-grained token. Fine-grained tokens don't support `admin:repo` scope.

## Required Permissions

The `setup-branch-protection.ts` script requires specific GitHub token scopes:

### Minimum Required
- **`repo`** - Full control of private repositories
  - Read and write access to code, pull requests, issues, etc.

### For Branch Protection
- **`admin:repo`** - Full control of repository settings
  - Required to configure branch protection rules
  - Required to change default branch
  - Required to configure merge settings
  - ⚠️ **Only available in classic tokens**

### For Team Creation (optional)
- **`admin:org`** - Full control of organization settings
  - Only needed if using `--create-teams` flag
  - Required to create teams in the organization
  - ⚠️ **Only available in classic tokens**

## How to Create/Update Token

1. **Go to GitHub Token Settings**
   - Visit: https://github.com/settings/tokens
   - Or: GitHub → Settings → Developer settings → Personal access tokens → **Tokens (classic)**

2. **Create New Classic Token**
   - Click "**Generate new token**" → "**Generate new token (classic)**"
   - ⚠️ **Important**: Select "classic", NOT "fine-grained"
   - Or edit an existing classic token

3. **Select Required Scopes**
   - ✅ **repo** (Full control of private repositories)
   - ✅ **admin:repo** (Full control of repository settings)
   - ✅ **admin:org** (Full control of organization settings) - Only if using `--create-teams`

4. **Generate Token**
   - Click "Generate token"
   - **Copy the token immediately** (you won't see it again!)

5. **Use the Token**
   ```bash
   export GITHUB_TOKEN=your_token_here
   bun scripts/setup-branch-protection.ts
   ```

## Token Security Best Practices

- ⚠️ **Never commit tokens to git**
- ⚠️ **Never share tokens publicly**
- ✅ Use environment variables
- ✅ Use GitHub Secrets in CI/CD
- ✅ Rotate tokens regularly
- ⚠️ **Note**: This script requires classic tokens (fine-grained tokens don't support admin scopes)

## Troubleshooting

### Error: "Insufficient permissions to configure merge settings"

**Problem:** Token doesn't have `admin:repo` scope

**Solution:**
1. Go to https://github.com/settings/tokens
2. Edit your token
3. Check the `admin:repo` scope
4. Save and regenerate if needed
5. Use the new token

### Error: "Missing 'admin:org' scope"

**Problem:** Using `--create-teams` but token doesn't have `admin:org` scope

**Solution:**
1. Add `admin:org` scope to your token, OR
2. Run without `--create-teams` and create teams manually:
   - Go to: https://github.com/orgs/{org}/teams
   - Create teams: `engine-team`, `ui-team`, `infra-ops`

### Error: "Repository not found or no access"

**Problem:** Token doesn't have access to the repository

**Solution:**
1. Verify the repository name is correct
2. Ensure the token has `repo` scope
3. For organization repos, ensure the token has organization access

## Alternative: Use GitHub CLI

If you prefer using GitHub CLI instead of a token:

```bash
# Authenticate with GitHub CLI
gh auth login

# The script will automatically use gh's credentials
bun scripts/setup-branch-protection.ts
```

Note: GitHub CLI tokens also need the same scopes when creating tokens.

## Related Documentation

- [GitHub Personal Access Tokens](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/managing-your-personal-access-tokens)
- [Token Scopes](https://docs.github.com/en/apps/oauth-apps/building-oauth-apps/scopes-for-oauth-apps)
- [Branch Protection Setup Script](../scripts/setup-branch-protection.ts)
