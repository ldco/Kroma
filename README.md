# image-api-tool

Standalone Node.js tool for staged image edits via OpenAI Images Edits API, with spend guards, reproducible run logs, output quality guard, project-scoped backend database, and optional post-iterations for upscale, color correction, and background removal.

## Requirements

- Node.js 20+
- Python 3 (for `rembg`, Real-ESRGAN python backend, and color correction)
- Pillow (`PIL`) for color correction and output QA guard
- SQLite (CLI `sqlite3` is recommended for manual inspection)
- AWS CLI v2 (optional, only for `sync-project-s3`)
- OpenAI API key with image edit access (only for `run` mode)

## Setup

```bash
cp .env.example .env
# edit .env

npm run rembg:setup
npm run realesrgan:python:setup
# optional ncnn fallback
npm run realesrgan:setup
```

Then add your own source files and update `settings/manifest.json`:

- `scene_refs` for input scenes
- `style_refs` for style anchors

## Project Isolation

All outputs are now project-scoped:

- `generated/projects/<project>/outputs/`
- `generated/projects/<project>/upscaled/`
- `generated/projects/<project>/color_corrected/`
- `generated/projects/<project>/background_removed/`
- `generated/projects/<project>/runs/`
- `generated/projects/<project>/archive/bad/`
- `generated/projects/<project>/archive/replaced/`

Use `--project <name>` on every command. Default is `default`.

## Commands

```bash
# dry run
npm run lab -- dry --project demo

# paid generation
npm run lab -- run --project demo --confirm-spend

# paid generation with 4 candidates per scene and auto-pick best
npm run lab -- run --project demo --confirm-spend --candidates 4

# run with explicit local directory for this project
npm run lab -- run --project demo --project-root /data/iat/demo --confirm-spend

# post chain in one run (order: bg-remove -> upscale -> color)
npm run lab -- run --project demo --confirm-spend --post-bg-remove --post-upscale --upscale-backend python --post-color --post-color-profile cinematic_warm

# only upscale
npm run upscale -- --project demo --input generated/projects/demo/outputs --output generated/projects/demo/upscaled --upscale-backend python --upscale-scale 2

# only bg-remove (production chain: rembg -> OpenAI refine)
npm run bgremove -- --project demo --input generated/projects/demo/outputs --output generated/projects/demo/background_removed --bg-remove-backends rembg --bg-refine-openai true

# quality audit only (no generation)
npm run qa -- --project demo --input generated/projects/demo/background_removed

# archive rejected files manually
npm run archivebad -- --project demo --input generated/projects/demo/background_removed
```

## Backend Data Layer

SQLite backend is managed by `scripts/backend.py` (future multi-user ready; currently no auth required).

Default DB path:

- `generated/backend/app.db`

Main entities:

- `schema_migrations`
- `users`
- `projects`
- `project_api_secrets` (encrypted)
- `runs`
- `run_jobs`
- `run_job_candidates`
- `assets`
- `project_snapshots`
- `project_exports`
- `chat_sessions`
- `chat_messages`
- `agent_instructions`
- `agent_instruction_events`
- `voice_requests`

Quick start:

```bash
# initialize DB + default local user
npm run backend:init

# apply/verify migrations
npm run backend:migrate

# create project record (owner: local)
python3 scripts/backend.py create-project --name "eugenia_prod" --slug eugenia_prod

# list projects
npm run backend:project:list

# export only one project (DB subset + files)
python3 scripts/backend.py export-project --project-slug eugenia_prod --output generated/exports/eugenia_prod.tar.gz

# set local storage root for one project
python3 scripts/backend.py set-project-storage-local --project-slug eugenia_prod --project-root /data/iat/eugenia_prod

# configure S3 storage for one project
python3 scripts/backend.py set-project-storage-s3 --project-slug eugenia_prod --enabled true --bucket my-art-bucket --prefix iat-prod --region us-east-1

# sync this project files to S3
python3 scripts/backend.py sync-project-s3 --project-slug eugenia_prod

# store encrypted project API secret
python3 scripts/backend.py set-project-secret --project-slug eugenia_prod --provider-code openai --secret-name api_key --secret-value sk-...

# list masked secrets
python3 scripts/backend.py list-project-secrets --project-slug eugenia_prod
```

`image-lab` auto-ingests each run log into backend DB by default.
`image-lab` can also resolve project local root from backend storage policy.

Backend ingest flags in `lab` command:

- `--backend-db-ingest true|false`
- `--backend-db-required true|false`
- `--backend-db <path>`
- `--backend-python-bin <python>`

Project storage flags in `lab` command:

- `--project-root <path>` (manual override, highest priority)
- `--backend-storage-resolve true|false` (default true)
- `--backend-storage-required true|false`

S3 sync flags in `lab` command:

- `--storage-sync-s3 true|false` (default false)
- `--storage-sync-dry-run true|false`
- `--storage-sync-delete true|false`
- `--storage-sync-required true|false`

## Backend API

Start REST API server:

```bash
npm run backend:api
```

Run instruction queue worker:

```bash
npm run backend:worker
# or single iteration
npm run backend:worker:once
```

