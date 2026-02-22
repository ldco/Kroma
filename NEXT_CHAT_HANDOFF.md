# Next Chat Handoff

Date: 2026-02-22
Branch: `master`
HEAD / upstream (`origin/master`): `868b71d`
Worktree: dirty (local uncommitted changes)

## Current Status

1. Rust backend (`src-tauri`, `axum` + SQLite) is the primary local API and remains the active backend surface.
2. Bootstrap prompt exchange is implemented and shipped on `master`:
   - `GET /api/projects/{slug}/bootstrap-prompt`
   - `POST /api/projects/{slug}/bootstrap-import` (`merge` / `replace`)
   - `dry_run` preview mode
   - change-summary / diff metadata in responses
3. Bootstrap import/export coverage now includes:
   - project metadata
   - provider accounts
   - style guides
   - `characters` (committed on `master`)
   - `reference_sets` + nested items (local, uncommitted)
   - `secrets` metadata only (`provider_code`, `secret_name`, `has_value`; no secret values) (local, uncommitted)
   - prompt templates
4. Product direction clarified: `scripts/` is transitional only; target architecture is a single Rust-owned application (API + runtime/orchestration + workers).

## What Landed (Latest Relevant Backend Work)

Latest commit on `master`: `868b71d`  
Commit message: `feat(bootstrap): support characters in prompt export/import`

### Implemented

1. `src-tauri/src/db/projects/bootstrap.rs`
   - Added `characters` to bootstrap export/import scope.
   - Added normalization/parsing for character inputs.
   - Added merge/replace application logic.
   - Added `dry_run` preview + diff/change-summary support.
   - Updated bootstrap prompt template/expected schema to include `characters`.
2. `src-tauri/tests/bootstrap_endpoints.rs`
   - Added/extended integration coverage for `characters` round-trip, replace safety, and dry-run no-write behavior.
3. `openapi/backend-api.openapi.yaml`
   - Added `characters` request schema coverage for bootstrap import.

## Local Work In Progress (Uncommitted)

1. Bootstrap support for `reference_sets` / `reference_set_items`
   - export/import/preview/diff/change-summary support added
   - replace-mode section semantics implemented
   - integration tests added/extended
   - OpenAPI bootstrap-import request schema updated
2. Bootstrap support for `secrets` metadata only
   - export includes metadata only (`provider_code`, `secret_name`, `has_value`)
   - import is metadata-only and merge-only for safety
   - no secret values exported or imported
   - existing secret values are preserved
   - integration tests added/extended
3. Project spec roadmap wording updated to make Rust runtime consolidation explicit as Phase 1 priority
   - `scripts/` documented as transitional, not end state
   - new Phase 1 runtime consolidation subsection added

## Code Analysis (This Pass)

Scope reviewed:
1. Local bootstrap backend changes in `src-tauri/src/db/projects/bootstrap.rs`
2. Bootstrap integration tests in `src-tauri/tests/bootstrap_endpoints.rs`
3. Bootstrap import request schema docs in `openapi/backend-api.openapi.yaml`
4. Project spec + handoff docs for roadmap consistency

Issues discovered:
1. Bug (fixed): `reference_sets` import accepted entries without an `items` field.
   - In merge mode, this could silently delete all items for a provided reference set because per-set item application is authoritative.

Fixes implemented:
1. Added validation requiring `reference_sets[].items` to be explicitly present (use `[]` for an empty set).
2. Added bootstrap validation test coverage for the missing `reference_sets[].items` case.
3. Clarified bootstrap prompt rules and OpenAPI docs for `reference_sets` item-array semantics.

Remaining risks / TODO:
1. `reference_sets` nested item behavior is authoritative per provided set (not per-item merge).
   - This is currently documented, but payloads do not yet include stable item IDs for fine-grained merge semantics.
2. `load_reference_sets()` uses per-set item queries (N+1 query pattern) during bootstrap export/snapshot loading.
   - Likely acceptable now, but may need optimization for large projects.

## Validation Status

From the feature pass that produced `868b71d` (per prior handoff):
1. `cargo fmt`
2. `cargo test --test bootstrap_endpoints`
3. `cargo test --test bootstrap_endpoints --test contract_parity --test http_contract_surface`
4. `cargo test`

Reported result: all passing.

Local validation run for the uncommitted bootstrap extensions:
1. `cargo fmt`
2. `cargo test --test bootstrap_endpoints`
3. `cargo test --test contract_parity --test http_contract_surface`
4. `cargo test`

Result: passing.

## Next Priority Work

1. Commit the local bootstrap extensions (`reference_sets` + secrets metadata) as a focused backend commit.
2. Decide whether `chat` / `agent instructions` / `voice` belong in bootstrap scope and define rules before implementation.
3. Start Phase 1 runtime consolidation (Rust app unification), not just CRUD/API completion:
   - Rust pipeline orchestration replacement for `scripts/image-lab.mjs`
   - Rust worker/dispatcher replacement for script workers
   - typed Rust tool adapters for external tools/APIs
4. Add desktop UI bootstrap flow:
   - export prompt action
   - paste/import modal
   - `dry_run` preview/diff confirmation
   - explicit merge vs replace UX
5. Improve OpenAPI response docs for bootstrap endpoints (responses currently less documented than request schema).

## Suggested Starting Point For Next Chat

1. Commit local bootstrap additions (`reference_sets` + secrets metadata) after reviewing diff.
2. Begin Rust runtime consolidation with a narrow first slice:
   - define Rust pipeline orchestration service interface + command execution adapters
   - wire a Rust-owned run trigger path (initially parity wrapper if needed)
3. Keep `scripts/` callable only as temporary fallback behind explicit migration boundaries.
