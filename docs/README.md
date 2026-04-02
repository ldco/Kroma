# Kroma Documentation Index

**Last Updated:** 2026-03-31
**Status:** Reorganized - Canonical structure established

---

## Documentation Structure

This documentation set is organized into clear sections based on purpose and audience. Each document is classified as **Canonical** (source of truth), **Reference** (supporting information), or **Archived** (historical context).

### 📁 Folder Structure

```
docs/
├── architecture/          # System design and technical decisions
├── api/                   # API contracts and integration guides
├── product/               # Product specs, roadmaps, user journeys
├── development/           # Developer guides and implementation plans
├── operations/            # Deployment, migration, release notes
├── locales/               # Translated documentation
└── archive/               # Historical reports and internal notes
    ├── status/            # Superseded status reports
    └── internal/          # Session notes, phase diaries
```

---

## Local Development Quick Start

**Two-Service Architecture:** Kroma consists of two services that must both be running for full-stack development:

| Service | Port | Description |
|---------|------|-------------|
| Backend | `127.0.0.1:8788` | Rust API server with SQLite persistence |
| Frontend | `localhost:3000` | Nuxt.js application |

### Startup Contract

**Prerequisites:**
1. Rust toolchain installed (`curl https://sh.rustup.rs -sSf | sh`)
2. Node.js >=20 installed
3. Frontend dependencies installed: `npm run frontend:install`

**Start both services:**
```bash
./start-dev.sh
```

**Verify both services are healthy:**
```bash
# Backend health
curl -s http://127.0.0.1:8788/health

# Frontend health (should return HTML)
curl -s http://localhost:3000/app
```

**For detailed setup instructions, see:** [README.md](../README.md#quick-start)

**Frontend prerequisites:** See [front-end-puppet-master/README.md](../front-end-puppet-master/README.md) for frontend-specific setup requirements.

---

## Canonical Documents (Source of Truth)

These documents are the authoritative source for their respective domains. **Always reference these first.**

| Document | Location | Purpose |
|----------|----------|---------|
| **Product Spec & Roadmap** | `product/Kroma_—_Project_Spec_(Current_State_&_Roadmap).md` | Complete product vision, features, and roadmap |
| **User Journey Map** | `product/USER_FLOW_JOURNEY_MAP.md` | Canonical user journey steps (J00-J08, U01, Rxx) |
| **Roadmap (Progress)** | `product/ROADMAP.md` | Current phase status and execution tracker |
| **Backend Contract** | `../openapi/backend-api.openapi.yaml` | **API source of truth** - all endpoints defined here |
| **Backend Contract Freeze** | `architecture/BACKEND_CONTRACT_FREEZE.md` | Frozen API contract for frontend integration |
| **Workflow** | `development/WORKFLOW.md` | Implementation rules and journey-step traceability |
| **Tech Spec** | `development/TECH_SPEC.md` | Technical architecture and decisions |
| **Frontend Plan** | `development/KROMA_TAURI_FRONTEND_PLAN.md` | Frontend implementation plan and phase status |
| **Metrics** | `operations/metrics.md` | Route counts, test totals, project metrics |

---

## Document Classifications

### Canonical (★)
- **Single source of truth** for its domain
- Must be updated when related changes are made
- Referenced by other documents

### Reference (ℹ️)
- Supporting information and context
- Historical decisions and rationale
- Implementation guides

### Archived (📦)
- Superseded by newer documents
- Historical context only
- Not maintained, kept for reference

---

## API Source of Truth

**All API endpoints are defined in:** `../openapi/backend-api.openapi.yaml`

**Contract Update Rule:** OpenAPI spec changes must be made in the same patch as:
- Backend route implementation changes
- Frontend API client updates
- Contract test modifications

**Deprecated Patterns:**
- ❌ Global endpoints (e.g., `/api/provider-accounts`)
- ✅ Project-scoped endpoints (e.g., `/api/projects/{slug}/provider-accounts`)

---

## Command Validation Checklist

Before merging documentation changes with command examples, verify:

- [ ] Commands exist in root `package.json` scripts
- [ ] Rust CLI commands work: `cargo run -- <command>`
- [ ] No references to removed script paths (`scripts/*.py`, `scripts/*.mjs`)
- [ ] Default values match current `.env.example`

**Current valid commands:**
```bash
# Startup (both services)
./start-dev.sh                # Start backend + frontend with health checks

# Backend
npm run backend:rust          # Start Rust API server
npm run backend:worker        # Start agent worker
npm run backend:init          # Initialize database
npm run backend:user:local    # Create local admin user

# Frontend
npm run frontend:install      # Install frontend dependencies
npm run frontend:dev          # Start frontend dev server

# Rust CLI
cargo run -- db:init
cargo run -- db:ensure-user
cargo run -- tools:install all
cargo run -- generate-one --project-slug <slug> --prompt "..."
cargo run -- upscale --project-slug <slug>
cargo run -- bgremove --project-slug <slug>
cargo run -- color --project-slug <slug>
cargo run -- qa --project-slug <slug>
cargo run -- agent-worker

# Frontend (front-end-puppet-master/)
npm run dev                   # Development server
npm run build                 # Production build
npm run lint:css-tokens       # CSS token validation
```

---

## Archival Policy

### Temporary Documents (Move to Archive After)
- **Phase diaries** (`PHASE_*_COMPLETE.md`): Move to `archive/internal/` after next phase starts
- **Session handoff notes** (`NEXT_CHAT_HANDOFF.md`): Move to `archive/internal/` after session
- **Status reports** (`*_STATUS.md`, `*_COMPLETE.md`): Move to `archive/status/` when superseded
- **Comment implementation reports**: Move to `archive/status/` after all comments addressed

### Permanent Documents (Stay in Root Sections)
- Product specs and roadmaps
- Architecture decisions
- API contracts
- Developer guides
- Release notes

---

## Metrics Source

For route counts, test totals, and project metrics, see: **`operations/metrics.md`**

**Do not hardcode metrics in other documents** - reference this file instead.

---

## Quick Navigation

### For New Contributors
1. Start with `product/Kroma_—_Project_Spec_(Current_State_&_Roadmap).md`
2. Read `development/WORKFLOW.md` for implementation rules
3. Review `product/USER_FLOW_JOURNEY_MAP.md` for user journey context

### For Frontend Developers
1. Check `development/KROMA_TAURI_FRONTEND_PLAN.md` for phase status
2. Review `../openapi/backend-api.openapi.yaml` for API contracts
3. See `architecture/BACKEND_CONTRACT_FREEZE.md` for integration guidelines

### For Backend Developers
1. Review `architecture/BACKEND_CONTRACT_FREEZE.md` for contract rules
2. Check `operations/metrics.md` for current route/test counts
3. See `development/WORKFLOW.md` for journey-step traceability

### For Release Planning
1. Check `product/ROADMAP.md` for current phase status
2. Review `operations/RELEASE_NOTES_v0.2.0.md` for last release
3. See `operations/metrics.md` for completion metrics

---

## Maintenance Rules

1. **Single Source of Truth:** Each domain has exactly one canonical document
2. **Archive Superseded Docs:** Move old status reports to `archive/status/`
3. **Update Metrics Centrally:** All numbers go in `operations/metrics.md`
4. **Validate Commands:** Command examples must work against current codebase
5. **Link, Don't Duplicate:** Reference canonical docs instead of copying content

---

*Last reorganized: 2026-03-28*
