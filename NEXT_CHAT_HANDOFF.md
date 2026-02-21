# Next Chat Handoff

Date: 2026-02-21
Branch: `master`
Last commit before this handoff update: `42edcd8`

## Current Architecture Decisions

1. Rust backend (`src-tauri`) remains the active implementation path.
2. Contract-first routing with parity and HTTP-surface tests is preserved.
3. Implemented domain coverage now includes:
- projects/storage
- runs/assets
- asset-links
- analytics
- exports
- prompt templates
- provider accounts
- style guides
- characters
- reference sets + nested items
4. Repository layer remains source-of-truth for validation, normalization, and SQL mapping.
5. Each implemented domain has dedicated integration tests aligned with contract routes.

## Completed Work In This Pass

1. Implemented reference-set CRUD endpoints:
- `GET /api/projects/{slug}/reference-sets`
- `POST /api/projects/{slug}/reference-sets`
- `GET /api/projects/{slug}/reference-sets/{referenceSetId}`
- `PUT /api/projects/{slug}/reference-sets/{referenceSetId}`
- `DELETE /api/projects/{slug}/reference-sets/{referenceSetId}`
2. Implemented nested reference-set item CRUD endpoints:
- `GET /api/projects/{slug}/reference-sets/{referenceSetId}/items`
- `POST /api/projects/{slug}/reference-sets/{referenceSetId}/items`
- `GET /api/projects/{slug}/reference-sets/{referenceSetId}/items/{itemId}`
- `PUT /api/projects/{slug}/reference-sets/{referenceSetId}/items/{itemId}`
- `DELETE /api/projects/{slug}/reference-sets/{referenceSetId}/items/{itemId}`
3. Added typed repository models + inputs for sets/items.
4. Added schema support:
- `reference_sets`
- `reference_set_items`
5. Added API module and route wiring:
- `src-tauri/src/api/reference_sets.rs`
- routing in `src-tauri/src/api/server.rs`
6. Added integration tests:
- `src-tauri/tests/reference_sets_endpoints.rs`
7. Updated HTTP contract-surface expected statuses for all reference-set routes.

## Major Refactors / Rewrites

1. Continued consistent CRUD architecture across new domains.
2. Introduced explicit nested-resource repository helpers (`fetch_reference_set_by_id`, `fetch_reference_set_item_by_id`).
3. Enforced item content invariant (`content_uri` or `content_text` required) at repository boundary.

## Key Issues Found

1. Reference-set and nested item routes were mounted but unimplemented.
2. No persistence schema existed for either resource.
3. Without explicit input schema in OpenAPI, endpoint-level behavior needed strong internal contracts and validation guards.

## Remaining Technical Debt

1. `db/projects.rs` is now very large and should be split into domain-focused modules.
2. Candidate schema overlap remains (`run_job_candidates` vs `run_candidates`).
3. Remaining unimplemented domains:
- chat sessions/messages
- agent instructions
- voice
- secrets

## Next Phase Goals (Immediate)

1. Implement chat sessions/messages domain skeleton with persisted list/create/read operations.
2. Implement agent-instructions list/create.
3. Start physical split of `db/projects.rs` (first extraction pass by domain).

## Validation Snapshot

1. `cargo fmt --all`
2. `cargo test`
3. `npm run backend:rust:test --silent`
4. Passing suites now include:
- contract parity
- HTTP contract-surface
- projects/storage
- runs/assets
- asset-links
- analytics
- exports
- prompt templates
- provider accounts
- style guides
- characters
- reference sets/items
