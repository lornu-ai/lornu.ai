# CODEOWNERS Setup Complete

## Current CODEOWNERS Structure

The CODEOWNERS file has been configured and matches the repository structure:

```
/services/engine/   → @engine-team    (Rust)
/apps/web/          → @ui-team        (Bun/Next.js)
/infra/             → @infra-ops      (TypeScript/CDK8s)
/ci/                → @infra-ops      (Dagger/Bun)
```

## Changes Made

- ✅ Removed `/agents/` section (not in current structure)
- ✅ Added GitHub documentation link
- ✅ Kept root config file ownership rules
- ✅ File committed to `ta` branch

## Next Steps

### 1. Create GitHub Teams (Required)

The teams referenced in CODEOWNERS must exist in your GitHub organization:

- `@engine-team` - For Rust engine development
- `@ui-team` - For Bun/Next.js web application
- `@infra-ops` - For infrastructure and CI/CD

**To create teams:**
1. Go to GitHub Settings → Teams
2. Create each team with appropriate members
3. Grant necessary repository permissions

**Alternative:** If teams don't exist yet, update CODEOWNERS to use individual usernames:
```gitignore
/services/engine/   @username1 @username2
/apps/web/          @username3 @username4
/infra/             @username5 @username6
/ci/                @username5 @username6
```

### 2. Configure Branch Protection

Run the branch protection setup script to enforce CODEOWNERS reviews:

```bash
GITHUB_TOKEN=your_token bun scripts/setup-branch-protection.ts
```

This will:
- Configure branch protection for `ta` branch
- Require CODEOWNERS reviews (enforced via `require_code_owner_reviews: true`)
- Set default branch to `ta`
- Configure merge settings (Squash & Merge only)

### 3. Verify CODEOWNERS Enforcement

After setting up branch protection:
1. Create a test PR that touches `/services/engine/`
2. Verify that `@engine-team` members are automatically requested as reviewers
3. Confirm that PR cannot be merged without approval from CODEOWNERS

## Related Files

- `.github/CODEOWNERS` - Code ownership rules
- `scripts/setup-branch-protection.ts` - Branch protection setup script
- `.github/BRANCH_SETUP.md` - Branch setup documentation

## Documentation

- [GitHub CODEOWNERS Documentation](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/about-code-owners)
