# Next Chat Handoff

Date: 2026-02-21
Branch: `master`
Last commit before this handoff update: `3fbaead`

## Current Architecture Decisions

1. Rust backend (`src-tauri`) is the active system of record.
2. Contract-first routing remains enforced with parity and mount tests.
3. Implemented API is segmented by domain modules:
- `api/projects.rs`
- `api/runs_assets.rs`
- `api/asset_links.rs`
- `api/analytics.rs`
- `api/exports.rs`
- `api/prompt_templates.rs`
- `api/provider_accounts.rs`
4. Repository-level validation remains the single authority for payload correctness and domain invariants.
5. All implemented domains include dedicated integration tests and are wired into HTTP contract-surface expectations.

## Completed Work In This Pass

1. Implemented provider-account CRUD endpoints end-to-end:
- `GET /api/projects/{slug}/provider-accounts`
- `POST /api/projects/{slug}/provider-accounts`
- `GET /api/projects/{slug}/provider-accounts/{providerCode}`
- `PUT /api/projects/{slug}/provider-accounts/{providerCode}`
- `DELETE /api/projects/{slug}/provider-accounts/{providerCode}`
2. Added typed provider-account repository model and inputs:
- `ProviderAccountSummary`
- `UpsertProviderAccountInput`
- `UpdateProviderAccountInput`
3. Added schema support:
- `provider_accounts` table
4. Added API module and route wiring:
- `src-tauri/src/api/provider_accounts.rs`
- routing in `src-tauri/src/api/server.rs`
5. Added integration tests:
- `src-tauri/tests/provider_accounts_endpoints.rs`
6. Updated HTTP contract-surface expected statuses for provider-account routes.

## Major Refactors / Rewrites

1. Continued domain isolation at API boundaries rather than extending existing handlers.
2. Added provider-account fetch/row-mapper helpers to keep SQL mapping consistent.
3. Reused strict CRUD validation pattern (required create fields, at-least-one update field, path-based not-found semantics).

## Key Issues Found

1. Provider-account routes were mounted but still `501` stubs.
2. No provider-account schema or typed persistence path existed.
3. POST contract lacked explicit request schema in OpenAPI; repository now enforces `provider_code` as required.

## Remaining Technical Debt

1. `db/projects.rs` is still too large and should be split by domain.
2. Candidate schema overlap remains (`run_job_candidates` and `run_candidates`).
3. Unimplemented contract domains now include:
- style guides
- characters
- reference sets
- chat
- agent instructions
- voice
- secrets

## Next Phase Goals (Immediate)

1. Implement style-guide CRUD domain.
2. Add style-guide integration tests with validation and lifecycle coverage.
3. Start extracting domain-specific repository modules from `db/projects.rs`.

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
- prompt templates
- provider accounts
