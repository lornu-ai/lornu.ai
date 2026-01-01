# lornu.ai - GitHub Copilot Instructions

## Repository Type
**Satellite Repository** for Lornu AI web application frontend.

## Tech Stack
- **Framework**: React 18 + TypeScript
- **Build**: Vite 6
- **Package Manager**: Bun (preferred) or npm/pnpm
- **Styling**: Tailwind CSS 4
- **UI**: Radix UI + Material UI components

## Quick Commands
```bash
bun install      # Install dependencies
bun run dev      # Start dev server
bun run build    # Build for production
bun run typecheck # Type check
```

## Key Files
- `src/app/App.tsx` - Main application component
- `src/app/components/` - React components
- `vite.config.ts` - Vite configuration
- `Dockerfile` - Container build (Nginx + static assets)

## Deployment
- Container images pushed to AWS ECR (`lornu-web` repository)
- Deployed via Flux CD (manifests in `private-lornu-ai` hub)
- Staging domain: `future.lornu.ai`

## Health Check
- Endpoint: `/healthz` (implemented in Nginx config)
- Required for Kubernetes liveness/readiness probes

## Standards
- Mobile-first responsive design
- Follow existing component patterns
- Use TypeScript strictly
- Maintain component organization in `src/app/components/`

