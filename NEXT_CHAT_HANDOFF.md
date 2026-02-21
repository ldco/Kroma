# Next Chat Handoff

Date: 2026-02-21
Branch: `master`
Last commit before this handoff update: `eb9b38e`

## Current Architecture Decisions

1. Rust backend (`src-tauri`) remains the single active backend implementation path.
2. Routing is still contract-first with parity tests against OpenAPI and deterministic `501` stubs for unimplemented domains.
3. Implemented API domains are now isolated into dedicated modules:
- `api/projects.rs`
- `api/runs_assets.rs`
- `api/asset_links.rs`
- `api/analytics.rs`
- `api/exports.rs`
- `api/prompt_templates.rs`
4. Repository layer exposes typed CRUD/read contracts for each implemented domain, keeping HTTP handlers thin.
5. Validation rules are centralized in repository methods for consistent error semantics.

## Completed Work In This Pass

1. Implemented prompt-template CRUD endpoints end-to-end:
- `GET /api/projects/{slug}/prompt-templates`
- `POST /api/projects/{slug}/prompt-templates`
- `GET /api/projects/{slug}/prompt-templates/{templateId}`
- `PUT /api/projects/{slug}/prompt-templates/{templateId}`
- `DELETE /api/projects/{slug}/prompt-templates/{templateId}`
2. Added typed prompt-template persistence model + inputs:
- `PromptTemplateSummary`
- `CreatePromptTemplateInput`
- `UpdatePromptTemplateInput`
3. Added schema support for prompt templates:
- `prompt_templates`
4. Added API module and router bindings:
- `src-tauri/src/api/prompt_templates.rs`
- route dispatch in `src-tauri/src/api/server.rs`
5. Added integration coverage:
- `src-tauri/tests/prompt_templates_endpoints.rs`
6. Updated HTTP contract-surface expected status table for prompt-template routes.

## Major Refactors / Rewrites

1. Continued strict domain separation in API layer instead of growing monolithic handlers.
2. Added repository fetch helpers and row mappers for prompt templates to avoid repetitive SQL/serialization logic.
3. Standardized CRUD validation patterns across domains (required fields, at-least-one update field, not-found semantics).

## Key Issues Found

1. Prompt-template routes were previously mounted but unimplemented (`501`).
2. No persistence schema existed for prompt templates.
3. Contract-required fields (`name`, `template_text`) needed enforced validation at write boundary.

## Remaining Technical Debt

1. `db/projects.rs` remains oversized and should be split by domain.
2. Candidate table overlap still exists (`run_job_candidates` and `run_candidates`).
3. Unimplemented contract domains still include:
- provider accounts
- style guides
- characters
- reference sets
- chat
- agent instructions
- voice
- secrets

## Next Phase Goals (Immediate)

1. Implement provider-account CRUD domain.
2. Add integration tests for provider-account validation and lifecycle.
3. Begin repository file split to reduce domain coupling in `db/projects.rs`.

## Validation Snapshot

1. `cargo fmt --all`
2. `cargo test`
3. `npm run backend:rust:test --silent`
4. Passing suites now include:
- contract parity
- HTTP contract-surface
- projects/storage
- runs/assets
- asset-links
- analytics
- exports
- prompt templates
