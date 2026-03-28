# Kroma Project Metrics

**Last Updated:** 2026-03-28  
**Status:** Canonical source for all project metrics

---

## Backend Metrics

### Route Counts

| Metric | Count | Source |
|--------|-------|--------|
| **Total API Routes** | 74 | `src-tauri/src/api/routes.rs` |
| Contract Routes | 74 | `CONTRACT_ROUTES` constant |
| System Routes | 1 | `/health` |
| Auth Routes | 3 | `/auth/*` |
| Project Routes | 74 total (all project-scoped) | All routes use `/api/projects/{slug}/*` pattern |

### Route Breakdown by Domain

| Domain | Route Count | Example |
|--------|-------------|---------|
| System | 1 | `GET /health` |
| Auth | 3 | `POST /auth/token` |
| Projects | 4 | `GET/POST /api/projects` |
| Storage | 3 | `GET/PUT /api/projects/{slug}/storage/*` |
| Runs | 4 | `GET/POST /api/projects/{slug}/runs/*` |
| Assets | 6 | `GET/POST /api/projects/{slug}/assets/*` |
| Analytics | 2 | `GET /api/projects/{slug}/quality-reports` |
| Exports | 2 | `GET /api/projects/{slug}/exports` |
| Prompt Templates | 4 | CRUD for templates |
| Provider Accounts | 5 | CRUD for providers |
| Style Guides | 4 | CRUD for style guides |
| Characters | 4 | CRUD for characters |
| Reference Sets | 9 | CRUD for sets + items |
| Chat | 5 | Chat sessions + messages |
| Agent Instructions | 5 | Instructions + events |
| Secrets | 5 | Secrets management |

**Note:** All routes except `/health` and `/auth/*` require Bearer token authentication.

---

## Test Metrics

### Test Counts

| Test Suite | Count | Location |
|------------|-------|----------|
| **Integration Tests** | 60+ | `src-tauri/tests/` |
| Contract Parity Tests | 20 files | `src-tauri/tests/*_endpoints.rs` |
| Error Taxonomy Tests | 16 files | All endpoint groups |
| Unit Tests | 109 | `cargo test --lib pipeline::` |
| **Total Tests** | 185+ | All suites combined |

### Test Coverage by Area

| Area | Test Files | Status |
|------|------------|--------|
| Projects | 2 | ✅ Passing |
| Pipeline Trigger | 2 | ✅ Passing |
| Reference Sets | 2 | ✅ Passing |
| Storage | 2 | ✅ Passing |
| Provider Accounts | 2 | ✅ Passing |
| Style Guides | 2 | ✅ Passing |
| Prompt Templates | 2 | ✅ Passing |
| Characters | 2 | ✅ Passing |
| Asset Links | 2 | ✅ Passing |
| Secrets | 3 | ✅ Passing |
| Runs/Assets | 2 | ✅ Passing |
| Exports | 2 | ✅ Passing |
| Analytics | 2 | ✅ Passing |
| Chat | 2 | ✅ Passing |
| Agent Instructions | 2 | ✅ Passing |
| Auth | 5 | ✅ Passing |
| Error Taxonomy | 16 | ✅ Passing |

---

## Frontend Metrics

### Puppet Master Frontend (`front-end-puppet-master/`)

| Metric | Count |
|--------|-------|
| Total Pages | 50+ (includes admin pages from PM framework) |
| Kroma App Pages | 7 (`/app/*` routes) |
| Components | 90+ (atoms, molecules, organisms) |
| Composables | 20+ |
| Pinia Stores | 4 (admin, projects, providers, runs, kromaAuth) |
| CSS Files | 110+ |
| TypeScript Files | 200+ |

### Kroma-Specific Pages

| Page | Route | Status |
|------|-------|--------|
| Dashboard | `/app` | ✅ Complete |
| Projects List | `/app/projects` | ✅ Complete |
| Project Providers | `/app/projects/[slug]/providers` | ✅ Complete |
| Runs List | `/app/projects/[slug]/runs` | ✅ Complete |
| Run Detail | `/app/projects/[slug]/runs/[id]` | ✅ Complete |
| Quick Tools | `/app/quick-tools` | ⏳ Placeholder |
| Settings | `/app/settings` | ⏳ Placeholder |

