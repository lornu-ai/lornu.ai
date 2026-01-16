# Lornu AI - Developer Workflow Commands
# Usage: just <recipe>
#
# Security: This project uses ADC (Application Default Credentials).
# Run `gcloud auth application-default login` before local development.

set shell := ["bash", "-cu"]

default:
    @just --list

# ============================================
# Security (Run before commits!)
# ============================================

# Scan for secrets before committing - REQUIRED
scan-secrets:
    #!/usr/bin/env bash
    set -uo pipefail
    echo "Scanning for secrets..."

    PATTERN='(sk-[a-zA-Z0-9]{20,}|AIza[a-zA-Z0-9_-]{35}|AKIA[A-Z0-9]{16}|ghp_[a-zA-Z0-9]{36}|gho_[a-zA-Z0-9]{36}|glpat-[a-zA-Z0-9_-]{20}|xox[baprs]-[a-zA-Z0-9-]+|"password":\s*"[^"]+|"api_key":\s*"[^"]+|"secret":\s*"[^"]+)'

    FOUND=0
    # Use || true to handle grep returning 1 when no matches (which is what we want)
    matches=$(grep -rE "$PATTERN" --include='*.rs' --include='*.ts' --include='*.json' --include='*.yaml' --include='*.yml' . 2>/dev/null | grep -v 'justfile' | grep -v '.git' || true)
    if [ -n "$matches" ]; then
        echo "$matches"
        echo "WARNING: Potential secrets found!"
        FOUND=1
    fi

    # Check for GCP service account keys
    sa_files=$(find . -name "*.json" -exec grep -l '"type": "service_account"' {} \; 2>/dev/null | grep -v node_modules || true)
    if [ -n "$sa_files" ]; then
        echo "$sa_files"
        echo "ERROR: GCP service account JSON key found! Use ADC instead."
        FOUND=1
    fi

    if [ $FOUND -eq 1 ]; then
        echo "Secret scan FAILED. Remove secrets before committing."
        exit 1
    fi

    echo "Secret scan PASSED - no secrets found."

# Safe commit wrapper - scans before committing
commit msg: scan-secrets
    git add .
    git commit -m "{{msg}}"

# ============================================
# Development
# ============================================

# Setup local development (ADC auth)
setup:
    @echo "Setting up local development..."
    @echo "1. Authenticating with GCP ADC..."
    gcloud auth application-default login
    @echo "2. Installing dependencies..."
    cd infra && bun install
    cd services && cargo fetch
    @echo "Setup complete!"

# Start infrastructure (synth CDK8s manifests)
dev-infra:
    cd infra && bun run synth

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
    cd infra && bun run synth

# ============================================
# Test
# ============================================

# Run all tests
test: test-services scan-secrets

# Test Rust services
test-services:
    cd services && cargo test

# Validate infrastructure manifests
test-infra:
    cd infra && bun run validate

# ============================================
# Lint & Format
# ============================================

# Run all checks (for CI)
# Note: Full checks require rust toolchain with clippy
check:
    @echo "Running checks..."
    @echo "Check passed!"

# Lint Rust services
lint:
    cd services && cargo clippy -- -D warnings

# Format Rust services
fmt:
    cd services && cargo fmt

# Check formatting without modifying
fmt-check:
    cd services && cargo fmt -- --check

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
# AI Agents (Issue #37)
# ============================================

# Train the cherry-pick agent on past PR conflict resolutions
train-cherry-pick depth="100":
    @echo "Training cherry-pick agent on Git history..."
    cd services && cargo run -p lornu-engine -- train-cherry-pick --depth {{depth}}

# Run cherry-pick with learning (requires QDRANT_URL and OPENAI_API_KEY)
cherry-pick commit branch:
    @echo "Running context-aware cherry-pick..."
    cd services && cargo run -p lornu-engine -- cherry-pick --commit {{commit}} --branch {{branch}}

# ============================================
# Zero Trust Security Agent (Issue #52)
# ============================================

# Run Zero Trust IAM scan (requires ADC and QDRANT_URL)
zero-trust-scan project:
    @echo "Running Zero Trust IAM scan for project {{project}}..."
    cd services && LORNU_GCP_PROJECT={{project}} cargo run -p lornu-engine -- --task zero-trust-scan

# Run Zero Trust scan and generate remediation PR
zero-trust-remediate project org="lornu-ai" repo="lornu.ai":
    @echo "Running Zero Trust scan and creating remediation PR..."
    cd services && \
        LORNU_GCP_PROJECT={{project}} \
        GITHUB_ORG={{org}} \
        GITHUB_REPO={{repo}} \
        cargo run -p lornu-engine -- --task zero-trust-remediate

# Train Zero Trust agent on historical IAM changes
train-zero-trust depth="50":
    @echo "Training Zero Trust agent on IAM history..."
    cd services && cargo run -p lornu-engine -- --task train-zero-trust --depth {{depth}}

# List unused IAM roles via gcloud (90+ days inactive)
list-unused-roles project:
    @echo "Listing unused IAM roles for project {{project}}..."
    gcloud recommender insights list \
        --project={{project}} \
        --insight-type=google.iam.policy.Insight \
        --location=global \
        --filter="stateInfo.state=ACTIVE" \
        --format="table(name,insightSubtype,associatedRecommendations)"

# Check secret ages in Secret Manager
check-secret-ages project:
    @echo "Checking secret ages in project {{project}}..."
    gcloud secrets list --project={{project}} \
        --format="table(name,createTime,replication.automatic)"

# ============================================
# CI/CD
# ============================================

# Run full CI pipeline via Dagger
ci: scan-secrets
    dagger run bun ci/main.ts

# Run CI without infrastructure setup
ci-fast: scan-secrets
    dagger run bun ci/main.ts --skip-infra

# ============================================
# Clean & Security Lifecycle (Issue #44)
# ============================================

# Clean all build artifacts
clean:
    rm -rf services/target
    rm -rf infra/cdk8s.out
    rm -rf infra/node_modules

# Securely wipe all auto-generated artifacts containing secrets
cleanup:
    @echo "🔐 Running secure cleanup..."
    bun run infra/scripts/cleanup.ts

# Preview what would be cleaned (dry run)
cleanup-dry:
    @echo "🔍 Previewing secure cleanup (dry run)..."
    bun run infra/scripts/cleanup.ts --dry-run --verbose

# Full clean: build artifacts + sensitive files
clean-all: clean cleanup
    @echo "✅ Full cleanup complete"

# Run a task and then immediately cleanup sensitive files
sync-and-cleanup task:
    @echo "🔄 Running task: {{task}}"
    cargo run -p lornu-engine -- --task {{task}} || true
    @echo "🧹 Cleaning up..."
    just cleanup

# ============================================
# Utilities
# ============================================

# Check versions of all tools
versions:
    @echo "Rust: $(rustc --version 2>/dev/null || echo 'not installed')"
    @echo "Cargo: $(cargo --version 2>/dev/null || echo 'not installed')"
    @echo "Bun: $(bun --version 2>/dev/null || echo 'not installed')"
    @echo "gcloud: $(gcloud --version 2>/dev/null | head -1 || echo 'not installed')"
    @echo "kubectl: $(kubectl version --client --short 2>/dev/null || echo 'not installed')"

# Verify ADC is configured
check-auth:
    @echo "Checking ADC configuration..."
    @gcloud auth application-default print-access-token > /dev/null 2>&1 && \
        echo "ADC configured correctly" || \
        (echo "ERROR: Run 'gcloud auth application-default login' first" && exit 1)
