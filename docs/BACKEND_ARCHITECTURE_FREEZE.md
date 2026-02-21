# Backend Architecture Freeze Plan

## Decision

1. Runtime target: Tauri app with Rust backend core.
2. Frontend target: Puppet Master (Nuxt-based) after backend freeze only.
3. Script-based Python/JS/Bash flows are transitional and will be eliminated from end-user runtime.

## Scope Of Backend Freeze

1. Domain model: users, projects, storage, runs, jobs, candidates, assets, quality, costs, audits, instructions, voice.
2. Data model: schema and migration policy.
3. Execution model: pipeline stages, retries, failures, idempotency, audit events.
4. Integration model: stable commands/API contract for frontend.
5. Deployment model: app data paths, local storage policy, optional S3 sync.

## Acceptance Gates

### Gate 1: Domain + Contracts

1. All backend use-cases mapped to explicit command/API contracts.
2. Request/response payloads versioned and documented.
3. Error taxonomy standardized (validation, provider, infra, policy, unknown).

### Gate 2: DB + Storage

1. Canonical schema finalized.
2. Migrations are additive, tested, reversible where possible.
3. App state path fixed (`var/backend` local default).
4. Project file paths derive only from project storage policy or explicit project root.

### Gate 3: Execution Engine

1. Stage pipeline formalized (`style -> time -> weather -> optional passes`).
2. Candidate selection, output guard, archive behavior formalized.
3. Background removal/upscale/color pass semantics formalized.
4. Cost/quality/audit persistence complete and tested.

### Gate 4: Rust Runtime Port

1. Rust modules created for DB, storage, pipeline, worker, provider adapters.
2. Rust command surface mirrors frozen contracts.
3. Script runtime dependency removed from end-user flow.

### Gate 5: Frontend Start Criteria

Frontend can begin only when:

1. Gates 1-4 are complete.
2. Contract smoke tests pass from Tauri command/API boundary.
3. Breaking-change policy is published.

## Step-By-Step Execution Order

1. Freeze backend contract shapes.
2. Freeze schema + migration IDs.
3. Freeze storage model and app-data paths.
4. Freeze pipeline semantics and failure handling.
5. Implement Rust backend core with parity tests.
6. Remove transitional script runtime paths from production flow.
7. Start Puppet Master frontend integration.

## Current Immediate Work

1. Freeze/lock current backend contract set as baseline for Rust parity.
2. Start Rust backend scaffold (`src-tauri` modules: db, storage, pipeline, worker, api bridge).
3. Add parity tests between current script backend and Rust command/API surface.

## Latest Progress Snapshot (2026-02-21)

1. Backend API contract now includes CRUD/read coverage for:
- `prompt_templates`
- `asset_links`
- `project_exports` list/detail
2. Legacy-to-canonical FK normalization is implemented and validated for:
- candidate output/final paths -> `run_candidates.output_asset_id/final_asset_id`
- job `final_output` -> `run_jobs.final_asset_id`
- export path -> `project_exports.export_asset_id`
3. Derived `asset_links` backfill is now seeded from existing run candidate/job graph.
4. Repository docs are normalized under `docs/`; project/user runtime knowledge is DB-owned.
