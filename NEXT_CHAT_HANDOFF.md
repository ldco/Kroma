# Next Chat Handoff

Date: 2026-02-21
Branch: `master`

## Current Architecture Decisions

1. Rust backend is now the authoritative forward path (`src-tauri`), with contract-first route mounting from OpenAPI path/method inventory.
2. API surface is split into:
- `api/routes.rs`: canonical route catalog and domain grouping.
- `api/server.rs`: router construction, health endpoint, stub fallback for not-yet-implemented endpoints.
- `api/projects.rs`: real project handlers (`list`, `upsert`, `detail`).
3. Data access for projects is centralized in a concrete store:
- `db/projects.rs` (`ProjectsStore`) is the single entrypoint for project persistence logic.
- Removed trait-object repository indirection for this phase to reduce abstraction overhead and improve clarity.
4. Endpoint behavior policy:
- Implemented endpoints return real data.
- Unimplemented contract routes are mounted and return deterministic `501` with structured details.
5. Runtime defaults:
- Bind: `127.0.0.1:8788` (override with `KROMA_BACKEND_BIND`).
- DB: `var/backend/app.db` repo-relative (override with `KROMA_BACKEND_DB`).

## Completed Work In This Pass

1. Created Rust backend scaffold and test harness.
2. Added OpenAPI contract parity tests (`tests/contract_parity.rs`).
3. Added HTTP route-mount coverage test for all contract routes (`tests/http_contract_surface.rs`).
4. Added real SQLite-backed project endpoints:
- `GET /api/projects`
- `POST /api/projects`
- `GET /api/projects/{slug}`
5. Refactored backend state/repository design:
- Replaced dynamic `Arc<dyn ProjectsRepository>` pattern with concrete `Arc<ProjectsStore>`.
6. Fixed slug normalization issue in list filtering:
- non-normalizable `username` query values no longer collapse to implicit fallback slug.
7. Added focused endpoint tests for create/list/detail and validation (`tests/projects_endpoints.rs`).
8. Added npm scripts:
- `backend:rust`
- `backend:rust:test`

## Major Refactors / Rewrites

1. Rewrote `api/projects.rs` to typed request/response flow instead of ad-hoc payload extraction.
2. Reworked `db/projects.rs` to expose explicit store methods and initialization API.
3. Simplified server composition by removing unnecessary compatibility-oriented abstraction layers.

## Key Issues Identified During Analysis

1. Over-abstraction for early-stage code (`dyn` repository interface with one implementation).
2. Monolithic project persistence file with mixed concerns (schema, parsing, storage defaults, CRUD logic).
3. Input normalization edge case in query filtering (`slugify` fallback behavior leaking into filter semantics).

## Remaining Technical Debt

1. `db/projects.rs` still contains multiple responsibilities and should be split into smaller modules:
- schema/migrations
- normalization
- storage policy resolution
- store operations
2. Most contract routes still return `501` stubs and need staged implementation.
3. OpenAPI currently lacks strict response schemas for typed payload validation.

## Next Phase Goals (Immediate)

1. Implement project storage endpoints with real persistence:
- `GET /api/projects/{slug}/storage`
- `PUT /api/projects/{slug}/storage/local`
- `PUT /api/projects/{slug}/storage/s3`
2. Add integration tests for storage read/update behavior.
3. Keep deterministic route-mount guarantee while replacing stubs endpoint-by-endpoint.

## Validation Snapshot

1. `cargo fmt --all`
2. `cargo test` (unit + parity + HTTP surface + project endpoint tests)
3. Runtime smoke checks:
- `GET /health`
- `GET /api/projects`
- `POST /api/projects`
- `GET /api/projects/{slug}`

