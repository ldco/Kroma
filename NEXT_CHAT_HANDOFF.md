# Next Chat Handoff

Date: 2026-02-22
Branch: `master`
HEAD / upstream (`origin/master`): `8b0fd91`
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
   - `reference_sets` + nested items (committed on `master`)
   - `secrets` metadata only (`provider_code`, `secret_name`, `has_value`; no secret values) (committed on `master`)
   - prompt templates
4. Product direction clarified: `scripts/` is transitional only; target architecture is a single Rust-owned application (API + runtime/orchestration + workers).
5. Rust pipeline runtime/trigger boundary and Rust-owned typed trigger endpoint (`POST /api/projects/{slug}/runs/trigger`) are now committed on `master` (script-backed execution remains behind the runtime boundary).
6. Product-scope cleanup is now committed on `master`: voice feature code and the old Python HTTP backend entrypoint (`scripts/backend_api.py`) were removed; active pipeline/runtime script dependencies remain.
7. `docs/ROADMAP.md` is now tracked on `master` as the day-to-day progress board (kept alongside the larger spec doc).

## What Landed (Latest Relevant Backend Work)

Latest commit on `master`: `8b0fd91`  
Commit message: `docs(api): document bootstrap responses and refresh roadmap`

### Implemented

1. Bootstrap endpoint OpenAPI response docs improved:
   - documented `GET /api/projects/{slug}/bootstrap-prompt` response body shape
   - documented `POST /api/projects/{slug}/bootstrap-import` response body shape (`200` / `400` / `404`)
2. Roadmap/handoff docs refreshed for recent Phase 1 progress (`6df48f7`, `4c283af`)
3. Voice legacy feature surface removed from Rust backend (previous commit `4c283af`):
   - deleted `src-tauri/src/api/voice.rs`
   - removed `/voice/*` routes from route catalog/router/OpenAPI parity expectations
   - deleted `src-tauri/tests/voice_endpoints.rs`
4. Secrets DB code split out of the old mixed voice/secrets module:
   - replaced `src-tauri/src/db/projects/voice_secrets.rs` with `src-tauri/src/db/projects/secrets.rs`
   - `projects.rs` schema setup now calls `ensure_secret_tables` / `ensure_secret_columns`
5. Legacy Python HTTP entrypoint removed:
   - deleted `scripts/backend_api.py`
   - removed `npm run backend:api` from `package.json`
6. Scope/docs/script cleanup:
   - removed voice schema remnants from `scripts/backend.py`
   - removed voice flows from `scripts/contract-smoke.sh` and `scripts/contract_smoke.py`
   - updated `README.md` and `docs/Kroma_—_Project_Spec_(Current_State_&_Roadmap).md` to reflect current scope
7. Prior commit (still relevant to current architecture): `6df48f7`
   - `runs/trigger` refactor is typed end-to-end and no longer accepts raw Rust-side `extra_args`

## Local Work In Progress (Uncommitted)

1. Bootstrap export performance improvement (current local change)
   - `load_reference_sets()` now batch-loads `reference_set_items` for the project and groups in Rust
   - removes per-reference-set nested item queries during bootstrap export/snapshot loading
2. Handoff + roadmap doc refresh (current local change)
   - Align notes with pushed commits `4c283af` and `8b0fd91`

## Code Analysis (This Pass)

Scope reviewed:
1. Local bootstrap backend changes in `src-tauri/src/db/projects/bootstrap.rs`
2. Bootstrap integration tests in `src-tauri/tests/bootstrap_endpoints.rs`
3. Bootstrap import request schema docs in `openapi/backend-api.openapi.yaml`
4. Project spec + handoff docs for roadmap consistency
5. Local runtime-consolidation WIP (`runs/trigger`, pipeline runtime/trigger modules)
6. Voice/legacy-scope cleanup across Rust API/DB/OpenAPI/tests/scripts/docs
7. New progress tracker doc (`docs/ROADMAP.md`) for milestone/status handoff alignment
8. Trigger contract refactor removing raw `extra_args` compatibility path from Rust layers
9. Trigger validation tightening started: Rust now mirrors script one-of input-source requirement
10. Voice/legacy cleanup completion pass (Rust + scripts + docs) and follow-up OpenAPI docs polish

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
   - Fixed locally with batched item loading; needs final commit/push and future profiling on larger datasets.
3. `scripts/backend.py` still exists and remains an active dependency of `scripts/image-lab.mjs`.
   - Do not delete it until a Rust replacement path is wired for the needed pipeline operations.
4. `runs/trigger` remains script-backed at execution time.
   - HTTP/API and service contracts are now typed; runtime internals still need script orchestration replacement.

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

Local validation run for next-phase runtime consolidation kickoff:
1. `cargo fmt`
2. `cargo test pipeline::runtime --lib`
3. `cargo test pipeline::trigger --lib`
4. `cargo test --test http_contract_surface`
5. `cargo test --test pipeline_trigger_endpoints`
6. `cargo test --test contract_parity --test http_contract_surface`

Result: passing.

Local validation run for voice/legacy-scope cleanup:
1. `cargo fmt`
2. `cargo test`
3. `python3 -m py_compile scripts/backend.py scripts/contract_smoke.py`
4. `bash -n scripts/contract-smoke.sh`

Result: passing.

Local validation run for bootstrap export `reference_sets` loader optimization:
1. `cargo fmt`
2. `cargo test --test bootstrap_endpoints`

Result: passing.

## Next Priority Work

1. Commit/push the local bootstrap `reference_sets` export loader optimization (batch item loading; removes N+1 query pattern).
2. Continue tightening typed trigger validation/rules for supported combinations (one-of input-source invariant is done; next: mode-specific combinations and clearer invariants).
3. Decide whether `chat` / `agent instructions` belong in bootstrap scope and define rules before implementation.
4. Continue Phase 1 runtime consolidation (Rust app unification):
   - Rust pipeline orchestration replacement for `scripts/image-lab.mjs`
   - Replace `backend.py`-dependent pipeline operations with Rust modules behind the existing runtime boundary
   - Rust worker/dispatcher replacement for script workers
   - typed Rust tool adapters for external tools/APIs
5. Add desktop UI bootstrap flow:
   - export prompt action
   - paste/import modal
   - `dry_run` preview/diff confirmation
   - explicit merge vs replace UX
6. Expand bootstrap OpenAPI response schemas/examples if client generation needs stricter typing (top-level responses are now documented).
7. Keep `docs/ROADMAP.md` and `NEXT_CHAT_HANDOFF.md` aligned when Phase 1 milestones or priorities shift.

## Suggested Starting Point For Next Chat

1. Commit/push the local `load_reference_sets()` batch-loading optimization, then re-evaluate bootstrap export performance with larger sample data.
2. Tighten remaining `runs/trigger` typed validation rules (the `input` xor `scene_refs` invariant is done; next define mode/stage-specific combinations).
3. Continue moving orchestration responsibilities out of `scripts/image-lab.mjs` and into Rust behind the existing runtime boundary.
4. Use `docs/ROADMAP.md` as the first status check, then update `NEXT_CHAT_HANDOFF.md` after any milestone-level change.
5. Keep `scripts/` callable only behind explicit Rust interfaces/migration boundaries.
