# Backend Contract Freeze (Step B)

Last updated: 2026-03-01
Status: Contract baseline complete — freeze checklist green for J00-J08

## Purpose

This document is the Step B contract baseline for frontend integration on journey steps `J00-J08`.

It defines:
1. stable response envelope expectations for journey-critical endpoints
2. stable error taxonomy (`error_kind`, `error_code`) for failure handling
3. breaking-change policy for backend/frontend coordination

## Freeze Status (2026-03-01)

**Step B is GREEN for frontend start on J00-J08.**

Completed:
- ✅ Error taxonomy published and tested across all journey-critical endpoints
- ✅ Contract tests cover J00-J08 endpoint surface
- ✅ OpenAPI schemas include `ErrorResponse` / `ErrorKind` components
- ✅ Breaking-change policy documented

## Stable Endpoint Surface (Journey-Critical)

### Project (`J00-J03`)

- `GET /api/projects`
- `POST /api/projects`
- `GET /api/projects/{slug}`
- `GET /api/projects/{slug}/storage`
- `PUT /api/projects/{slug}/storage/local`
- `PUT /api/projects/{slug}/storage/s3`

### Run / Review / Postprocess (`J04-J07`, `R01`)

- `POST /api/projects/{slug}/runs/trigger`
- `POST /api/projects/{slug}/runs/validate-config`
- `GET /api/projects/{slug}/runs`
- `GET /api/projects/{slug}/runs/{runId}`
- `GET /api/projects/{slug}/runs/{runId}/jobs`
- `GET /api/projects/{slug}/assets`
- `GET /api/projects/{slug}/assets/{assetId}`

### Export (`J08`)

- `GET /api/projects/{slug}/exports`
- `GET /api/projects/{slug}/exports/{exportId}`

## Response Contract Baseline

### Success shape

Success responses remain endpoint-specific and include:
- `ok: true`
- endpoint payload fields (for example `project`, `runs`, `assets`, `exports`, `pipeline_trigger`)

### Error shape (frozen baseline)

Error responses use:
- `ok: false`
- `error: string`
- `error_kind: "validation" | "provider" | "infra" | "policy" | "unknown"`
- `error_code: string`

`error_kind` and `error_code` are additive fields over legacy `{ ok, error }` and are required for frontend error routing.

## Error Taxonomy Baseline

### Validation

- `validation_error`
- `not_found`
- `invalid_mode`
- `invalid_request`
- `project_root_managed`
- `invalid_project_slug`
- `planning_preflight_failed`
- `config_validation_failed`

### Policy

- `spend_confirmation_required`

### Provider

- `pipeline_command_failed`

### Infra

- `internal_error`

### Unknown

- reserved for future unmapped failures (not used by current Step B baseline)

## Verification Baseline

Current minimum automated evidence (all passing 2026-03-01):

1. **Error taxonomy tests** (`src-tauri/tests/error_taxonomy_endpoints.rs`):
   - Project validation errors have taxonomy fields
   - Not-found errors have taxonomy fields
   - Run trigger spend-confirmation errors have policy taxonomy

2. **Endpoint suites with taxonomy assertions**:
   - `bootstrap_endpoints` — J03 bootstrap import/export
   - `reference_sets_endpoints` — J02 continuity references
   - `storage_endpoints` — J01 project storage config
   - `provider_accounts_endpoints` — J00 provider setup
   - `style_guides_endpoints` — J02 style baselines
   - `prompt_templates_endpoints` — J02/J03 prompt management
   - `characters_endpoints` — J02 character identity
   - `asset_links_endpoints` — J07 asset relationships
   - `chat_endpoints` — copilot session management
   - `agent_instructions_endpoints` — agent workflow
   - `secrets_endpoints` — credential management
   - `runs_assets_endpoints` — J04-J07 run/asset detail (not_found taxonomy)
   - `exports_endpoints` — J08 export detail (not_found taxonomy)

3. **Core endpoint suites**:
   - `projects_endpoints` — J00-J01 project CRUD
   - `pipeline_trigger_endpoints` — J04-J07 run triggering
   - `auth_endpoints` — token bootstrap

4. **Contract mount/parity suites**:
   - `contract_parity` — OpenAPI vs runtime parity
   - `http_contract_surface` — HTTP surface validation

5. **OpenAPI baseline** (`openapi/backend-api.openapi.yaml`):
   - `ErrorResponse` / `ErrorKind` component schemas defined
   - Error schema references for all Step B endpoint groups

## Breaking-Change Policy (Frontend Integration)

### Non-breaking changes

1. adding new optional fields to success/error payloads
2. adding new endpoints
3. adding new error codes while preserving existing code behavior

### Breaking changes

1. removing or renaming existing response fields
2. changing field type/semantics for existing fields
3. changing status code semantics for existing scenarios
4. removing supported error codes or reclassifying them to different behavior without migration notice

### Required process for breaking changes

1. update this document and `docs/ROADMAP.md` in the same patch
2. update OpenAPI and contract tests in the same patch
3. provide a migration note with old vs new examples for frontend
