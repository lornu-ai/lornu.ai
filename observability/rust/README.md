# Rust Observability Microservice

This is a minimal observability microservice using Axum and Tokio.

- `/healthz` — Health check endpoint
- `/metrics` — Example static metrics (Prometheus-style)

## Running

```bash
cargo run
```

The service listens on port 8082 by default.
