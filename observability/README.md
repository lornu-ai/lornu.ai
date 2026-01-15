
# Observability & SRE Track

This directory contains observability and SRE tooling for the lornu.ai project.

## Directory Structure

```
observability/
	README.md                # Overview and integration guide
	ts/
		server.ts              # Bun-based observability service (healthz, metrics, logging)
		README.md
	rust/
		Cargo.toml
		src/
			main.rs              # Minimal Axum-based microservice (healthz, metrics)
		README.md
```

## Components

### TypeScript (Bun) Service
- Exposes `/healthz` (health check) and `/metrics` (Prometheus-style metrics) endpoints.
- Simple in-memory metrics and logging to stdout.
- Easily run with `bun run server.ts`.

### Rust Microservice
- Exposes `/healthz` and `/metrics` endpoints using Axum and Tokio.
- Example static metrics, ready for extension.
- Run with `cargo run`.

## Integration Guidance

- Both services are self-contained and can be run independently for local or production observability.
- Designed for easy extension (add tracing, more metrics, etc.).
- Can be deployed as sidecars or standalone services in your infrastructure.

## Goals
- Provide health checks, metrics, and logging endpoints
- Enable easy integration with the main app and infrastructure
- Demonstrate best practices for SRE/observability in modern web apps

See subdirectory READMEs for details and usage instructions.