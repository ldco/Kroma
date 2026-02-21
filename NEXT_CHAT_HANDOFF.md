# Next Chat Handoff

Date: 2026-02-21
Branch: `master`
Last commit before this handoff update: `1bb129c`

## Current Architecture Decisions

1. Rust backend (`src-tauri`) is the active delivery path.
2. Contract-first router remains enforced:
- `api/routes.rs` is the canonical route list.
- parity tests ensure OpenAPI and route catalog alignment.
- unimplemented domains return structured `501` stubs.
3. API is now split by domain modules:
- `api/projects.rs`
- `api/runs_assets.rs`
- `api/asset_links.rs`
- `api/analytics.rs`
4. Repository API (`db/projects.rs`) now covers four concrete domains beyond projects/storage:
- runs/assets read
- asset-links CRUD
- analytics read (`quality_reports`, `cost_events`)
5. Validation stays repository-first for deterministic behavior and reduced handler complexity.

## Completed Work In This Pass

1. Implemented analytics endpoints end-to-end:
- `GET /api/projects/{slug}/quality-reports`
- `GET /api/projects/{slug}/cost-events`
2. Added typed analytics persistence models:
- `QualityReportSummary`
- `CostEventSummary`
3. Added schema support for analytics tables:
- `quality_reports`
- `cost_events`
4. Added analytics API module and router wiring:
- `src-tauri/src/api/analytics.rs`
- dispatch in `src-tauri/src/api/server.rs`
5. Added analytics integration tests:
- `src-tauri/tests/analytics_endpoints.rs`
6. Updated contract-surface status expectations for analytics routes.

## Major Refactors / Rewrites

1. Continued domain decomposition at API level by introducing `api/analytics.rs`.
2. Added explicit analytics row mappers to avoid handler-side data shaping.
3. Kept contracts strongly typed and list limits explicit (default 500, clamped in store).

## Key Issues Found

1. Analytics routes were mounted but unimplemented.
2. Analytics persistence schema was absent.
3. First pass had a mapper fallback to a non-selected column (`meta_json`) causing `500`; fixed by canonicalizing on `summary_json` for quality reports.

## Remaining Technical Debt

1. `db/projects.rs` still carries multiple domains and should be split into per-domain modules.
2. `run_job_candidates` remains in schema path while reads use `run_candidates` as canonical source.
3. Remaining unimplemented contract domains:
- exports
- prompt templates
- provider accounts
- style guides
- characters
- reference sets
- chat
- agent instructions
- voice
- secrets

## Next Phase Goals (Immediate)

1. Implement exports read surfaces:
- `GET /api/projects/{slug}/exports`
- `GET /api/projects/{slug}/exports/{exportId}`
2. Add integration tests for export listing + detail not-found behavior.
3. Begin first repository file split (`db/projects.rs` -> domain submodules).

## Validation Snapshot

1. `cargo fmt --all`
2. `cargo test`
3. `npm run backend:rust:test --silent`
4. Passing suites include:
- contract parity
- HTTP contract surface checks
- projects/storage endpoints
- runs/assets endpoints
- asset-links endpoints
- analytics endpoints
