# Migration Status (Rust vs Scripts)

Last updated: 2026-02-27
Status: Active migration (partial)

## Summary

Kroma is in a partial migration state:

1. Rust (`src-tauri`) is now the primary backend for metadata/API.
2. `scripts/` is still required for pipeline execution and local tool orchestration.
3. Some Python fallback paths remain for compatibility while Rust migration closes remaining gaps.

## What Is Already in Rust (Primary)

These areas are implemented in the Rust backend and are the main path today:

| Subsystem | Status | Notes |
| --- | --- | --- |
| HTTP API server (`axum`) | `Primary (Rust)` | `npm run backend:rust`, default `127.0.0.1:8788` |
| OpenAPI contract + route parity | `Primary (Rust)` | Contract-first route catalog and parity tests |
| SQLite schema management | `Primary (Rust)` | Tables created/normalized on startup |
| Projects CRUD | `Primary (Rust)` | Project creation, detail, listing |
| Storage config API | `Primary (Rust)` | Local + S3 settings |
| Runs read APIs | `Primary (Rust)` | Runs/run detail/job list |
| Assets + asset links APIs | `Primary (Rust)` | Asset registry and relationships |
| Analytics read APIs | `Primary (Rust)` | `quality-reports`, `cost-events` |
| Exports read APIs | `Primary (Rust)` | Export listing/detail |
| Prompt templates CRUD | `Primary (Rust)` | Implemented and tested |
| Provider accounts CRUD | `Primary (Rust)` | Implemented and tested |
| Style guides CRUD | `Primary (Rust)` | Implemented and tested |
| Characters CRUD | `Primary (Rust)` | Implemented and tested |
| Reference sets CRUD | `Primary (Rust)` | Sets + items |
| Chat / instructions / voice / secrets APIs | `Primary (Rust)` | Implemented and tested |
| Bootstrap prompt exchange | `Primary (Rust)` | `bootstrap-prompt`, `bootstrap-import`, `dry_run` preview |

## What Still Lives in `scripts/` (Active)

These are still actively used and not yet migrated to Rust:

| Subsystem | Current Runtime | Status | Why it remains |
| --- | --- | --- | --- |
| Image generation pipeline | `scripts/image-lab.mjs` (Node.js) | `Active (Scripts)` | Main generation orchestration, spend guards, staged runs |
| QA guard helpers | Python scripts | `Active (Scripts)` | Existing image QA implementation/tooling |
| Local post-process wrappers | Python scripts | `Active (Scripts)` | rembg / Real-ESRGAN / color correction wrappers |
| Tool setup/install helpers | Python/Bash scripts | `Active (Scripts)` | Environment/tool bootstrapping |

## Legacy / Compatibility Paths (Still Present)

These are still in repo and usable, but no longer the recommended primary path:

| Subsystem | Current Runtime | Status | Migration Intent |
| --- | --- | --- | --- |
| `scripts/backend.py` | Python | `Legacy-Compatible` | Keep temporarily while Rust parity gaps close |

## Not Yet Migrated to Rust (Key Gaps)

| Area | Status | Notes |
| --- | --- | --- |
| Pipeline execution mutation parity | `Planned` | Rust API does not yet replace full script-run execution flow |
| Export mutation parity (`create export`, `sync-s3`) | `Planned / Partial` | Rust has export read APIs; write/sync parity still pending |
| Agent worker runtime | `Primary (Rust)` | Rust `agent-worker` loop now processes confirmed instructions with retry/backoff and secret fallback target resolution |
| Auth/token system | `Planned` | No `/auth/*` endpoints yet |
| `audit_events` schema + API | `Planned` | Table still missing |

## Recommended Golden Path (Today)

Use this split until migration is complete:

1. Use Rust backend (`npm run backend:rust`) for project metadata and UI-facing APIs.
2. Use `scripts/image-lab.mjs` for generation/post-process pipeline runs.
3. Use legacy Python scripts only when a Rust parity feature is not available yet.

## How to Read "Migration Complete"

Migration is complete only when all three are true:

1. Rust replaces the Python metadata/API paths for normal operation (mostly true now).
2. Rust (or Rust-native workers/services) replaces script-only operational runtimes where intended.
3. `scripts/` is reduced to optional tooling wrappers, not core product runtime.
