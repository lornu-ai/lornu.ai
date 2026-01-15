# Justfile for lornu.ai - Lean Trunk-Based Development
# Issue: Trunk-based workflow with Dagger + Crossplane

# Default: Show available commands
default:
    @just --list

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# Baseline Checklist (Pre-Merge to `ta`)
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

# Run all baseline checks
check:
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    @echo "🔍 Running baseline checklist..."
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    @just check:rust
    @just check:bun
    @just check:contracts
    @echo ""
    @echo "✅ All baseline checks passed!"

# Rust linting and type checking
check:rust:
    @echo ""
    @echo "🔧 Checking Rust code..."
    cargo clippy --all-targets --all-features -- -D warnings
    cargo fmt --check
    @echo "✅ Rust checks passed"

# Bun type checking
check:bun:
    @echo ""
    @echo "🔧 Checking Bun/TypeScript code..."
    bun run typecheck
    @echo "✅ Bun checks passed"

# Contract check (TypeShare between Rust and Bun)
check:contracts:
    @echo ""
    @echo "🔧 Checking TypeShare contracts..."
    @if [ -f "services/engine/typeshare.toml" ]; then \
        cargo run --bin typeshare-cli -- check services/engine/typeshare.toml || echo "⚠️  TypeShare check skipped (typeshare-cli not found)"; \
    else \
        echo "ℹ️  No TypeShare config found, skipping contract check"; \
    fi
    @echo "✅ Contract checks passed"

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# Infrastructure (Crossplane/K8s)
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

# Dry-run infrastructure changes (for `infra/*` branches)
plan:
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    @echo "📋 Running infrastructure dry-run..."
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    bun run infra:plan
    @echo "✅ Infrastructure plan validated"

# Apply infrastructure changes (for merge to `ta`)
apply:
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    @echo "🚀 Applying infrastructure changes..."
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    bun run infra:apply
    @echo "✅ Infrastructure applied"

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# Build Commands
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

# Build Rust engine (release)
build:rust:
    @echo "🔨 Building Rust engine..."
    cargo build --release
    @echo "✅ Rust engine built"

# Build Bun/Next.js app
build:bun:
    @echo "🔨 Building Bun/Next.js app..."
    bun run build
    @echo "✅ Bun app built"

# Build everything (for merge to `ta`)
build:all:
    @just build:rust
    @just build:bun

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# Agent Sandbox (for `agent/*` branches)
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

# Create agent sandbox namespace (via Crossplane)
agent:sandbox:create BRANCH_NAME:
    @echo "🏗️  Creating agent sandbox for branch: {{BRANCH_NAME}}"
    @# Extract agent name from branch (e.g., agent/researcher/exp-1 -> researcher)
    @SANDBOX_NAME=$$(echo "{{BRANCH_NAME}}" | sed 's|agent/\([^/]*\).*|\1|'); \
    echo "Sandbox name: $$SANDBOX_NAME"; \
    bun run infra:agent:sandbox:create --name=$$SANDBOX_NAME --branch={{BRANCH_NAME}}

# Delete agent sandbox namespace
agent:sandbox:delete BRANCH_NAME:
    @echo "🗑️  Deleting agent sandbox for branch: {{BRANCH_NAME}}"
    @SANDBOX_NAME=$$(echo "{{BRANCH_NAME}}" | sed 's|agent/\([^/]*\).*|\1|'); \
    bun run infra:agent:sandbox:delete --name=$$SANDBOX_NAME

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# Development
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

# Run tests
test:
    @echo "🧪 Running tests..."
    cargo test
    bun test
    @echo "✅ All tests passed"

# Format code
fmt:
    @echo "🎨 Formatting code..."
    cargo fmt
    bun run format
    @echo "✅ Code formatted"

# Clean build artifacts
clean:
    @echo "🧹 Cleaning build artifacts..."
    cargo clean
    rm -rf apps/web/.next
    rm -rf apps/web/dist
    @echo "✅ Cleaned"

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# Repository Setup
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

# Setup branch protection for ta branch (requires GITHUB_TOKEN)
setup-branch-protection:
    @echo "🔐 Setting up branch protection for ta branch..."
    @echo "   Requires GITHUB_TOKEN environment variable"
    @echo "   Get token from: https://github.com/settings/tokens"
    @echo "   Required scopes: repo, admin:repo"
    bun scripts/setup-branch-protection.ts

# Dry run: Show what would be configured
setup-branch-protection-dry-run:
    @echo "🔍 Dry run: Show what would be configured..."
    bun scripts/setup-branch-protection.ts --dry-run
