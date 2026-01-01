# Agent Protocols for lornu.ai

This satellite repository hosts the Lornu AI web application frontend.

## Repository Type
**Satellite Repository** - Application source code lives here; deployment manifests managed in the [hub repository](https://github.com/lornu-ai/private-lornu-ai).

## Tech Stack
- React 18 + TypeScript
- Vite 6 (build tool)
- Tailwind CSS 4 (styling)
- Radix UI + Material UI (components)
- Bun (preferred) or npm/pnpm (package manager)

## Deployment
- **Container Registry**: AWS ECR (`lornu-web`)
- **Kubernetes Cluster**: AWS EKS (lornu-hub-aws)
- **GitOps**: Flux CD (managed in hub repo)
- **Staging Domain**: `future.lornu.ai`

## Health Check
- Endpoint: `/healthz`
- Required for Kubernetes liveness/readiness probes
- Implemented via Nginx in Dockerfile

## Related Repositories
- **[private-lornu-ai](https://github.com/lornu-ai/private-lornu-ai)**: Hub repository (deployment manifests, Flux Kustomizations)

