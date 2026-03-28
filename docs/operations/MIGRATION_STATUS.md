# Migration Status (Rust vs Scripts)

Last updated: 2026-03-24
Status: **Migration COMPLETE — 100% Rust**

## Summary

Kroma migration to Rust is **COMPLETE**:

1. Rust (`src-tauri`) is the exclusive backend for metadata/API/pipeline.
2. All Python/Node.js scripts have been migrated to Rust.
3. Zero Python/Node.js dependencies in backend runtime.

## What Is Already in Rust (Complete)

All subsystems are now in Rust:

| Subsystem | Status | Notes |
| --- | --- | --- |
| HTTP API server (`axum`) | `✅ Rust` | `npm run backend:rust`, default `127.0.0.1:8788` |
| OpenAPI contract + route parity | `✅ Rust` | 74 routes, contract-first |
| SQLite schema management | `✅ Rust` | Tables created/normalized on startup |
| Projects CRUD | `✅ Rust` | Project creation, detail, listing |
| Storage config API | `✅ Rust` | Local + S3 settings |
| Runs read/write APIs | `✅ Rust` | Runs, trigger, review, jobs |
| Assets + asset links APIs | `✅ Rust` | Asset registry and relationships |
| Analytics read APIs | `✅ Rust` | `quality-reports`, `cost-events` |
| Exports read APIs | `✅ Rust` | Export listing/detail |
| Prompt templates CRUD | `✅ Rust` | Implemented and tested |
| Provider accounts CRUD | `✅ Rust` | Implemented and tested |
| Style guides CRUD | `✅ Rust` | Implemented and tested |
| Characters CRUD | `✅ Rust` | Implemented and tested |
| Reference sets CRUD | `✅ Rust` | Sets + items |
| Chat / instructions / secrets APIs | `✅ Rust` | Implemented and tested |
| Bootstrap prompt exchange | `✅ Rust` | `bootstrap-prompt`, `bootstrap-import` |
| Auth/token system | `✅ Rust` | `/auth/token`, `/auth/tokens`, `/auth/tokens/{id}` |
| Agent worker runtime | `✅ Rust` | `agent-worker` with retry/backoff |
| Pipeline execution | `✅ Rust` | Rust pipeline runtime |
| QA guard helpers | `✅ Rust` | Native Rust QA checks |
| Post-process (bg-remove, upscale, color) | `✅ Rust` | CLI commands via Rust |
| Tool setup/install | `✅ Rust` | `cargo run -- tools:install` |

## Recommended Golden Path (Today)

Use Rust exclusively:

1. **Start backend:** `npm run backend:rust` (or `cargo run --manifest-path src-tauri/Cargo.toml`)
2. **Database init:** `npm run backend:init` (or `cargo run -- db:init`)
3. **Create user:** `npm run backend:user:local -- --username local --display-name "Local User"`
4. **Install tools:** `npm run tools:setup` (or `cargo run -- tools:install all`)
5. **Run pipeline:** `cargo run -- generate-one --project-slug <slug> --prompt "..."`
6. **Post-process:** `cargo run -- upscale`, `cargo run -- bgremove`, `cargo run -- color`
7. **QA checks:** `cargo run -- qa --project-slug <slug>`
8. **Worker:** `npm run backend:worker` (or `cargo run -- agent-worker`)

**All operations are now Rust-native. No Python/Node.js scripts required.**

## How to Read "Migration Complete"

Migration is complete only when all three are true:

1. Rust replaces the Python metadata/API paths for normal operation (mostly true now).
2. Rust (or Rust-native workers/services) replaces script-only operational runtimes where intended.
3. `scripts/` is reduced to optional tooling wrappers, not core product runtime.
