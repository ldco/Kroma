# Kroma Tauri Frontend Plan

**Document Created:** 2026-03-23 18:30 (UTC)
**Last Updated:** 2026-03-24 (UTC)
**Status:** Phase 1 Complete - Phase 2 In Progress
**Project:** Kroma - Comic/Graphic-Novel Production Tool
**Frontend:** Puppet Master 2 (Nuxt 3 + Pure CSS) - **CANONICAL FRONTEND**
**Backend:** Rust (Axum + SQLite) - Step B Complete

---

## Architecture Decision (2026-03-24)

**Canonical Frontend:** `front-end-puppet-master/`

**Rationale:**
- Puppet Master 2 provides a cleaner Nuxt 3 + Pure CSS architecture
- Tauri 2 integration already built-in
- Lighter-weight and appropriate for a desktop tool
- Admin/RBAC/i18n complexity from other frameworks is unnecessary overhead for Kroma's use case

**Non-Canonical:** `frontend-nuxt/` has been archived and removed from the codebase.

---

## Executive Summary

### Project Goal

Build a **desktop-first comic/graphic-novel production UI** using Tauri, enabling artists to:

1. Create and manage project universes (continuity boundaries)
2. Configure AI provider accounts (OpenAI, Stability, etc.)
3. Trigger generation runs with prompts and references
4. Review and approve candidates
5. Manage assets with lineage tracking
6. Build continuity references (characters, style guides, reference sets)
7. Export production-ready packages

### Current Status

| Component | Status | Details |
|-----------|--------|---------|
| **Backend (Rust)** | ✅ Step B Complete | 74 API routes, contract-frozen, 60+ integration tests passing |
| **Frontend Framework** | ✅ Puppet Master 2 | Nuxt 3 + Pure CSS system - CANONICAL |
| **Phase 1: App Shell & Projects** | ✅ Complete | Layout, navigation, projects CRUD |
| **Phase 2: Provider Setup** | 🔄 In Progress | Provider account management (API endpoints fixed) |
| **Tauri Integration** | ⏳ Future | Desktop app wrapper (after web UI complete) |

**Metrics Source:** See `../operations/metrics.md` for current route counts, test totals, and project metrics.

### Recent Changes (2026-03-24)

**Backend Contract Alignment:**
- ✅ Provider store updated to use `/api/projects/{slug}/provider-accounts` endpoints
- ✅ Provider type updated to use `provider_code` instead of `id`
- ✅ Removed `testProvider` action (no backend endpoint)
- ✅ Run store cleaned up: removed `approveCandidate`, `retryRun`, `cancelRun`
- ✅ Projects store: `deleteProject` marked as not supported by backend
- ✅ `frontend-nuxt/` removed - consolidated on `front-end-puppet-master/`

### Architecture Decision

**Web-First, Tauri Later:**
1. ✅ Complete web UI (Phases 1-8)
2. ✅ Test with Rust backend
3. ⏳ Wrap in Tauri for desktop distribution

This approach allows:
- Faster iteration on UI/UX
- Easier debugging in browser dev tools
- Clean separation of concerns
- Same codebase for web and desktop

---

## Phase 1: Complete ✅

**Duration:** 2026-03-23 (4 hours)  
**Status:** Complete  

### What Was Delivered

#### Layout System
- ✅ `kroma.vue` layout with responsive navigation
- ✅ Vertical sidebar (desktop ≥640px)
- ✅ Bottom navigation (mobile <640px)
- ✅ Dark/light mode toggle
- ✅ User menu with logout

#### Pages
- ✅ `/app` - Dashboard with stats and recent projects
- ✅ `/app/projects` - Project list with CRUD modals
- ✅ `/app/providers` - Placeholder
- ✅ `/app/quick-tools` - Placeholder
- ✅ `/app/settings` - Placeholder

#### Components
- ✅ `ProjectCard.vue` (atom) - Project summary display

#### State Management
- ✅ `stores/projects.ts` - Pinia store
- ✅ `composables/useProjects.ts` - Composable

#### Type Definitions
- ✅ `types/kroma.ts` - 20+ domain types

