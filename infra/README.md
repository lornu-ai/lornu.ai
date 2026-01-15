# Lornu AI Infrastructure (Bun + CDK8s)

**No YAML. No State Files. Pure TypeScript.**

This directory contains the programmable infrastructure for Lornu AI using:
- **Bun** - Lightning-fast TypeScript runtime
- **CDK8s** - Type-safe Kubernetes manifest generation
- **Crossplane** - Multi-cloud infrastructure orchestration
- **Server-Side Apply** - State-free GitOps pattern

## Quick Start

```bash
# Install dependencies
bun install

# Import Crossplane CRDs (one-time setup)
bun run import:crossplane

# Synthesize manifests for dev
bun run synth:dev

# Apply directly to cluster (state-free)
bun run apply
```

## Architecture

```
infra/
├── main.ts           # Entry point - all charts defined here
├── cdk8s.yaml        # CDK8s config (imports Crossplane CRDs)
├── package.json      # Bun dependencies
├── ci/
│   └── apply_ssa.ts  # Direct-to-cluster SSA applicator
└── dist/             # Generated manifests (gitignored or committed for GitOps)
```

## Environment Targeting

```bash
# Development
LORNU_ENV=dev bun run synth

# Staging
LORNU_ENV=staging bun run synth

# Production
LORNU_ENV=prod bun run synth
```

## Why Bun + CDK8s?

| Feature | Traditional YAML | Bun + CDK8s |
|---------|------------------|-------------|
| Type Safety | ❌ None | ✅ Full TypeScript |
| Speed | N/A | ✅ Instant synth |
| Abstraction | ❌ Copy-paste | ✅ Constructs |
| Validation | ❌ Runtime errors | ✅ Compile-time |
| State Files | ❌ Terraform state | ✅ Zero state |

## Mandatory Labels

All resources auto-include:
- `lornu.ai/environment` - dev/staging/prod
- `lornu.ai/managed-by` - crossplane
- `app.kubernetes.io/name` - component name
- `app.kubernetes.io/part-of` - lornu-ai
