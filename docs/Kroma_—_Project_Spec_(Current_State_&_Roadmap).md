# Kroma — Project Spec (Current State & Roadmap)

# Kroma — Full Project Specification

**Last updated:** 2026-02-21  
**Status:** Active development — Phase 1 in progress

---

## 1. What Is Kroma?

Kroma (package: `image-api-tool`) is a **self-hosted AI image production pipeline** for large visual projects — comics, visual stories, illustration campaigns, character-consistent series.

It is **not** a GUI app today. It is a **backend-first, CLI-driven system** that orchestrates AI image generation, quality control, and post-processing in a reproducible, cost-safe, project-isolated way.

### Core Value Propositions

| Value | How it's delivered |
|---|---|
| Style consistency | Staged workflow: style lock → time/light → weather → characters |
| Cost safety | Dry-run mode, `--confirm-spend` guard, batch limits, per-run logs |
| Reproducibility | Every run writes a machine-readable JSON log |
| Quality control | Automated QA guard (chroma/grayscale checks), multi-candidate selection |
| Project isolation | All data, files, and DB records are scoped to a project slug |
| Post-processing | Background removal → upscale → color correction, all in one chain |

---

## 2. System Architecture

```mermaid
graph TD
    CLI["CLI Runner\nimage-lab.mjs (Node.js)"]
    API["REST API Server\nbackend_api.py (Python)"]
    DB["SQLite Database\nvar/backend/app.db"]
    Worker["Instruction Worker\nagent_worker.py"]
    Dispatch["Agent Dispatcher\nagent_dispatch.py"]
    OpenAI["OpenAI API\ngpt-image-1"]
    ESRGAN["Real-ESRGAN\n(local upscaler)"]
    Rembg["rembg\n(local BG removal)"]
    S3["S3 Storage\n(optional)"]
    FS["Local Filesystem\nproject_root/"]

    CLI --> OpenAI
    CLI --> ESRGAN
    CLI --> Rembg
    CLI --> DB
    CLI --> FS
    API --> DB
    API --> FS
    API --> S3
    Worker --> DB
    Worker --> Dispatch
    Dispatch --> OpenAI
```

### Layer Breakdown

#### Layer 1 — CLI Pipeline (`scripts/image-lab.mjs`, Node.js)
The main creative runner. Accepts commands: `run`, `dry`, `upscale`, `bgremove`, `color`, `qa`, `archive-bad`.

- Reads config from `settings/manifest.json` and `settings/presets.json`
- Builds a job queue from `scene_refs` + `style_refs`
- Calls **OpenAI Images Edits API** (`/v1/images/edits`) with `gpt-image-1`
- Enforces spend guards (`--confirm-spend`, batch limits)
- Writes reproducible run logs (JSON) per run
- Runs post-processing chains: bg-remove → upscale → color correction
- Runs QA guard (`scripts/output-guard.py`) to reject bad outputs automatically
- Auto-ingests run results into the backend DB

#### Layer 2 — Backend Data Layer (`scripts/backend.py`, Python)
SQLite-backed data management and CLI admin tool.

- Schema initialization and migration tracking (`schema_migrations` table)
- Project/user CRUD, storage policy resolution (local + S3)
- Asset upsert with SHA-256 deduplication
- Run log ingestion (runs → jobs → candidates → assets)
- Encrypted secret management (Fernet encryption)
- Project export as portable `.tar.gz` (DB slice + files)
- S3 sync via AWS CLI

#### Layer 3 — REST API Server (`scripts/backend_api.py`, Python stdlib HTTP)
Thin HTTP server wrapping the backend layer. Serves at `http://127.0.0.1:8787`.

- Full project/storage/secrets CRUD
- Chat sessions and messages
- Agent instruction lifecycle (create → confirm → queue → run → done/failed)
- Voice STT/TTS request tracking
- Run ingestion and project export endpoints

#### Layer 4 — Agent Instruction Worker (`scripts/agent_worker.py`, Python)
Async polling worker that processes queued agent instructions.

