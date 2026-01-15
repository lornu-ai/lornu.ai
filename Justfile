set dotenv-load := true
set shell := ["bash", "-c"]

# Default task: Show available commands
default:
    @just --list

# -----------------------------------------------------------------------------
# 🛠 Setup & Installation
# -----------------------------------------------------------------------------

# Initial setup for a fresh clone
bootstrap: install
    @echo "🚀 Lornu.ai environment ready."

# Install all dependencies
install:
    @echo "📦 Installing dependencies..."
    bun install

# -----------------------------------------------------------------------------
# 🧪 Validation & Testing
# -----------------------------------------------------------------------------

# Run the full validation suite
check: check-web check-engine check-infra
    @echo "✅ All systems validated."

# Lint and test the Rust engine (if present)
check-engine:
    @if [ -f Cargo.toml ]; then \
        echo "🦀 Checking Rust engine..."; \
        cargo clippy --workspace -- -D warnings; \
        cargo test --workspace; \
    else \
        echo "ℹ️  No Cargo.toml found; skipping Rust checks."; \
    fi

# Type-check and test the frontend
check-web:
    @echo "🍞 Checking Web Frontend..."
    bun run typecheck
    bun run test

# Validate infrastructure logic (if present)
check-infra:
    @if [ -d infra ]; then \
        echo "☁️  Checking Infrastructure Logic..."; \
        (cd infra && bun install && bun run synth); \
    else \
        echo "ℹ️  No infra/ directory found; skipping infra checks."; \
    fi

# -----------------------------------------------------------------------------
# 🚀 Infrastructure Deployment (SSA)
# -----------------------------------------------------------------------------

# Preview infrastructure changes
plan:
    @if [ -f infra/ci/apply_ssa.ts ]; then \
        echo "🔍 Planning infrastructure changes..."; \
        bun run infra/ci/apply_ssa.ts --dry-run; \
    else \
        echo "ℹ️  No infra/ci/apply_ssa.ts found; skipping plan."; \
    fi

# Apply infrastructure directly to the cluster
apply:
    @if [ -f infra/ci/apply_ssa.ts ]; then \
        echo "🛰️  Applying infrastructure to cluster..."; \
        bun run infra/ci/apply_ssa.ts; \
    else \
        echo "ℹ️  No infra/ci/apply_ssa.ts found; skipping apply."; \
    fi

# -----------------------------------------------------------------------------
# 🏗 CI/CD & Agents
# -----------------------------------------------------------------------------

# Run the full Dagger pipeline (if present)
pipeline:
    @if command -v dagger >/dev/null 2>&1; then \
        echo "🗡️  Executing Dagger Pipeline..."; \
        dagger run bun ci/main.ts; \
    else \
        echo "ℹ️  Dagger not installed; skipping pipeline."; \
    fi

# Watch logs for agent pods (optional)
agent-logs:
    @if command -v kubectl >/dev/null 2>&1; then \
        kubectl logs -l app=lornu-agent -f --tail=100; \
    else \
        echo "ℹ️  kubectl not installed; skipping agent logs."; \
    fi

# -----------------------------------------------------------------------------
# 🧹 Cleanup
# -----------------------------------------------------------------------------

clean:
    @echo "🧹 Cleaning build artifacts..."
    rm -rf dist
    rm -rf node_modules/.vite
    rm -rf .vite
    @if [ -d infra/dist ]; then rm -rf infra/dist; fi
    @if [ -d apps/web/.next ]; then rm -rf apps/web/.next; fi
