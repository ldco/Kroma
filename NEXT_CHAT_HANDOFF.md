# Next Chat Handoff

Date: 2026-02-21
Branch: `master`
Last commit before this handoff update: `6b39bb5`

## Current Architecture Decisions

1. Rust backend (`src-tauri`) is the active and authoritative implementation path.
2. Contract-first routing is enforced via:
- route catalog (`src-tauri/src/api/routes.rs`)
- mounted-route surface checks (`src-tauri/tests/http_contract_surface.rs`)
- OpenAPI parity checks (`src-tauri/tests/contract_parity.rs`)
3. Repository-level validation remains the single authority for payload rules and state transitions.
4. API handlers stay thin: parse/extract input, delegate to store, map typed errors to HTTP.
5. Secrets API returns non-sensitive summaries only (`has_value`, metadata), never raw secret value.

## Completed Work In This Pass

1. Implemented voice endpoints:
- `POST /api/projects/{slug}/voice/stt`
- `POST /api/projects/{slug}/voice/tts`
- `GET /api/projects/{slug}/voice/requests/{requestId}`
2. Implemented secrets endpoints:
- `GET /api/projects/{slug}/secrets`
- `POST /api/projects/{slug}/secrets`
- `DELETE /api/projects/{slug}/secrets/{providerCode}/{secretName}`
3. Added repository models + inputs:
- `VoiceRequestSummary`
- `SecretSummary`
- `CreateVoiceSttInput`
- `CreateVoiceTtsInput`
- `UpsertSecretInput`
4. Added persistence schema support:
- `voice_requests`
- `project_secrets`
5. Added endpoint integration tests:
- `src-tauri/tests/voice_endpoints.rs`
- `src-tauri/tests/secrets_endpoints.rs`
6. Updated contract-surface expected statuses for all voice/secrets routes.

## Major Refactors / Rewrites

1. Added first-class voice request persistence (rather than route stubs), including detail lookup by request id.
2. Added explicit secrets repository operations with normalized provider codes and strict required fields.
3. Ensured secret responses are safe-by-default (metadata only, no secret echo).

## Key Issues Found

1. Voice and secrets contract routes were listed but had no Rust implementations.
2. No Rust persistence schema existed for voice requests or project secrets.
3. No integration test coverage existed for either domain.

## Remaining Technical Debt

1. `src-tauri/src/db/projects.rs` is still oversized and should be physically split by domain.
2. API modules still repeat identical error/JSON helpers and can be centralized cleanly.
3. Candidate overlap remains (`run_job_candidates` and `run_candidates`) and needs consolidation.

## Next Phase Goals (Immediate)

1. Begin physical repository split for `db/projects.rs` by domain slices (start with newly added voice/secrets section).
2. Introduce shared API response/error helpers to reduce duplicated handler boilerplate.
3. Keep parity and endpoint integration suites green after each structural refactor step.

## Validation Snapshot

1. `cargo fmt --all`
2. `cargo test`
3. `npm run backend:rust:test --silent`
4. Passing suites include all prior domains plus:
- voice
- secrets
