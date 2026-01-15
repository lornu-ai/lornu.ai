# Branch Setup for lornu.ai

## Trunk Branch: `ta`

The `ta` branch is the **trunk** (main branch) for this repository. All pull requests should be merged into `ta` using **Squash & Merge**.

## Branch Strategy

```
┌─────────────────┐
│  Feature Branch │  (feat/*, infra/*, agent/*)
│       ↓         │
│  Squash Merge   │
│       ↓         │
│       ta        │  ← Trunk (always deployable)
│       ↓         │
│   Production    │
└─────────────────┘
```

## Setting `ta` as Default Branch

To set `ta` as the default branch in GitHub:

1. Go to **Settings** → **Branches**
2. Under **Default branch**, click **Switch to another branch**
3. Select `ta` and click **Update**
4. Confirm the change

## Branch Protection Rules

Recommended branch protection for `ta`:

- ✅ Require pull request reviews before merging
- ✅ Require status checks to pass before merging
- ✅ Require branches to be up to date before merging
- ✅ Require conversation resolution before merging
- ✅ Require linear history (enforces Squash & Merge)
- ✅ Do not allow bypassing the above settings

## Workflow

1. **Create feature branch**: `git checkout -b feat/my-feature ta`
2. **Make changes and commit**
3. **Push branch**: `git push origin feat/my-feature`
4. **Create PR targeting `ta`**
5. **Wait for CI to pass** (Dagger pipeline runs automatically)
6. **Squash & Merge** into `ta`
7. **Delete feature branch** (GitHub can do this automatically)

## Related Documentation

- [Trunk-Based Workflow](./docs/TRUNK_BASED_WORKFLOW.md) - Complete workflow guide
- [CODEOWNERS](./.github/CODEOWNERS) - Code ownership rules
