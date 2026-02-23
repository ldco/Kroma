# Next Chat Handoff

Date: 2026-02-23
Branch: `master`
HEAD / upstream (`origin/master`): `c0fa57f`
Worktree: dirty (local uncommitted changes: `NEXT_CHAT_HANDOFF.md`)

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
8. Rust pipeline runtime now has a file-backed layered settings path for pipeline planning/runtime defaults:
   - app settings (preferred): `config/pipeline.settings.toml`
   - app settings fallback: `config/pipeline.settings.json`
   - project settings: `<project_root>/.kroma/pipeline.settings.json`
   - precedence enforced in Rust: request overrides > project settings > app settings > Rust defaults
9. Rust now owns additional `image-lab.mjs` planning/runtime configuration logic:
   - manifest parsing for `generation`, `safe_batch_limit`, and `output_guard`
   - postprocess planning config parsing/validation + planned postprocess record shaping
   - Rust dry-run run-log top-level/job planned postprocess + output-guard fields (manifest/settings-aware)
10. Example config templates now exist under `config/`:
   - `pipeline.settings.toml.example`
   - `pipeline.manifest.json.example`
   - `postprocess.json.example`

## Latest Review Pass (2026-02-23, settings layer + runtime wiring)

### Scope Reviewed

1. `src-tauri/src/pipeline/settings_layer.rs` (new layered config loader/parser)
2. `src-tauri/src/pipeline/runtime.rs` (effective request merge + settings application in dry/script paths)
3. `src-tauri/src/pipeline/postprocess_planning.rs` (planning subset parser/resolver)
4. `README.md` + `config/*.example` templates (new config documentation/examples)

### Issues Discovered (this pass)

1. Bug (fixed): explicit relative project settings paths were resolved from process CWD instead of `project_root`.
   - Impact: `project_settings_path` overrides could silently miss the intended file depending on launch directory.
2. Validation bug (fixed): settings parser accepted empty strings for path/profile fields.
   - Impact: invalid values like `"manifest_path": "   "` flowed into runtime resolution and failed later with less clear errors.
3. Performance/behavior issue (fixed): `RustDryRunPipelineOrchestrator` resolved layered settings even for `run` mode before delegating to the inner script orchestrator, which also resolves settings.
   - Impact: duplicated config file reads on run path and unnecessary extra parsing work.

### Fixes Implemented (this pass)

1. `settings_layer`: explicit relative project settings paths now resolve relative to `project_root`.
2. `settings_layer`: empty-string values are rejected as invalid typed settings fields.
3. `runtime`: `RustDryRunPipelineOrchestrator` now short-circuits non-dry requests to the inner orchestrator before layered settings resolution (removes duplicate run-path config loads).
4. Added regression coverage for:
   - empty-string settings rejection
   - explicit relative project settings path resolution
5. Added app-settings TOML support (new) with JSON fallback:
   - app loader now prefers `config/pipeline.settings.toml`
   - legacy `config/pipeline.settings.json` remains supported as fallback
6. Added config examples + README docs for the Rust-owned layered pipeline settings path.

### Validation (this pass)

1. `cargo test pipeline::settings_layer --lib`
2. `cargo test pipeline::runtime --lib`
3. `cargo test pipeline::trigger --lib`

Result: passing.

### Remaining Risks / TODO (this pass)

1. Request-level postprocess toggle booleans in `PipelinePostprocessOptions` are not tri-state (`bool` instead of `Option<bool>`).
   - Impact: runtime can express "enable" via request overrides, but cannot cleanly express an explicit request-level "disable" that overrides a settings-file `true` value.
   - Current mitigation: not exposed on typed API yet; mostly affects future request-level config override surfaces.
2. Explicit app/project settings paths are still treated as optional if the file is missing (loader returns defaults).
   - Could be hardened to fail fast for explicitly provided paths to avoid silent misconfiguration.
3. Project settings format is JSON-only for now (intentional compatibility choice); TOML support is app-layer only.

### Recommended Next Steps

1. Add a Rust config validation command/path (CLI or endpoint-internal utility) that validates layered app/project settings + manifest + postprocess config together before pipeline runs.
2. Convert request-level postprocess toggles in the typed runtime/request model to tri-state semantics (`Option<bool>` or equivalent) before exposing them on the API/UI.
3. Continue run-mode migration: move generation/post-process execution loop from `scripts/image-lab.mjs` into Rust execution services and replace script-owned run-log writing.