#### CSS
- ✅ 6 new CSS files following Puppet Master conventions
- ✅ Responsive grid layouts
- ✅ Dark/light mode support

#### Translations
- ✅ `i18n/en.ts` - English translations

### Files Created (25 total)

```
front-end-puppet-master/
├── app/
│   ├── layouts/
│   │   └── kroma.vue
│   ├── pages/
│   │   └── app/
│   │       ├── index.vue
│   │       ├── projects/
│   │       │   └── index.vue
│   │       ├── providers.vue
│   │       ├── quick-tools.vue
│   │       └── settings.vue
│   ├── components/
│   │   └── atoms/
│   │       └── ProjectCard.vue
│   ├── stores/
│   │   └── projects.ts
│   ├── composables/
│   │   └── useProjects.ts
│   ├── types/
│   │   └── kroma.ts
│   └── assets/
│       └── css/
│           ├── layout/
│           │   ├── kroma-variables.css
│           │   ├── kroma-sidebar.css
│           │   ├── kroma-main.css
│           │   └── kroma-bottom-nav.css
│           └── ui/
│               ├── projects/
│               │   ├── project-card.css
│               │   ├── project-grid.css
│               │   └── index.css
│               └── content/
│                   └── kroma-dashboard.css
├── i18n/
│   └── en.ts
└── .env.example (updated)

docs/
├── FRONTEND_DEVELOPMENT_PLAN.md
└── PHASE_1_COMPLETE.md
```

### Metrics

| Metric | Count |
|--------|-------|
| New Pages | 5 |
| New Components | 1 |
| New Stores | 1 |
| New Composables | 1 |
| New CSS Files | 6 |
| New Types | 20+ |
| Lines of Code | ~2,500 |
| Time Spent | ~4 hours |

---

## Phase 2: Provider Setup (Next)

**Duration:** 2 days (16 hours)  
**Priority:** P0  
**Start Date:** 2026-03-27  
**Target Completion:** 2026-03-29  

### Goal

Users can configure AI provider accounts (OpenAI, Stability, Anthropic, etc.) with encrypted API key storage.

### Tasks

#### 2.1 Create Providers Page
- [ ] Replace placeholder `/app/providers.vue`
- [ ] Provider account list grid
- [ ] Add provider modal
- [ ] Edit/delete functionality

#### 2.2 Build Components

**Atoms:**
- [ ] `ProviderStatusIcon.vue` - Health status indicator
- [ ] `ApiKeyInput.vue` - Secure password-style input

**Molecules:**
- [ ] `ProviderAccountForm.vue` - CRUD form
- [ ] `ProviderCard.vue` - Provider display card

**Organisms:**
- [ ] `ProviderAccountManager.vue` - Full manager interface

#### 2.3 State Management
- [ ] `stores/providers.ts` - Pinia store
- [ ] `composables/useProviders.ts` - Composable

#### 2.4 CSS
- [ ] `ui/providers/provider-card.css`
- [ ] `ui/providers/provider-status.css`
- [ ] `ui/providers/index.css`

#### 2.5 API Integration
- [ ] Connect to `GET /api/provider-accounts`
- [ ] Connect to `POST /api/provider-accounts`
- [ ] Connect to `PUT /api/provider-accounts/{id}`
- [ ] Connect to `DELETE /api/provider-accounts/{id}`
- [ ] Handle encrypted API keys
- [ ] Display health check status

### Deliverables

- ✅ Provider account list
- ✅ Add/edit provider form (API key input with encryption)
- ✅ Provider health status display
- ✅ Delete provider (with confirmation)
- ✅ Provider selection available in run form (future)

---

## Phase 3: Run Workflow

**Duration:** 4 days (32 hours)  
**Priority:** P0  
**Start Date:** 2026-03-30  
**Target Completion:** 2026-04-03  

### Goal

Users can trigger generation runs and review/approve candidates.

### Key Features

- Run trigger form (prompt, negative prompt, settings)
- Run history with status
- Candidate viewer with comparison
- Approve/retry workflow
- Real-time status updates (polling → WebSocket later)

