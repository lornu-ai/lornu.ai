# Trunk-Based Development Workflow

**Lean, YAML-free, Pulumi-free workflow for lornu.ai using Dagger + Crossplane**

## Philosophy

- **Trunk (`ta`) is always deployable** - Long-lived branches cause configuration drift
- **Features are atomic** - Short-lived (1-3 days), merged via Squash & Merge
- **Branches trigger automations** - Not just tests, but infrastructure provisioning
- **Ownership via CODEOWNERS** - Not branch naming

## Branch Patterns

| Pattern | Trigger | Action |
|---------|---------|--------|
| `feat/*` | Push | **Dagger** builds Rust/Bun + runs `just check`. No infra changes. |
| `infra/*` | Push | **Dagger** runs `bun run infra:plan`. Crossplane validates the plan. |
| `agent/*` | Push | **Dagger** spins up temporary **Knative** service for agent integration tests. |
| **Merge to `ta`** | Merge | **Dagger** runs `bun run infra:apply` (SSA) + `cargo build --release`. |

## The "ta" Trunk Flow

```
┌─────────────────┐
│  feat/new-api   │  (1-3 days)
│       ↓         │
│  Squash Merge   │
│       ↓         │
│       ta        │  (always deployable)
│       ↓         │
│   Production    │
└─────────────────┘
```

### Rules

1. **Only `ta` is long-lived** - All other branches are temporary
2. **Squash & Merge only** - Keeps `ta` history clean and atomic
3. **No direct commits to `ta`** - All changes come via feature branches
4. **`ta` is always green** - If CI fails, fix immediately or revert

## Feature Branches (`feat/*`)

### Purpose
- Add new features
- Fix bugs
- Refactor code

### What Happens
1. **Dagger Pipeline** runs automatically on push
2. Builds Rust engine (`cargo build`)
3. Builds Bun/Next.js app (`bun run build`)
4. Runs baseline checks (`just check`)
   - Rust linting (`cargo clippy`)
   - Bun type checking (`bun run typecheck`)
   - Contract checks (TypeShare)

### Example
```bash
git checkout -b feat/user-authentication
# ... make changes ...
git push origin feat/user-authentication
# Dagger automatically runs: build + check
```

## Infrastructure Branches (`infra/*`)

### Purpose
- Add/modify Kubernetes resources
- Update Crossplane compositions
- Change infrastructure configuration

### What Happens
1. **Dagger Pipeline** runs automatically on push
2. Runs infrastructure dry-run (`bun run infra:plan`)
3. Crossplane validates the plan
4. **No actual changes** - Just validation

### Example
```bash
git checkout -b infra/add-database
# ... modify infra/ files ...
git push origin infra/add-database
# Dagger automatically runs: infra:plan (dry-run)
```

## Agent Branches (`agent/*`)

### Purpose
- Experiment with new agents
- Test agent integrations
- Fail fast without breaking `ta`

### What Happens
1. **Dagger Pipeline** detects `agent/*` pattern
2. Extracts agent name (e.g., `agent/researcher/exp-1` → `researcher`)
3. Creates temporary Kubernetes namespace via Crossplane
4. Deploys agent to Knative service in sandbox
5. Runs integration tests
6. **Sandbox auto-deletes** when branch is deleted

### Example
```bash
git checkout -b agent/researcher/exp-1
# ... develop agent ...
git push origin agent/researcher/exp-1
# Dagger automatically:
#   1. Creates namespace: agent-researcher-{timestamp}
#   2. Deploys to Knative
#   3. Runs integration tests
```

### Sandbox Cleanup

Sandboxes are automatically deleted when:
- Branch is deleted (via GitHub webhook)
- Branch is merged to `ta` (sandbox no longer needed)

Manual cleanup:
```bash
bun infra/agent-sandbox.ts --name researcher-exp-1 --action delete
```

## Merge to `ta`

