# Next Chat Handoff

Date: 2026-02-21
Branch: `master`
Last commit before this handoff update: `6b61f5b`

## Current Architecture Decisions

1. Rust backend (`src-tauri`) remains the active implementation path.
2. Contract-first route catalog + parity tests remain strict guardrails.
3. Implemented domain modules now include:
- projects/storage
- runs/assets
- asset-links
- analytics
- exports
- prompt templates
- provider accounts
- style guides
- characters
4. Repository-layer validation stays central; handlers perform orchestration only.
5. Every implemented domain has explicit integration coverage and contract-surface expectations.

## Completed Work In This Pass

1. Implemented character CRUD endpoints end-to-end:
- `GET /api/projects/{slug}/characters`
- `POST /api/projects/{slug}/characters`
- `GET /api/projects/{slug}/characters/{characterId}`
- `PUT /api/projects/{slug}/characters/{characterId}`
- `DELETE /api/projects/{slug}/characters/{characterId}`
2. Added typed repository model + inputs:
- `CharacterSummary`
- `CreateCharacterInput`
- `UpdateCharacterInput`
3. Added schema support:
- `characters` table
4. Added API module + router wiring:
- `src-tauri/src/api/characters.rs`
- route dispatch updates in `src-tauri/src/api/server.rs`
5. Added integration tests:
- `src-tauri/tests/characters_endpoints.rs`
6. Updated HTTP contract-surface expected statuses for character routes.

## Major Refactors / Rewrites

1. Continued clean per-domain API module pattern.
2. Added character fetch/mapper helpers to keep SQL mapping explicit and reusable.
3. Reused repository-level CRUD validation semantics for consistency across domains.

## Key Issues Found

1. Character routes were mounted but still `501` stubs.
2. No character persistence schema existed.
3. Contract lacked request body schema details; repository now enforces required `name` semantics and update field constraints.

## Remaining Technical Debt

1. `db/projects.rs` remains too large and should be split by domain.
2. Candidate schema overlap (`run_job_candidates` vs `run_candidates`) remains unresolved.
3. Remaining unimplemented contract domains:
- reference sets (+ items)
- chat sessions/messages
- agent instructions
- voice
- secrets

## Next Phase Goals (Immediate)

1. Implement reference-set CRUD and item CRUD surfaces.
2. Add integration tests for nested reference-set item routes.
3. Start physically splitting `db/projects.rs` into domain submodules.

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
- characters
