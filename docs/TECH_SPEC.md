# IAT Toolkit - Technical Specification (v1)

Date: 2026-02-20  
Status: Draft for implementation

## 1. Product Goal

Build a production-grade toolkit for large visual projects (comics, visual stories, campaigns) where:

1. style consistency is mandatory ("one hand");
2. characters must remain visually stable across many images;
3. background removal and post-processing are first-class features;
4. both local models and paid API providers can be used in one pipeline.

## 2. What The Application Must Do

## 2.1 Project and User Layer

1. Support multiple users (for now app runs in single-user mode, but DB is multi-user ready).
2. Each user can own many projects.
3. Each project has isolated data, runs, assets, storage policy, and exports.
4. Project must be exportable as a standalone package (DB slice + files).

## 2.2 Creative Pipeline

1. Generate images in staged workflow: style lock -> light/time -> weather/effects -> optional advanced passes.
2. Multi-candidate generation per task with automatic best-candidate selection.
3. Background removal with fallback chain (`rembg`, optional paid APIs, optional refine pass).
4. Optional post-processing: upscale, color correction, QA guard.
5. Strict run logging for reproducibility.

## 2.3 Style and Identity Consistency

1. Store style guides and reference sets per project.
2. Store character profiles and reference images per character.
3. Track quality metrics for each candidate/result.
4. Keep prompt versions and model/provider parameters for every run.

## 2.4 Storage and Deployment

1. Support project-specific local storage root.
2. Support optional project-specific S3 sync.
3. Allow mixed mode: local first + S3 backup/sync.

## 2.5 Providers and Cost Tracking

1. Support connectors for free/local tools and paid API providers.
2. Persist cost and usage per provider/run/project.
3. Persist failures and retries for provider calls.

## 2.6 Conversational Copilot (Voice + Agent Instructions)

1. Provide chat bot interface that accepts text and voice input.
2. Convert voice to text (STT), generate assistant response, optionally synthesize voice output (TTS).
3. Allow bot to create structured instructions for AI agent via internal API.
4. Support instruction lifecycle: `draft -> queued -> running -> done|failed|canceled`.
5. Persist all conversations, messages, instruction payloads, and execution logs.
6. Enforce guardrails: project scope, allowed actions, confirmation for destructive actions.

## 3. Non-Goals for Initial MVP

1. No authentication UI yet (single-user runtime mode).
2. No final GUI yet (backend-first implementation).
3. No training/fine-tuning pipeline yet (only inference + orchestration).

## 4. Architecture Baseline

1. Backend service: REST API + job orchestration.
2. DB: PostgreSQL in production, SQLite allowed for local/dev bootstrap.
3. File storage: local filesystem and optional S3.
4. Queue: asynchronous job execution layer (can start with in-process worker, later external queue).

## 4.1 Product Runtime Target (Authoritative)

1. Final desktop product runtime is **Tauri app + Rust backend core**.
2. Frontend (Puppet Master / Nuxt-based framework) is deferred until backend architecture and contracts are frozen.
3. End-user runtime must not require manual Python/Node/Bash installation.
4. Any current script-based tooling is transitional implementation scaffolding and not the target runtime architecture.

## 4.2 Backend-First Gate

Frontend implementation can start only after all backend gates are complete:

1. Stable domain architecture (projects, storage, runs, assets, quality, costs, audit, instructions).
2. Stable command/API contracts for frontend integration.
3. Stable DB schema + migration policy.
4. Stable execution pipeline semantics and error model.

## 5. Database Design (Target)

Note: table names below are canonical target schema. Existing schema can be migrated to this structure incrementally.

## 5.1 Core Entities

### `app_users`

Purpose: user identity (single-user now, multi-user later).

Columns:

1. `id UUID PK`
2. `username TEXT UNIQUE NOT NULL`
3. `display_name TEXT NOT NULL`
4. `email TEXT NULL`
5. `is_active BOOLEAN NOT NULL DEFAULT TRUE`
6. `created_at TIMESTAMPTZ NOT NULL`
7. `updated_at TIMESTAMPTZ NOT NULL`

### `projects`

Purpose: root entity for all creative work.

Columns:

1. `id UUID PK`
2. `owner_user_id UUID NOT NULL FK -> app_users.id`
3. `slug TEXT NOT NULL`
4. `name TEXT NOT NULL`
5. `description TEXT NOT NULL DEFAULT ''`
6. `status TEXT NOT NULL` (`active|archived|deleted`)
7. `settings_json JSONB NOT NULL DEFAULT '{}'`
8. `created_at TIMESTAMPTZ NOT NULL`
9. `updated_at TIMESTAMPTZ NOT NULL`