## Latest Patch (2026-02-23, runtime dry-run log shaping cleanup)

### Scope Reviewed

1. `src-tauri/src/pipeline/runtime.rs` (Rust dry-run run-log JSON assembly)
2. `src-tauri/src/pipeline/execution.rs` (typed planned run-log job record builders)

### Issues Discovered

1. Design debt (fixed): `RustDryRunPipelineOrchestrator` still hand-built script-shaped planned job JSON inline.
   - Impact: duplicate schema defaults in `runtime.rs`; drift risk vs typed `pipeline::execution` helpers during migration off `image-lab.mjs`.

### Fixes Implemented

1. Added typed planned run-log job record builder in `pipeline::execution`:
   - `build_planned_run_log_job_record(...)`
   - script-parity defaults for planned generation/postprocess/output-guard fields centralized in Rust
2. `RustDryRunPipelineOrchestrator` now maps planned jobs through the typed builder instead of inline `serde_json::json!` object shaping.
3. Added regression coverage:
   - `pipeline::execution` unit test for planned-job default fields
   - `pipeline::runtime` dry-run test reads generated run log and asserts planned job JSON fields (`planned_generation`, `planned_postprocess.pipeline_order`, `planned_output_guard`)

### Validation (latest patch)

1. `cargo fmt`
2. `cargo test pipeline::execution --lib`
3. `cargo test pipeline::runtime --lib`

Result: passing.

## Next Phase Started (2026-02-23, typed dry-run log envelope extraction)

### Scope

1. `src-tauri/src/pipeline/execution.rs`
2. `src-tauri/src/pipeline/runtime.rs`

### What Landed

1. `pipeline::execution` now defines typed dry-run run-log envelope structs/builders:
   - `ExecutionPlannedRunLogRecord`
   - `ExecutionPlannedRunLogContext`
   - `build_planned_run_log_record(...)`
   - shared defaults for planned postprocess/output-guard fields
2. `pipeline::runtime::RustDryRunPipelineOrchestrator` now serializes the typed dry-run run-log record directly instead of hand-assembling the top-level JSON envelope.
3. Added unit coverage for the new top-level typed dry-run run-log builder.

### Validation

1. `cargo fmt`
2. `cargo test pipeline::execution --lib`
3. `cargo test pipeline::runtime --lib`

Result: passing.

## Latest Patch (2026-02-23, runtime/execution extraction)

### Scope Reviewed

1. `src-tauri/src/pipeline/runtime.rs` (Rust dry-run path + `--input` file discovery)
2. `src-tauri/src/pipeline/execution.rs` (new Rust run-mode execution foundations)
3. Script parity behavior in `scripts/image-lab.mjs` for file output naming / candidate winner ranking / generation directory layout

### Issues Discovered

1. Bug (fixed): Rust recursive image discovery for typed dry `--input` runs used filesystem iteration order.
   - Impact: nondeterministic job ordering across machines/filesystems; unstable run-log job order and migration parity checks.
2. Parity gap (fixed): Rust dry-run path only created `runs/` while the script generation path creates `outputs`, `runs`, and archive directories up front.
   - Impact: dry-run directory side effects diverged from script behavior and from the future Rust run-mode path.

### Fixes Implemented

1. `pipeline::runtime` now sorts recursive directory entries during Rust `--input` image discovery for deterministic planning order.
2. `pipeline::runtime` dry-run path now uses `pipeline::execution` project directory helpers (`execution_project_dirs`, `ensure_generation_mode_dirs`) instead of hardcoded path joins.
3. `pipeline::execution` extracted additional script-parity helpers:
   - generation directory creation (`outputs`, `runs`, `archive/*`)
   - sanitized output file path builder (port of `buildFileOutputPath`)
   - candidate winner selection ranking (port of `pickBestCandidate`)
   - stricter candidate filename contract (`total_candidates >= 1`)
