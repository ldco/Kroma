# DB Migration Phased Plan (Current Code Baseline)

Date: 2026-02-21

## 1. Decision Summary

1. For strict alignment with `docs/TECH_SPEC.md` and Phase 1 roadmap completion, all canonical target tables are still needed.
2. Several target tables already exist physically in `scripts/backend.py:init_schema()`, but many are schema-incompatible and/or unused by runtime paths.
3. The main gap is no longer table creation only. The gap is canonical naming, column semantics, data backfill, and API/ingest wiring.

## 2. Baseline Facts (from current code)

1. Legacy names still drive runtime behavior: `users`, `projects.user_id`, `run_job_candidates`, `assets.asset_kind`, `assets.rel_path`, `project_exports.export_path`, `project_api_secrets.key_ref`.
2. API layer still joins `users` and `projects.user_id` in active endpoints.
3. Many "target" tables are present but do not match target columns or meaning.
4. Run ingestion writes only legacy run/candidate fields and does not persist normalized `quality_reports` or `cost_events`.

## 3. Table Priority Matrix

| Target table/entity | Current state in code | Priority | If deferred, what is missing |
|---|---|---|---|
| `app_users` + FK migration | Not present (still `users`) | Must now | Spec mismatch, FK contract mismatch for chat/agent/audit, blocks canonical user model |
| `projects.owner_user_id` | Not present (still `user_id`) | Must now | Cannot satisfy required unique/index model `(owner_user_id, slug)` |
| `project_storage` (policy fields) | Exists with wrong quota-like fields | Must now | Storage remains embedded in `projects.settings_json`; no normalized local/S3 policy rows |
| `assets` canonical (`kind`, `storage_uri`, `metadata_json`) | Exists with legacy names | Must now | Asset URI model and downstream FK conversions stay blocked |
| `runs` canonical (`run_mode`, `model_name`, `settings_snapshot_json`) | Exists with legacy names | Must now | Target query contract and API schema remain inconsistent |
| `run_jobs` canonical FK fields | Exists with legacy names/semantics | Must now | Cannot reliably link selected/final outputs to `assets.id` |
| `run_candidates` | Not present (still `run_job_candidates`) | Must now | Quality/report FK target mismatch; endpoint contracts drift |
| `project_api_secrets.kms_key_ref` | Still `key_ref` | Must now | Security schema mismatch vs spec |
| `quality_reports` | Exists but wrong columns and not populated | Must now | No normalized QA history despite roadmap requirement |
| `cost_events` | Exists but wrong columns and not populated | Must now | No per-provider/per-run cost ledger |
| `audit_events` | Exists but wrong columns and weak semantics | Must now | Cannot meet auditable-operation requirement stated in spec |
| `project_exports` canonical fields | Exists with path-based fields | Should now | Export records cannot be linked to asset registry |
| `provider_accounts` canonical config | Exists with wrong fields (`api_key`, `meta_json`) | Should now | No normalized provider enable/config model |
| `style_guides` | Exists but missing target fields (`rules_json`, `is_default`) | Should now | Character/style CRUD cannot match spec contract |
| `characters` | Exists but missing `identity_constraints_json` | Should now | Cannot store stable identity constraints |
| `reference_sets` | Exists with wrong fields (`title`, `notes`) | Should now | No `kind`/`metadata_json` reference-pack semantics |
| `reference_items` | Exists but missing `weight` | Should now | Cannot model weighted references |
| `asset_links` | Mostly aligned and present | Can defer | Asset graph remains underused but does not block current run ingestion |
| `prompt_templates` | Exists but missing `stage`, `version`, `is_active` | Can defer | Prompt-versioning in DB remains unavailable (prompts still in manifest) |
| `chat_sessions`/`chat_messages`/`agent_*`/`voice_requests` | Functionally close; FK names depend on `users` migration | Must now only for FK realignment | If deferred, these continue working but stay non-canonical |

## 4. Phased Execution Plan

## Phase 0: Safety Baseline (pre-migration)

1. Freeze a DB backup and snapshot migration state.
2. Add migration smoke tests against a fresh DB and a representative existing DB.
3. Add read-only schema assertions for all target columns/indexes to prevent drift.

Exit criteria:
1. Reproducible backup path documented.
2. CI/local check can fail on schema drift.

## Phase 1: Canonical Core (Must now)

Scope:
1. Introduce canonical user/project ownership names: `app_users`, `projects.owner_user_id`.
2. Introduce canonical run/asset/export column names and candidate table naming.
3. Normalize `project_storage` to local/S3 policy fields.
4. Align secrets key reference field: `kms_key_ref`.
5. Align index set required by `docs/TECH_SPEC.md` for core run/asset queries.

Implementation strategy:
1. Use additive migration pattern first: add canonical columns/tables before removing legacy.
2. Backfill canonical columns from legacy values.
3. Update code to dual-read/dual-write during transition.
4. After stable cutover, switch primary reads to canonical fields.

Exit criteria:
1. All core runtime paths operate using canonical names internally.
2. Existing projects/runs remain queryable and writable without data loss.

## Phase 2: Data Semantics and Event Tables (Must now)

Scope:
1. Redefine `quality_reports`, `cost_events`, `audit_events` to target schema.
2. Backfill quality from historical run logs where available.
3. Start writing `quality_reports` and `cost_events` during run ingestion.
4. Emit `audit_events` for secrets, storage updates, export, and instruction status transitions.

Exit criteria:
1. New runs produce normalized quality and cost rows.
2. Security-critical mutations produce audit events.
3. Existing log-based data has documented backfill coverage and known limits.

## Phase 3: API Contract Completion (Phase 1 roadmap)

Scope:
1. Add missing endpoints for runs/assets/quality/cost.
2. Add CRUD for `characters`, `style_guides`, `reference_sets`.
3. Update OpenAPI schema to canonical field names and endpoint responses.

Exit criteria:
1. Endpoints listed in roadmap section 10.2 are implemented.
2. OpenAPI matches runtime responses.

## Phase 4: Extended Canonicalization (Should now / Can defer)

Scope:
1. Align `provider_accounts`, `style_guides`, `characters`, `reference_sets`, `reference_items`, `prompt_templates` to exact target fields.
2. Adopt `asset_links` in ingest/export logic where lineage is available.

Exit criteria:
1. Creative-knowledge and provider tables have spec-complete shape.
2. API responses no longer expose legacy table semantics.

## Phase 5: Legacy Cleanup (deferred cleanup window)

Scope:
1. Remove or archive legacy columns once no code paths depend on them.
2. Remove compatibility shims and temporary dual-write logic.
3. Keep `project_snapshots` as an intentional extension table unless explicitly deprecated.

Exit criteria:
1. No production path reads/writes legacy names.
2. Schema is canonical plus approved extension tables only.

## 5. What We Still Miss Even After DDL

1. Ingestion behavior must change, not just schema.
2. Path-string fields must resolve to `assets.id` for canonical FK usage.
3. API handlers must stop joining legacy `users`/`projects.user_id` fields.
4. Storage read/write must move from `projects.settings_json` to `project_storage` source-of-truth.
5. OpenAPI and tests must be updated in lockstep to avoid silent contract drift.

## 6. Recommended Build Order for Next Implementation Pass

1. Implement Phase 1 migrations with compatibility reads/writes.
2. Migrate API and ingestion logic to canonical fields.
3. Implement Phase 2 event-table writes and backfills.
4. Implement missing Phase 1 endpoints and OpenAPI updates.
5. Execute cleanup phase only after one full test cycle on migrated real data.
