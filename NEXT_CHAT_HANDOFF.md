# Next Chat Handoff

Date: 2026-02-21
Branch: `master`
Last commit before this handoff update: `2d28311`

## Current Architecture Decisions

1. Rust backend (`src-tauri`) is the authoritative implementation.
2. Contract-first routing remains enforced by:
- route catalog (`src-tauri/src/api/routes.rs`)
- mounted-route contract surface test (`src-tauri/tests/http_contract_surface.rs`)
- OpenAPI parity test (`src-tauri/tests/contract_parity.rs`)
3. Repository layer keeps business validation and persistence logic centralized.
4. API handlers remain thin and map typed repo errors to HTTP responses.
5. `db/projects.rs` is being physically split by domain; voice+secrets is now extracted to a dedicated submodule.

## Completed Work In This Pass

1. Implemented and validated voice endpoints:
- `POST /api/projects/{slug}/voice/stt`
- `POST /api/projects/{slug}/voice/tts`
- `GET /api/projects/{slug}/voice/requests/{requestId}`
2. Implemented and validated secrets endpoints:
- `GET /api/projects/{slug}/secrets`
- `POST /api/projects/{slug}/secrets`
- `DELETE /api/projects/{slug}/secrets/{providerCode}/{secretName}`
3. Added persistence schema support for:
- `voice_requests`
- `project_secrets`
4. Added integration suites:
- `src-tauri/tests/voice_endpoints.rs`
- `src-tauri/tests/secrets_endpoints.rs`
5. Updated contract-surface expected status mapping for all voice/secrets routes.
6. Started repository modularization by extracting voice/secrets domain code from `src-tauri/src/db/projects.rs` into:
- `src-tauri/src/db/projects/voice_secrets.rs`
7. `src-tauri/src/db/projects.rs` now re-exports voice/secrets input+summary types and delegates schema setup to module helpers.

## Major Refactors / Rewrites

1. Replaced in-file voice/secrets repository section with a dedicated domain module (`voice_secrets.rs`) to reduce core file sprawl.
2. Isolated voice/secrets schema creation and column migration logic behind module-level helper functions.
3. Removed duplicated voice/secrets structs/methods/helpers from `projects.rs` and kept public API surface stable via re-exports.

## Key Issues Found

1. Voice and secrets routes previously existed in contract scope but lacked Rust implementation.
2. `src-tauri/src/db/projects.rs` had continued to grow monolithically; maintainability risk increased each domain addition.
3. Voice/secrets domain logic and schema handling were tightly embedded in the monolithic file before extraction.

## Remaining Technical Debt

1. `src-tauri/src/db/projects.rs` is still large and should continue splitting (next candidates: chat/instructions, reference sets, provider/style/character).
2. API modules still duplicate `map_repo_error` / `internal_error` / JSON envelope helper code.
3. Candidate table overlap remains (`run_job_candidates` and `run_candidates`).

## Next Phase Goals (Immediate)

1. Continue physical repository split with the next domain slice (chat + agent instructions).
2. Introduce shared API handler utility layer for repeated error mapping and JSON response helpers.
3. Preserve strict parity checks and full test green status during each refactor step.

## Validation Snapshot

1. `cargo fmt --all`
2. `cargo test`
3. `npm run backend:rust:test --silent`
4. All suites passing, including new:
- `voice_endpoints`
- `secrets_endpoints`