- Polls DB every 2 seconds for `status = 'queued'` instructions
- Acquires optimistic lock (`locked_by`, `locked_at`) to prevent double-processing
- Resolves agent API target URL + token (env vars or encrypted DB secrets)
- Dispatches instruction payload via HTTP POST (`agent_dispatch.py`)
- Handles retries with exponential backoff (default: 3 attempts)
- Emits structured events to `agent_instruction_events` table

---

## 3. File Structure

```
app/
├── scripts/
│   ├── image-lab.mjs          # Main CLI runner (Node.js)
│   ├── backend.py             # DB layer + CLI admin commands
│   ├── backend_api.py         # REST API server
│   ├── agent_worker.py        # Instruction queue worker
│   ├── agent_dispatch.py      # HTTP dispatcher for agent API
│   ├── db_migrate.py          # Migration runner
│   ├── output-guard.py        # QA guard (Pillow/chroma checks)
│   ├── apply-color-correction.py
│   ├── rembg-remove.py
│   └── realesrgan-python-upscale.py
├── settings/
│   ├── manifest.json          # Single source of truth (prompts, refs, guards)
│   ├── presets.json           # Reusable time/weather bundles
│   ├── postprocess.json       # Upscale, bg-remove, color config
│   └── color-correction.json  # Color profiles (neutral, cinematic_warm, cold_rain)
├── openapi/
│   └── backend-api.openapi.yaml
├── var/                       # Runtime state (not versioned)
│   ├── backend/               # app.db, master.key
│   └── projects/              # default local project roots (if project_root not explicit)
├── tools/                     # Local tool runtime dirs (not versioned)
└── docs/
    └── WORKFLOW.md
```

---

## 4. Creative Pipeline (Staged Workflow)

```mermaid
flowchart TD
    A["Stage 1: Style Lock\nConsistent artistic hand\nGeometry preserved"] --> B["Stage 2: Time/Light Lock\nDay/night variation\nStyle stays stable"]
    B --> C["Stage 3: Weather/Effects\nRain, clear, etc.\nNever replaces structure"]
    C --> D["Stage 4+: Characters\nOnly after scene style is locked"]
    D --> E["Post-Processing Chain\nbg_remove → upscale → color"]
    E --> F["QA Guard\nChroma/grayscale check\nAuto-archive bad outputs"]
    F --> G["DB Ingest\nRun log → SQLite backend"]
```

### Post-Processing Chain (fixed order)

1. `rembg` — local background removal (free, fast)
2. OpenAI refine pass — edge cleanup for hair/transparency
3. Real-ESRGAN — local upscaling (2x or 4x)
4. Color correction — profile-based (neutral / cinematic_warm / cold_rain)

### Multi-Candidate Selection

- Request N candidates per scene (`--candidates N`, max 6 by default)
- QA guard runs on each candidate
- Best candidate selected by: fewer hard failures → fewer soft warnings → lower avg chroma exceed
- All candidates and the selected winner are stored in the run log and DB

---

## 5. Configuration System

### `settings/manifest.json`
Single source of truth for a run:

| Key | Purpose |
|---|---|
| `style_refs` | Style anchor image paths |
| `scene_refs` | Input scene image paths |
| `safe_batch_limit` | Max jobs per run (default: 20) |
| `generation.max_candidates` | Hard cap on candidates (default: 6) |
| `output_guard` | Grayscale enforcement + chroma thresholds |
| `policy.allowed_style_roots` | Prevents style-anchor contamination |
| `prompts.*` | All prompt text lives here, not in code |

### `settings/postprocess.json`
Controls upscale, bg-remove, and color correction backends and parameters.

### `settings/color-correction.json`
Named color profiles: `neutral`, `cinematic_warm`, `cold_rain`.

---

## 6. Database — Current vs Target Schema

The DB is managed by `scripts/backend.py` via `init_schema()`. Migrations are tracked in `schema_migrations`.

### Current Tables (implemented in `backend.py`)

