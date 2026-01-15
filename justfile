# Lornu AI - Developer Workflow Commands
# Usage: just <recipe>

set shell := ["bash", "-cu"]

# Default recipe shows available commands
default:
    @just --list

# ============================================
# Development
# ============================================

# Start infrastructure (synth CDK8s manifests)
dev-infra:
    cd infra && bun install && bun run synth

# Build Rust services in development mode
dev-services:
    cd services && cargo build

# ============================================
# Build
# ============================================

# Build all components
build: build-services build-infra

# Build Rust services (release mode)
build-services:
    cd services && cargo build --release

# Build/synthesize infrastructure manifests
build-infra:
    cd infra && bun install && bun run synth

# ============================================
# Test
# ============================================

# Run all tests
test: test-services test-infra

# Test Rust services
test-services:
    cd services && cargo test

# Validate infrastructure manifests
test-infra:
    cd infra && bun run validate

# ============================================
# Lint & Format
# ============================================

# Lint Rust services
lint:
    cd services && cargo clippy -- -D warnings

# Format Rust services
fmt:
    cd services && cargo fmt

# ============================================
# Infrastructure
# ============================================

# Synthesize CDK8s manifests
synth env="dev":
    cd infra && LORNU_ENV={{env}} bun run synth

# Apply manifests to cluster (requires kubectl)
apply env="dev":
    cd infra && LORNU_ENV={{env}} bun run apply

# Apply with dry-run validation
apply-dry-run env="dev":
    cd infra && LORNU_ENV={{env}} bun run apply:dry-run

# ============================================
# CI/CD
# ============================================

# Run full CI pipeline via Dagger
ci:
    dagger run bun ci/main.ts

# Run CI without infrastructure setup
ci-fast:
    dagger run bun ci/main.ts --skip-infra

# ============================================
# Clean
# ============================================

# Clean all build artifacts
clean: clean-services clean-infra

# Clean Rust build artifacts
clean-services:
    cd services && cargo clean

# Clean infrastructure artifacts
clean-infra:
    rm -rf infra/cdk8s.out

# ============================================
# Utilities
# ============================================

# Check versions of all tools
versions:
    @echo "Rust: $(rustc --version)"
    @echo "Cargo: $(cargo --version)"
    @echo "Bun: $(bun --version)"
    @echo "Just: $(just --version)"
    @echo "Kubectl: $(kubectl version --client --short 2>/dev/null || echo 'not installed')"

# Install development dependencies
setup:
    cd infra && bun install
    cd services && cargo fetch
    @echo "Setup complete!"
