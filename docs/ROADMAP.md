# Kroma Roadmap (Progress Tracker)

Last updated: 2026-02-27
Status: Phase 1 in progress (Rust runtime consolidation) + journey map freeze active

## Purpose

This file is the working progress tracker for Kroma.

- `docs/Kroma_—_Project_Spec_(Current_State_&_Roadmap).md` remains the full product/spec document.
- This file is the day-to-day roadmap status board: what is done, in progress, and next.

## Planning Control Docs (Source of Truth)

1. `docs/ROADMAP.md`:
   - fixed execution order, phase status, and current priorities
2. `docs/USER_FLOW_JOURNEY_MAP.md`:
   - canonical user journey (`J00-J08`, `U01`, recovery flows) and acceptance gates
3. `docs/WORKFLOW.md`:
   - implementation rules and required journey-step traceability for every feature

## Product North Star (Aligned 2026-02-27)

1. Primary outcome:
   - Kroma exists to produce long-form comic/graphic-novel universes with stable style and stable character identity across many generated images.
2. Primary unit of work:
   - `project` is the core product unit (one universe/story world with its heroes, style constraints, assets, runs, and history).
   - a single artist/user can own multiple independent projects.
3. Secondary convenience mode:
   - quick one-off graphics utilities (for example background removal or single-image generation without project context) are supported, but they are not the product driver.
4. Future direction (after core image stack maturity):
   - continuity-preserving video generation from the same project character/style identity model.

## Journey-First Execution Rule (Frozen)

1. We implement journey steps, not random features.
2. Any roadmap task must map to `Jxx`, `Uxx`, or `Rxx` in `docs/USER_FLOW_JOURNEY_MAP.md`.
3. If a proposed task has no mapped journey step, it is out of scope until the journey map is updated first.

## Architecture Direction (Explicit)

- Kroma is a desktop app and the target architecture is Rust-owned end to end (`src-tauri`).
- `scripts/` are transitional migration scaffolding only, not a supported long-term runtime.
- No permanent script wrappers/adapters as an end state:
  - temporary Rust boundaries around scripts are allowed only to preserve momentum while replacing them
  - every script-backed path must have a Rust replacement milestone and removal milestone
- Phase 1 is not complete until core runtime/orchestration, worker flows, and active backend integrations are owned by Rust modules.

### Persistence Model Decision (2026-02-24)

1. Default (desktop) data model remains:
   - SQLite for metadata/state
   - local filesystem for image assets
2. Cloud storage is an optional tier:
   - S3 sync/backups are additive, not mandatory for local runtime
3. PostgreSQL is deferred:
   - not required for current desktop single-user architecture
   - planned only for future hosted/shared multi-user mode
4. Scope guard:
   - do not spend roadmap capacity on full PostgreSQL backend wiring until hosted mode requirements are active

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
- Starts after Phase 1 backend/runtime freeze gates are complete for target journey steps
- Frontend scope is project-first (comic/graphic-novel workflow), not utility-first

## Completed Work (Done / Pushed)

### Backend Core / API Foundation

- Rust backend (`axum` + SQLite) is the primary local API
- Auth/audit foundation started in Rust:
  - hashed `api_tokens` table + `/auth/token(s)` routes
  - Bearer middleware with local dev bypass flag (`KROMA_API_AUTH_DEV_BYPASS`)
  - auth bypass now defaults to `false` when unset; dev bypass is explicit opt-in
  - first-token bootstrap path restored safely: unauthenticated `POST /auth/token` is allowed only when there are no active tokens and bind is loopback (toggle: `KROMA_API_AUTH_BOOTSTRAP_FIRST_TOKEN`)
  - test routers now use explicit dev-bypass constructors instead of relying on implicit default bypass
  - `audit_events` table + initial audit writes for key mutating endpoints
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
- Secrets-at-rest hardening landed for Rust `project_secrets` writes:
  - `secret_value` now stores ciphertext (AES-256-GCM) instead of plaintext
  - key source matches legacy behavior (`IAT_MASTER_KEY` or `IAT_MASTER_KEY_FILE`, default `var/backend/master.key`)
  - local key file is auto-generated on first use when not provided
  - explicit per-project key rotation path added: `POST /api/projects/{slug}/secrets/rotate`
    - supports active key refs (`IAT_MASTER_KEY_REF`) and previous-key fallback (`IAT_MASTER_KEY_PREVIOUS`) for re-encryption
  - explicit migration visibility endpoint added: `GET /api/projects/{slug}/secrets/rotation-status`
    - reports encrypted/plaintext/empty row counts and key-ref distribution
  - local operator CLI fallback added for key maintenance without HTTP dependency:
    - `cargo run -- secrets-rotation-status --project-slug <slug>`
    - `cargo run -- secrets-rotate --project-slug <slug> [--from-key-ref <ref>] [--force]`

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
- Legacy script safety gate: remaining Python fallback scripts are opt-in only (`KROMA_ENABLE_LEGACY_SCRIPTS=1`) to prevent accidental production/runtime drift during migration

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
  - native ingest+sync runtime wiring no longer depends on `ScriptPipelineBackendOps` wrapper state
    - Rust post-run default now injects runner/app-root directly for AWS sync execution
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
- Legacy npm script entrypoints now explicitly set `KROMA_ENABLE_LEGACY_SCRIPTS=1` for Python fallback commands (`backend:init`, `backend:migrate`); worker runtime is Rust-owned by default

## In Progress

### Phase 1 Runtime Consolidation Into Rust (Started)