| Table | Status |
|---|---|
| `schema_migrations` | ✅ Implemented |
| `users` | ✅ Implemented (single-user `local`) |
| `projects` | ✅ Implemented |
| `project_api_secrets` | ✅ Implemented (Fernet encrypted) |
| `runs` | ✅ Implemented |
| `run_jobs` | ✅ Implemented |
| `run_job_candidates` | ✅ Implemented |
| `assets` | ✅ Implemented |
| `project_snapshots` | ✅ Implemented |
| `project_exports` | ✅ Implemented |
| `chat_sessions` | ✅ Implemented |
| `chat_messages` | ✅ Implemented |
| `agent_instructions` | ✅ Implemented (with retry/lock fields) |
| `agent_instruction_events` | ✅ Implemented |
| `voice_requests` | ✅ Implemented |

### Target Tables (from `docs/TECH_SPEC.md` — not yet migrated)

| Table | Status | Notes |
|---|---|---|
| `app_users` | ⚠️ Rename from `users` | Add `email`, `is_active` columns |
| `project_storage` | ⚠️ Extract from `projects` | Normalize storage policy into own table |
| `provider_accounts` | ❌ Missing | Provider config per project |
| `style_guides` | ❌ Missing | Style lock definitions |
| `characters` | ❌ Missing | Character identity profiles |
| `reference_sets` | ❌ Missing | Reusable reference packs |
| `reference_items` | ❌ Missing | Files linked into reference sets |
| `asset_links` | ❌ Missing | Derived-from graph between assets |
| `run_candidates` | ⚠️ Rename from `run_job_candidates` | Add ranking columns |
| `quality_reports` | ❌ Missing | Normalized QA/guard outcomes |
| `prompt_templates` | ❌ Missing | Versioned prompt templates |
| `cost_events` | ❌ Missing | Per-provider cost tracking |
| `audit_events` | ❌ Missing | Full audit trail |

---

## 7. REST API Surface

Base URL: `http://127.0.0.1:8787`  
Contract: `file:openapi/backend-api.openapi.yaml`

### Implemented Endpoints

| Group | Endpoints |
|---|---|
| Health | `GET /health` |
| Projects | `GET/POST /api/projects`, `GET /api/projects/:slug` |
| Storage | `GET /api/projects/:slug/storage`, `PUT .../local`, `PUT .../s3` |
| Secrets | `GET/POST /api/projects/:slug/secrets`, `DELETE .../secrets/:provider/:name` |
| Runs | `POST /api/projects/:slug/runs/ingest` |
| Export | `POST /api/projects/:slug/export`, `POST .../sync-s3` |
| Chat | `POST/GET .../chat/sessions`, `POST/GET .../chat/sessions/:id/messages` |
| Agent | `POST/GET .../agent/instructions`, `GET/POST .../instructions/:id`, `POST .../confirm`, `POST .../cancel`, `GET .../events` |
| Voice | `POST .../voice/stt`, `POST .../voice/tts`, `GET .../voice/requests/:id` |

### Missing / Planned Endpoints

| Group | Missing |
|---|---|
| Projects | `GET /api/projects/:slug/chat/sessions/:sessionId` (single session detail) |
| Runs | `GET /api/projects/:slug/runs`, `GET .../runs/:runId`, `GET .../runs/:runId/jobs` |
| Assets | `GET /api/projects/:slug/assets`, `GET .../assets/:assetId` |
| Quality | `GET /api/projects/:slug/quality-reports` |
| Cost | `GET /api/projects/:slug/cost-events` |
| Characters | Full CRUD for `characters`, `style_guides`, `reference_sets` |
| Auth | `POST /auth/login`, `POST /auth/token` (Phase 2) |

---

## 8. Agent Instruction System

```mermaid
stateDiagram
    [*] --> draft
    draft --> queued : dispatch_to_agent=true
    queued --> running : worker picks up
    running --> done : agent responds OK
    running --> failed : max retries exceeded
    running --> queued : retry scheduled
    queued --> canceled : user cancels
    draft --> canceled : user cancels
```

### Instruction Payload Contract

