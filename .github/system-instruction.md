# lornu.ai - System Instruction

## Overview
This is a **satellite repository** for the Lornu AI web application frontend. Deployment manifests are managed in the [private-lornu-ai](https://github.com/lornu-ai/private-lornu-ai) hub repository.

## Tech Stack
- React 18 + TypeScript
- Vite 6 (build tool)
- Tailwind CSS 4 (styling)
- Radix UI + Material UI (components)
- Bun or npm/pnpm (package manager)

## Development
- Run `bun install` to install dependencies
- Run `bun run dev` to start development server
- Run `bun run build` to build for production

## Deployment Pipeline
1. Push to `main` → Triggers GitHub Actions
2. Build container image (`linux/amd64` platform required)
3. Push to AWS ECR (`lornu-web` repository)
4. Flux CD reconciles from hub repo
5. Kubernetes deploys to EKS cluster
6. Available at `future.lornu.ai` (staging)

## Container Build
- **Dockerfile**: Multi-stage build with Nginx
- **Platform**: Must build for `linux/amd64` (EKS requirement)
- **Health Check**: `/healthz` endpoint via Nginx

## Repository Structure
```
lornu.ai/
├── src/app/        # React components
├── public/         # Static assets
├── Dockerfile      # Container build
└── package.json    # Dependencies
```

## Related Repositories
- **[private-lornu-ai](https://github.com/lornu-ai/private-lornu-ai)**: Hub repository (deployment manifests)