Constraints:

1. unique `(owner_user_id, slug)`

### `project_storage`

Purpose: normalized storage policy (local + S3) per project.

Columns:

1. `id UUID PK`
2. `project_id UUID UNIQUE NOT NULL FK -> projects.id`
3. `local_base_dir TEXT NOT NULL DEFAULT 'var/projects'`
4. `local_project_root TEXT NULL`
5. `s3_enabled BOOLEAN NOT NULL DEFAULT FALSE`
6. `s3_bucket TEXT NULL`
7. `s3_prefix TEXT NULL`
8. `s3_region TEXT NULL`
9. `s3_profile TEXT NULL`
10. `s3_endpoint_url TEXT NULL`
11. `created_at TIMESTAMPTZ NOT NULL`
12. `updated_at TIMESTAMPTZ NOT NULL`

### `provider_accounts`

Purpose: provider configuration (OpenAI, remove.bg, PhotoRoom, local engines).

Columns:

1. `id UUID PK`
2. `project_id UUID NOT NULL FK -> projects.id`
3. `provider_code TEXT NOT NULL` (`openai|removebg|photoroom|local_realesrgan|...`)
4. `is_enabled BOOLEAN NOT NULL DEFAULT TRUE`
5. `config_json JSONB NOT NULL DEFAULT '{}'`
6. `created_at TIMESTAMPTZ NOT NULL`
7. `updated_at TIMESTAMPTZ NOT NULL`

Constraint:

1. unique `(project_id, provider_code)`

### `project_api_secrets`

Purpose: encrypted secrets for paid providers.

Columns:

1. `id UUID PK`
2. `project_id UUID NOT NULL FK -> projects.id`
3. `provider_code TEXT NOT NULL`
4. `secret_name TEXT NOT NULL`
5. `secret_ciphertext TEXT NOT NULL`
6. `kms_key_ref TEXT NULL`
7. `created_at TIMESTAMPTZ NOT NULL`
8. `updated_at TIMESTAMPTZ NOT NULL`

Constraint:

1. unique `(project_id, provider_code, secret_name)`

## 5.2 Creative Knowledge Layer

### `style_guides`

Purpose: style lock definition for project.

Columns:

1. `id UUID PK`
2. `project_id UUID NOT NULL FK -> projects.id`
3. `name TEXT NOT NULL`
4. `description TEXT NOT NULL DEFAULT ''`
5. `rules_json JSONB NOT NULL DEFAULT '{}'`
6. `is_default BOOLEAN NOT NULL DEFAULT FALSE`
7. `created_at TIMESTAMPTZ NOT NULL`
8. `updated_at TIMESTAMPTZ NOT NULL`

### `characters`

Purpose: stable character identity in long projects.

Columns:

1. `id UUID PK`
2. `project_id UUID NOT NULL FK -> projects.id`
3. `code TEXT NOT NULL` (stable short id)
4. `name TEXT NOT NULL`
5. `bio TEXT NOT NULL DEFAULT ''`
6. `identity_constraints_json JSONB NOT NULL DEFAULT '{}'`
7. `created_at TIMESTAMPTZ NOT NULL`
8. `updated_at TIMESTAMPTZ NOT NULL`

Constraint:

1. unique `(project_id, code)`

### `reference_sets`

Purpose: reusable packs of references (style, character, scene, mood).

Columns:

1. `id UUID PK`
2. `project_id UUID NOT NULL FK -> projects.id`
3. `name TEXT NOT NULL`
4. `kind TEXT NOT NULL` (`style|character|scene|lighting|other`)
5. `metadata_json JSONB NOT NULL DEFAULT '{}'`
6. `created_at TIMESTAMPTZ NOT NULL`
7. `updated_at TIMESTAMPTZ NOT NULL`

### `reference_items`

Purpose: files linked into reference sets.

Columns:

1. `id UUID PK`
2. `reference_set_id UUID NOT NULL FK -> reference_sets.id`
3. `asset_id UUID NOT NULL FK -> assets.id`
4. `weight NUMERIC(8,4) NOT NULL DEFAULT 1.0`
5. `notes TEXT NOT NULL DEFAULT ''`
6. `created_at TIMESTAMPTZ NOT NULL`

Constraint:

1. unique `(reference_set_id, asset_id)`

## 5.3 Asset and File Layer

### `assets`

Purpose: registry of every relevant file in project lifecycle.

Columns:

