# Kroma Roadmap (Progress Tracker)

Last updated: 2026-03-01
Status: Step A COMPLETE — Phase 1 runtime consolidation done. Step B (contract freeze) next.

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

### Phase 1 Runtime Consolidation — Step A (COMPLETE 2026-03-01)

**Goal:** Full run orchestration ownership in Rust for project journey `J04-J07`

**Completed:**
1. ✅ Replaced `scripts/image-lab.mjs` generation/post-process orchestration with Rust modules
2. ✅ Replaced Python worker runtime (`agent_worker.py`, `agent_dispatch.py`) with Rust service
3. ✅ External tool/provider calls behind typed Rust adapters only
4. ✅ Script-removal parity rule maintained: each migrated script path removed in same slice

**Acceptance gate:**
1. ✅ App-triggered project runs execute without script runtime dependency
2. ✅ Parity tests pass for `J04-J07` flows (`cargo test --lib pipeline::` — 109 passed)

**Rust CLI utility commands (replacement for image-lab.mjs):**
```bash
cargo run -- generate-one --project-slug <slug> --prompt <text> --input-images-file <file> --output <path>
cargo run -- upscale --project-slug <slug> [--input PATH] [--output PATH] [--upscale-backend ncnn|python]
cargo run -- color --project-slug <slug> [--input PATH] [--output PATH] [--profile PROFILE]
cargo run -- bgremove --project-slug <slug> [--input PATH] [--output PATH]
cargo run -- qa --project-slug <slug> [--input PATH]
cargo run -- archive-bad --project-slug <slug> --input PATH
```

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
- **Removed `scripts/image-lab.mjs` entirely (2026-03-01)** — all utility workflows now Rust CLI commands:
  - `cargo run -- generate-one`, `upscale`, `color`, `bgremove`, `qa`, `archive-bad`
- Worker runtime is Rust-owned by default (`cargo run -- agent-worker`)

## In Progress

### Step B — Backend Contract Freeze for Frontend (ACTIVE)

**Goal:** Lock stable backend contracts for UI journey `J00-J08`

**Must complete:**
1. Publish stable request/response and error taxonomy for project/run/review/postprocess/export steps
2. Complete contract + integration tests for all journey-critical endpoints
3. Publish breaking-change policy for frontend integration

**Acceptance gate:**
1. Backend freeze checklist is green
2. Frontend can implement without contract churn

**Current progress:**
- ✅ Published `docs/BACKEND_CONTRACT_FREEZE.md` baseline
- ✅ Error taxonomy fields (`error_kind`, `error_code`) added to Rust API responses
- ✅ Contract-freeze regression tests added (`error_taxonomy_endpoints.rs`)
- ✅ Taxonomy assertions expanded across 10+ endpoint groups
- ✅ OpenAPI `ErrorResponse` / `ErrorKind` schemas published
- ⏳ Remaining: finalize journey-critical endpoint coverage and freeze checklist

### Phase 1 Runtime Consolidation Into Rust (COMPLETE)

All Step A items completed 2026-03-01. See "Phase 1 Runtime Consolidation — Step A (COMPLETE)" above.

## Fixed Execution Plan (Frozen 2026-02-27, Updated 2026-03-01)

### Step A — Finish Phase 1 Runtime Consolidation (COMPLETE ✅)

**Goal:** Full run orchestration ownership in Rust for project journey `J04-J07`

**Status:** COMPLETED 2026-03-01

**Completed:**
1. ✅ Replaced `scripts/image-lab.mjs` generation/post-process orchestration with Rust modules
2. ✅ Replaced Python worker runtime (`agent_worker.py`, `agent_dispatch.py`) with Rust service
3. ✅ External tool/provider calls behind typed Rust adapters only
4. ✅ Script-removal parity rule maintained: each migrated script path removed in same slice

**Acceptance gate:**
1. ✅ App-triggered project runs execute without script runtime dependency
2. ✅ Parity tests pass for `J04-J07` flows

### Step B — Backend Contract Freeze for Frontend (ACTIVE)

**Goal:** Lock stable backend contracts for UI journey `J00-J08`

**Status:** IN PROGRESS — see "In Progress" section above for details

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

**Completed:**
1. ✅ Rust-run parity slice for `J04-J07` — COMPLETE (image-lab.mjs removed, Rust CLI commands active)
2. ✅ Worker migration slice — COMPLETE (Rust `agent-worker` active, Python scripts removed)

**Next:**
3. Journey contract freeze slice:
   - Finalize and test backend contracts needed by frontend `J00-J08` implementation
   - Complete Step B acceptance checklist
   - Mark backend freeze as green for frontend start

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
