# Kroma Roadmap (Progress Tracker)

Last updated: 2026-02-22
Status: Phase 1 in progress (backend + runtime consolidation into Rust)

## Purpose

This file is the working progress tracker for Kroma.

- `docs/Kroma_—_Project_Spec_(Current_State_&_Roadmap).md` remains the full product/spec document.
- This file is the day-to-day roadmap status board: what is done, in progress, and next.

## Architecture Direction (Explicit)

- Kroma is a desktop app and the target architecture is Rust-owned end to end (`src-tauri`).
- `scripts/` are transitional migration scaffolding only, not a supported long-term runtime.
- No permanent script wrappers/adapters as an end state:
  - temporary Rust boundaries around scripts are allowed only to preserve momentum while replacing them
  - every script-backed path must have a Rust replacement milestone and removal milestone
- Phase 1 is not complete until core runtime/orchestration, worker flows, and active backend integrations are owned by Rust modules.

## Current Phase Status

### Phase 1 — Stabilize & Complete Backend (In Progress)

Progress summary:
- Rust backend is primary API surface (`src-tauri`)
- Metadata/database APIs are mostly migrated and contract-tested
- Bootstrap import/export is implemented and expanding
- Runtime consolidation into Rust has started (script fallback still exists, but removal is the direction and milestone target)

### Phase 2 — GUI Frontend (Not Started)

Status:
- Planned, but not current priority
- Depends on continued Phase 1 backend/runtime consolidation

## Completed Work (Done / Pushed)

### Backend Core / API Foundation

- Rust backend (`axum` + SQLite) is the primary local API
- Contract-first route catalog + parity tests + HTTP mount checks
- Most metadata CRUD/read endpoints implemented in Rust:
  - projects
  - storage
  - runs/assets read APIs
  - prompt templates
  - provider accounts
  - style guides
  - characters
  - reference sets (+ items) CRUD
  - chat
  - agent instructions
  - secrets CRUD

### Bootstrap Flow (Rust)

- `GET /api/projects/{slug}/bootstrap-prompt`
- `POST /api/projects/{slug}/bootstrap-import`
  - `merge` / `replace`
  - `dry_run` preview
  - change-summary / diff metadata

### Bootstrap Coverage (Pushed)

- project metadata
- provider accounts
- style guides
- prompt templates
- characters
- reference sets (+ nested items)
- secrets metadata only (`provider_code`, `secret_name`, `has_value`; no secret values)

### Bootstrap Safety / Quality Improvements (Pushed)

- `reference_sets[].items` now required explicitly (use `[]` for empty set)
- secrets import is metadata-only and merge-only
- existing secret values are preserved
- validation + round-trip + replace-scope + dry-run tests expanded

### Roadmap / Direction Clarification (Pushed)

- `scripts/` documented as transitional, not end state
- Phase 1 explicitly includes runtime consolidation into Rust (not just CRUD endpoint work)
- Desktop-app principle reaffirmed: move active runtime/workers/integrations into Rust; reduce and delete script paths as replacements land
- Script removal rule: when a script responsibility is fully migrated to Rust, delete the script path in the same phase/patch (do not keep dormant legacy script code)

### Runtime Consolidation Foundation (Pushed)

- Rust pipeline runtime boundary (`src-tauri/src/pipeline/runtime.rs`) added
- Rust trigger service (`src-tauri/src/pipeline/trigger.rs`) added
- Rust-owned `POST /api/projects/{slug}/runs/trigger` endpoint added
- Trigger request contract is typed end-to-end (no raw `extra_args` passthrough)
- Script parity validation enforced for input source (`input` xor `scene_refs`)
- Typed script-backed backend ops boundary added for pipeline post-run operations (`pipeline::backend_ops`)
- Typed post-run service added for backend ingest / S3 sync orchestration (`pipeline::post_run`)
- Native Rust run-log ingest (`ProjectsStore::ingest_run_log`) added and wired into the typed trigger post-run path
  - `backend.py ingest-run` is no longer used for the default Rust `runs/trigger` path
- Rust-owned S3 sync prechecks + AWS CLI execution added in `pipeline::backend_ops`
  - `backend.py sync-project-s3` is no longer used for the default Rust `runs/trigger` post-run path
- Removed script-owned post-run backend ingest/sync calls from `scripts/image-lab.mjs`
  - script run path now emits run log + summary marker only; Rust runtime owns post-run backend operations
- Typed trigger path now injects `project_root` from Rust project storage
  - avoids script-side backend storage lookup (`backend.py get-project-storage`) for the app-triggered run path
- Rust-native dry-run execution is now in place for typed trigger app flows (`scene_refs` and `input`)
  - planning + run-log writing + summary marker emitted from Rust (no Node on those dry paths)

### Scope Cleanup / Legacy Removal (Pushed)

- Removed Rust voice endpoint surface (`/voice/*`) and associated tests
- Removed legacy Python HTTP backend entrypoint `scripts/backend_api.py`
- Split secrets DB code into `src-tauri/src/db/projects/secrets.rs`
- Removed voice schema remnants from `scripts/backend.py`
- Updated contract smoke scripts, README, and spec docs to match current scope

## In Progress

### Phase 1 Runtime Consolidation Into Rust (Started)

