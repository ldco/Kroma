# Next Chat Handoff

Date: 2026-02-22
Branch: `master`
HEAD / upstream (`origin/master`): `9f972e0`
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

## Architecture Direction (Non-Negotiable)

1. Kroma is a desktop app and the target architecture is Rust-owned end to end (`src-tauri`).
2. `scripts/` are transitional migration scaffolding only, not a supported long-term runtime.
3. Temporary Rust wrappers around script behavior are allowed only while replacing that behavior with Rust modules.
4. Every script-backed runtime path should have:
   - a Rust replacement milestone
   - a script deprecation/removal milestone
5. Phase 1 is not complete until core runtime/orchestration, worker flows, and active backend integrations are owned by Rust modules.

## Runtime Consolidation Update (Newest)

1. Native Rust run-log ingest is now implemented in `src-tauri/src/db/projects/pipeline_ingest.rs` (`ProjectsStore::ingest_run_log`).
2. Default typed `runs/trigger` post-run path now uses:
   - Rust post-run wrapper (`pipeline::runtime`)
   - Rust post-run service (`pipeline::post_run`)
   - native Rust ingest (via `pipeline::backend_ops` hybrid adapter)
3. `backend.py ingest-run` is no longer used on the default Rust `runs/trigger` path.
4. Remaining post-run script dependency on that path is `backend.py sync-project-s3` only.
5. Script handoff hardening is in place:
   - `scripts/image-lab.mjs` emits `KROMA_PIPELINE_SUMMARY_JSON: {...}`
   - Rust parser prefers the structured marker and keeps text-line fallback during migration

## What Landed (Latest Relevant Backend Work)

Latest backend commit on `master`: `a620df7`  
Latest docs/handoff commit on `master`: `9f972e0`  
Commit messages:
- `fix(trigger): validate semantic errors before project lookup`
- `refactor(trigger): remove redundant validation clones`

### Implemented

1. Trigger validation regression fix (new):
   - extracted shared trigger semantic validation (`validate_trigger_input`) in `src-tauri/src/pipeline/trigger.rs`
   - handler now runs semantic validation before project lookup in `src-tauri/src/api/runs_assets.rs`
   - restores consistent `400` validation errors before `404` missing-project for invalid typed payloads
2. Trigger validation hardening (recent commits):
   - `ac769cd`: reject stage-incompatible `time`/`weather` typed filters
   - `e50bdf6`: reject empty typed list inputs (`scene_refs`, `style_refs`)
   - `d62c556`: tighten `runs/trigger` OpenAPI schema and response docs
3. Bootstrap endpoint OpenAPI response docs improved:
   - documented `GET /api/projects/{slug}/bootstrap-prompt` response body shape
   - documented `POST /api/projects/{slug}/bootstrap-import` response body shape (`200` / `400` / `404`)
4. Roadmap/handoff docs refreshed for recent Phase 1 progress (`6df48f7`, `4c283af`)
5. Voice legacy feature surface removed from Rust backend (previous commit `4c283af`):
   - deleted `src-tauri/src/api/voice.rs`
   - removed `/voice/*` routes from route catalog/router/OpenAPI parity expectations
   - deleted `src-tauri/tests/voice_endpoints.rs`
6. Secrets DB code split out of the old mixed voice/secrets module:
   - replaced `src-tauri/src/db/projects/voice_secrets.rs` with `src-tauri/src/db/projects/secrets.rs`
   - `projects.rs` schema setup now calls `ensure_secret_tables` / `ensure_secret_columns`
7. Legacy Python HTTP entrypoint removed:
   - deleted `scripts/backend_api.py`
   - removed `npm run backend:api` from `package.json`
8. Scope/docs/script cleanup:
   - removed voice schema remnants from `scripts/backend.py`
   - removed voice flows from `scripts/contract-smoke.sh` and `scripts/contract_smoke.py`
   - updated `README.md` and `docs/Kroma_—_Project_Spec_(Current_State_&_Roadmap).md` to reflect current scope
9. Prior commit (still relevant to current architecture): `6df48f7`
   - `runs/trigger` refactor is typed end-to-end and no longer accepts raw Rust-side `extra_args`

## Local Work In Progress (Uncommitted)

1. Next-phase runtime-consolidation kickoff (current local change)
   - added `src-tauri/src/pipeline/backend_ops.rs` typed Rust boundary for script-backed `backend.py` operations
   - covers `ingest-run` and `sync-project-s3` command building/execution shell with unit tests
2. `src-tauri/src/pipeline/mod.rs`
   - exports new `backend_ops` module
3. Unrelated local file change remains outside backend work:
   - `logo.png` modified (not part of backend/runtime work)

## Code Analysis (This Pass)

Scope reviewed:
1. Latest `runs/trigger` validation hardening in `src-tauri/src/pipeline/trigger.rs`
2. HTTP trigger handler validation/ordering in `src-tauri/src/api/runs_assets.rs`
3. Trigger endpoint coverage in `src-tauri/tests/pipeline_trigger_endpoints.rs`
4. Contract/mount parity after recent trigger/OpenAPI changes
5. Newly extracted shared trigger validation path for duplication/perf issues
6. Next-phase `backend.py` dependency surface in `scripts/image-lab.mjs` / `scripts/backend.py` (`ingest-run`, `sync-project-s3`)