### Pages
- `/app/projects/[slug]/runs` - Run history
- `/app/projects/[slug]/runs/[id]` - Run review

### Components (10+)
- `RunTriggerForm.vue`
- `RunCandidateViewer.vue`
- `RunStatusBadge.vue`
- `CandidateCard.vue`
- `QualityScoreBadge.vue`
- `WorkflowStepIndicator.vue`
- `RunHistoryTable.vue`
- `RunReviewWorkflow.vue`

---

## Phase 4: Asset Management

**Duration:** 3 days (24 hours)  
**Priority:** P0  
**Start Date:** 2026-04-04  
**Target Completion:** 2026-04-07  

### Goal

Users can view, filter, upload, and manage project assets.

### Key Features

- Asset gallery grid
- Asset detail panel (metadata, lineage)
- Upload/replace asset
- Filter by run, character, style
- Asset selection (single, multi)

### Pages
- `/app/projects/[slug]/assets` - Asset gallery

### Components (5+)
- `AssetGallery.vue`
- `AssetThumbnail.vue`
- `AssetDetailPanel.vue`
- `AssetFilterBar.vue`
- `AssetManager.vue`

---

## Phase 5: Continuity Features

**Duration:** 4 days (32 hours)  
**Priority:** P1  
**Start Date:** 2026-04-08  
**Target Completion:** 2026-04-12  

### Goal

Users can manage characters, style guides, and reference sets.

### Features

- Character roster with reference images
- Style guide editor with constraints
- Reference set builder

### Pages
- `/app/projects/[slug]/characters`
- `/app/projects/[slug]/style-guides`
- `/app/projects/[slug]/references`

---

## Phase 6: Project Settings

**Duration:** 2 days (20 hours)  
**Priority:** P1  
**Start Date:** 2026-04-13  
**Target Completion:** 2026-04-15  

### Goal

Users can configure project storage, secrets, and bootstrap settings.

### Features

- Storage configuration (local, S3)
- Secrets management (encrypted)
- Bootstrap import/export
- Project defaults

### Pages
- `/app/projects/[slug]/settings`

---

## Phase 7: Quick Tools

**Duration:** 3 days (24 hours)  
**Priority:** P2  
**Start Date:** 2026-04-16  
**Target Completion:** 2026-04-19  

### Goal

Utility mode for quick operations without project context.

### Tools

- Background removal
- Image upscaling
- Color correction

### Pages
- `/app/quick-tools` (replace placeholder)

---

## Phase 8: Advanced Features

**Duration:** 3 days (24 hours)  
**Priority:** P2  
**Start Date:** 2026-04-20  
**Target Completion:** 2026-04-23  

### Features

- Prompt template library
- Export history and package download
- Chat with agent
- Quality reports and analytics

### Pages
- `/app/templates`
- `/app/exports`
- `/app/chat`

---

## Tauri Integration (Future)

**After Phases 1-8 Complete**

### Goal

Wrap the completed web UI in a Tauri desktop application.

### Tasks

1. **Tauri Setup**
   - [ ] Initialize Tauri in project
   - [ ] Configure `tauri.conf.json`
   - [ ] Set up app icons, window settings

2. **Backend Integration**
   - [ ] Rust backend runs as Tauri backend process
   - [ ] IPC communication (frontend ↔ backend)
   - [ ] File system access for local storage

3. **Native Features**
   - [ ] System tray integration
   - [ ] Native file dialogs
   - [ ] Auto-updates
   - [ ] Menu bar customization

4. **Build & Distribution**
   - [ ] Windows installer (.msi, .exe)
   - [ ] macOS app bundle (.dmg, .app)
   - [ ] Linux packages (.deb, .AppImage)

### Timeline

**Estimated:** 1-2 weeks after web UI complete

---

## API Reference

### Backend Endpoints

Full API documentation: `openapi/backend-api.openapi.yaml`

#### Projects
```
GET    /api/projects                      # List projects
POST   /api/projects                      # Create project
GET    /api/projects/{slug}               # Get project detail
PUT    /api/projects/{slug}               # Update project
DELETE /api/projects/{slug}               # Delete project
```

