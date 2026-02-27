# Backend Contract Freeze (Step B)

Last updated: 2026-02-27
Status: In progress (contract baseline published)

## Purpose

This document is the Step B contract baseline for frontend integration on journey steps `J00-J08`.

It defines:
1. stable response envelope expectations for journey-critical endpoints
2. stable error taxonomy (`error_kind`, `error_code`) for failure handling
3. breaking-change policy for backend/frontend coordination

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

Current minimum automated evidence:
1. `src-tauri/tests/error_taxonomy_endpoints.rs` (taxonomy fields on project/run/export error paths)
2. endpoint suites with taxonomy assertions:
- `bootstrap_endpoints`
- `reference_sets_endpoints`
- `storage_endpoints`
- `provider_accounts_endpoints`
- `style_guides_endpoints`
- `prompt_templates_endpoints`
- `characters_endpoints`
- `asset_links_endpoints`
- `chat_endpoints`
- `agent_instructions_endpoints`
- `secrets_endpoints`
3. endpoint suites:
- `projects_endpoints`
- `runs_assets_endpoints`
- `pipeline_trigger_endpoints`
- `exports_endpoints`
4. contract mount/parity suites:
- `contract_parity`
- `http_contract_surface`
5. OpenAPI baseline:
- `openapi/backend-api.openapi.yaml` includes `ErrorResponse` + `ErrorKind` component schemas and error-schema references for Step B baseline + remaining journey endpoint groups (`provider-accounts`, `style-guides`, `prompt-templates`, `characters`, `asset-links`, `chat/sessions`, `agent/instructions`, `secrets`).

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
