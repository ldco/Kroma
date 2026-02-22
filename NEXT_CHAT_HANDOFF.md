# Next Chat Handoff

Date: 2026-02-22
Branch: `master`
Last pushed commit before this handoff update: `4fc5fdb`

## Current Status

1. Rust backend is the primary local API (`axum` + SQLite) and is passing the full backend test suite.
2. Bootstrap prompt exchange is live and shipped on `master`:
   - export prompt endpoint
   - import endpoint (`merge` / `replace`)
   - `dry_run` preview mode
   - change-summary/diff metadata in responses
3. Bootstrap import/export currently covers:
   - project metadata
   - provider accounts
   - style guides
   - prompt templates
4. This pass added local (not yet committed) bootstrap support for `characters` across export/import/preview/diff.

## Scope (This Pass)

1. Continued backend implementation of the bootstrap flow.
2. Extended bootstrap domain coverage to include `characters`.
3. Updated integration tests to exercise `characters` in:
   - export/import round-trip
   - replace scope safety
   - dry-run preview no-write behavior
4. Updated lightweight OpenAPI request schema docs for `bootstrap-import` to include `characters`.

## Issues Found / Risks Addressed

1. Coverage gap (fixed):
   - bootstrap feature previously did not support `characters`, forcing users to recreate them manually after AI bootstrap imports.
2. Regression risk (mitigated):
   - added tests to ensure `replace` mode still does not delete `characters` when that section is omitted.
   - added dry-run assertions to confirm preview includes `characters` changes without persisting writes.

## Fixes / Changes Implemented (Local, Uncommitted)

1. `src-tauri/src/db/projects/bootstrap.rs`
   - Added `characters` to bootstrap export settings.
   - Added `characters` input parsing/normalization (`ProjectBootstrapCharacterInput`).
   - Added merge/replace apply logic for `characters`.
   - Added dry-run preview generation for `characters`.
   - Added diff/change-summary computation for `characters`.
   - Added `load_characters()` snapshot helper.
   - Updated bootstrap prompt template + expected response schema to include `characters`.
2. `src-tauri/tests/bootstrap_endpoints.rs`
   - Extended round-trip test with seeded/imported `characters`.
   - Extended replace-scope safety test with `characters`.
   - Extended dry-run test with `characters` preview and persistence checks.
3. `openapi/backend-api.openapi.yaml`
   - Added `characters` request schema under `POST /api/projects/{slug}/bootstrap-import`.

## Validation

Commands run:
1. `cargo fmt`
2. `cargo test --test bootstrap_endpoints`
3. `cargo test --test bootstrap_endpoints --test contract_parity --test http_contract_surface`
4. `cargo test`

Result: all passing.

## Open Tasks

1. Commit and push the local `characters` bootstrap support.
2. Extend bootstrap coverage to the next domains:
   - reference sets (likely highest value)
   - secrets (metadata only, no secret values)
   - voice/chat instructions (if included in bootstrap scope)
3. Add desktop UI actions for bootstrap prompt export/import:
   - export prompt button
   - import modal (paste AI JSON)
   - dry-run preview / diff confirmation
   - merge vs replace confirmation UX
4. Improve OpenAPI response documentation for bootstrap endpoints (currently request schema is only lightly documented).

## Recommended Next Steps

1. Commit/push this `characters` bootstrap backend phase as one focused commit.
2. Implement `reference_sets` bootstrap import/export next (same pattern as `characters`).
3. Start desktop UI integration using `dry_run` preview before enabling destructive replace imports.