#### Providers
```
GET    /api/provider-accounts             # List providers
POST   /api/provider-accounts             # Create provider
PUT    /api/provider-accounts/{id}        # Update provider
DELETE /api/provider-accounts/{id}        # Delete provider
```

#### Runs
```
GET    /api/projects/{slug}/runs          # List runs
POST   /api/projects/{slug}/runs/trigger  # Trigger run
GET    /api/projects/{slug}/runs/{id}     # Get run detail
```

#### Assets
```
GET    /api/projects/{slug}/assets        # List assets
POST   /api/projects/{slug}/assets        # Upload asset
DELETE /api/projects/{slug}/assets/{id}   # Delete asset
```

---

## Development Workflow

### Setup

```bash
# 1. Backend (Terminal 1)
cd /run/media/ldco/3734114f-7123-41f5-8f63-7f43c94879eb/CURRENT_WORKING_DEV/Kroma/app
npm run backend:rust

# 2. Frontend (Terminal 2)
cd /run/media/ldco/3734114f-7123-41f5-8f63-7f43c94879eb/CURRENT_WORKING_DEV/Kroma/app/front-end-puppet-master
cp .env.example .env
npm run dev
```

### Access

- **Frontend:** http://localhost:3000/app
- **Backend API:** http://127.0.0.1:8788

### Git Workflow

```bash
# Feature branches
git checkout -b feature/phase-2-providers
git checkout -b feature/phase-3-runs

# Commit format
feat(providers): add provider account CRUD
fix(projects): resolve project card hover state
test(api): add integration tests for providers API
```

---

## Success Metrics

### MVP (Phases 1-4)

- [ ] User can create and manage projects
- [ ] User can configure AI provider accounts
- [ ] User can trigger generation runs
- [ ] User can review and approve candidates
- [ ] User can manage assets with filters
- [ ] All data comes from Rust backend API
- [ ] Zero Python/script dependencies
- [ ] User can complete journey J01 → J04

### Full Feature Set (Phases 1-8)

- [ ] All MVP features complete
- [ ] User can manage characters, style guides, references
- [ ] User can configure project settings
- [ ] User can use quick tools
- [ ] User can save/load templates
- [ ] User can export packages
- [ ] User can chat with agent
- [ ] User can view analytics

### Tauri Desktop

- [ ] Web UI wrapped in Tauri
- [ ] Native file dialogs
- [ ] System tray integration
- [ ] Auto-updates working
- [ ] Installers for Windows, macOS, Linux

---

## Timeline Summary

| Phase | Start | End | Duration | Status |
|-------|-------|-----|----------|--------|
| **Phase 1:** App Shell & Projects | 2026-03-23 | 2026-03-23 | 4 hours | ✅ Complete |
| **Phase 2:** Provider Setup | 2026-03-27 | 2026-03-29 | 2 days | ⏳ Next |
| **Phase 3:** Run Workflow | 2026-03-30 | 2026-04-03 | 4 days | ⏳ Planned |
| **Phase 4:** Asset Management | 2026-04-04 | 2026-04-07 | 3 days | ⏳ Planned |
| **Phase 5:** Continuity Features | 2026-04-08 | 2026-04-12 | 4 days | ⏳ Planned |
| **Phase 6:** Project Settings | 2026-04-13 | 2026-04-15 | 2 days | ⏳ Planned |
| **Phase 7:** Quick Tools | 2026-04-16 | 2026-04-19 | 3 days | ⏳ Planned |
| **Phase 8:** Advanced Features | 2026-04-20 | 2026-04-23 | 3 days | ⏳ Planned |
| **Tauri Integration** | TBD | TBD | 1-2 weeks | ⏳ Future |

### Total Estimates

| Metric | Value |
|--------|-------|
| **Total Pages** | 15 new pages |
| **Total Components** | 47 new components |
| **Total Composables** | 15 new composables |
| **Total Stores** | 4 new stores |
| **Total CSS Files** | 20+ new CSS files |
| **Total Development Time** | 267 hours (~6.7 weeks) |
| **MVP (Phases 1-4)** | 96 hours (~2.4 weeks) |

---

## Risks & Mitigations

