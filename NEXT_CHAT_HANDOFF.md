# Next Chat Handoff

Date: 2026-02-22
Branch: `master`
HEAD / upstream (`origin/master`): `12981a3`
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
5. Next phase has started locally: Rust pipeline runtime/trigger boundary exists and is now used by an initial Rust-owned HTTP trigger endpoint (`POST /api/projects/{slug}/runs/trigger`) with script-backed fallback execution.
6. Product-scope cleanup (local, uncommitted) removed voice feature code and the old Python HTTP backend entrypoint (`scripts/backend_api.py`); active pipeline/runtime script dependencies remain.
7. A new progress tracker (`docs/ROADMAP.md`, local/untracked) now complements the full spec doc and should be kept in sync with major Phase 1 milestones.

## What Landed (Latest Relevant Backend Work)

Latest commit on `master`: `12981a3`  
Commit message: `feat(bootstrap): add reference sets and secrets metadata support`

### Implemented

1. `src-tauri/src/db/projects/bootstrap.rs`
   - Added bootstrap export/import/preview/diff support for `reference_sets` + nested items.
   - Added bootstrap export/import (metadata-only, merge-only) support for `secrets`.
   - Added safety validation requiring explicit `reference_sets[].items` arrays (use `[]` for empty set).
   - Preserves existing secret values on metadata import.
2. `src-tauri/tests/bootstrap_endpoints.rs`
   - Extended round-trip / replace-scope / dry-run coverage for `reference_sets` and secrets metadata.
   - Added validation coverage for missing `reference_sets[].items`.
3. `openapi/backend-api.openapi.yaml`
   - Added bootstrap-import request schema for `reference_sets` and `secrets`.
   - Documented `reference_sets` object requires `name` and `items`.
4. `docs/Kroma_—_Project_Spec_(Current_State_&_Roadmap).md`
   - Clarified `scripts/` are transitional and added explicit Phase 1 runtime consolidation priority.

## Local Work In Progress (Uncommitted)

1. Phase 1 runtime consolidation kickoff (`src-tauri/src/pipeline/runtime.rs`, `src-tauri/src/pipeline/trigger.rs`, `src-tauri/src/pipeline/mod.rs`)
   - Added typed Rust pipeline orchestration interface (`PipelineOrchestrator`)
   - Added command execution adapter boundary (`PipelineCommandRunner`)
   - Added temporary script-backed orchestrator (`ScriptPipelineOrchestrator`) targeting `scripts/image-lab.mjs`
   - Added Rust trigger service (`PipelineTriggerService`) with run-mode spend confirmation enforcement and CLI flag injection
   - Added unit tests for command building, slug validation, success/error execution paths, and trigger semantics
2. Initial Rust-owned trigger endpoint (still script-backed under the Rust boundary)
   - `POST /api/projects/{slug}/runs/trigger` added to route catalog + router + OpenAPI
   - handler implemented in `src-tauri/src/api/runs_assets.rs`
   - Rust-side validation for `mode` (`dry`/`run`)
   - project existence check before invoking fallback orchestrator
   - run-mode spend confirmation enforcement via `PipelineTriggerService`
   - command failure mapping to API `400` with summarized message
   - typed request fields for trigger input (`project_root`, `input`, `scene_refs`, `style_refs`, `stage`, `time`, `weather`, `candidates`)
   - no raw CLI arg pass-through in Rust API/service/runtime (`extra_args` removed)
   - CLI flag construction is isolated to the runtime orchestrator implementation
   - exact one-of `input` / `scene_refs` invariant enforced in Rust (script parity)
3. `src-tauri/src/api/server.rs`
   - `AppState` owns a `PipelineTriggerService` initialized with the script-backed orchestrator (temporary fallback implementation)
4. Added endpoint tests for the new trigger endpoint:
   - validation + missing-project + missing confirmation paths
   - success path using injected fake orchestrator/service (no `node` dependency)
   - typed-field translation + validation-order coverage
   - exact one-of input-source validation (`input` xor `scene_refs`) coverage
   - conflicting typed inputs (`input` vs `scene_refs`) and `candidates` range validation coverage
5. Scope cleanup (local, uncommitted):
   - Removed Rust/OpenAPI/test/script voice request feature paths (`/voice/*`)
   - Split `src-tauri/src/db/projects/voice_secrets.rs` into `src-tauri/src/db/projects/secrets.rs` and removed voice DB code while preserving secrets
   - Deleted legacy Python HTTP backend entrypoint `scripts/backend_api.py`
   - Removed voice schema remnants from `scripts/backend.py` (`voice_requests`, `voice_asset_id` schema/index bits)
   - Updated README/spec/docs/handoff references to match removals
6. Roadmap/progress tracking docs (local, uncommitted):
   - Added `docs/ROADMAP.md` as the day-to-day status board
   - Kept `docs/Kroma_—_Project_Spec_(Current_State_&_Roadmap).md` as the fuller architecture/product reference

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
2. `cargo test --test bootstrap_endpoints --test contract_parity --test http_contract_surface --test pipeline_trigger_endpoints`

Result: passing.

## Next Priority Work

1. Continue tightening typed trigger validation/rules for supported combinations (one-of input-source invariant is done; next: mode-specific combinations and clearer invariants).
2. Decide whether `chat` / `agent instructions` belong in bootstrap scope and define rules before implementation.
3. Continue Phase 1 runtime consolidation (Rust app unification):
   - Rust pipeline orchestration replacement for `scripts/image-lab.mjs`
   - Replace `backend.py`-dependent pipeline operations with Rust modules behind the existing runtime boundary
   - Rust worker/dispatcher replacement for script workers
   - typed Rust tool adapters for external tools/APIs
4. Add desktop UI bootstrap flow:
   - export prompt action
   - paste/import modal
   - `dry_run` preview/diff confirmation
   - explicit merge vs replace UX
5. Improve OpenAPI response docs for bootstrap endpoints (responses currently less documented than request schema).
6. Keep `docs/ROADMAP.md` and `NEXT_CHAT_HANDOFF.md` aligned when Phase 1 milestones or priorities shift.

## Suggested Starting Point For Next Chat

1. Tighten remaining `runs/trigger` typed validation rules (the `input` xor `scene_refs` invariant is done; next define mode/stage-specific combinations).
2. Continue moving orchestration responsibilities out of `scripts/image-lab.mjs` and into Rust behind the existing runtime boundary.
3. Use `docs/ROADMAP.md` as the first status check, then update `NEXT_CHAT_HANDOFF.md` after any milestone-level change.
4. Keep `scripts/` callable only behind explicit Rust interfaces/migration boundaries.