Every instruction dispatched to an external agent API must include:

| Field | Type | Description |
|---|---|---|
| `instruction_id` | UUID | Stable identifier |
| `project_slug` | TEXT | Project scope |
| `instruction_type` | TEXT | `pipeline_run\|asset_edit\|qa_check\|storage_sync\|custom` |
| `objective` | TEXT | Plain-language goal |
| `constraints` | object | Hard limits and rules |
| `inputs` | object | Paths / asset IDs / references |
| `execution` | object | Model / provider / options |
| `confirmation_required` | boolean | Whether user must confirm before execution |
| `requested_by` | TEXT | User ID or username |
| `callback` | object | Status/event endpoint for agent to report back |

### Worker Configuration

| Parameter | Default | Description |
|---|---|---|
| `--poll-interval-seconds` | 2.0 | DB polling frequency |
| `--max-locked-seconds` | 120 | Stale lock timeout |
| `--default-max-attempts` | 3 | Retry limit per instruction |
| `--retry-backoff-seconds` | 10 | Base backoff multiplier |
| `--dispatch-timeout` | 20.0 | HTTP timeout per attempt |
| `--dispatch-retries` | 2 | HTTP-level retries |

---

## 9. External Integrations

| Integration | Purpose | Required | Config |
|---|---|---|---|
| OpenAI `gpt-image-1` | Image generation & editing | Yes (for `run` mode) | `OPENAI_API_KEY` in `.env` |
| OpenAI (bg refine) | Edge cleanup on cutouts | Optional | Same key |
| rembg (Python) | Local background removal | Optional | `tools/rembg/.venv` |
| Real-ESRGAN (Python) | Local upscaling | Optional | `tools/realesrgan-python/.venv` |
| Real-ESRGAN (ncnn) | Fallback upscaling binary | Optional | `tools/realesrgan/` |
| PhotoRoom API | Paid BG removal fallback | Optional | `PHOTOROOM_API_KEY` |
| remove.bg API | Paid BG removal fallback | Optional | `REMOVE_BG_API_KEY` |
| AWS CLI / S3 | Project file sync | Optional | AWS profile or env vars |
| External Agent API | Receives instruction payloads | Optional | `IAT_AGENT_API_URL` + `IAT_AGENT_API_TOKEN` |

---

## 10. What Needs to Be Built — Roadmap

### Phase 1 — Stabilize & Complete Backend (Current Priority)

#### 10.1 DB Schema Migration to Target Spec

The current DB schema partially matches the target defined in `file:docs/TECH_SPEC.md`. The following migrations are needed:

1. **Rename `users` → `app_users`**, add `email` and `is_active` columns
2. **Extract `project_storage` table** — normalize storage policy out of `projects` table
3. **Add `provider_accounts` table** — per-project provider config (OpenAI, rembg, etc.)
4. **Add `style_guides` table** — style lock definitions per project
5. **Add `characters` table** — stable character identity profiles
6. **Add `reference_sets` + `reference_items` tables** — reusable reference packs
7. **Add `asset_links` table** — derived-from graph between assets
8. **Rename `run_job_candidates` → `run_candidates`**, add ranking columns (`rank_hard_failures`, `rank_soft_warnings`, `rank_avg_chroma_exceed`)
9. **Add `quality_reports` table** — normalized QA/guard outcomes (currently only in run log JSON)
10. **Add `prompt_templates` table** — versioned prompt templates
11. **Add `cost_events` table** — per-provider cost tracking per run
12. **Add `audit_events` table** — full audit trail for all mutations

All migrations must be additive and tracked in `schema_migrations`.

#### 10.2 Missing API Endpoints

1. `GET /api/projects/:slug/runs` — list runs for a project
2. `GET /api/projects/:slug/runs/:runId` — run detail with jobs
3. `GET /api/projects/:slug/runs/:runId/jobs` — job list with candidates
4. `GET /api/projects/:slug/assets` — asset registry
5. `GET /api/projects/:slug/quality-reports` — QA report history
6. `GET /api/projects/:slug/cost-events` — cost tracking
7. Full CRUD for `characters`, `style_guides`, `reference_sets`

