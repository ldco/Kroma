# Next Chat Handoff

Date: 2026-02-21
Branch: `master`
Last commit before this handoff update: `ba03f4d`

## Current Architecture Decisions

1. Rust backend (`src-tauri`) is the primary forward path; script backend remains reference only.
2. Routing is contract-first:
- Route catalog is explicit (`api/routes.rs`) and parity-checked against OpenAPI.
- Every OpenAPI route is mounted; unimplemented routes return deterministic `501` stubs.
3. State uses concrete services, not dynamic repository interfaces:
- `AppState` carries `Arc<ProjectsStore>`.
- Removed `dyn` repository layer to reduce accidental complexity.
4. Project domain persistence is SQLite-backed via `db/projects.rs` with explicit methods for:
- project list/upsert/detail
- storage read/update (local + s3)
5. Runtime config:
- `KROMA_BACKEND_BIND` for HTTP bind (default `127.0.0.1:8788`)
- `KROMA_BACKEND_DB` for DB path (default `var/backend/app.db` repo-relative)

## Completed Work In This Pass

1. Rust backend scaffold finalized with executable server and tests.
2. Contract safety rails implemented:
- `tests/contract_parity.rs`
- `tests/http_contract_surface.rs`
3. Implemented real project endpoints:
- `GET /api/projects`
- `POST /api/projects`
- `GET /api/projects/{slug}`
4. Implemented real storage endpoints:
- `GET /api/projects/{slug}/storage`
- `PUT /api/projects/{slug}/storage/local`
- `PUT /api/projects/{slug}/storage/s3`
5. Added storage integration tests:
- `tests/storage_endpoints.rs`
6. Fixed normalization bug:
- username filter no longer collapses invalid input to fallback slug.

## Major Refactors / Rewrites

1. Removed dynamic repository abstraction (`Arc<dyn ...>`) in favor of concrete `ProjectsStore`.
2. Reworked `api/projects.rs` into typed request/response handlers.
3. Added explicit storage update semantics:
- local updates require at least one explicit field
- S3 fields can be updated independently
- empty string input clears optional storage fields

## Key Issues Found

1. Early code had abstraction layers without multiple implementations.
2. Slug fallback behavior leaked into query filtering and created implicit incorrect filters.
3. Storage endpoints were contract-declared but not implemented.

## Remaining Technical Debt

1. `db/projects.rs` still combines schema, normalization, storage policy, and CRUD concerns in one file.
2. Most non-project contract routes are still `501` stubs.
3. OpenAPI response schemas remain underspecified for strict typed validation.

## Next Phase Goals (Immediate)

1. Implement run and asset read endpoints:
- `GET /api/projects/{slug}/runs`
- `GET /api/projects/{slug}/runs/{runId}`
- `GET /api/projects/{slug}/runs/{runId}/jobs`
- `GET /api/projects/{slug}/assets`
- `GET /api/projects/{slug}/assets/{assetId}`
2. Add endpoint integration tests for those read paths.
3. Start splitting `db/projects.rs` into focused modules once read paths stabilize.

## Validation Snapshot

1. `cargo fmt --all`
2. `cargo test` (unit + parity + HTTP surface + project + storage endpoint tests)
3. Runtime smoke checks:
- `GET /health`
- `GET/POST /api/projects`
- `GET /api/projects/{slug}`
- `GET /api/projects/{slug}/storage`
- `PUT /api/projects/{slug}/storage/local`
- `PUT /api/projects/{slug}/storage/s3`
