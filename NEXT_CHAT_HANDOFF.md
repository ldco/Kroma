# Next Chat Handoff

Date: 2026-02-21
Branch: `master`
Last commit before this handoff update: `10dd6ff`

## Current Architecture Decisions

1. Rust backend (`src-tauri`) remains the active architecture path.
2. OpenAPI parity + contract mount guarantees stay enforced through tests.
3. Implemented API domains now include:
- projects/storage
- runs/assets
- asset-links
- analytics
- exports
- prompt templates
- provider accounts
- style guides
4. Domain handlers stay thin; validation and normalization remain repository responsibilities.
5. Every implemented domain has integration tests and explicit contract-surface status expectations.

## Completed Work In This Pass

1. Implemented style-guide CRUD endpoints end-to-end:
- `GET /api/projects/{slug}/style-guides`
- `POST /api/projects/{slug}/style-guides`
- `GET /api/projects/{slug}/style-guides/{styleGuideId}`
- `PUT /api/projects/{slug}/style-guides/{styleGuideId}`
- `DELETE /api/projects/{slug}/style-guides/{styleGuideId}`
2. Added typed style-guide repository model + inputs:
- `StyleGuideSummary`
- `CreateStyleGuideInput`
- `UpdateStyleGuideInput`
3. Added persistence schema:
- `style_guides` table
4. Added API module + routing:
- `src-tauri/src/api/style_guides.rs`
- route wiring in `src-tauri/src/api/server.rs`
5. Added integration coverage:
- `src-tauri/tests/style_guides_endpoints.rs`
6. Updated contract-surface expectations for style-guide routes.

## Major Refactors / Rewrites

1. Continued strict per-domain module boundaries.
2. Reused shared CRUD validation pattern for deterministic behavior.
3. Kept schema and row mappers explicit for clear API contracts.

## Key Issues Found

1. Style-guide routes were mounted but unimplemented (`501`).
2. No style-guide schema existed.
3. Contract lacked requestBody details; create/update validation now explicitly enforced by repository.

## Remaining Technical Debt

1. `db/projects.rs` is still large and should be split into domain modules.
2. Candidate overlap (`run_job_candidates` + `run_candidates`) remains.
3. Remaining unimplemented domains:
- characters
- reference sets
- chat
- agent instructions
- voice
- secrets

## Next Phase Goals (Immediate)

1. Implement character CRUD domain.
2. Add character integration tests (validation + lifecycle).
3. Start first physical split of `db/projects.rs` into submodules.

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
- style guides