1. `id UUID PK`
2. `project_id UUID NOT NULL FK -> projects.id`
3. `kind TEXT NOT NULL` (`input|reference|generated|mask|upscaled|bg_removed|final|run_log|export|other`)
4. `storage_uri TEXT NOT NULL` (local path or `s3://...`)
5. `storage_backend TEXT NOT NULL` (`local|s3`)
6. `mime_type TEXT NULL`
7. `width INT NULL`
8. `height INT NULL`
9. `sha256 TEXT NULL`
10. `metadata_json JSONB NOT NULL DEFAULT '{}'`
11. `created_at TIMESTAMPTZ NOT NULL`

Index:

1. `(project_id, kind, created_at DESC)`

### `asset_links`

Purpose: explicit graph between assets (derived-from relation).

Columns:

1. `id UUID PK`
2. `project_id UUID NOT NULL FK -> projects.id`
3. `parent_asset_id UUID NOT NULL FK -> assets.id`
4. `child_asset_id UUID NOT NULL FK -> assets.id`
5. `link_type TEXT NOT NULL` (`derived_from|variant_of|mask_for|reference_of`)
6. `created_at TIMESTAMPTZ NOT NULL`

Constraint:

1. unique `(parent_asset_id, child_asset_id, link_type)`

## 5.4 Run Orchestration Layer

### `runs`

Purpose: one full pipeline execution for a project.

Columns:

1. `id UUID PK`
2. `project_id UUID NOT NULL FK -> projects.id`
3. `run_mode TEXT NOT NULL` (`dry|run`)
4. `status TEXT NOT NULL` (`planned|running|done|failed|partial`)
5. `stage TEXT NULL`
6. `time_of_day TEXT NULL`
7. `weather TEXT NULL`
8. `model_name TEXT NULL`
9. `provider_code TEXT NULL`
10. `settings_snapshot_json JSONB NOT NULL DEFAULT '{}'`
11. `started_at TIMESTAMPTZ NULL`
12. `finished_at TIMESTAMPTZ NULL`
13. `created_at TIMESTAMPTZ NOT NULL`

### `run_jobs`

Purpose: unit of work inside run (scene/shot request).

Columns:

1. `id UUID PK`
2. `run_id UUID NOT NULL FK -> runs.id`
3. `job_key TEXT NOT NULL`
4. `status TEXT NOT NULL` (`planned|running|done|failed_output_guard|failed`)
5. `prompt_text TEXT NOT NULL`
6. `selected_candidate_index INT NULL`
7. `final_asset_id UUID NULL FK -> assets.id`
8. `meta_json JSONB NOT NULL DEFAULT '{}'`
9. `created_at TIMESTAMPTZ NOT NULL`

Constraint:

1. unique `(run_id, job_key)`

### `run_candidates`

Purpose: all generated variants for each job.

Columns:

1. `id UUID PK`
2. `job_id UUID NOT NULL FK -> run_jobs.id`
3. `candidate_index INT NOT NULL`
4. `status TEXT NOT NULL` (`generated|done|failed_output_guard|failed`)
5. `output_asset_id UUID NULL FK -> assets.id`
6. `final_asset_id UUID NULL FK -> assets.id`
7. `rank_hard_failures INT NOT NULL DEFAULT 0`
8. `rank_soft_warnings INT NOT NULL DEFAULT 0`
9. `rank_avg_chroma_exceed NUMERIC(12,6) NOT NULL DEFAULT 0`
10. `meta_json JSONB NOT NULL DEFAULT '{}'`
11. `created_at TIMESTAMPTZ NOT NULL`

Constraint:

1. unique `(job_id, candidate_index)`

### `quality_reports`

Purpose: QA/guard outcomes and scoring details.

Columns:

1. `id UUID PK`
2. `project_id UUID NOT NULL FK -> projects.id`
3. `run_id UUID NULL FK -> runs.id`
4. `job_id UUID NULL FK -> run_jobs.id`
5. `candidate_id UUID NULL FK -> run_candidates.id`
6. `report_type TEXT NOT NULL` (`output_guard|model_score|human_review`)
7. `summary_json JSONB NOT NULL`
8. `created_at TIMESTAMPTZ NOT NULL`

## 5.5 Prompt, Cost, Export, Audit

### `prompt_templates`

1. `id UUID PK`
2. `project_id UUID NOT NULL FK -> projects.id`
3. `name TEXT NOT NULL`
4. `stage TEXT NOT NULL`
5. `template_text TEXT NOT NULL`
6. `version INT NOT NULL`
7. `is_active BOOLEAN NOT NULL DEFAULT TRUE`
8. `created_at TIMESTAMPTZ NOT NULL`