### Technical Risks

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Backend API changes | High | Low | Contract freeze in place |
| CSS conflicts | Medium | Medium | Namespace prefixes for Kroma CSS |
| Performance with large galleries | Medium | Medium | Pagination, lazy loading |
| Real-time updates complex | Medium | Medium | Start with polling, upgrade later |

### Scope Risks

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Feature creep | High | High | Strict journey step adherence |
| UI perfectionism | Medium | High | MVP-first approach |
| Integration complexity | Medium | Medium | Build tests alongside features |

---

## Journey Step Mapping

| Journey Step | Frontend Pages | Phase |
|--------------|----------------|-------|
| `J01` Create/Select Project | `/app/projects`, `/app/projects/[slug]` | Phase 1 |
| `J02` Build Continuity References | `/characters`, `/style-guides`, `/references` | Phase 5 |
| `J03` Bootstrap Story Settings | `/settings` | Phase 6 |
| `J04` Lock Style Baseline | `/runs`, `/runs/[id]` | Phase 3 |
| `J05` Controlled Variation | `/runs/[id]` | Phase 3 |
| `J06` Character Identity | `/characters` | Phase 5 |
| `J07` Local Post-Process | `/quick-tools` | Phase 7 |
| `J08` Review, Curate, Export | `/exports`, `/assets` | Phase 4, 8 |

---

## Appendix: File Structure

```
front-end-puppet-master/
├── app/
│   ├── layouts/
│   │   ├── kroma.vue              # Kroma app layout
│   │   ├── admin.vue              # Admin layout (existing)
│   │   └── default.vue            # Default layout (existing)
│   ├── pages/
│   │   ├── app/
│   │   │   ├── index.vue          # Dashboard ✅
│   │   │   ├── projects/
│   │   │   │   └── index.vue      # Projects list ✅
│   │   │   ├── providers.vue      # Providers (placeholder) ✅
│   │   │   ├── quick-tools.vue    # Quick tools (placeholder) ✅
│   │   │   └── settings.vue       # Settings (placeholder) ✅
│   │   │   └── [slug]/            # Project detail (Phase 3)
│   │   │       ├── runs/          # Run management (Phase 3)
│   │   │       ├── assets/        # Asset gallery (Phase 4)
│   │   │       ├── characters/    # Characters (Phase 5)
│   │   │       ├── style-guides/  # Style guides (Phase 5)
│   │   │       ├── references/    # References (Phase 5)
│   │   │       └── settings/      # Project settings (Phase 6)
│   │   └── admin/                 # Existing admin pages
│   ├── components/
│   │   ├── atoms/
│   │   │   ├── ProjectCard.vue    ✅
│   │   │   ├── RunStatusBadge.vue # Phase 3
│   │   │   └── ...
│   │   ├── molecules/
│   │   │   ├── RunTriggerForm.vue # Phase 3
│   │   │   └── ...
│   │   └── organisms/
│   │       ├── ProjectDashboard.vue # Phase 1
│   │       └── ...
│   ├── stores/
│   │   ├── projects.ts            ✅
│   │   ├── runs.ts                # Phase 3
│   │   ├── assets.ts              # Phase 4
│   │   └── providers.ts           # Phase 2
│   ├── composables/
│   │   ├── useProjects.ts         ✅
│   │   ├── useRuns.ts             # Phase 3
│   │   ├── useAssets.ts           # Phase 4
│   │   └── useProviders.ts        # Phase 2
│   └── types/
│       └── kroma.ts               ✅
├── assets/
│   └── css/
│       ├── layout/
│       │   ├── kroma-*.css        ✅
│       │   └── ...
│       └── ui/
│           ├── projects/          ✅
│           ├── providers/         # Phase 2
│           ├── runs/              # Phase 3
│           └── ...
└── i18n/
    ├── en.ts                      ✅
    ├── ru.ts                      # Future
    └── he.ts                      # Future
```

---

**Document Status:** Active  
**Next Review:** After Phase 2 completion  
**Next Phase:** Phase 2 - Provider Setup (2 days)

---

*Last updated: 2026-03-23 18:30 (UTC)*