#### 1. Rust Pipeline Runtime Boundary (WIP)

  - `src-tauri/src/pipeline/runtime.rs`
  - `PipelineOrchestrator` trait
  - `PipelineCommandRunner` trait
  - `RustPostRunPipelineOrchestrator` wrapper owns post-run finalize flow for typed HTTP trigger path
    - preserves typed request options end-to-end for run execution
    - Rust post-run ingest uses native DB transaction path (`ProjectsStore::ingest_run_log`)
    - structured summary marker is emitted and parsed (`KROMA_PIPELINE_SUMMARY_JSON`)
  - `RustDryRunPipelineOrchestrator` now executes typed dry scene-ref and input-path runs in Rust
    - deterministic Rust image discovery for `--input` (sorted recursive listing)
    - generation directory layout creation delegated to `pipeline::execution`

#### 2. Rust Pipeline Trigger Service (WIP)

- `src-tauri/src/pipeline/trigger.rs`
  - `PipelineTriggerService`
  - `TriggerMode` (`dry` / `run`)
  - run-mode spend confirmation enforcement
  - `--confirm-spend` injection for Rust run path

#### 3. Rust-Owned Trigger Endpoint (Initial Slice, WIP)

- `POST /api/projects/{slug}/runs/trigger`
- Implemented in Rust API handler (`src-tauri/src/api/runs_assets.rs`)
- Calls Rust trigger service with Rust-native runtime stack
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

### Step B Contract Freeze Kickoff (In Progress)

- Published Step B baseline contract doc:
  - `docs/BACKEND_CONTRACT_FREEZE.md`
  - includes journey-critical endpoint surface (`project`, `run/review/postprocess`, `export`)
  - includes additive frozen error taxonomy fields:
    - `error_kind`
    - `error_code`
  - includes backend/frontend breaking-change policy
- Rust API error responses now include taxonomy fields on baseline paths while preserving legacy `ok/error` compatibility shape.
- Contract-freeze regression test added:
  - `src-tauri/tests/error_taxonomy_endpoints.rs`
  - validates taxonomy on project validation errors, not-found errors, and run policy errors.
- Taxonomy assertions expanded on journey endpoints:
  - `bootstrap_endpoints`
  - `reference_sets_endpoints`
  - `storage_endpoints`
- Taxonomy assertions expanded on additional endpoint groups:
  - `provider_accounts_endpoints`
  - `style_guides_endpoints`
  - `prompt_templates_endpoints`
  - `characters_endpoints`
  - `asset_links_endpoints`
  - `chat_endpoints`
  - `agent_instructions_endpoints`
  - `secrets_endpoints`
- OpenAPI contract now includes shared `ErrorResponse` / `ErrorKind` schemas and Step B path-level error schema references across:
  - project/bootstrap/storage/reference-set routes
  - `provider-accounts`, `style-guides`, `prompt-templates`, `characters`
  - `asset-links`, `chat/sessions`, `agent/instructions`, `secrets`

## Fixed Execution Plan (Frozen 2026-02-27)

### Step A — Finish Phase 1 Runtime Consolidation (Now)

Goal:
- full run orchestration ownership in Rust for project journey `J04-J07`

Must complete:
1. replace remaining `scripts/image-lab.mjs` generation/post-process orchestration paths with Rust modules
2. replace Python worker runtime (`agent_worker.py`, `agent_dispatch.py`) with Rust service modules
3. keep external tool/provider calls behind typed Rust adapters only
4. maintain script-removal parity rule: remove each migrated script path in the same slice

Acceptance gate:
1. app-triggered project runs (`POST /api/projects/{slug}/runs/trigger`) execute without script runtime dependency in normal path
2. parity tests pass for `J04-J07` flows

### Step B — Backend Contract Freeze for Frontend (Next)

Goal:
- lock stable backend contracts for UI journey `J00-J08`

Must complete:
1. publish stable request/response and error taxonomy for project/run/review/postprocess/export steps
2. complete contract + integration tests for all journey-critical endpoints
3. publish breaking-change policy for frontend integration

Acceptance gate:
1. backend freeze checklist is green
2. frontend can implement without contract churn

### Step C — Frontend Phase Starts (After A+B)

Goal:
- implement UI by journey order, not by random page list

Execution order:
1. `J00-J03`: onboarding, project creation, references, bootstrap
2. `J04-J06`: staged run compose/review/identity continuity
3. `J07-J08`: post-process and export
4. `U01`: utility mode only after primary journey baseline works end-to-end

Acceptance gate:
1. every UI slice cites journey step IDs and done criteria from `docs/USER_FLOW_JOURNEY_MAP.md`

## Immediate Next 3 Slices (Locked)

1. Rust-run parity slice for `J04-J07`:
   - extract remaining generation/post-process orchestration logic from scripts into Rust runtime modules
2. Worker migration slice:
   - replace Python agent worker/dispatch runtime with Rust worker modules
3. Journey contract freeze slice:
   - finalize and test backend contracts needed by frontend `J00-J08` implementation

## Later (Phase 2+)

### Phase 2 — GUI Frontend

- project dashboard
- project universe overview (story + core cast continuity)
- run viewer
- asset browser
- chat / copilot
- instruction queue
- settings
- QA reports

This starts only after Step A + Step B in the fixed execution plan are complete.

### Phase 3+ — Continuity Video (Future)

- keep style and faces consistent across shot sequences
- reuse project continuity assets/rules from image pipeline as video conditioning inputs
- remains out of scope until Phase 1 and Phase 2 foundations are stable
