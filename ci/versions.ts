/**
 * Single source of truth for versions of tools and container images.
 */
export const versions = {
  bun: "1.0.25",
  dagger: "0.11.9",
  rust: "1.75-slim",
  images: {
    /**
     * Base image for Bun with a pinned Alpine version.
     */
    bun: "oven/bun:1.0.25-alpine",
    rust: "rust:1.75-slim",
    dind: "docker:dind",
    ciRunner: "gcr.io/lornu-ai/ci-runner",
  },
};