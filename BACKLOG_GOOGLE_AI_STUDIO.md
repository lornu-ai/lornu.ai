# Backlog: Google AI Studio Integration

This backlog tracks the tasks required to integrate Google AI Studio (Gemini) into the **lornu.ai** platform.

## 1. Infrastructure & Secret Management

| ID | Task | Description | Priority | Status |
|----|------|-------------|----------|--------|
| GAS-001 | Secret Provisioning | Add `GOOGLE_AI_STUDIO_KEY` to `lornu-legacy` Secret Manager. | P0 | ⏳ Todo |
| GAS-002 | Cross-Project IAM | Update `scripts/setup-cross-project-secrets.sh` to grant `engine-sa@lornu-v2` access to the Gemini secret. | P0 | ⏳ Todo |
| GAS-003 | Secret Sync | Add Gemini secret to `secrets.json.example` and verify `ci/sync_secrets.ts` compatibility. | P1 | ⏳ Todo |

## 2. Rust Engine Integration

| ID | Task | Description | Priority | Status |
|----|------|-------------|----------|--------|
| GAS-004 | Gemini Provider | Implement a Gemini API client in `services/engine/src/tools/gemini.rs`. | P0 | ⏳ Todo |
| GAS-005 | Engine API Extension | Add `/api/ai/gemini/generate` endpoint to Axum server. | P0 | ⏳ Todo |
| GAS-006 | Engine Observability | Integrate Gemini API calls into OpenTelemetry tracing. | P2 | ⏳ Todo |
| GAS-007 | Streaming Support | Implement Server-Sent Events (SSE) for Gemini streaming responses. | P1 | ⏳ Todo |

## 3. Agent Framework & Examples

| ID | Task | Description | Priority | Status |
|----|------|-------------|----------|--------|
| GAS-008 | Gemini Agent Template | Create `agents/gemini-starter` as a reference implementation. | P1 | ⏳ Todo |
| GAS-009 | Gemini Tooling (TS) | Create TS library for agents to call the engine's Gemini API. | P1 | ⏳ Todo |
| GAS-010 | Multimodal Support | Add support for image/video inputs in the Gemini tool. | P2 | ⏳ Todo |

## 4. CI/CD & Testing

| ID | Task | Description | Priority | Status |
|----|------|-------------|----------|--------|
| GAS-011 | Integration Tests | Add Dagger task to verify Gemini connectivity during CI. | P1 | ⏳ Todo |
| GAS-012 | Documentation | Create `docs/GOOGLE_AI_STUDIO_SETUP.md`. | P0 | ⏳ Todo |

## 5. Developer Experience (DX)

| ID | Task | Description | Priority | Status |
|----|------|-------------|----------|--------|
| GAS-013 | `just` Commands | Add `just test-gemini` to verify local configuration. | P2 | ⏳ Todo |
| GAS-014 | Sandbox Integration | Update `infra/agent-sandbox.ts` to inject Gemini secrets into agent pods. | P1 | ⏳ Todo |

---

## Implementation Notes

### Google AI Studio API
- Base URL: `https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent`
- Auth: `x-goog-api-key` header.
- Models: `gemini-1.5-pro`, `gemini-1.5-flash`.

### Secret Access Pattern
Agents should not call Google AI Studio directly with the API key. Instead:
1. Agent calls `lornu-engine`.
2. `lornu-engine` retrieves secret from GCP Secret Manager (cross-project).
3. `lornu-engine` performs the request to Google AI Studio.
4. `lornu-engine` returns the result to the agent.
