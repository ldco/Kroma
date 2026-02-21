# Next Chat Handoff

Date: 2026-02-21
Branch: `master`
Last commit before this handoff update: `54741c1`

## Current Architecture Decisions

1. Rust backend (`src-tauri`) is the authoritative implementation.
2. Contract-first routing remains enforced by:
- route catalog (`src-tauri/src/api/routes.rs`)
- mounted-route contract surface test (`src-tauri/tests/http_contract_surface.rs`)
- OpenAPI parity test (`src-tauri/tests/contract_parity.rs`)
3. Repository layer keeps business validation and persistence logic centralized.
4. API handlers remain thin and map typed repo errors to HTTP responses.
5. `db/projects.rs` is being physically split by domain; voice+secrets and chat+agent-instructions are extracted to dedicated submodules.

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
6. Continued repository modularization by extracting voice/secrets domain code from `src-tauri/src/db/projects.rs` into:
- `src-tauri/src/db/projects/voice_secrets.rs`
7. Finished chat/instruction repository extraction into:
- `src-tauri/src/db/projects/chat_instructions.rs`
8. `src-tauri/src/db/projects.rs` now declares and re-exports both extracted domain modules, and delegates schema setup to module helpers.
9. Introduced shared API handler utilities to remove duplicated per-module code:
- `src-tauri/src/api/handler_utils.rs`
- centralized `ApiObject` alias
- centralized `map_repo_error`, `internal_error`, `into_json`
10. Refactored API modules (`projects`, `analytics`, `runs_assets`, `asset_links`, `characters`, `chat`, `agent_instructions`, `provider_accounts`, `style_guides`, `prompt_templates`, `reference_sets`, `exports`, `voice`, `secrets`) to use shared utilities.

## Major Refactors / Rewrites

1. Replaced in-file voice/secrets repository section with a dedicated domain module (`voice_secrets.rs`) to reduce core file sprawl.
2. Isolated voice/secrets schema creation and column migration logic behind module-level helper functions.
3. Removed duplicated voice/secrets structs/methods/helpers from `projects.rs` and kept public API surface stable via re-exports.
4. Removed duplicated chat/instruction structs/methods/helpers from `projects.rs` and moved them to `chat_instructions.rs` with stable re-exports.
5. Removed duplicated API-layer error/JSON helper code by introducing a shared utility module.

## Key Issues Found

1. Voice and secrets routes previously existed in contract scope but lacked Rust implementation.
2. `src-tauri/src/db/projects.rs` had continued to grow monolithically; maintainability risk increased each domain addition.
3. Voice/secrets domain logic and schema handling were tightly embedded in the monolithic file before extraction.

## Remaining Technical Debt

1. `src-tauri/src/db/projects.rs` is still large and should continue splitting (next candidates: reference sets, provider/style/character).
2. Candidate table overlap remains (`run_job_candidates` and `run_candidates`).

## Next Phase Goals (Immediate)

1. Continue physical repository split with the next domain slice (reference sets or provider/style/character).
2. Evaluate whether `api/response.rs` and `api/handler_utils.rs` should be unified into one response abstraction.
3. Preserve strict parity checks and full test green status during each refactor step.

## Validation Snapshot

1. `cargo fmt --all`
2. `cargo test`
3. `npm run backend:rust:test --silent`
4. All suites passing, including new:
- `voice_endpoints`
- `secrets_endpoints`

## Code Analysis Update (2026-02-21)

### Scope of analysis

1. Reviewed all newly added/refactored files in this pass:
- `src-tauri/src/db/projects/chat_instructions.rs`
- `src-tauri/src/db/projects.rs` (module wiring + re-exports)
- `src-tauri/src/api/handler_utils.rs`
- all API modules migrated to shared handler utilities.
2. Re-validated behavior through full Rust test suites and contract tests.
3. Attempted `cargo clippy --all-targets -- -D warnings` (tooling missing in this environment: `cargo-clippy` component not installed).

### Issues discovered

1. Internal server errors returned raw backend error details to clients (DB/join failure strings), creating information disclosure risk.

### Fixes implemented

1. Sanitized API internal error responses in `src-tauri/src/api/handler_utils.rs`:
- client now receives `Internal server error` for 500 responses.
- detailed error is logged via `tracing::error!`.
2. Added unit tests for shared error mapping in `src-tauri/src/api/handler_utils.rs`:
- custom not-found message mapping
- validation message mapping
- sanitized 500 response behavior
3. Re-ran validation:
- `cargo fmt --all`
- `cargo test`
- `npm run backend:rust:test --silent`

### Open tasks

1. Continue repository modularization by extracting the next domain from `src-tauri/src/db/projects.rs` (recommended: reference sets).
2. Decide whether to consolidate `src-tauri/src/api/response.rs` and `src-tauri/src/api/handler_utils.rs` into one response abstraction.
3. Install clippy component (`rustup component add clippy`) and enforce lint checks in CI/local workflow.

### Recommended next steps

1. Start reference-set domain extraction into `src-tauri/src/db/projects/reference_sets.rs` with stable public re-exports in `projects.rs`.
2. Keep parity/contract suite green after each extraction step (`contract_parity`, `http_contract_surface`, endpoint suites).
3. After extraction, evaluate shared helper consolidation (`response.rs` + `handler_utils.rs`) and standardize one API envelope style.