#### 10.3 Cost Tracking

- Capture OpenAI API cost per run/job/candidate (tokens, image units)
- Write `cost_events` rows during run ingestion
- Expose via `GET /api/projects/:slug/cost-events`

#### 10.4 Quality Reports Normalization

- Currently QA results live only in run log JSON
- Migrate to `quality_reports` table during run ingestion
- Link to `run_candidates` for per-candidate scoring

---

### Phase 2 — GUI Frontend

No GUI exists today. The backend-first approach is intentional, but a GUI is the next major milestone.

#### Proposed Frontend Stack

- **Framework:** React (or SvelteKit for lighter footprint)
- **API:** Consumes the existing REST API at `http://127.0.0.1:8787`
- **Hosting:** Local dev server (same machine as backend)

#### Required Views

| View | Description |
|---|---|
| Project Dashboard | List projects, create new, view status |
| Run Viewer | Browse runs, jobs, candidates with thumbnails |
| Asset Browser | Browse all assets by kind, filter, preview |
| Chat / Copilot | Text chat interface, send messages, view agent instructions |
| Instruction Queue | View queued/running/done instructions, confirm/cancel |
| Settings | Project storage config, secrets management |
| QA Report | Per-run quality report with pass/fail breakdown |

#### Project Dashboard Wireframe

