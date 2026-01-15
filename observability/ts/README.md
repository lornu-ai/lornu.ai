# TypeScript Observability Service

This service provides basic observability endpoints using Bun:
- `/healthz` — Health check endpoint
- `/metrics` — Simple in-memory metrics (Prometheus-style)
- Logging to stdout

## Running

```bash
bun run server.ts
```

Set `PORT` env variable to change the default port (8081).
