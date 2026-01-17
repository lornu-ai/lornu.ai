# =============================================================================
# ai-agent-pr-comment container (Dockworker base image pattern)
# =============================================================================
ARG RUST_VERSION=1.74.1
ARG BINARY_NAME=ai-agent-pr-comment

FROM rust:${RUST_VERSION}-slim-bookworm AS builder

WORKDIR /build

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY . .

RUN cargo build --release --bin ${BINARY_NAME}

FROM gcr.io/distroless/cc-debian12:nonroot

LABEL org.opencontainers.image.source="https://github.com/lornu-ai/lornu.ai" \
      org.opencontainers.image.vendor="Lornu AI" \
      org.opencontainers.image.licenses="Apache-2.0" \
      org.opencontainers.image.title="ai-agent-pr-comment" \
      org.opencontainers.image.description="Rust GitHub PR comment agent webhook service" \
      org.opencontainers.image.url="https://dockworker.ai" \
      lornu.ai/component="ai-agent-pr-comment" \
      lornu.ai/managed-by="dockworker"

WORKDIR /app

COPY --from=builder /build/target/release/${BINARY_NAME} /app/agent

ENV PORT=8080 \
    HOST=0.0.0.0 \
    RUST_LOG=info

USER nonroot:nonroot

EXPOSE 8080

ENTRYPOINT ["/app/agent"]