```wireframe
<!DOCTYPE html>
<html>
<head>
<style>
* { box-sizing: border-box; margin: 0; padding: 0; font-family: sans-serif; }
body { background: #f5f5f5; color: #222; }
.topbar { background: #1a1a1a; color: #fff; padding: 12px 24px; display: flex; align-items: center; gap: 16px; }
.topbar .logo { font-weight: 700; font-size: 18px; letter-spacing: 1px; }
.topbar .nav { display: flex; gap: 16px; font-size: 13px; color: #aaa; }
.topbar .nav span { cursor: pointer; }
.topbar .nav span.active { color: #fff; }
.main { display: flex; height: calc(100vh - 48px); }
.sidebar { width: 220px; background: #fff; border-right: 1px solid #e0e0e0; padding: 16px; }
.sidebar h3 { font-size: 11px; text-transform: uppercase; color: #888; margin-bottom: 12px; letter-spacing: 1px; }
.sidebar .proj-item { padding: 8px 10px; border-radius: 6px; font-size: 13px; cursor: pointer; margin-bottom: 4px; }
.sidebar .proj-item.active { background: #f0f0f0; font-weight: 600; }
.sidebar .proj-item .badge { float: right; font-size: 10px; background: #e0e0e0; border-radius: 10px; padding: 1px 6px; }
.sidebar .new-btn { margin-top: 16px; width: 100%; padding: 8px; background: #1a1a1a; color: #fff; border: none; border-radius: 6px; font-size: 13px; cursor: pointer; }
.content { flex: 1; padding: 24px; overflow-y: auto; }
.content h2 { font-size: 20px; margin-bottom: 4px; }
.content .sub { font-size: 13px; color: #888; margin-bottom: 24px; }
.stats { display: flex; gap: 16px; margin-bottom: 24px; }
.stat-card { background: #fff; border: 1px solid #e0e0e0; border-radius: 8px; padding: 16px 20px; flex: 1; }
.stat-card .label { font-size: 11px; color: #888; text-transform: uppercase; letter-spacing: 1px; }
.stat-card .value { font-size: 28px; font-weight: 700; margin-top: 4px; }
.section-title { font-size: 13px; font-weight: 600; text-transform: uppercase; color: #888; letter-spacing: 1px; margin-bottom: 12px; }
.run-table { background: #fff; border: 1px solid #e0e0e0; border-radius: 8px; overflow: hidden; }
.run-table table { width: 100%; border-collapse: collapse; font-size: 13px; }
.run-table th { background: #f8f8f8; padding: 10px 16px; text-align: left; font-weight: 600; color: #555; border-bottom: 1px solid #e0e0e0; }
.run-table td { padding: 10px 16px; border-bottom: 1px solid #f0f0f0; }
.run-table tr:last-child td { border-bottom: none; }
.badge-done { background: #d4edda; color: #155724; border-radius: 10px; padding: 2px 8px; font-size: 11px; }
.badge-running { background: #fff3cd; color: #856404; border-radius: 10px; padding: 2px 8px; font-size: 11px; }
.badge-failed { background: #f8d7da; color: #721c24; border-radius: 10px; padding: 2px 8px; font-size: 11px; }
.actions { display: flex; gap: 8px; margin-bottom: 20px; }
.btn { padding: 8px 16px; border-radius: 6px; font-size: 13px; cursor: pointer; border: 1px solid #ccc; background: #fff; }
.btn-primary { background: #1a1a1a; color: #fff; border-color: #1a1a1a; }
</style>
</head>
<body>
<div class="topbar">
  <div class="logo">KROMA</div>
  <div class="nav">
    <span class="active">Projects</span>
    <span>Assets</span>
    <span>Settings</span>
  </div>
</div>
<div class="main">
  <div class="sidebar">
    <h3>Projects</h3>
    <div class="proj-item active">eugenia_prod <span class="badge">12</span></div>
    <div class="proj-item">noir_city <span class="badge">5</span></div>
    <div class="proj-item">demo</div>
    <button class="new-btn">+ New Project</button>
  </div>
  <div class="content">
    <h2>eugenia_prod</h2>
    <div class="sub">Local storage · Last run 2h ago</div>
    <div class="stats">
      <div class="stat-card"><div class="label">Total Runs</div><div class="value">12</div></div>
      <div class="stat-card"><div class="label">Assets</div><div class="value">84</div></div>
      <div class="stat-card"><div class="label">Cost (USD)</div><div class="value">$4.20</div></div>
      <div class="stat-card"><div class="label">QA Pass Rate</div><div class="value">91%</div></div>
    </div>
    <div class="actions">
      <button class="btn btn-primary" data-element-id="btn-new-run">+ New Run</button>
      <button class="btn" data-element-id="btn-open-chat">💬 Copilot</button>
      <button class="btn" data-element-id="btn-export">Export</button>
    </div>
    <div class="section-title">Recent Runs</div>
    <div class="run-table">
      <table>
        <thead><tr><th>Run ID</th><th>Stage</th><th>Jobs</th><th>Status</th><th>Started</th></tr></thead>
        <tbody>
          <tr><td>run_2026-02-21...</td><td>style</td><td>6/6</td><td><span class="badge-done">done</span></td><td>2h ago</td></tr>
          <tr><td>run_2026-02-20...</td><td>time</td><td>6/6</td><td><span class="badge-done">done</span></td><td>1d ago</td></tr>
          <tr><td>run_2026-02-19...</td><td>weather</td><td>4/6</td><td><span class="badge-failed">partial</span></td><td>2d ago</td></tr>
          <tr><td>run_2026-02-18...</td><td>style</td><td>6/6</td><td><span class="badge-running">running</span></td><td>3d ago</td></tr>
        </tbody>
      </table>
    </div>
  </div>
</div>
</body>
</html>
```

#### Chat / Copilot View Wireframe

