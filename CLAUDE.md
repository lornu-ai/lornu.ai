# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Development Commands

All commands use `just` (command runner). Run `just` to see available commands.

```bash
# Setup
just setup              # Setup ADC auth + install dependencies
just check-auth         # Verify GCP ADC is configured

# Build
just build              # Build all (Rust release + infra)
just build-services     # Build Rust services (release)
just dev-services       # Build Rust services (dev mode)
just build-infra        # Synthesize CDK8s manifests

# Test
just test               # Run all tests + secret scan
just test-services      # Rust tests only (cd services && cargo test)

# Lint & Format
just lint               # Rust clippy (cd services && cargo clippy -- -D warnings)
just fmt                # Format Rust code
just fmt-check          # Check formatting without changes

# Single test (Rust)
cd services && cargo test <test_name>
cd services && cargo test -p lornu-engine <test_name>

# Security (required before commits)
just scan-secrets       # Scan for leaked secrets
just commit "msg"       # Safe commit with auto-scan

# Infrastructure
just synth env="dev"    # Synthesize CDK8s manifests for environment
just apply-dry-run env="dev"  # Validate manifests without applying

# CI
just ci                 # Full CI pipeline via Dagger
just ci-fast            # CI without infra setup

# Multi-Cloud OIDC Promotion (Tekton)
just promote-multi-cloud <image> <tag>  # Trigger OIDC promotion pipeline
just apply-tekton-promote               # Apply Tekton promotion resources
just validate-tekton-promote            # Dry-run validation
```

## Architecture Overview

This is a monorepo with a Rust backend, TypeScript infrastructure-as-code, and Dagger-based CI/CD.

### Directory Structure

- **services/**: Rust workspace with 4 crates
  - `engine/`: Core agent orchestrator (DNS sync, cherry-pick, zero-trust agents)
  - `gateway/`: API gateway (WIP)
  - `agent-worker/`: Long-running task executor (WIP)
  - `github-bot/`: GitHub automation
- **infra/**: Infrastructure as Code using CDK8s + Crossplane (TypeScript/Bun)
  - `main.ts`: Entry point - Hub/Spoke pattern with XRDs, Compositions, Claims
  - `src/constructs/`: Agent XRDs and Compositions for multi-cloud
- **ci/**: Dagger CI/CD pipeline (TypeScript) + Tekton
  - `main.ts`: Pipeline orchestrator
  - `dagger.ts`: Branch-triggered automations
  - `tekton/`: OIDC multi-cloud promotion (TEP-0147, ESO integration)
- **agents/**, **ai-agents/**: Experimental agent code

### Key Patterns

**Trunk-Based Development**: Single long-lived branch `ta` (trunk). Features via short-lived branches:
- `feat/*` → Dagger builds + checks (no infra changes)
- `infra/*` → Dagger validates infrastructure plan
- `agent/*` → Creates temporary K8s sandbox for testing
- Merge to `ta` → Full apply + release builds

**Infrastructure**: CDK8s synthesizes Kubernetes manifests, Crossplane handles multi-cloud orchestration. No Terraform, no YAML configs.

**Secrets**: External Secrets Operator (ESO) with OIDC authentication. Never commit secrets - `just scan-secrets` enforces this.

**Multi-Cloud CI/CD**: Tekton pipelines with OIDC-only authentication (no static credentials):
- EKS (`eks-lornu`): CI hub with IRSA for AWS auth
- GKE (`lornu-ai-production-cluster`): Promotion target
- AKS (`lornu-aks-hub`): Future primary hub (Phase 2)

### Tech Stack

- **Rust**: Tokio, Axum, kube-rs for K8s client
- **TypeScript**: Bun runtime, CDK8s for infrastructure
- **CI/CD**: Dagger + Tekton (OIDC promotion via TEP-0147 artifacts)
- **Cloud**: Multi-cloud via Crossplane (AWS primary, GCP, Azure)
- **Secrets**: ESO with OIDC (AWS IRSA, GCP Workload Identity, Azure WI)