### What Happens
1. **Dagger Pipeline** runs on merge
2. Applies infrastructure changes (`bun run infra:apply`)
   - Uses Server-Side Apply (SSA)
   - Crossplane reconciles changes
3. Builds release artifacts
   - `cargo build --release` (Rust)
   - `bun run build` (Bun/Next.js)
4. **Ready for deployment**

### Pre-Merge Checklist

The `just check` command ensures:

1. ✅ **Rust checks**
   - `cargo clippy` (linting)
   - `cargo fmt --check` (formatting)

2. ✅ **Bun checks**
   - `bun run typecheck` (type checking)

3. ✅ **Contract checks**
   - TypeShare validation (Rust ↔ Bun interfaces)

4. ✅ **Infrastructure plan** (for `infra/*` branches)
   - `bun run infra:plan` (dry-run validation)

## Ownership (CODEOWNERS)

Ownership is enforced via `.github/CODEOWNERS`, not branch naming:

```
/services/engine/   @engine-team    (Rust)
/apps/web/          @ui-team        (Bun/Next.js)
/infra/             @infra-ops      (TypeScript/CDK8s)
/ci/                @infra-ops      (Dagger/Bun)
/agents/            @agent-team     (Experimental)
```

## Local Development

### Running Checks Locally

```bash
# Run all baseline checks
just check

# Run specific checks
just check:rust
just check:bun
just check:contracts

# Format code
just fmt

# Run tests
just test
```

### Creating Agent Sandbox Locally

```bash
# Create sandbox
bun infra/agent-sandbox.ts \
  --name researcher-exp-1 \
  --branch agent/researcher/exp-1

# List sandboxes
bun infra/agent-sandbox.ts --action list

# Delete sandbox
bun infra/agent-sandbox.ts --name researcher-exp-1 --action delete
```

## Dagger Pipeline

The Dagger pipeline (`ci/dagger.ts`) automatically detects branch patterns and runs appropriate actions:

```bash
# Run manually
bun ci/dagger.ts --branch feat/new-feature --event push

# Run for infrastructure
bun ci/dagger.ts --branch infra/add-database --event push

# Run for agent
bun ci/dagger.ts --branch agent/researcher/exp-1 --event push

# Run for trunk merge
bun ci/dagger.ts --branch ta --event merge
```

## Benefits

### vs. GitFlow

- ✅ **Simpler** - No `develop`, `release`, `hotfix` branches
- ✅ **Faster** - Direct path from feature to production
- ✅ **Less drift** - `ta` is always deployable

### vs. GitHub Flow

- ✅ **Infrastructure-aware** - Branches trigger infra validation
- ✅ **Agent experiments** - Isolated sandboxes for testing
- ✅ **Automated** - Dagger handles all the complexity

### vs. Standard Enterprise

- ✅ **YAML-free** - TypeScript/Bun instead of YAML
- ✅ **Pulumi-free** - Crossplane for infrastructure
- ✅ **Lean** - No "standard enterprise" fluff

## Troubleshooting

### Dagger Pipeline Fails

```bash
# Run locally to debug
bun ci/dagger.ts --branch feat/test --event push

# Check Dagger logs
export DAGGER_LOG_LEVEL=debug
bun ci/dagger.ts --branch feat/test --event push
```

### Agent Sandbox Not Created

```bash
# Check branch pattern
echo $BRANCH | grep -E "^agent/"

# Manually create sandbox
bun infra/agent-sandbox.ts \
  --name researcher-exp-1 \
  --branch agent/researcher/exp-1
```

### Infrastructure Plan Fails

```bash
# Run plan locally
bun run infra:plan

# Check Crossplane resources
kubectl get compositions
kubectl get xrd
```

## Related Documentation

- [CODEOWNERS](../.github/CODEOWNERS) - Ownership rules
- [Justfile](../Justfile) - Available commands
- [Dagger Pipeline](../ci/dagger.ts) - Pipeline implementation
