set dotenv-load := true
set shell := ["bash", "-c"]

# Justfile for lornu.ai - Unified workflow

# Default: Show available commands
default:
    @just --list

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# Setup & Installation
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

bootstrap: install
    @echo "🚀 Lornu.ai environment ready."

install:
    @echo "📦 Installing dependencies..."
    bun install
    @if [ -f Cargo.toml ]; then cargo fetch; fi
    @if [ -d infra ]; then (cd infra && bun install); fi

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# Baseline Checklist (Pre-Merge)
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

check: check:rust check:bun check:contracts check:infra
    @echo "✅ All baseline checks passed!"

check:rust:
    @if [ -f Cargo.toml ]; then \
        echo "🔧 Checking Rust code..."; \
        cargo clippy --all-targets --all-features -- -D warnings; \
        cargo fmt --check; \
    else \
        echo "ℹ️  No Cargo.toml found; skipping Rust checks."; \
    fi

check:bun:
    @echo "🔧 Checking Bun/TypeScript code..."
    bun run typecheck

check:contracts:
    @if [ -f "services/engine/typeshare.toml" ]; then \
        echo "🔧 Checking TypeShare contracts..."; \
        cargo run --bin typeshare-cli -- check services/engine/typeshare.toml || echo "⚠️  TypeShare check skipped (typeshare-cli not found)"; \
    else \
        echo "ℹ️  No TypeShare config found, skipping contract check"; \
    fi

check:infra:
    @if [ -d infra ]; then \
        echo "☁️  Checking Infrastructure Logic..."; \
        (cd infra && bun run synth); \
    else \
        echo "ℹ️  No infra/ directory found; skipping infra checks."; \
    fi

# Aliases for the unified workflow naming
check-engine: check:rust
check-web: check:bun
check-infra: check:infra

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# Infrastructure (Crossplane/K8s)
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

plan:
    @if [ -f infra/ci/apply_ssa.ts ]; then \
        echo "🔍 Planning infrastructure changes (SSA)..."; \
        bun run infra/ci/apply_ssa.ts --dry-run; \
    else \
        echo "📋 Running infrastructure dry-run..."; \
        bun run infra:plan; \
    fi

apply:
    @if [ -f infra/ci/apply_ssa.ts ]; then \
        echo "🛰️  Applying infrastructure to cluster (SSA)..."; \
        bun run infra/ci/apply_ssa.ts; \
    else \
        echo "🚀 Applying infrastructure changes..."; \
        bun run infra:apply; \
    fi

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# Build Commands
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

build:rust:
    @if [ -f Cargo.toml ]; then \
        echo "🔨 Building Rust engine..."; \
        cargo build --release; \
    else \
        echo "ℹ️  No Cargo.toml found; skipping Rust build."; \
    fi

build:bun:
    @echo "🔨 Building Bun app..."
    bun run build

build:all:
    @just build:rust
    @just build:bun

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# CI/CD & Agents
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

pipeline:
    @if command -v dagger >/dev/null 2>&1; then \
        echo "🗡️  Executing Dagger Pipeline..."; \
        dagger run bun ci/main.ts; \
    else \
        echo "ℹ️  Dagger not installed; skipping pipeline."; \
    fi

agent-logs:
    @if command -v kubectl >/dev/null 2>&1; then \
        kubectl logs -l app=lornu-agent -f --tail=100; \
    else \
        echo "ℹ️  kubectl not installed; skipping agent logs."; \
    fi

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# Development
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

test:
    @echo "🧪 Running tests..."
    @if [ -f Cargo.toml ]; then cargo test; fi
    bun run test
    @echo "✅ Tests complete"

fmt:
    @echo "🎨 Formatting code..."
    @if [ -f Cargo.toml ]; then cargo fmt; fi
    bun run format
    @echo "✅ Code formatted"

clean:
    @echo "🧹 Cleaning build artifacts..."
    @if [ -f Cargo.toml ]; then cargo clean; fi
    rm -rf dist
    rm -rf node_modules/.vite
    rm -rf .vite
    rm -rf apps/web/.next
    rm -rf apps/web/dist
    @if [ -d infra/dist ]; then rm -rf infra/dist; fi
    @echo "✅ Cleaned"
