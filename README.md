<div align="center"><img src="logo.png" alt="Project Logo" width="500"/></div>

<div align="center">

[![License](https://img.shields.io/badge/license-GPLv3-green)](./LICENSE)
[![Version](https://img.shields.io/badge/version-0.1.0-blue)](https://github.com/ldco/Kroma)
[![Build Status](https://img.shields.io/badge/build-%5BINSERT_STATUS%5D-lightgrey)](https://example.com/build-status)
[![Language](https://img.shields.io/badge/language-Rust%20%7C%20Node.js%20%7C%20Python-informational)](https://github.com/ldco/Kroma)

</div>

# Kroma

Prompt-driven image workflow tooling with a contract-first backend API, project-scoped storage, and reproducible run logs.

## Introduction

Kroma solves a common production pain: image generation/editing pipelines often become hard to reproduce, hard to audit, and hard to scale across multiple projects.

This repository is for developers who need:
- a local-first backend API for project/runs/assets metadata
- deterministic schema + route contract checks
- CLI-driven image workflows with optional post-processing stages

Why it exists:
- keep project data isolated
- keep API contract and implementation in sync
- make local development fast and predictable

## Features

- Contract-first backend (`openapi/backend-api.openapi.yaml`) with parity tests.
- Rust API server (`src-tauri`) with SQLite persistence.
- CLI workflows for dry run, paid run, upscale, color correction, background removal, and QA.
- Project-scoped storage and export/sync primitives.
- Integration tests across API domains (`src-tauri/tests/*`).

## Architecture Decision (Desktop-First Persistence)

Current product architecture is explicitly desktop-first:

1. Metadata database: local SQLite per user/app install.
2. Image/blob storage: local filesystem (project-scoped directories).
3. Cloud storage: optional S3 sync as an add-on capability (for backup/sync/team workflows), not a required runtime dependency.
4. PostgreSQL: deferred until a hosted multi-user deployment mode is introduced.

This keeps local UX fast and zero-ops while preserving a clean upgrade path for paid cloud features.

## Current Backend State (2026-02-22)

- Primary backend is Rust (`src-tauri`), started with `npm run backend:rust` on `127.0.0.1:8788`.
- Rust API contract currently mounts 68 routes and is covered by contract + endpoint tests.
- Bootstrap prompt exchange is implemented:
  - `GET /api/projects/{slug}/bootstrap-prompt`
  - `POST /api/projects/{slug}/bootstrap-import` (`merge`, `replace`, `dry_run`)

### Why `scripts/` still exists

`scripts/` is still required because migration is partial, not complete:

1. The image generation/post-process pipeline is still script-driven (`scripts/image-lab.mjs`).
2. Local tool wrappers and setup flows still live in Python/Node scripts.
3. Some migration paths (`backend.py`, worker scripts) are retained while parity migration continues.

So: backend data/API is largely migrated to Rust, but runtime pipeline orchestration is not fully migrated yet.

## 🚀 Quick Start

This is the recommended golden path for first-time setup: start the Rust backend and verify it responds to real API requests.

### Prerequisites

- Git
- Node.js `>=20` (from `package.json`)
- Python 3 (`python3` available on PATH)
- Rust toolchain (`cargo`, `rustc`)
- `curl`

### Installation

```bash
git clone https://github.com/ldco/Kroma.git
cd Kroma/app
npm install
```

### Environment Setup

Copy the template env file:

```bash
cp .env.example .env
```

Default `.env.example`:

```dotenv
OPENAI_API_KEY=[YOUR_API_KEY]
OPENAI_IMAGE_MODEL=gpt-image-1
OPENAI_IMAGE_SIZE=768x1152
OPENAI_IMAGE_QUALITY=medium
PHOTOROOM_API_KEY=[YOUR_PHOTOROOM_API_KEY]
REMOVE_BG_API_KEY=[YOUR_REMOVE_BG_API_KEY]
```

`OPENAI_API_KEY` is only required for paid image generation (`run` mode). You can start and test the backend API without it.

### Run Locally

```bash
npm run backend:rust
```

This starts the Rust API on `127.0.0.1:8788` by default (`KROMA_BACKEND_BIND` overrides it).

### ✅ Success Check

In a second terminal:

```bash
curl -s http://127.0.0.1:8788/health
```

You should see JSON containing:
- `"ok": true`
- `"status": "ok"`
- `"service": "kroma-backend-core"`

Now verify a full API round-trip:

```bash
curl -s -X POST http://127.0.0.1:8788/api/projects \
  -H 'Content-Type: application/json' \
  -d '{"name":"DX Demo","slug":"dx_demo"}'

curl -s http://127.0.0.1:8788/api/projects
```

The second command should return a project list containing `dx_demo`.

## Usage

### API Example (curl)

```bash
curl -s -X POST http://127.0.0.1:8788/api/projects \
  -H 'Content-Type: application/json' \
  -d '{"name":"My Project","slug":"my_project"}'

curl -s http://127.0.0.1:8788/api/projects/my_project
```

### CLI Example (Dry Image Workflow)

```bash
npm run lab -- dry \
  --project my_project \
  --project-root [ABS_PROJECT_ROOT] \
  --input [PATH_TO_SCENES]
```

### CLI Example (Paid Run)

```bash
npm run lab -- run \
  --project my_project \
  --project-root [ABS_PROJECT_ROOT] \
  --input [PATH_TO_SCENES] \
  --confirm-spend
```

### JavaScript API Call Example

```js
const response = await fetch("http://127.0.0.1:8788/api/projects", {
  method: "POST",
  headers: { "Content-Type": "application/json" },
  body: JSON.stringify({ name: "SDK Demo", slug: "sdk_demo" })
});

const data = await response.json();
console.log(data);
```

## Configuration

| Key | Description | Default | Required |
| --- | --- | --- | --- |
| `OPENAI_API_KEY` | OpenAI key for paid image generation (`run` mode) | None | Yes (for `run` mode) |
| `OPENAI_IMAGE_MODEL` | OpenAI image model | `gpt-image-1` | No |
| `OPENAI_IMAGE_SIZE` | Default image size | `768x1152` | No |
| `OPENAI_IMAGE_QUALITY` | Default image quality | `medium` | No |
| `PHOTOROOM_API_KEY` | Optional PhotoRoom fallback for BG removal | None | No |
| `REMOVE_BG_API_KEY` | Optional remove.bg fallback for BG removal | None | No |
| `KROMA_BACKEND_BIND` | Rust API bind address | `127.0.0.1:8788` | No |
| `KROMA_BACKEND_DB` | Rust API SQLite database path | `var/backend/app.db` | No |
| `KROMA_BACKEND_DB_URL` | Reserved for future hosted DB mode (`postgres://...`) | None | No |
| `IAT_AGENT_API_URL` | Optional agent dispatch target URL | None | No |
| `IAT_AGENT_API_TOKEN` | Optional agent dispatch bearer token | None | No |

For desktop/local mode, keep `KROMA_BACKEND_DB_URL` unset and use SQLite (`KROMA_BACKEND_DB`).

### Pipeline Runtime Settings (Layered, Rust-Owned)

Pipeline runtime settings are now resolved in Rust (not ad hoc in scripts) with layered precedence:

1. Request/runtime overrides
2. Project settings file
3. App settings file
4. Rust built-in defaults

Current file formats:
- App settings: `TOML` (preferred) at `config/pipeline.settings.toml`
- App settings fallback (legacy): `JSON` at `config/pipeline.settings.json`
- Project settings: `JSON` at `<project_root>/.kroma/pipeline.settings.json`

What this currently controls on the Rust runtime path:
- default `manifest_path`
- default `postprocess_config_path`
- postprocess planning defaults (`upscale`, `upscale_backend`, `color`, `color_profile`, `bg_remove`, `bg_remove_backends`, `bg_refine_openai`, `bg_refine_openai_required`)

Start from:
- `config/pipeline.settings.toml.example`
- `config/pipeline.manifest.json.example`
- `config/postprocess.json.example`

Typical setup:
1. Create `config/pipeline.settings.toml` from the example.
2. Create `config/pipeline.manifest.json` and `config/postprocess.json` from the example files (or point the TOML file at your own paths).

Validate the layered config stack (Rust-owned validation path):

```bash
cd src-tauri
cargo run -- validate-pipeline-config --project-root ../var/projects/my_project
```

Validate only specific files (without settings files) by overriding manifest/postprocess paths:

```bash
cd src-tauri
cargo run -- validate-pipeline-config \
  --manifest ../config/pipeline.manifest.json \
  --postprocess-config ../config/postprocess.json
```

This validates:
- app settings (`config/pipeline.settings.toml` or JSON fallback)
- project settings (`<project_root>/.kroma/pipeline.settings.json`)
- referenced pipeline manifest JSON (if configured)
- referenced postprocess config JSON (if configured)

Project-scoped API validation endpoint (backend resolves `project_root` from stored project storage when omitted):

```bash
POST /api/projects/{slug}/runs/validate-config
```

Example app settings (`TOML`):

```toml
[pipeline]
manifest_path = "config/pipeline.manifest.json"
postprocess_config_path = "config/postprocess.json"

[pipeline.postprocess]
upscale = true
upscale_backend = "ncnn"
color = true
color_profile = "studio"
bg_remove = true
bg_remove_backends = ["rembg"]
bg_refine_openai = false
bg_refine_openai_required = false
```

Example project overrides (`JSON`):

```json
{
  "pipeline": {
    "postprocess": {
      "color_profile": "project-profile",
      "bg_remove_backends": ["photoroom"]
    }
  }
}
```

## Project Structure

```text
app/
├─ config/
│  ├─ pipeline.settings.toml.example
│  ├─ pipeline.manifest.json.example
│  └─ postprocess.json.example
├─ package.json
├─ .env.example
├─ openapi/
│  └─ backend-api.openapi.yaml
├─ scripts/
│  ├─ image-lab.mjs
│  ├─ backend.py
│  ├─ contract_smoke.py
│  └─ setup_tools.py
├─ src-tauri/
│  ├─ src/
│  │  ├─ api/
│  │  └─ db/
│  └─ tests/
├─ docs/
└─ var/
```

## Contributing

### Run Tests

```bash
npm run backend:rust:test
```

If you want API-contract smoke validation against a running server:

```bash
npm run backend:contract:smoke -- --base-url http://127.0.0.1:8788 --project-slug dx_smoke
```

### Commit / PR Expectations

- Keep changes focused and scoped.
- Update `openapi/backend-api.openapi.yaml` when endpoint contracts change.
- Ensure tests pass before opening a PR.
- Include a short verification note in your PR description.

## Troubleshooting / FAQ

### 1) Dependency install issue (`cargo` or `rustc` not found)

Install Rust toolchain, then retry:

```bash
curl https://sh.rustup.rs -sSf | sh
source "$HOME/.cargo/env"
rustc --version
cargo --version
```

### 2) Env/config issue (`Missing OPENAI_API_KEY` during `run`)

`run` mode requires an API key. Add it to `.env`:

```bash
sed -n '1,20p' .env
```

Ensure `OPENAI_API_KEY=[YOUR_API_KEY]` is present and non-empty.

### 3) Runtime/startup issue (server fails to bind)

If port `8788` is already in use, run on another port:

```bash
KROMA_BACKEND_BIND=127.0.0.1:8790 npm run backend:rust
```

Then verify:

```bash
curl -s http://127.0.0.1:8790/health
```

## License & Contact

- License: `GPLv3` (see `LICENSE`)
- Contact: open an issue at `https://github.com/ldco/Kroma/issues`
