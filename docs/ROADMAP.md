# Kroma Roadmap (Progress Tracker)

Last updated: 2026-02-22
Status: Phase 1 in progress (backend + runtime consolidation into Rust)

## Purpose

This file is the working progress tracker for Kroma.

- `docs/Kroma_—_Project_Spec_(Current_State_&_Roadmap).md` remains the full product/spec document.
- This file is the day-to-day roadmap status board: what is done, in progress, and next.

## Current Phase Status

### Phase 1 — Stabilize & Complete Backend (In Progress)

Progress summary:
- Rust backend is primary API surface (`src-tauri`)
- Metadata/database APIs are mostly migrated and contract-tested
- Bootstrap import/export is implemented and expanding
- Runtime consolidation into Rust has started (still uses script fallback behind Rust boundary)

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

### Runtime Consolidation Foundation (Pushed)

- Rust pipeline runtime boundary (`src-tauri/src/pipeline/runtime.rs`) added
- Rust trigger service (`src-tauri/src/pipeline/trigger.rs`) added
- Rust-owned `POST /api/projects/{slug}/runs/trigger` endpoint added
- Trigger request contract is typed end-to-end (no raw `extra_args` passthrough)
- Script parity validation enforced for input source (`input` xor `scene_refs`)

### Scope Cleanup / Legacy Removal (Pushed)

- Removed Rust voice endpoint surface (`/voice/*`) and associated tests
- Removed legacy Python HTTP backend entrypoint `scripts/backend_api.py`
- Split secrets DB code into `src-tauri/src/db/projects/secrets.rs`
- Removed voice schema remnants from `scripts/backend.py`
- Updated contract smoke scripts, README, and spec docs to match current scope

## In Progress (Local WIP)

### Phase 1 Runtime Consolidation Into Rust (Started)

#### 1. Rust Pipeline Runtime Boundary (WIP)

- `src-tauri/src/pipeline/runtime.rs`
  - `PipelineOrchestrator` trait
  - `PipelineCommandRunner` trait
  - `ScriptPipelineOrchestrator` (temporary fallback adapter to `scripts/image-lab.mjs`)

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

### Near-Term Backend / Bootstrap Work

1. Decide bootstrap scope for:
   - chat
   - agent instructions
2. Improve OpenAPI response documentation for bootstrap endpoints
   - Done locally: response bodies for `/bootstrap-prompt` and `/bootstrap-import` are now documented
   - next: expand nested field schemas/examples if SDK/client generation needs stronger typing
3. Consider optimizing bootstrap `reference_sets` export loading (`N+1` query risk for large projects)

### Phase 1 Remaining (Larger Milestones)

1. Replace `scripts/image-lab.mjs` orchestration with Rust pipeline orchestration module
2. Move run trigger + ingest parity fully into Rust services/endpoints
3. Replace Python worker runtime (`agent_worker.py`, `agent_dispatch.py`) with Rust worker/service modules
4. Build typed Rust adapters for external tools/integrations
   - OpenAI image calls
   - rembg
   - ESRGAN
   - S3 sync
5. Add parity tests and deprecation milestones for script paths

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