```wireframe
<!DOCTYPE html>
<html>
<head>
<style>
* { box-sizing: border-box; margin: 0; padding: 0; font-family: sans-serif; }
body { background: #f5f5f5; color: #222; height: 100vh; display: flex; flex-direction: column; }
.topbar { background: #1a1a1a; color: #fff; padding: 12px 24px; display: flex; align-items: center; gap: 16px; font-size: 14px; }
.topbar .logo { font-weight: 700; font-size: 16px; }
.topbar .breadcrumb { color: #aaa; font-size: 13px; }
.chat-layout { display: flex; flex: 1; overflow: hidden; }
.sessions-panel { width: 220px; background: #fff; border-right: 1px solid #e0e0e0; padding: 16px; overflow-y: auto; }
.sessions-panel h3 { font-size: 11px; text-transform: uppercase; color: #888; margin-bottom: 12px; letter-spacing: 1px; }
.session-item { padding: 8px 10px; border-radius: 6px; font-size: 13px; cursor: pointer; margin-bottom: 4px; }
.session-item.active { background: #f0f0f0; font-weight: 600; }
.session-item .date { font-size: 11px; color: #aaa; }
.new-session-btn { width: 100%; padding: 8px; background: #1a1a1a; color: #fff; border: none; border-radius: 6px; font-size: 13px; cursor: pointer; margin-bottom: 16px; }
.chat-main { flex: 1; display: flex; flex-direction: column; }
.messages { flex: 1; overflow-y: auto; padding: 24px; display: flex; flex-direction: column; gap: 16px; }
.msg { max-width: 70%; }
.msg.user { align-self: flex-end; }
.msg.assistant { align-self: flex-start; }
.msg .bubble { padding: 10px 14px; border-radius: 12px; font-size: 14px; line-height: 1.5; }
.msg.user .bubble { background: #1a1a1a; color: #fff; border-bottom-right-radius: 4px; }
.msg.assistant .bubble { background: #fff; border: 1px solid #e0e0e0; border-bottom-left-radius: 4px; }
.msg .meta { font-size: 11px; color: #aaa; margin-top: 4px; }
.msg.user .meta { text-align: right; }
.instruction-card { background: #fff8e1; border: 1px solid #ffe082; border-radius: 8px; padding: 12px 14px; font-size: 13px; margin-top: 8px; }
.instruction-card .label { font-size: 11px; text-transform: uppercase; color: #888; margin-bottom: 4px; }
.instruction-card .actions { display: flex; gap: 8px; margin-top: 10px; }
.instruction-card .btn { padding: 5px 12px; border-radius: 5px; font-size: 12px; cursor: pointer; border: 1px solid #ccc; background: #fff; }
.instruction-card .btn-confirm { background: #1a1a1a; color: #fff; border-color: #1a1a1a; }
.input-bar { padding: 16px 24px; background: #fff; border-top: 1px solid #e0e0e0; display: flex; gap: 10px; align-items: flex-end; }
.input-bar textarea { flex: 1; padding: 10px 14px; border: 1px solid #ccc; border-radius: 8px; font-size: 14px; resize: none; height: 44px; }
.input-bar .mic-btn { padding: 10px 14px; background: #f0f0f0; border: 1px solid #ccc; border-radius: 8px; cursor: pointer; font-size: 16px; }
.input-bar .send-btn { padding: 10px 20px; background: #1a1a1a; color: #fff; border: none; border-radius: 8px; font-size: 14px; cursor: pointer; }
</style>
</head>
<body>
<div class="topbar">
  <div class="logo">KROMA</div>
  <div class="breadcrumb">eugenia_prod › Copilot</div>
</div>
<div class="chat-layout">
  <div class="sessions-panel">
    <button class="new-session-btn" data-element-id="btn-new-session">+ New Session</button>
    <h3>Sessions</h3>
    <div class="session-item active">Scene planning<div class="date">Today</div></div>
    <div class="session-item">Style review<div class="date">Yesterday</div></div>
    <div class="session-item">Weather pass<div class="date">Feb 19</div></div>
  </div>
  <div class="chat-main">
    <div class="messages">
      <div class="msg user">
        <div class="bubble">Run a style lock pass on all 6 scenes with 3 candidates each.</div>
        <div class="meta">You · 14:32</div>
      </div>
      <div class="msg assistant">
        <div class="bubble">
          I'll queue a pipeline run for stage=style with candidates=3 across all 6 scenes. This will cost approximately $1.80. Shall I proceed?
          <div class="instruction-card">
            <div class="label">Agent Instruction · pipeline_run</div>
            <strong>Stage:</strong> style · <strong>Candidates:</strong> 3 · <strong>Scenes:</strong> 6<br>
            <strong>Status:</strong> Awaiting confirmation
            <div class="actions">
              <button class="btn btn-confirm" data-element-id="btn-confirm-instruction">Confirm</button>
              <button class="btn" data-element-id="btn-cancel-instruction">Cancel</button>
            </div>
          </div>
        </div>
        <div class="meta">Kroma Copilot · 14:32</div>
      </div>
    </div>
    <div class="input-bar">
      <textarea placeholder="Ask Kroma to run a pipeline, check quality, manage assets..." data-element-id="chat-input"></textarea>
      <button class="mic-btn" data-element-id="btn-voice">🎤</button>
      <button class="send-btn" data-element-id="btn-send">Send</button>
    </div>
  </div>
</div>
</body>
</html>
```

