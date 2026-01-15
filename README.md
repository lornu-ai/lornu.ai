# lornu.ai

The remake of lornu.ai with Rust Engine and modern architecture.

## Features

- **Rust Engine**: High-performance backend built with Rust
- **Trunk-Based Development**: Lean workflow with `ta` branch (always deployable)
- **Dagger CI/CD**: Branch-based automations (no YAML, no Pulumi)
- **Cross-Project Secrets**: Access secrets from legacy project (`lornu-legacy`) while running in new project (`lornu-v2`)
- **Agent Sandboxes**: Temporary K8s namespaces for agent experiments
- **Bun Runtime**: Fast TypeScript/JavaScript execution

## Quick Start

### Prerequisites

- [Bun](https://bun.sh) installed
- [Rust](https://rustup.rs) installed
- [Just](https://github.com/casey/just) installed (for `just` commands)
- [Dagger](https://dagger.io) installed
- GCP credentials configured
- Access to `lornu-legacy` and `lornu-v2` projects

### Install Dependencies

```bash
# Install Bun dependencies
bun install

# Install Rust dependencies (when you have Cargo.toml)
cargo build
```

### Development Workflow

```bash
# Run baseline checks
just check

# Format code
just fmt

# Run tests
just test

# Build everything
just build:all
```

### Branch Patterns

| Branch | What Happens |
|--------|--------------|
| `feat/*` | Dagger builds + runs `just check` (no infra changes) |
| `infra/*` | Dagger runs `bun run infra:plan` (Crossplane validates) |
| `agent/*` | Dagger creates temporary Knative service for integration tests |
| **Merge to `ta`** | Dagger runs `bun run infra:apply` + `cargo build --release` |

### Setup Cross-Project Secrets

```bash
# Setup IAM bindings for cross-project secret access
./scripts/setup-cross-project-secrets.sh

# Or manually sync secrets
bun ci/sync_secrets.ts \
  --project-id lornu-legacy \
  --secret-name OPENAI_KEY \
  --accessor-project-id lornu-v2 \
  --accessor-service-account engine-sa@lornu-v2.iam.gserviceaccount.com
```

### Create Agent Sandbox

```bash
# Create temporary sandbox for agent experiment
bun infra/agent-sandbox.ts \
  --name researcher-exp-1 \
  --branch agent/researcher/exp-1

# List all sandboxes
bun infra/agent-sandbox.ts --action list

# Delete sandbox
bun infra/agent-sandbox.ts --name researcher-exp-1 --action delete
```

## Documentation

- [Trunk-Based Workflow](./docs/TRUNK_BASED_WORKFLOW.md) - Complete guide on the lean development workflow
- [Cross-Project Secrets](./docs/CROSS_PROJECT_SECRETS.md) - Guide on accessing secrets across GCP projects
- [Rust Examples](./examples/rust-cross-project-secrets.rs) - Example code for accessing cross-project secrets

## Architecture

```
┌─────────────────────────────────┐
│  Trunk (`ta`) Branch            │
│  (Always Deployable)            │
└─────────────────────────────────┘
              │
    ┌─────────┼─────────┐
    │         │         │
┌───▼───┐ ┌──▼───┐ ┌───▼───┐
│ feat/ │ │infra/│ │agent/ │
│       │ │      │ │       │
│ Build │ │ Plan │ │Sandbox│
│ Check │ │Valid │ │ Deploy│
└───────┘ └──────┘ └───────┘
              │
    ┌─────────┼─────────┐
    │         │         │
┌───▼───┐ ┌──▼───┐ ┌───▼───┐
│ Rust  │ │Bun/  │ │K8s/   │
│Engine │ │Next  │ │Knative│
└───────┘ └──────┘ └───────┘
```

## Project Structure

```
lornu.ai/
├── services/engine/   # Rust engine (@engine-team)
├── apps/web/          # Bun/Next.js app (@ui-team)
├── infra/             # Infrastructure (TypeScript/CDK8s) (@infra-ops)
├── ci/                # CI/CD (Dagger/Bun) (@infra-ops)
├── agents/            # Experimental agents (@agent-team)
├── Justfile           # Development commands
└── .github/
    ├── CODEOWNERS     # Ownership rules
    └── workflows/     # GitHub Actions (Dagger pipeline)
```

## License

Apache-2.0