Issues discovered:
1. Bug (fixed): `reference_sets` import accepted entries without an `items` field.
   - In merge mode, this could silently delete all items for a provided reference set because per-set item application is authoritative.
2. Regression (fixed): stage-aware trigger semantic validation ran only inside `PipelineTriggerService` after project lookup.
   - Invalid typed requests (e.g. `time` without compatible `stage`) could return `404 Project not found` before the expected `400` validation error.
3. Performance/maintenance issue (fixed): `PipelineTriggerService::trigger` reconstructed/cloned all `TriggerRunParams` fields just to call shared validation.
   - This added avoidable allocations and created a field-sync maintenance hazard for future trigger params.

Fixes implemented:
1. Added validation requiring `reference_sets[].items` to be explicitly present (use `[]` for an empty set).
2. Added bootstrap validation test coverage for the missing `reference_sets[].items` case.
3. Clarified bootstrap prompt rules and OpenAPI docs for `reference_sets` item-array semantics.
4. Extracted shared trigger semantic validation helper and reused it in both the HTTP handler and trigger service.
5. Restored pre-lookup validation precedence for stage-aware trigger semantic errors.
6. Updated endpoint tests to assert validation errors are returned before missing-project lookup for stage-aware invalid payloads.
7. Removed redundant param cloning by validating `TriggerRunParams` by reference before destructuring in the trigger service.
8. Started a typed Rust boundary (`pipeline::backend_ops`) for script-backed backend ingest/S3 sync operations.

Remaining risks / TODO:
1. `reference_sets` nested item behavior is authoritative per provided set (not per-item merge).
   - This is currently documented, but payloads do not yet include stable item IDs for fine-grained merge semantics.
2. Bootstrap `reference_sets` export item loading was optimized (batched query), but larger-dataset profiling is still TODO.
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

Local validation run for trigger semantic-validation regression fix:
1. `cargo test pipeline::trigger --lib`
2. `cargo test --test pipeline_trigger_endpoints`
3. `cargo test --test contract_parity --test http_contract_surface`

Result: passing.

Local validation run for trigger validation clone-removal follow-up:
1. `cargo test pipeline::trigger --lib`
2. `cargo test --test pipeline_trigger_endpoints`

Result: passing.

Local validation run for next-phase `pipeline::backend_ops` boundary kickoff:
1. `cargo fmt`
2. `cargo test pipeline::backend_ops --lib`

Result: passing.

Local validation run for runtime post-run ingest wiring (Rust-owned wrapper around script fallback):
1. `cargo fmt`
2. `cargo test pipeline::runtime --lib`
3. `cargo test pipeline::trigger --lib`
4. `cargo test pipeline::post_run --lib`
5. `cargo test --test pipeline_trigger_endpoints`
6. `cargo test --test contract_parity --test http_contract_surface`

Result: passing.

## Runtime Consolidation Update (Newest)

1. `pipeline::backend_ops` and `pipeline::post_run` are now wired into the default Rust pipeline runtime path for typed `runs/trigger`.
2. The default runtime now wraps `ScriptPipelineOrchestrator` with a Rust post-run wrapper:
   - disables script-side backend ingest (`--backend-db-ingest false`) to avoid duplicate ingest
   - keeps script-side S3 sync disabled (`--storage-sync-s3 false`) until Rust owns sync policy/options
   - runs Rust `PipelinePostRunService` ingest after successful script execution (best-effort; warning on failure)
3. Current transitional risk:
   - Rust post-run wrapper extracts `run_log_path` from the script stdout line `Run log: ...`
   - this is intentionally temporary and should be replaced by structured handoff (JSON stdout/metadata) or full Rust orchestration ownership

## Next Priority Work

1. Continue Phase 1 runtime consolidation (Rust app unification):
   - Rust pipeline orchestration replacement for `scripts/image-lab.mjs`
   - Replace `backend.py`-dependent pipeline operations with Rust modules behind the existing runtime boundary (in progress: `pipeline::backend_ops` + `pipeline::post_run`; backend ingest now routed through Rust wrapper on typed trigger path)
   - Rust worker/dispatcher replacement for script workers
   - typed Rust tool adapters for external tools/APIs
2. Decide whether `chat` / `agent instructions` belong in bootstrap scope and define rules before implementation.
3. Add desktop UI bootstrap flow:
   - export prompt action
   - paste/import modal
   - `dry_run` preview/diff confirmation
   - explicit merge vs replace UX
4. Expand bootstrap/OpenAPI nested response schemas/examples if client generation needs stricter typing.
5. Keep `docs/ROADMAP.md` and `NEXT_CHAT_HANDOFF.md` aligned when Phase 1 milestones or priorities shift.

## Suggested Starting Point For Next Chat

1. Wire `pipeline::backend_ops` into the Rust runtime/orchestration path as the boundary for backend ingest + S3 sync post-run operations (still script-backed initially).
   - Status: done for backend ingest on the default typed trigger path (best-effort Rust post-run wrapper)
2. Replace stdout `Run log:` parsing with structured handoff from the script runtime (JSON metadata output), or move run-log creation + post-run orchestration fully into Rust.
3. Extend Rust-owned post-run path to S3 sync once typed trigger/runtime policy/options are defined (keep sync disabled in script while migrating).
4. Use `docs/ROADMAP.md` as the first status check, then update `NEXT_CHAT_HANDOFF.md` after milestone-level changes.
5. Keep `scripts/` callable only behind explicit Rust interfaces/migration boundaries.
