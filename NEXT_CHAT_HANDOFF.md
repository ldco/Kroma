# Next Chat Handoff

Date: 2026-02-21
Branch: `master`
Last commit before this handoff update: `bbc5a82`

## Current Architecture Decisions

1. Rust backend (`src-tauri`) is the primary implementation path.
2. Router remains contract-first:
- explicit route catalog in `api/routes.rs`
- all OpenAPI routes mounted
- unimplemented domains return deterministic `501`
3. API layer is split by domain:
- `api/projects.rs`: projects + storage
- `api/runs_assets.rs`: runs + assets read surfaces
- `api/asset_links.rs`: asset-link CRUD and filtering
4. Repository layer (`db/projects.rs`) exposes typed domain operations for projects, storage, runs, assets, and asset-links.
5. Asset-link integrity rules are enforced in repository methods:
- link types constrained to contract enum values
- parent/child assets must exist within the same project
- parent and child must differ

## Completed Work In This Pass

1. Implemented asset-links domain end-to-end:
- `GET /api/projects/{slug}/asset-links`
- `POST /api/projects/{slug}/asset-links`
- `GET /api/projects/{slug}/asset-links/{linkId}`
- `PUT /api/projects/{slug}/asset-links/{linkId}`
- `DELETE /api/projects/{slug}/asset-links/{linkId}`
2. Added typed repository models + inputs for asset-links:
- `AssetLinkSummary`
- `CreateAssetLinkInput`
- `UpdateAssetLinkInput`
3. Added persistent schema support for `asset_links` table and repository row/query helpers.
4. Added integration test coverage:
- `src-tauri/tests/asset_links_endpoints.rs`
5. Updated contract-surface status expectations for newly implemented asset-link endpoints.

## Major Refactors / Rewrites

1. Kept domain boundaries explicit by adding `api/asset_links.rs` instead of expanding unrelated modules.
2. Reused consistent error-mapping semantics (`NotFound` / `Validation` / internal DB errors) across all implemented API domains.
3. Consolidated asset-link validation at repository boundary instead of scattering checks across handlers.

## Key Issues Found

1. Asset-link routes were mounted but previously returned `501` stubs.
2. No persistence model existed for asset-link relations.
3. Link validation and project asset-ownership checks were absent.

## Remaining Technical Debt

1. `db/projects.rs` is still oversized and should be split into focused modules.
2. `run_job_candidates` remains in schema paths while `run_candidates` is the current canonical read source.
3. Remaining unimplemented contract domains:
- analytics (`quality-reports`, `cost-events`)
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

1. Implement analytics read surfaces:
- `GET /api/projects/{slug}/quality-reports`
- `GET /api/projects/{slug}/cost-events`
2. Add integration tests for analytics list/read semantics and filter/limit behavior.
3. Begin first extraction pass of repository file layout (`db/projects.rs` split by domain).

## Validation Snapshot

1. `cargo fmt --all`
2. `cargo test`
3. Passing suites include:
- contract parity
- HTTP contract mount checks
- projects endpoints
- storage endpoints
- runs/assets endpoints
- asset-links endpoints
