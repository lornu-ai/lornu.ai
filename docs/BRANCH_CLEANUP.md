# Branch Cleanup Recommendations

## Trunk-Based Workflow Rule

> **"Only `ta` is long-lived - All other branches are temporary"**

## Current Branch Analysis

### ✅ Keep: `ta` (trunk)
- **Status**: Main branch, always deployable
- **Action**: Keep - this is the trunk

### ❓ Review: `develop`
- **Status**: Has 1 unique commit not in `ta`
  - `feat(agents): add Cloudflare DNS agent in Rust`
- **Recommendation**:
  - **If work needs to be merged**: Create PR from `develop` → `ta`, then delete `develop`
  - **If work is obsolete**: Delete `develop`
- **Action**: Review the Cloudflare DNS agent work and decide

### ❌ Delete: `build`
- **Status**: No unique commits vs `ta`
- **Commits**: Same as `deploy` branch
- **Recommendation**: **DELETE** - Obsolete, violates trunk-based workflow
- **Action**: Delete after confirming no active work

### ❌ Delete: `deploy`
- **Status**: No unique commits vs `ta`
- **Commits**: Same as `build` branch
- **Recommendation**: **DELETE** - Obsolete, violates trunk-based workflow
- **Action**: Delete after confirming no active work

## Deletion Commands

```bash
# Delete build branch (remote)
git push origin --delete build

# Delete deploy branch (remote)
git push origin --delete deploy

# Delete develop branch (after merging work to ta)
git push origin --delete develop
```

## Summary

**Recommended Actions:**
1. ✅ Keep `ta` (trunk)
2. ❓ Review `develop` - merge Cloudflare DNS agent work to `ta` if needed, then delete
3. ❌ Delete `build` - obsolete
4. ❌ Delete `deploy` - obsolete

This aligns with the trunk-based workflow where only `ta` is long-lived.