Default bind:

- `http://127.0.0.1:8787`

Main endpoints:

- `GET /health`
- `GET /api/projects`
- `POST /api/projects`
- `GET /api/projects/:slug`
- `GET /api/projects/:slug/storage`
- `PUT /api/projects/:slug/storage/local`
- `PUT /api/projects/:slug/storage/s3`
- `POST /api/projects/:slug/runs/ingest`
- `POST /api/projects/:slug/export`
- `POST /api/projects/:slug/sync-s3`
- `POST /api/projects/:slug/chat/sessions`
- `GET /api/projects/:slug/chat/sessions`
- `GET /api/projects/:slug/chat/sessions/:sessionId/messages`
- `POST /api/projects/:slug/chat/sessions/:sessionId/messages`
- `POST /api/projects/:slug/agent/instructions`
- `GET /api/projects/:slug/agent/instructions`
- `GET /api/projects/:slug/agent/instructions/:instructionId`
- `POST /api/projects/:slug/agent/instructions/:instructionId/confirm`
- `POST /api/projects/:slug/agent/instructions/:instructionId/cancel`
- `GET /api/projects/:slug/agent/instructions/:instructionId/events`
- `POST /api/projects/:slug/voice/stt`
- `POST /api/projects/:slug/voice/tts`
- `GET /api/projects/:slug/voice/requests/:requestId`
- `GET /api/projects/:slug/secrets`
- `POST /api/projects/:slug/secrets`
- `DELETE /api/projects/:slug/secrets/:providerCode/:secretName`

Agent dispatch env vars (optional):

- `IAT_AGENT_API_URL` (where instruction payloads are POSTed)
- `IAT_AGENT_API_TOKEN` (Bearer token for agent API)
- `IAT_MASTER_KEY` (optional, encryption master key override)
- `IAT_MASTER_KEY_FILE` (optional, local fallback key file path)

OpenAPI contract:

- `openapi/backend-api.openapi.yaml`

Contract smoke test:

```bash
npm run backend:contract:smoke
```

Example requests:

```bash
curl -s http://127.0.0.1:8787/health

curl -s -X POST http://127.0.0.1:8787/api/projects \
  -H 'Content-Type: application/json' \
  -d '{"name":"Eugenia Prod","slug":"eugenia_prod"}'

curl -s -X PUT http://127.0.0.1:8787/api/projects/eugenia_prod/storage/local \
  -H 'Content-Type: application/json' \
  -d '{"project_root":"/data/iat/eugenia_prod"}'

curl -s -X POST http://127.0.0.1:8787/api/projects/eugenia_prod/chat/sessions \
  -H 'Content-Type: application/json' \
  -d '{"title":"Scene planning"}'

curl -s -X POST http://127.0.0.1:8787/api/projects/eugenia_prod/agent/instructions \
  -H 'Content-Type: application/json' \
  -d '{"instruction_type":"pipeline_run","dispatch_to_agent":false,"payload_json":{"stage":"style","candidates":3}}'
```

## Auto Archiving

When a target output file already exists, the old file is auto-moved to:

- `generated/projects/<project>/archive/replaced/`

Disable this behavior with `--no-archive-replaced`.

## Production Background Removal

Default production chain:

- `rembg` (base matte extraction)
- `openai` refine pass (edge cleanup for hair/transparency and smoother compositing look)

You can control refine behavior:

- `--bg-refine-openai true|false`
- `--bg-refine-openai-required true|false`
- `--bg-refine-model gpt-image-1`
- `--bg-refine-prompt "..."`

Required key in `.env` for refine:

- `OPENAI_API_KEY`

Optional external fallbacks are still available via `--bg-remove-backends rembg,photoroom,removebg` with keys:

- `PHOTOROOM_API_KEY`
- `REMOVE_BG_API_KEY`

## Output Quality Guard

`run` mode now checks final outputs with `scripts/output-guard.py` using `settings/manifest.json`:

- `output_guard.enforce_grayscale`
- `output_guard.max_chroma_delta`
- `output_guard.fail_on_chroma_exceed`

Behavior:

- if hard-fail rules are triggered, job status becomes `failed_output_guard`;
- failed output is auto-moved to `generated/projects/<project>/archive/bad/`;
- run exits with non-zero status after writing run log.

CLI overrides:

- `--output-guard-enabled true|false`
- `--enforce-grayscale true|false`
- `--max-chroma-delta <number>`
- `--fail-on-chroma-exceed true|false`
- `--qa-python-bin <python>`

## Multi-Candidate Selection

For each generation job you can request multiple candidates:

- `--candidates <N>` (default `1`)
- hard limit via `settings/manifest.json -> generation.max_candidates` (default `6`)
- override limit with `--allow-many-candidates` (run mode)

Selection logic:

- candidates that fail output guard are marked `failed_output_guard` and archived to `archive/bad`;
- among passing candidates, tool picks best by:
  - fewer hard failures,
  - fewer soft warnings,
  - lower average chroma exceed.

Run log stores all candidates and selected winner (`selected_candidate`, `final_output`).
