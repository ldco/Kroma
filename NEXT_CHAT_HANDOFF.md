# Next Chat Handoff

Date: 2026-02-21
Branch: `master`
Last commit before this handoff update: `9bdacc4`

## Current Architecture Decisions

1. Rust backend (`src-tauri`) continues as the primary implementation track.
2. Router remains contract-first and parity-tested against OpenAPI.
3. Implemented API is decomposed by domain modules:
- `api/projects.rs`
- `api/runs_assets.rs`
- `api/asset_links.rs`
- `api/analytics.rs`
- `api/exports.rs`
4. Repository layer has typed read/write surfaces for implemented domains; handlers remain thin and policy-free.
5. Domain list endpoints use explicit default limits and repository-side clamping.

## Completed Work In This Pass

1. Implemented exports read endpoints end-to-end:
- `GET /api/projects/{slug}/exports`
- `GET /api/projects/{slug}/exports/{exportId}`
2. Added typed export persistence model:
- `ProjectExportSummary`
3. Added schema support for exports table:
- `project_exports`
4. Added exports API module and route binding:
- `src-tauri/src/api/exports.rs`
- router dispatch in `src-tauri/src/api/server.rs`
5. Added exports integration tests:
- `src-tauri/tests/exports_endpoints.rs`
6. Updated HTTP contract-surface expectations for exports routes.

## Major Refactors / Rewrites

1. Continued clean domain separation by introducing dedicated exports API module.
2. Added explicit row mappers for exports to keep serialization contracts stable.
3. Fixed two mapper anti-patterns encountered during development:
- quality reports: removed fallback read from non-selected `meta_json`
- exports: removed fallback read from non-selected `meta_json`

## Key Issues Found

1. Exports routes were mounted but previously returned `501` stubs.
2. Exports persistence schema was missing.
3. Mapper fallback-to-missing-column patterns caused runtime `500` failures during test pass; corrected to canonical JSON columns.

## Remaining Technical Debt

1. `db/projects.rs` still contains multiple domains and should be split into domain-focused modules.
2. Legacy overlap remains in candidate schema paths (`run_job_candidates` vs `run_candidates`).
3. Unimplemented contract domains remain:
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

1. Implement prompt-template domain (`GET/POST/GET by id/PUT/DELETE`).
2. Add integration tests for prompt-template CRUD + validation.
3. Start physical repository split to reduce `db/projects.rs` blast radius.

## Validation Snapshot

1. `cargo fmt --all`
2. `cargo test`
3. `npm run backend:rust:test --silent`
4. Passing suites include:
- contract parity
- HTTP contract-surface
- projects/storage
- runs/assets
- asset-links
- analytics
- exports