4. Next-phase starter landed after this patch:
   - `pipeline::execution::plan_candidate_output_paths(...)` now ports script path planning for generate/bg-remove/upscale/color pass order and naming (typed + unit tested)
   - `pipeline::execution::plan_job_candidate_output_paths(...)` now ports the per-job candidate loop expansion into ordered typed plans
   - `pipeline::execution::summarize_output_guard_report(...)` now ports script output-guard ranking math (`summarizeGuardReport`) with rounding parity tests
   - `pipeline::execution::resolve_job_outcome_from_candidates(...)` now ports script winner/failure job outcome shaping (`selected_candidate`, `final_output`/`output`, failure reason) with strict missing-output validation
   - `pipeline::execution::finalize_job_from_candidates(...)` now ports script `jobMeta` final field shaping and failed-output-guard counter increment behavior
   - `pipeline::execution::build_run_log_candidate_record(...)` / `build_run_log_job_record(...)` now assemble typed run-log candidate/job entries from execution results (generated `output` preserved separately from `final_output`)
   - `pipeline::execution::build_run_log_output_guard_record(...)` now ports `candidateMeta.output_guard` log shaping (checked_input/summary/files/bad_archive)

### Validation (latest patch)

1. `cargo test pipeline::execution --lib`
2. `cargo test pipeline::runtime --lib`
3. `cargo test --test pipeline_trigger_endpoints --test http_contract_surface`
4. `cargo fmt`

Result: passing.

### Open Tasks / Recommended Next Steps (updated)

1. Use the new Rust `pipeline::execution` path-building/ranking helpers inside an actual Rust run-mode loop (not only dry-run support/foundations).
   - started: candidate output/post-process path planning is now available as a typed Rust helper
2. Port candidate generation/post-process orchestration from `scripts/image-lab.mjs` into Rust execution services:
   - generation call step
   - optional bg-remove/upscale/color pass ordering
   - output-guard evaluation + candidate ranking/winner selection
3. Replace script run-log writing for run-mode with `pipeline::runlog` and remove `image-lab.mjs` run-mode ownership.

## Security / Audit Foundation Update (2026-02-23)

### Completed

1. Added Rust DB schema support for `api_tokens` (hashed bearer tokens) and `audit_events` (spec-aligned target columns).
2. Added auth token APIs:
   - `POST /auth/token`
   - `GET /auth/tokens`
   - `DELETE /auth/tokens/{tokenId}`
3. Added Bearer auth middleware in the Rust router:
   - enforces auth on mutating endpoints and `/auth/*`
   - keeps local dev bypass via `KROMA_API_AUTH_DEV_BYPASS` (default enabled)
4. Added audit writes and `audit_id` response fields for mutating handlers in scope:
   - `provider_accounts`
   - `secrets`
   - `projects` (project/storage upserts)
   - `agent_instructions` create/confirm/cancel
5. Added legacy script safety gates (`KROMA_ENABLE_LEGACY_SCRIPTS=1`) to:
   - `scripts/backend.py`
   - `scripts/agent_worker.py`
   - `scripts/agent_dispatch.py`
6. Added `scripts/README.md` deprecation note pointing to Rust runtime as the only supported backend/runtime.

### Remaining (Important)

1. Full removal of `scripts/image-lab.mjs` from the HTTP pipeline path is still not complete.
   - Rust owns more execution/planning/log shaping logic, but run-mode generation/post-process orchestration still uses script fallback.
2. Audit writes for export mutations are not implemented because the Rust HTTP surface currently has no export mutation endpoints.
3. OpenAPI security docs are only partially annotated (auth routes + bearer scheme added; endpoint-level security metadata still needs full pass).

## Latest Review Pass (2026-02-23)

### Scope (newly added Rust migration code reviewed)

1. `src-tauri/src/db/projects/pipeline_ingest.rs` (native Rust run-log ingest)
2. `src-tauri/src/pipeline/backend_ops.rs` (native ingest + Rust S3 sync backend ops)
3. Endpoint/runtime regression surface after recent trigger/runtime consolidation commits

### Issues Discovered (this pass)

1. Bug (fixed): fallback `candidate_index` generation in native ingest used a run-global counter instead of a per-job counter.
   - Impact: jobs with missing `candidate_index` fields could get incorrect candidate numbering (e.g. second job starting at `2` instead of `1`).
2. Edge-case bug (fixed): Rust S3 sync precheck only validated `project_root.exists()`.
   - Impact: a file at the configured `project_root` path would incorrectly pass precheck and fail later in `aws s3 sync` with a less clear runtime error.

### Fixes Implemented (this pass)

