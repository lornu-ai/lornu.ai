# Branch Cleanup Recommendations

## Trunk-Based Workflow Rule

> **"Only `ta` is long-lived - All other branches are temporary"**

## How to Identify Stale Branches

### 1. Find branches already merged into `ta`
These branches are generally safe to delete.

```bash
# List local branches merged into ta
git branch --merged ta

# To delete them (example for 'my-feature-branch'):
git branch -d my-feature-branch
```

### 2. Find branches with unique work
These branches need review before deletion.

```bash
# See commits on a branch that are not in ta
git log ta..my-feature-branch

# If the work is needed, create a PR. If not, delete the remote branch.
git push origin --delete my-feature-branch
```

## Deletion Commands

```bash
# Delete a remote branch
git push origin --delete branch-name

# After deleting remote branches, prune local stale branches
git fetch --prune
```

## Summary

**Workflow:**
1. ✅ Keep `ta` (trunk) - the only long-lived branch
2. ❓ Review feature branches - check if work needs to be merged
3. ❌ Delete merged/obsolete branches - keep repository clean

This aligns with the trunk-based workflow where only `ta` is long-lived.