#### 1. Rust Pipeline Runtime Boundary (WIP)

  - `src-tauri/src/pipeline/runtime.rs`
  - `PipelineOrchestrator` trait
  - `PipelineCommandRunner` trait
  - `ScriptPipelineOrchestrator` (temporary fallback adapter to `scripts/image-lab.mjs`)
  - `RustPostRunPipelineOrchestrator` wrapper now owns backend ingest for typed HTTP trigger path
    - disables script-side backend ingest (`--backend-db-ingest false`) to avoid duplicate ingestion
    - keeps script-side S3 sync disabled (`--storage-sync-s3 false`) to prevent duplicate post-run sync execution
    - Rust post-run ingest now uses native DB transaction path (`ProjectsStore::ingest_run_log`)
    - structured script summary marker is emitted and parsed (`KROMA_PIPELINE_SUMMARY_JSON`) with text fallback retained during migration
  - `RustDryRunPipelineOrchestrator` now executes typed dry scene-ref and input-path runs in Rust
    - deterministic Rust image discovery for `--input` (sorted recursive listing)
    - generation directory layout creation delegated to `pipeline::execution`

#### 2. Rust Pipeline Trigger Service (WIP)

- `src-tauri/src/pipeline/trigger.rs`
  - `PipelineTriggerService`
  - `TriggerMode` (`dry` / `run`)
  - run-mode spend confirmation enforcement
  - `--confirm-spend` injection for script fallback path

#### 3. Rust-Owned Trigger Endpoint (Initial Slice, WIP)

- `POST /api/projects/{slug}/runs/trigger`
- Implemented in Rust API handler (`src-tauri/src/api/runs_assets.rs`)
- Calls Rust trigger service, which then uses script fallback adapter
- Current request shape:
  - `mode: dry|run`
  - `confirm_spend: bool`
  - typed fields only: `project_root`, `input`, `scene_refs`, `style_refs`, `stage`, `time`, `weather`, `candidates`
  - no raw CLI arg pass-through (`extra_args` removed)
  - `input` and `scene_refs` now follow script parity: exactly one is required

#### 4. Tests (WIP)

- Unit tests for runtime/orchestrator and trigger service
- Endpoint tests for safe non-script paths:
  - invalid mode
  - missing project
  - missing spend confirmation for `run`
- Endpoint success-path test with fake orchestrator injection (no `node` dependency)
  - validates Rust-owned endpoint flow without requiring script runtime
- Typed-field request translation + validation tests for `runs/trigger`
  - typed request compiles into typed runtime request, then CLI flags inside runtime orchestrator
  - validation runs before project lookup
  - exact one-of validation for `input` / `scene_refs` enforced in Rust and covered by tests
- Runtime wrapper tests
  - verifies script backend-ingest disable flags are injected
  - verifies Rust post-run ingest executes on successful script run output
  - verifies missing `Run log:` stdout line degrades to warning (best-effort ingest)

## Next Work (Priority Queue)

### Immediate Next (Phase 1 / 10.0)

1. Add success-path test strategy for `POST /api/projects/{slug}/runs/trigger`
   - Done locally (fake pipeline orchestrator/service injected into test router state)
2. Replace `extra_args` pass-through with typed request DTO fields
   - Done locally: Rust API/service/runtime path is typed end-to-end
   - `extra_args` removed from Rust endpoint + OpenAPI
   - exact one-of `input`/`scene_refs` invariant added for script parity
   - next: keep tightening typed validation/rules as real callers are added
3. Keep script runtime behind explicit Rust interfaces only
   - no new direct script calls from handlers/routes
4. Continue replacing script-backed orchestration behavior inside the runtime boundary (without widening the HTTP contract)
   - current default runtime path routes post-run ingest through Rust `pipeline::post_run` + native `ProjectsStore::ingest_run_log`
   - post-run backend operations for typed trigger path are now Rust-owned (ingest + S3 sync)
   - `scripts/image-lab.mjs` post-run backend calls removed; remaining script responsibility is generation/post-process orchestration
   - next: extract generation/orchestration stages from `scripts/image-lab.mjs` into Rust modules
   - latest extraction: `pipeline::execution` now owns script-parity helpers for candidate filename, output path sanitization, candidate winner ranking, project directory layout, candidate post-process output path planning, per-job candidate loop expansion into ordered typed plans, output-guard rank summarization, job outcome resolution/finalization, and typed run-log candidate/job/output-guard record assembly

### Near-Term Backend / Bootstrap Work

1. Decide bootstrap scope for:
   - chat
   - agent instructions
2. Improve OpenAPI response documentation for bootstrap endpoints
   - Pushed: response bodies for `/bootstrap-prompt` and `/bootstrap-import` are now documented
   - next: expand nested field schemas/examples if SDK/client generation needs stronger typing
3. Consider optimizing bootstrap `reference_sets` export loading (`N+1` query risk for large projects)
   - Pushed: batched `reference_set_items` query replaces per-set item queries in bootstrap export loading
   - next: profile with larger seed data before spending time on further query tuning

### Phase 1 Remaining (Larger Milestones)

1. Replace `scripts/image-lab.mjs` orchestration with Rust pipeline orchestration modules (desktop app owns generation/post-process flow)
2. Move run trigger + ingest + sync parity fully into Rust services/endpoints (typed trigger post-run ingest+sync path now Rust-owned; generation/orchestration still script-backed)
3. Replace Python worker runtime (`agent_worker.py`, `agent_dispatch.py`) with Rust worker/service modules
4. Build typed Rust adapters for external tools/integrations (called from Rust, not shell scripts)
   - OpenAI image calls
   - rembg
   - ESRGAN
   - S3 sync
5. Add parity tests plus explicit deprecation/removal milestones for script paths
6. Delete each script (or migrated script subcommand/path) immediately after Rust parity lands for that responsibility; no "kept just in case" script retention
6. Remove remaining production runtime dependencies on `scripts/` (retain only dev/setup utilities if still needed)

## Later (Phase 2+)

### Phase 2 — GUI Frontend

- project dashboard
- run viewer
- asset browser
- chat / copilot
- instruction queue
- settings
- QA reports

This starts after more of Phase 1 runtime consolidation is in place.