1. `pipeline_ingest`: fallback `candidate_index` now resets per job (`enumerate()` within each job candidate list).
2. Added ingest regression test covering two jobs with missing candidate indexes (`candidate_index` must start at `1` for each job).
3. `backend_ops`: Rust S3 sync precheck now requires `project_root` to be a directory (`is_dir()`).
4. Added backend ops regression test asserting file-backed `project_root` fails precheck without invoking AWS CLI.

### Validation (this pass)

1. `cargo test pipeline_ingest --lib`
2. `cargo test pipeline::backend_ops --lib`
3. `cargo fmt`
4. `cargo test --test pipeline_trigger_endpoints --test http_contract_surface`

Result: passing.

### Follow-up Hardening (2026-02-23)

1. Added native Rust ingest re-ingest/idempotency regression coverage for the same `run_log_path`.
2. Test verifies second ingest replaces the prior run state cleanly (single run row, refreshed job/candidate state, refreshed quality/cost rows).
3. File: `src-tauri/src/db/projects/pipeline_ingest.rs`

### Review Follow-up (2026-02-23, planning/runtime preflight)

Scope reviewed:
1. `src-tauri/src/pipeline/planning.rs` (typed planning manifest parsing/defaults)
2. `src-tauri/src/pipeline/runtime.rs` (Rust planning preflight + parity warning integration)
3. `src-tauri/src/api/runs_assets.rs` / `src-tauri/src/pipeline/trigger.rs` fallout from new runtime error variants/options

Issues discovered (fixed):
1. Regression: `ScriptPipelineOrchestrator::execute` ran Rust manifest planning preflight before `build_command`.
   - Impact: invalid manifest payloads could mask earlier validation/infrastructure errors (e.g. invalid project slug) and change error precedence.
2. Typed parser issue: planning manifest `scene_refs` / `style_refs` silently dropped empty-string entries.
   - Impact: Rust preflight could pass malformed arrays that the script would later fail on, weakening migration parity and early validation.

Fixes implemented:
1. Restored error precedence by building/validating the command (`build_command`) before running Rust planning preflight.
2. Added runtime regression test asserting invalid project slug is returned before manifest preflight errors.
3. Tightened planning manifest parser to reject empty-string entries in string arrays (new explicit error).
4. Added planning parser regression test for empty array entries.

Validation (review follow-up):
1. `cargo test pipeline::planning --lib`
2. `cargo test pipeline::runtime --lib`
3. `cargo test --test pipeline_trigger_endpoints --test http_contract_surface`

Result: passing.

Remaining risks / TODO (review follow-up):
1. Rust preflight still duplicates manifest parsing/planning work already performed by `image-lab.mjs` (temporary migration cost).
2. Rust now covers scene expansion parity for typed dry `input` runs, but run-mode execution/post-process remains script-owned.
3. Next migration step remains to replace script-owned generation/post-process execution with Rust execution modules.
4. Rust now passes planned jobs via `--jobs-file` for script-backed runs, but `image-lab.mjs` still owns the run-mode execution loop and final run-log writing on that path.

Review follow-up (2026-02-23, latest runtime slice):
1. Fixed regression in Rust preflight activation guard: typed dry `input` path runs were still delegating to script because the guard skipped `InputPath`.
2. Fixed temp-file leak edge case in script-backed `--jobs-file` path when the second `build_command` fails after writing the temp file.
3. Validation:
   - `cargo test pipeline::runtime --lib`
   - `cargo test --test pipeline_trigger_endpoints --test http_contract_surface`
4. Result: passing.

### Current Status / Open Tasks (updated)

