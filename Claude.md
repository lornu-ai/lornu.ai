# lornu.ai - External LLM Context

## Project Summary
This is the **lornu.ai** web application - a modern SaaS landing page and frontend for the Lornu AI platform.

## Tech Stack
- **Framework**: React 18 + TypeScript
- **Build Tool**: Vite 6
- **Package Manager**: Bun (preferred) or npm/pnpm
- **Styling**: Tailwind CSS 4
- **UI Components**: Radix UI + Material UI
- **Runtime**: Node.js 18+ or Bun

## Repository Type
**Satellite Repository** - Application source code. Deployment manifests managed in [private-lornu-ai](https://github.com/lornu-ai/private-lornu-ai) hub.

## Quick Start
```bash
# Install dependencies
bun install  # or npm install

# Run development server
bun run dev  # or npm run dev

# Build for production
bun run build  # or npm run build
```

## Deployment
- **Container**: Docker with Nginx (serves static assets)
- **Platform**: `linux/amd64` (required for EKS)
- **Registry**: AWS ECR (`lornu-web` repository)
- **Domain**: `future.lornu.ai` (staging)
- **Managed By**: Flux CD (in hub repo)

## Health Check
- Endpoint: `/healthz` (required for Kubernetes)
- Implemented in Nginx config

## Structure
```
lornu.ai/
├── src/
│   └── app/          # React components
├── public/           # Static assets
├── Dockerfile        # Container build
└── package.json      # Dependencies
```