---

### Phase 2 — Authentication

Currently the system runs in single-user mode with a hardcoded `local` user. Auth is a Phase 2 concern.

#### Required Work

1. Add `api_tokens` table (hashed tokens, scoped to user + project)
2. Add `Authorization: Bearer <token>` check to all API endpoints
3. Add `POST /auth/token` endpoint (generate token for local user)
4. Add token rotation and expiry
5. Keep single-user bootstrap path (`--with-default-user`) working without auth for local dev

---

### Phase 2 — Voice Pipeline (STT/TTS)

The DB schema and API endpoints for voice are already implemented. The actual STT/TTS processing is not yet wired.

#### Required Work

1. Implement `POST /api/projects/:slug/voice/stt` — accept audio file, call STT provider (OpenAI Whisper or local), return transcript
2. Implement `POST /api/projects/:slug/voice/tts` — accept text, call TTS provider (OpenAI TTS), return audio asset
3. Wire voice assets to `assets` table with `kind = 'voice'`
4. Link voice requests to chat messages (`voice_asset_id` on `chat_messages`)
5. Support streaming TTS for low-latency playback

---

### Phase 2 — Advanced Scoring & Consistency Metrics

1. Add model-based scoring for candidate ranking (beyond chroma/grayscale)
2. Add character consistency scoring (face/pose similarity across scenes)
3. Add style drift detection (compare output style to reference set)
4. Expose scoring results via `quality_reports` table

---

## 11. Security & Secret Handling

| Rule | Implementation |
|---|---|
| API keys never stored in plaintext | Fernet symmetric encryption in `project_api_secrets` |
| Master key from env or file | `IAT_MASTER_KEY` env var or `IAT_MASTER_KEY_FILE` path |
| Secrets masked in API responses | `list-project-secrets` returns masked values only |
| Run logs never print secret values | Enforced in `backend.py` ingest path |
| S3 sync operations are auditable | `audit_events` table (target schema) |

---

## 12. MVP Delivery Checklist

### Phase 1 (Current)

- [ ] DB schema migration to target spec (12 tables to add/rename)
- [ ] `quality_reports` table + ingest from run logs
- [ ] `cost_events` table + ingest from run logs
- [ ] Missing REST endpoints (runs, assets, quality, cost, characters)
- [ ] OpenAPI spec updated to match all implemented endpoints
- [ ] Contract smoke test coverage for all endpoints

### Phase 2

- [ ] Frontend GUI (React/Svelte) — project dashboard, run viewer, chat, asset browser
- [ ] Authentication (API tokens, single-user bootstrap)
- [ ] Voice STT/TTS pipeline (OpenAI Whisper + TTS)
- [ ] Advanced scoring models for candidate ranking
- [ ] Character consistency metrics
- [ ] PostgreSQL migration path (from SQLite)

---

## 13. Key Design Principles

1. **Prompts are data** — no hardcoded project text in scripts; all prompts live in `settings/manifest.json`
2. **Dry-run first** — every new prompt pack must be tested in dry mode before spending
3. **Project isolation** — all data, files, and DB records are scoped to a project slug; no cross-project references
4. **Reproducibility** — every run writes a complete JSON log; runs can be replayed
5. **Additive migrations only** — DB schema changes must never destroy existing data
6. **Backend-first** — API is the source of truth; GUI is a consumer, not the owner