1. Typed trigger post-run backend ops are Rust-owned end-to-end (Rust ingest + Rust S3 sync execution).
2. Remaining major script dependency on the app trigger path is `scripts/image-lab.mjs` generation/orchestration itself.
3. Next phase started: Rust `pipeline::planning` module now ports `image-lab` prompt composition + generation job planning.
4. Follow-up landed: typed planning manifest defaults + JSON parser (`scene_refs`, `style_refs`, `policy.default_no_invention`, `prompts`) with tests for script-subset parity.
5. Follow-up landed: `pipeline::runtime` now supports typed `manifest_path` and runs Rust planning preflight (manifest parse + job planning) before script execution when a manifest is provided.
6. Follow-up landed: runtime now compares Rust planned job count vs script summary job count (when available) and emits a warning on mismatch (parity signal during migration).
7. Next phase starter landed: runtime preflight now retains Rust-planned job IDs (not only count) and includes IDs in parity mismatch warnings.
8. Follow-up landed: Rust runtime now writes planned jobs to a temp JSON file and passes `--jobs-file` to `image-lab.mjs` for manifest-backed scene-ref runs, bypassing script-side planning on the typed app path.
9. Next phase starter landed: new `pipeline::runlog` Rust module provides typed summary-marker formatting and pretty JSON run-log writing helpers (foundation for replacing script run-log output).
10. Major follow-up landed: typed dry runs with `scene_refs` now execute in Rust (planning + run-log writing + summary marker) via `RustDryRunPipelineOrchestrator` and no longer require Node/script execution on that path.
11. Review+follow-up landed: typed dry runs with `input` path (file/dir) now also execute in Rust using Rust image-file discovery (recursive image scan parity) and no longer delegate to the script path.
12. Next phase starter landed: `pipeline::execution` Rust module (run-mode execution foundation) with script-parity candidate output filename logic and tests.
13. Follow-up landed: `pipeline::execution` now owns script-parity project directory layout conventions (`outputs`, `runs`, `upscaled`, `color_corrected`, `background_removed`, `archive/*`) with tests.
14. Continue extracting generation/orchestration stages from `scripts/image-lab.mjs` into Rust modules and remove the script fallback (next: move candidate generation/post-process loop pieces into `pipeline::execution` and reuse `pipeline::runlog` for final log output).

### Recommended Next Steps

1. Start a Rust orchestration module for run planning + job expansion currently handled in `scripts/image-lab.mjs`.
   - Started: `src-tauri/src/pipeline/planning.rs` (prompt composition + generation job planning parity helpers/tests)
   - Extended: planning manifest default values + typed JSON/file parsing for the planning subset used by `image-lab`
   - Wired (preflight): `ScriptPipelineOrchestrator::execute` validates manifest/planning in Rust before invoking `image-lab.mjs` when `manifest_path` is set
2. Move script stdout/run-log ownership into Rust (structured in-memory result + Rust log writer), then delete script summary parsing fallback.
3. Add re-ingest/idempotency tests for native Rust ingest (especially changed candidate/output paths across repeated `run_log_path` imports).
   - Started: same `run_log_path` replacement coverage added in `pipeline_ingest` tests; still expand for multi-job/missing-path edge cases

## Architecture Direction (Non-Negotiable)

1. Kroma is a desktop app and the target architecture is Rust-owned end to end (`src-tauri`).
2. `scripts/` are transitional migration scaffolding only, not a supported long-term runtime.
3. Temporary Rust wrappers around script behavior are allowed only while replacing that behavior with Rust modules.
4. Every script-backed runtime path should have:
   - a Rust replacement milestone
   - a script deprecation/removal milestone
   - script deletion in the same phase once Rust parity lands (no dormant legacy script retention)
5. Phase 1 is not complete until core runtime/orchestration, worker flows, and active backend integrations are owned by Rust modules.

## Runtime Consolidation Update (Newest)

1. Native Rust run-log ingest is now implemented in `src-tauri/src/db/projects/pipeline_ingest.rs` (`ProjectsStore::ingest_run_log`).
2. Default typed `runs/trigger` post-run path now uses:
   - Rust post-run wrapper (`pipeline::runtime`)
   - Rust post-run service (`pipeline::post_run`)
   - native Rust ingest + Rust AWS CLI sync execution (via `pipeline::backend_ops` hybrid adapter)
3. `backend.py ingest-run` and `backend.py sync-project-s3` are no longer used on the default Rust `runs/trigger` post-run path.
4. Remaining runtime script dependency on that path is `scripts/image-lab.mjs` generation/orchestration itself (post-run backend ops are Rust-owned).
5. Script handoff hardening is in place:
   - `scripts/image-lab.mjs` emits `KROMA_PIPELINE_SUMMARY_JSON: {...}`
   - Rust parser prefers the structured marker and keeps text-line fallback during migration
6. `scripts/image-lab.mjs` no longer executes backend ingest/S3 sync directly (post-run backend calls removed from the script run path).
7. Typed `runs/trigger` now injects `project_root` from Rust project storage when omitted, reducing dependency on script-side backend storage resolution for the app path.

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