### `cost_events`

1. `id UUID PK`
2. `project_id UUID NOT NULL FK -> projects.id`
3. `run_id UUID NULL FK -> runs.id`
4. `provider_code TEXT NOT NULL`
5. `operation_code TEXT NOT NULL`
6. `units NUMERIC(18,6) NOT NULL DEFAULT 0`
7. `cost_usd NUMERIC(18,6) NOT NULL DEFAULT 0`
8. `currency TEXT NOT NULL DEFAULT 'USD'`
9. `meta_json JSONB NOT NULL DEFAULT '{}'`
10. `created_at TIMESTAMPTZ NOT NULL`

### `project_exports`

1. `id UUID PK`
2. `project_id UUID NOT NULL FK -> projects.id`
3. `export_asset_id UUID NULL FK -> assets.id`
4. `format TEXT NOT NULL` (`tar.gz|zip|folder`)
5. `sha256 TEXT NULL`
6. `created_at TIMESTAMPTZ NOT NULL`

### `audit_events`

1. `id UUID PK`
2. `project_id UUID NULL FK -> projects.id`
3. `actor_user_id UUID NULL FK -> app_users.id`
4. `event_code TEXT NOT NULL`
5. `payload_json JSONB NOT NULL DEFAULT '{}'`
6. `created_at TIMESTAMPTZ NOT NULL`

## 5.6 Chatbot and Agent Instruction Layer

### `chat_sessions`

Purpose: one conversation context for a user in one project.

Columns:

1. `id UUID PK`
2. `project_id UUID NOT NULL FK -> projects.id`
3. `user_id UUID NOT NULL FK -> app_users.id`
4. `title TEXT NOT NULL DEFAULT ''`
5. `status TEXT NOT NULL` (`active|archived`)
6. `context_json JSONB NOT NULL DEFAULT '{}'`
7. `created_at TIMESTAMPTZ NOT NULL`
8. `updated_at TIMESTAMPTZ NOT NULL`

### `chat_messages`

Purpose: persistent transcript (user/assistant/system/tool).

Columns:

1. `id UUID PK`
2. `session_id UUID NOT NULL FK -> chat_sessions.id`
3. `role TEXT NOT NULL` (`user|assistant|system|tool`)
4. `content_text TEXT NOT NULL`
5. `content_json JSONB NOT NULL DEFAULT '{}'`
6. `voice_asset_id UUID NULL FK -> assets.id`
7. `token_usage_json JSONB NOT NULL DEFAULT '{}'`
8. `created_at TIMESTAMPTZ NOT NULL`

### `agent_instructions`

Purpose: normalized instruction objects emitted by chatbot and sent to AI agent API.

Columns:

1. `id UUID PK`
2. `project_id UUID NOT NULL FK -> projects.id`
3. `session_id UUID NULL FK -> chat_sessions.id`
4. `message_id UUID NULL FK -> chat_messages.id`
5. `instruction_type TEXT NOT NULL` (`pipeline_run|asset_edit|qa_check|storage_sync|custom`)
6. `payload_json JSONB NOT NULL`
7. `status TEXT NOT NULL` (`draft|queued|running|done|failed|canceled`)
8. `priority INT NOT NULL DEFAULT 100`
9. `requires_confirmation BOOLEAN NOT NULL DEFAULT FALSE`
10. `confirmed_by_user_id UUID NULL FK -> app_users.id`
11. `queued_at TIMESTAMPTZ NULL`
12. `started_at TIMESTAMPTZ NULL`
13. `finished_at TIMESTAMPTZ NULL`
14. `created_at TIMESTAMPTZ NOT NULL`
15. `updated_at TIMESTAMPTZ NOT NULL`

### `agent_instruction_events`

Purpose: execution/audit trail for each instruction.

Columns:

1. `id UUID PK`
2. `instruction_id UUID NOT NULL FK -> agent_instructions.id`
3. `event_type TEXT NOT NULL` (`queued|started|log|result|error|status_change`)
4. `event_payload_json JSONB NOT NULL DEFAULT '{}'`
5. `created_at TIMESTAMPTZ NOT NULL`

### `voice_requests`

Purpose: STT/TTS requests tied to chat messages.

Columns:

1. `id UUID PK`
2. `project_id UUID NOT NULL FK -> projects.id`
3. `session_id UUID NULL FK -> chat_sessions.id`
4. `message_id UUID NULL FK -> chat_messages.id`
5. `direction TEXT NOT NULL` (`stt|tts`)
6. `provider_code TEXT NOT NULL`
7. `input_asset_id UUID NULL FK -> assets.id`
8. `output_asset_id UUID NULL FK -> assets.id`
9. `status TEXT NOT NULL` (`queued|running|done|failed`)
10. `latency_ms INT NULL`
11. `meta_json JSONB NOT NULL DEFAULT '{}'`
12. `created_at TIMESTAMPTZ NOT NULL`

## 6. Required Indexes

1. `projects(owner_user_id, slug)`
2. `runs(project_id, created_at DESC)`
3. `run_jobs(run_id, status)`
4. `run_candidates(job_id, candidate_index)`
5. `assets(project_id, kind, created_at DESC)`
6. `assets(project_id, sha256)`
7. `cost_events(project_id, created_at DESC)`
8. `quality_reports(project_id, created_at DESC)`
9. `chat_sessions(project_id, updated_at DESC)`
10. `chat_messages(session_id, created_at ASC)`
11. `agent_instructions(project_id, status, priority, created_at DESC)`
12. `agent_instruction_events(instruction_id, created_at ASC)`
13. `voice_requests(project_id, status, created_at DESC)`

## 7. Data Isolation Rules

1. All query paths must include `project_id` scope.
2. No cross-project asset references are allowed.
3. Export operation must only include rows/files bound to one `project_id`.
4. Storage policy is defined per project, not globally.

## 8. Security and Secret Handling

1. API keys must not be stored in plaintext.
2. Secrets must be encrypted at rest (`project_api_secrets.secret_ciphertext`).
3. Run logs must not print secret values.
4. S3 sync operations must be auditable (`audit_events` + optional run logs).

## 9. API Surface (Backend-First)

Required API groups:

1. users/projects
2. project storage (local/s3)
3. runs/jobs/candidates
4. assets and file links
5. quality reports
6. provider config and cost
7. export and snapshot
8. chatbot sessions/messages
9. agent instructions and execution events
10. voice STT/TTS endpoints

## 9.1 Required Chatbot/Agent Endpoints (MVP)

### Chat Sessions

1. `POST /api/projects/:slug/chat/sessions`
2. `GET /api/projects/:slug/chat/sessions`
3. `GET /api/projects/:slug/chat/sessions/:sessionId`
4. `POST /api/projects/:slug/chat/sessions/:sessionId/messages`
5. `GET /api/projects/:slug/chat/sessions/:sessionId/messages`

### Agent Instructions

1. `POST /api/projects/:slug/agent/instructions`
2. `GET /api/projects/:slug/agent/instructions`
3. `GET /api/projects/:slug/agent/instructions/:instructionId`
4. `POST /api/projects/:slug/agent/instructions/:instructionId/confirm`
5. `POST /api/projects/:slug/agent/instructions/:instructionId/cancel`
6. `GET /api/projects/:slug/agent/instructions/:instructionId/events`

### Voice

1. `POST /api/projects/:slug/voice/stt`
2. `POST /api/projects/:slug/voice/tts`
3. `GET /api/projects/:slug/voice/requests/:requestId`

## 9.2 Agent Instruction Payload Contract (MVP)

All chatbot-generated instructions sent to AI agent API must follow this envelope:

1. `instruction_id` (UUID)
2. `project_slug` (TEXT)
3. `instruction_type` (`pipeline_run|asset_edit|qa_check|storage_sync|custom`)
4. `objective` (TEXT, plain-language goal)
5. `constraints` (object, optional hard limits and rules)
6. `inputs` (object, paths/asset_ids/references)
7. `execution` (object, model/provider/options)
8. `confirmation_required` (BOOLEAN)
9. `requested_by` (user id/username)
10. `callback` (object with status/event endpoint)

Minimum response contract from agent API:

1. `instruction_id`
2. `status` (`accepted|rejected|running|done|failed`)
3. `message`
4. `result` (object, optional)
5. `error` (object, optional)

## 10. MVP Delivery Plan

Phase 1:

1. finalize schema migration to tables in this spec;
2. keep single-user runtime with `app_users` seeded as `local`;
3. stabilize project storage (local + optional S3);
4. persist runs/jobs/candidates/assets/quality and export;
5. implement chatbot text mode + instruction dispatch to agent API;
6. implement instruction status tracking and execution event log.

Phase 2:

1. add auth and multi-user API tokens;
2. add human review workflow for candidate ranking;
3. add advanced scoring models and richer consistency metrics;
4. add voice STT/TTS pipeline for chatbot (streaming + low-latency mode).