---

## Release Metrics

### v0.2.0 — Step B Complete (2026-03-08)

| Metric | Value |
|--------|-------|
| Backend Routes | 74 |
| Integration Tests | 60+ |
| Contract Tests | 20 files |
| Error Taxonomy Coverage | 100% |
| Python Dependencies | 0 (100% Rust) |
| Script Dependencies | 0 (all migrated) |

### Current Phase (Phase 2 - In Progress)

| Metric | Value |
|--------|-------|
| Frontend Framework | Puppet Master 2 (Nuxt 3) |
| Canonical Frontend | `front-end-puppet-master/` |
| Removed Frontends | `frontend-nuxt/` (archived) |
| API Integration | ✅ Complete (useKromaApi) |
| Auth Bootstrapping | ✅ Complete (kromaAuth store) |

---

## Migration Status

### Script → Rust Migration

| Script | Rust Replacement | Status |
|--------|-----------------|--------|
| `scripts/backend.py` | `src-tauri/src/api/` | ✅ Complete |
| `scripts/image-lab.mjs` | `cargo run -- <command>` | ✅ Complete |
| `scripts/agent_worker.py` | `cargo run -- agent-worker` | ✅ Complete |
| `scripts/agent_dispatch.py` | Rust pipeline runtime | ✅ Complete |

### Current Rust CLI Commands

```bash
cargo run -- db:init
cargo run -- db:ensure-user
cargo run -- tools:install all
cargo run -- generate-one --project-slug <slug> --prompt "..."
cargo run -- upscale --project-slug <slug>
cargo run -- bgremove --project-slug <slug>
cargo run -- color --project-slug <slug>
cargo run -- qa --project-slug <slug>
cargo run -- archive-bad --project-slug <slug>
cargo run -- agent-worker
cargo run -- secrets-rotation-status --project-slug <slug>
cargo run -- secrets-rotate --project-slug <slug>
```

---

## Data Model Metrics

### Database Tables (SQLite)

| Table | Purpose |
|-------|---------|
| `users` | User accounts |
| `api_tokens` | Bearer tokens |
| `audit_events` | Audit log |
| `projects` | Project metadata |
| `project_storage` | Storage configuration |
| `project_secrets` | Encrypted secrets |
| `runs` | Generation runs |
| `run_jobs` | Run job queue |
| `assets` | Asset registry |
| `asset_links` | Asset relationships |
| `quality_reports` | QA reports |
| `cost_events` | Cost tracking |
| `exports` | Export history |
| `prompt_templates` | Template library |
| `provider_accounts` | Provider configs |
| `style_guides` | Style constraints |
| `characters` | Character roster |
| `reference_sets` | Reference collections |
| `reference_items` | Reference items |
| `chat_sessions` | Chat history |
| `chat_messages` | Chat messages |
| `agent_instructions` | Instruction queue |
| `instruction_events` | Instruction events |

**Total Tables:** 23

---

## Performance Metrics

### Lighthouse Scores (front-end-puppet-master)

| Metric | Score | Target |
|--------|-------|--------|
| Performance | 95+ | ✅ |
| Accessibility | 95+ | ✅ |
| Best Practices | 95+ | ✅ |
| SEO | 90+ | ✅ |
| PWA | N/A | Desktop app |

### CSS Architecture

| Metric | Count | Status |
|--------|-------|--------|
| Total CSS Files | 110+ | ✅ |
| CSS Token Violations | Pre-existing PM issues | ⚠️ Not Kroma-specific |
| Inline Style Blocks | 0 | ✅ Complete |
| Global CSS Files | 100% | ✅ Complete |

---

## Notes

- **Route counts** are sourced from `src-tauri/src/api/routes.rs::CONTRACT_ROUTES`
- **Test counts** from `cargo test` output and `src-tauri/tests/` file count
- **Frontend metrics** from `front-end-puppet-master/` file structure
- **All metrics** should be updated when significant changes are made

---

*This is the canonical source for project metrics. Reference this document instead of hardcoding values in other docs.*
