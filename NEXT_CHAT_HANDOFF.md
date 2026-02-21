# Next Chat Handoff

Date: 2026-02-21
Branch: `master`
Last commit before this handoff update: `37c3587`

## Current Architecture Decisions

1. Rust backend (`src-tauri`) remains the only active implementation path.
2. HTTP routing is contract-first and deterministic:
- `api/routes.rs` is the explicit route catalog.
- Every OpenAPI route is mounted; non-implemented routes return structured `501`.
3. Domain handlers are split by responsibility:
- `api/projects.rs` handles project + storage lifecycle.
- `api/runs_assets.rs` handles run/asset read surfaces.
4. Persistence stays in `db/projects.rs` for now, but run/asset mapping is explicit and typed:
- Run summaries, run details with jobs/candidates, and asset summaries/details are concrete store methods.
- JSON fields are parsed once at repository boundaries into `serde_json::Value`.
5. Candidate model direction is `run_candidates` as canonical read source for run detail surfaces.

## Completed Work In This Pass

1. Fixed broken in-progress repository state that did not compile:
- Implemented missing row mappers and run job/candidate fetch logic.
2. Implemented run read endpoints:
- `GET /api/projects/{slug}/runs`
- `GET /api/projects/{slug}/runs/{runId}`
- `GET /api/projects/{slug}/runs/{runId}/jobs`
3. Implemented asset read endpoints:
- `GET /api/projects/{slug}/assets`
- `GET /api/projects/{slug}/assets/{assetId}`
4. Added dedicated API module:
- `src-tauri/src/api/runs_assets.rs`
5. Wired all new handlers in router dispatch:
- `src-tauri/src/api/server.rs`
6. Added integration coverage for seeded run/job/candidate/asset data:
- `src-tauri/tests/runs_assets_endpoints.rs`
7. Updated contract surface status expectations for newly implemented routes:
- `src-tauri/tests/http_contract_surface.rs`
8. Strengthened run candidate schema for new DB creation by including full candidate fields at table creation.

## Major Refactors / Rewrites

1. Replaced placeholder references to missing helper functions with complete repository-level implementations.
2. Introduced a clean API boundary for runs/assets instead of extending `api/projects.rs` with unrelated logic.
3. Normalized JSON decoding and string fallback rules in repository row mapping helpers.

## Key Issues Found

1. Prior in-progress code introduced unresolved symbols (`row_to_run_summary`, `row_to_asset_summary`, `fetch_jobs_with_candidates`) that fully blocked compilation.
2. Run/asset contract routes were mounted but still routed to generic stubs.
3. Candidate schema definition was partially split between table creation and migration-style column adds, increasing drift risk.

## Remaining Technical Debt

1. `db/projects.rs` is still oversized and mixes schema management, normalization utilities, and domain query logic.
2. `run_job_candidates` still exists in schema paths while current read path treats `run_candidates` as canonical.
3. Many contract domains remain `501` stubs (asset-links, analytics, exports, templates, provider accounts, style guides, characters, reference sets, chat, instructions, voice, secrets).

## Next Phase Goals (Immediate)

1. Implement asset-links domain end-to-end:
- `GET /api/projects/{slug}/asset-links`
- `POST /api/projects/{slug}/asset-links`
- `GET /api/projects/{slug}/asset-links/{linkId}`
- `PUT /api/projects/{slug}/asset-links/{linkId}`
- `DELETE /api/projects/{slug}/asset-links/{linkId}`
2. Add integration tests for asset-link CRUD and filtering.
3. Start first extraction pass for `db/projects.rs` by moving run/asset query logic into a dedicated submodule.

## Validation Snapshot

1. `cargo fmt --all`
2. `cargo test`
3. Passing suites include:
- contract parity tests
- HTTP contract mount test
- projects endpoints tests
- storage endpoints tests
- runs/assets endpoints tests
