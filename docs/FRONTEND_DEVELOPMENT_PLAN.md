# Kroma Frontend Development Plan

**Document Created:** 2026-03-23 14:30 (UTC)  
**Last Updated:** 2026-03-23 14:30 (UTC)  
**Status:** Ready to Start - Phase 1 (App Shell & Projects)  
**Frontend Framework:** Puppet Master 2 (Nuxt 3 + Pure CSS)  
**Backend API:** Rust (Axum + SQLite) - Step B Complete

---

## Executive Summary

### Current State

| Component | Status | Details |
|-----------|--------|---------|
| **Backend (Rust)** | ✅ Step B Complete | 68 API routes, contract-frozen, 60+ tests passing |
| **Frontend Framework** | ✅ Ready | Puppet Master - mature CSS system, auth, admin panel |
| **Kroma Frontend** | ⏳ Not Started | Needs custom app UI (not website) |

### Product Goal

Build a **project-first comic/graphic-novel production UI** that connects to the Rust backend API, enabling users to:

1. Create and manage project universes (continuity boundaries)
2. Configure AI provider accounts
3. Trigger generation runs with prompts and references
4. Review and approve candidates
5. Manage assets with lineage tracking
6. Build continuity references (characters, style guides, reference sets)
7. Export production-ready packages

---

## 1. Current Inventory Analysis

### 1.1 Puppet Master Frontend (What We Have)

#### Pages (5 existing)

| Page | Purpose | Relevance to Kroma |
|------|---------|-------------------|
| `index.vue` | Website homepage | ❌ Not needed (App mode) |
| `[section].vue` | Dynamic section renderer | ❌ Not needed (App mode) |
| `login.vue` | Admin login | ✅ Reuse for app login |
| `init.vue` | Setup wizard | ⚠️ Adapt for Kroma onboarding |
| `admin/*` | 16 admin pages | ⚠️ Adapt for Kroma admin |

#### Components (53 total)

**Atoms (11):**
```
✅ AppImage.vue          - Reusable image component
✅ BackToTop.vue         - Scroll helper
✅ CtaButton.vue         - Button variants (primary, secondary, ghost, outline)
✅ HamburgerIcon.vue     - Mobile nav icon
✅ LangSwitcher.vue      - Language selector
✅ Logo.vue              - Brand logo
✅ MadeWith.vue          - Footer branding
✅ NavLink.vue           - Navigation link
✅ PictureImage.vue      - Responsive picture element
✅ SocialIcon.vue        - Social media icons
✅ ThemeToggle.vue       - Dark/light mode toggle
```

**Loading States (4):**
```
✅ LoadingBase.vue       - Base loading wrapper
✅ LoadingCard.vue       - Card skeleton
✅ LoadingTable.vue      - Table skeleton
✅ LoadingText.vue       - Text skeleton
```

**Molecules (16):**
```
✅ AppBottomNav.vue      - Mobile bottom navigation (App UX)
✅ BlogPostCard.vue      - Content card pattern
✅ ContactInfo.vue       - Contact display
✅ EmptyState.vue        - Empty state pattern
✅ FooterCta.vue         - CTA section
✅ FooterNav.vue         - Footer navigation
✅ HeaderActions.vue     - Header action buttons
✅ HeaderContact.vue     - Header contact info
✅ HeroParallaxScene.vue - Hero section
✅ LegalInfo.vue         - Legal links
✅ NavLinks.vue          - Nav link collection
✅ PortfolioLightbox.vue - Image lightbox
✅ PricingCard.vue       - Pricing display
✅ SocialNav.vue         - Social links
✅ TeamMemberCard.vue    - Person card pattern
✅ TwoFactorSetup.vue    - 2FA setup flow
```

**Organisms (12):**
```
✅ BlogPostsGrid.vue     - Grid layout pattern
✅ ChangePasswordModal.vue - Modal form
✅ ConfirmDialog.vue     - Confirmation dialog
✅ PricingComparison.vue - Comparison table
✅ PricingTiers.vue      - Tier display
✅ SectionRenderer.vue   - Dynamic section renderer
✅ TeamGrid.vue          - Team grid layout
✅ TheFooter.vue         - Site footer
✅ TheHeader.vue         - Site header
✅ ToastContainer.vue    - Notification system
```

**Sections (11):**
```
✅ SectionAbout.vue
✅ SectionBlog.vue
✅ SectionClients.vue
✅ SectionContact.vue
✅ SectionFaq.vue
✅ SectionFeatures.vue
✅ SectionHero.vue
✅ SectionPortfolio.vue
✅ SectionPricing.vue
✅ SectionTeam.vue
✅ SectionTestimonials.vue
```

#### Composables (22 existing)

| Composable | Purpose | Reuse for Kroma |
|------------|---------|-----------------|
| `apiFetch.ts` | API client with CSRF | ✅ Yes - extend for Kroma API |
| `useAuth.ts` | Authentication state | ✅ Yes - direct reuse |
| `useToast.ts` | Toast notifications | ✅ Yes - direct reuse |
| `useConfirm.ts` | Confirmation dialogs | ✅ Yes - direct reuse |
| `useConfig.ts` | Configuration access | ✅ Yes - direct reuse |
| `useSiteSettings.ts` | Site settings | ⚠️ Adapt for project settings |
| `usePagination.ts` | Pagination logic | ✅ Yes - direct reuse |
| `useMediaQuery.ts` | Responsive queries | ✅ Yes - direct reuse |
| `usePerformance.ts` | Performance monitoring | ✅ Yes - direct reuse |
| `useA11y.ts` | Accessibility helpers | ✅ Yes - direct reuse |
| `useCsrf.ts` | CSRF token management | ✅ Yes - direct reuse |
| `useWebSocket.ts` | WebSocket connection | ⚠️ Adapt for run progress |
| Others (scroll, reveal, etc.) | UI enhancements | ⚠️ As needed |

#### Stores (1 existing)

```typescript
admin.ts - Admin panel state (adapt for Kroma app state)
```

#### CSS System (93 files)

**Architecture:** 5-layer cascade system
```css
@layer reset, primitives, semantic, components, utilities;
```

**Design Tokens:**
- **Colors:** 4 primitives (`--p-black`, `--p-white`, `--p-brand: #aa0000`, `--p-accent`)
- **Spacing:** `--space-1` (4px) to `--space-32` (128px)
- **Typography:** 4-layer font system (fallbacks, brand, semantic, sizes)
- **Breakpoints:** Phone (≤639px), Tablet (≥640px), Desktop (≥1024px), Large (≥1280px)

**Key Features:**
- ✅ Pure CSS (no Tailwind, no Bootstrap)
- ✅ CSS variables everywhere (no magic numbers)
- ✅ Logical properties (RTL support)
- ✅ Dark/light mode via `light-dark()` and `.dark/.light` classes
- ✅ One file per component (easy to find/maintain)

---

### 1.2 Rust Backend API (What to Connect To)

#### 22 API Domains

| Domain | Base Path | Methods | Purpose |
|--------|-----------|---------|---------|
| `auth` | `/auth` | GET, POST, DELETE | Authentication tokens |
| `projects` | `/api/projects` | GET, POST, PUT | Project CRUD, storage config |
| `runs` | `/api/projects/{slug}/runs` | GET, POST | Run trigger, review, jobs |
| `assets` | `/api/projects/{slug}/assets` | GET, POST | Asset upload, gallery |
| `storage` | `/api/projects/{slug}/storage` | GET, PUT | Storage configuration |
| `analytics` | `/quality-reports`, `/cost-events` | GET | Quality reports, costs |
| `exports` | `/exports` | GET, POST | Export packages |
| `prompt-templates` | `/prompt-templates` | GET, POST, PUT, DELETE | Template library |
| `provider-accounts` | `/provider-accounts` | GET, POST, PUT, DELETE | AI provider config |
| `style-guides` | `/style-guides` | GET, POST, PUT, DELETE | Visual constraints |
| `characters` | `/characters` | GET, POST, PUT, DELETE | Character identities |
| `reference-sets` | `/reference-sets` | GET, POST, PUT, DELETE | Continuity references |
| `reference-items` | `/reference-items` | GET, POST, PUT, DELETE | Reference items |
| `chat` | `/chat` | GET, POST | Agent chat |
| `agent/instructions` | `/agent/instructions` | GET, POST | Agent configuration |
| `secrets` | `/api/projects/{slug}/secrets` | GET, POST, PUT | Encrypted secrets |
| `bootstrap` | `/api/projects/{slug}/bootstrap-*` | GET, POST | Bootstrap import/export |

#### Key Data Models

```rust
// Project - Main continuity boundary
struct Project {
    id: String,
    slug: String,
    name: String,
    description: Option<String>,
    created_at: String,
    updated_at: String,
    owner_id: String,
}

// Run - Generation job with candidates
struct Run {
    id: String,
    project_slug: String,
    status: RunStatus, // pending, running, completed, failed
    prompt: String,
    candidates: Vec<Candidate>,
    created_at: String,
}

// Asset - Images with lineage tracking
struct Asset {
    id: String,
    project_slug: String,
    run_id: Option<String>,
    file_path: String,
    metadata: AssetMetadata,
    lineage: Vec<String>, // parent asset IDs
}

// Character - Recurring identity anchors
struct Character {
    id: String,
    project_slug: String,
    name: String,
    description: Option<String>,
    reference_images: Vec<String>,
}

// StyleGuide - Visual constraints
struct StyleGuide {
    id: String,
    project_slug: String,
    name: String,
    style_constraints: Vec<StyleConstraint>,
    reference_images: Vec<String>,
}

// ReferenceSet - Continuity references
struct ReferenceSet {
    id: String,
    project_slug: String,
    name: String,
    items: Vec<ReferenceItem>,
}

// ProviderAccount - AI API credentials
struct ProviderAccount {
    id: String,
    name: String,
    provider_type: ProviderType, // openai, stability, etc.
    api_key_encrypted: String,
    health_status: HealthStatus,
}
```

---

## 2. Gap Analysis: What's Missing for Kroma

### 2.1 Missing Pages (App UX)

| Page | Route | Purpose | Priority | Estimate |
|------|-------|---------|----------|----------|
| Dashboard | `/app` | Project overview, recent runs, quick stats | P0 | 4h |
| Projects List | `/app/projects` | Project list, create, edit, delete | P0 | 6h |
| Project Detail | `/app/projects/[slug]` | Project overview, tabs for runs/assets/etc. | P0 | 8h |
| Run History | `/app/projects/[slug]/runs` | Run list, trigger new run | P0 | 6h |
| Run Review | `/app/projects/[slug]/runs/[id]` | Candidate review, approve/retry | P0 | 12h |
| Asset Gallery | `/app/projects/[slug]/assets` | Asset grid, filters, detail view | P0 | 8h |
| Characters | `/app/projects/[slug]/characters` | Character management | P1 | 6h |
| Style Guides | `/app/projects/[slug]/style-guides` | Style guide editor | P1 | 6h |
| References | `/app/projects/[slug]/references` | Reference set builder | P1 | 8h |
| Project Settings | `/app/projects/[slug]/settings` | Config, storage, secrets | P1 | 6h |
| Providers | `/app/providers` | AI provider account setup | P0 | 6h |
| Templates | `/app/templates` | Prompt template library | P2 | 4h |
| Exports | `/app/exports` | Export history, download | P2 | 4h |
| Quick Tools | `/app/quick-tools` | Utility mode (bg-remove, upscale) | P2 | 8h |
| Chat | `/app/chat` | Chat with agent | P2 | 6h |

**Total Pages:** 15 new pages  
**Estimated Time:** 98 hours (~2.5 weeks full-time)

---

### 2.2 Missing Components

#### Atoms (need ~15 new)

| Component | Purpose | Priority |
|-----------|---------|----------|
| `ProjectCard.vue` | Project summary card | P0 |
| `RunStatusBadge.vue` | Run status indicator (pending, running, completed, failed) | P0 |
| `AssetThumbnail.vue` | Asset thumbnail with metadata | P0 |
| `CharacterAvatar.vue` | Character avatar display | P1 |
| `StyleSwatch.vue` | Style color/constraint swatch | P1 |
| `ReferenceThumbnail.vue` | Reference item thumbnail | P1 |
| `ProviderStatusIcon.vue` | Provider health status icon | P0 |
| `PromptInput.vue` | Multi-line prompt input with suggestions | P0 |
| `CandidateCard.vue` | Run candidate display | P0 |
| `QualityScoreBadge.vue` | QA score indicator | P0 |
| `LineageGraph.vue` | Mini asset lineage visualization | P1 |
| `StorageUsageIndicator.vue` | Storage usage progress bar | P1 |
| `ApiKeyInput.vue` | Secure API key input field | P0 |
| `TokenBalanceDisplay.vue` | Provider token balance | P2 |
| `WorkflowStepIndicator.vue` | Step progress indicator | P0 |

#### Molecules (need ~20 new)

| Component | Purpose | Priority |
|-----------|---------|----------|
| `ProjectHeader.vue` | Project title, actions, metadata | P0 |
| `RunTriggerForm.vue` | Run configuration form | P0 |
| `RunCandidateViewer.vue` | Candidate comparison viewer | P0 |
| `AssetGallery.vue` | Asset grid with selection | P0 |
| `AssetDetailPanel.vue` | Asset metadata, lineage, actions | P0 |
| `CharacterEditor.vue` | Character CRUD form | P1 |
| `StyleGuideEditor.vue` | Style guide constraint editor | P1 |
| `ReferenceSetBuilder.vue` | Reference set add/remove items | P1 |
| `ProviderAccountForm.vue` | Provider account CRUD | P0 |
| `PromptTemplateSelector.vue` | Template picker with preview | P2 |
| `RunHistoryTable.vue` | Run history with filters | P0 |
| `AssetFilterBar.vue` | Asset filter controls | P0 |
| `ProjectStorageConfig.vue` | Storage configuration form | P1 |
| `ExportPackageBuilder.vue` | Export package configuration | P2 |
| `QuickToolSelector.vue` | Quick tool selection cards | P2 |
| `BgRemoveTool.vue` | Background removal workflow | P2 |
| `UpscaleTool.vue` | Image upscaling workflow | P2 |
| `ColorCorrectionTool.vue` | Color correction workflow | P2 |
| `ChatMessageBubble.vue` | Chat message display | P2 |
| `AgentInstructionsEditor.vue` | Agent instruction editor | P2 |

#### Organisms (need ~12 new)

| Component | Purpose | Priority |
|-----------|---------|----------|
| `ProjectDashboard.vue` | Dashboard with project stats | P0 |
| `RunReviewWorkflow.vue` | Multi-step run review flow | P0 |
| `AssetManager.vue` | Full asset management interface | P0 |
| `CharacterRoster.vue` | Character roster grid | P1 |
| `StyleGuideLibrary.vue` | Style guide collection | P1 |
| `ReferenceSetGallery.vue` | Reference set gallery | P1 |
| `ProviderAccountManager.vue` | Provider account management | P0 |
| `PromptTemplateLibrary.vue` | Template library browser | P2 |
| `ExportHistoryPanel.vue` | Export history list | P2 |
| `QuickToolsPanel.vue` | Quick tools dashboard | P2 |
| `ProjectSettingsPanel.vue` | Project settings tabs | P1 |
| `AppShell.vue` | App layout (sidebar + header + content) | P0 |

**Total Components:** 47 new components  
**Estimated Time:** 140 hours (~3.5 weeks full-time)

---

### 2.3 Missing Composables (API Integration)

| Composable | Purpose | Priority | Estimate |
|------------|---------|----------|----------|
| `useProjects.ts` | Project CRUD, storage config | P0 | 4h |
| `useRuns.ts` | Run trigger, review, jobs | P0 | 6h |
| `useAssets.ts` | Asset upload, gallery, lineage | P0 | 6h |
| `useCharacters.ts` | Character CRUD | P1 | 4h |
| `useStyleGuides.ts` | Style guide CRUD | P1 | 4h |
| `useReferences.ts` | Reference sets/items CRUD | P1 | 4h |
| `useProviders.ts` | Provider account CRUD | P0 | 4h |
| `useTemplates.ts` | Prompt templates CRUD | P2 | 3h |
| `useChat.ts` | Chat with agent | P2 | 4h |
| `useExports.ts` | Export history, download | P2 | 3h |
| `useSecrets.ts` | Project secrets (encrypted) | P1 | 4h |
| `useBootstrap.ts` | Bootstrap prompt/import | P1 | 4h |
| `useAnalytics.ts` | Quality reports, cost events | P2 | 4h |
| `useStorage.ts` | Storage config (local, S3) | P1 | 3h |
| `useQuickTools.ts` | Quick tools (bg-remove, upscale) | P2 | 4h |

**Total Composables:** 15 new composables  
**Estimated Time:** 61 hours (~1.5 weeks full-time)

---

### 2.4 Missing Stores (Pinia)

| Store | Purpose | Priority | Estimate |
|-------|---------|----------|----------|
| `projects.ts` | Active project state, selection | P0 | 3h |
| `runs.ts` | Current run workflow state | P0 | 3h |
| `assets.ts` | Asset selection, filters | P0 | 3h |
| `chat.ts` | Chat message history | P2 | 2h |

**Total Stores:** 4 new stores  
**Estimated Time:** 11 hours

---

### 2.5 Missing CSS (Domain-Specific)

| Directory | Files | Purpose | Priority |
|-----------|-------|---------|----------|
| `ui/projects/` | `project-card.css`, `project-grid.css`, `project-header.css` | Project UI | P0 |
| `ui/runs/` | `run-status.css`, `run-candidate.css`, `run-review.css`, `workflow-steps.css` | Run UI | P0 |
| `ui/assets/` | `asset-gallery.css`, `asset-thumbnail.css`, `asset-detail.css`, `lineage-graph.css` | Asset UI | P0 |
| `ui/characters/` | `character-roster.css`, `character-card.css` | Character UI | P1 |
| `ui/style-guides/` | `style-swatch.css`, `style-guide-editor.css` | Style guide UI | P1 |
| `ui/references/` | `reference-set.css`, `reference-item.css` | Reference UI | P1 |
| `ui/providers/` | `provider-account.css`, `provider-status.css` | Provider UI | P0 |
| `ui/quick-tools/` | `tool-card.css`, `tool-workflow.css` | Quick tools UI | P2 |
| `layout/` | `app-sidebar.css`, `app-header.css`, `app-content.css` | App layout | P0 |

**Total CSS Files:** 20+ new files  
**Estimated Time:** 30 hours

---

## 3. Development Phases (Priority Order)

### Phase 1: Core App Shell & Projects (P0)
**Goal:** Users can create projects and see project list  
**Estimated Time:** 24 hours (3 days)  
**Start Date:** 2026-03-23  
**Target Completion:** 2026-03-26

#### Tasks

- [ ] **1.1** Configure `puppet-master.config.ts` for App mode
  ```typescript
  entities: { website: false, app: true }
  pmMode: 'build'
  ```

- [ ] **1.2** Create `app/layouts/app.vue` (vertical nav desktop, bottom nav mobile)

- [ ] **1.3** Build `AppShell.vue` organism (sidebar + header + content area)

- [ ] **1.4** Add CSS: `layout/app-sidebar.css`, `layout/app-header.css`, `layout/app-content.css`

- [ ] **1.5** Create pages:
  - `/app` (dashboard)
  - `/app/projects` (project list)

- [ ] **1.6** Build components:
  - `ProjectCard.vue` (atom)
  - `ProjectHeader.vue` (molecule)
  - `ProjectDashboard.vue` (organism)

- [ ] **1.7** Create composables:
  - `useProjects.ts` (project CRUD)

- [ ] **1.8** Create stores:
  - `projects.ts` (active project state)

- [ ] **1.9** Add CSS: `ui/projects/project-card.css`, `ui/projects/project-grid.css`

- [ ] **1.10** Connect to backend APIs:
  - `GET /api/projects` (list)
  - `POST /api/projects` (create)
  - `GET /api/projects/{slug}` (detail)

#### Deliverables

- ✅ App shell with navigation (vertical sidebar desktop, bottom nav mobile)
- ✅ Project list page with create/edit/delete
- ✅ Project detail page (basic info, tabs for runs/assets/etc.)
- ✅ Dashboard with recent runs, project stats
- ✅ All data comes from Rust backend API

#### Acceptance Criteria

- [ ] User can create a new project
- [ ] User can see list of projects
- [ ] User can navigate to project detail
- [ ] User can edit project name/description
- [ ] User can delete project (with confirmation)
- [ ] Navigation works on desktop and mobile
- [ ] Dark/light mode works

---

### Phase 2: Provider Setup (P0)
**Goal:** Users can configure AI provider accounts  
**Estimated Time:** 16 hours (2 days)  
**Start Date:** 2026-03-27  
**Target Completion:** 2026-03-29

#### Tasks

- [ ] **2.1** Create page: `/app/providers`

- [ ] **2.2** Build components:
  - `ProviderAccountForm.vue` (molecule)
  - `ProviderStatusIcon.vue` (atom)
  - `ApiKeyInput.vue` (atom)
  - `ProviderAccountManager.vue` (organism)

- [ ] **2.3** Create composables:
  - `useProviders.ts` (provider CRUD)

- [ ] **2.4** Add CSS: `ui/providers/provider-account.css`, `ui/providers/provider-status.css`

- [ ] **2.5** Connect to provider-accounts CRUD APIs

- [ ] **2.6** Implement secrets encryption UI (if needed)

#### Deliverables

- ✅ Provider account list
- ✅ Add/edit provider form (API key input with encryption)
- ✅ Provider health check status display
- ✅ Delete provider (with confirmation)

#### Acceptance Criteria

- [ ] User can add provider account (OpenAI, Stability, etc.)
- [ ] API key is stored encrypted
- [ ] Provider health status is visible
- [ ] User can delete provider account
- [ ] Provider selection available in run form

---

### Phase 3: Run Workflow (P0)
**Goal:** Users can trigger runs and review candidates  
**Estimated Time:** 32 hours (4 days)  
**Start Date:** 2026-03-30  
**Target Completion:** 2026-04-03

#### Tasks

- [ ] **3.1** Create pages:
  - `/app/projects/[slug]/runs` (run history)
  - `/app/projects/[slug]/runs/[id]` (run review)

- [ ] **3.2** Build components:
  - `RunTriggerForm.vue` (molecule)
  - `RunCandidateViewer.vue` (molecule)
  - `RunStatusBadge.vue` (atom)
  - `CandidateCard.vue` (atom)
  - `QualityScoreBadge.vue` (atom)
  - `WorkflowStepIndicator.vue` (atom)
  - `RunHistoryTable.vue` (molecule)
  - `RunReviewWorkflow.vue` (organism)

- [ ] **3.3** Create composables:
  - `useRuns.ts` (run trigger, review, jobs)

- [ ] **3.4** Create stores:
  - `runs.ts` (current run workflow state)

- [ ] **3.5** Add CSS: `ui/runs/run-status.css`, `ui/runs/run-candidate.css`, `ui/runs/run-review.css`, `ui/runs/workflow-steps.css`

- [ ] **3.6** Connect to runs CRUD, trigger, review APIs

- [ ] **3.7** Implement candidate approval/retry flow

#### Deliverables

- ✅ Run trigger form (prompt, references, settings)
- ✅ Run history table with filters
- ✅ Run review page (candidate comparison)
- ✅ Approve/retry workflow
- ✅ Real-time run status updates (polling or WebSocket)

#### Acceptance Criteria

- [ ] User can trigger a new run with prompt and settings
- [ ] User can see run progress (pending → running → completed)
- [ ] User can view candidates for completed runs
- [ ] User can approve candidates (mark as winner)
- [ ] User can retry failed runs
- [ ] Run status updates in real-time
- [ ] Candidate images are displayed with metadata

---

### Phase 4: Asset Management (P0)
**Goal:** Users can view, filter, and manage project assets  
**Estimated Time:** 24 hours (3 days)  
**Start Date:** 2026-04-04  
**Target Completion:** 2026-04-07

#### Tasks

- [ ] **4.1** Create page: `/app/projects/[slug]/assets`

- [ ] **4.2** Build components:
  - `AssetGallery.vue` (molecule)
  - `AssetThumbnail.vue` (atom)
  - `AssetDetailPanel.vue` (molecule)
  - `AssetFilterBar.vue` (molecule)
  - `AssetManager.vue` (organism)

- [ ] **4.3** Create composables:
  - `useAssets.ts` (asset upload, gallery, lineage)

- [ ] **4.4** Create stores:
  - `assets.ts` (asset selection, filters)

- [ ] **4.5** Add CSS: `ui/assets/asset-gallery.css`, `ui/assets/asset-thumbnail.css`, `ui/assets/asset-detail.css`

- [ ] **4.6** Connect to assets CRUD, upload APIs

- [ ] **4.7** Implement asset filters (by run, character, style)

#### Deliverables

- ✅ Asset gallery grid with infinite scroll/pagination
- ✅ Asset detail panel (metadata, lineage, actions)
- ✅ Upload/replace asset
- ✅ Filter by tags, runs, characters, style
- ✅ Asset selection (single, multi)

#### Acceptance Criteria

- [ ] User can view all project assets in grid
- [ ] User can filter assets by run, character, style
- [ ] User can view asset metadata (prompt, run, lineage)
- [ ] User can upload new assets
- [ ] User can delete assets (with confirmation)
- [ ] Asset lineage is visible (parent/child relationships)

---

### Phase 5: Continuity Features (P1)
**Goal:** Users can manage characters, style guides, references  
**Estimated Time:** 32 hours (4 days)  
**Start Date:** 2026-04-08  
**Target Completion:** 2026-04-12

#### Tasks

- [ ] **5.1** Create pages:
  - `/app/projects/[slug]/characters`
  - `/app/projects/[slug]/style-guides`
  - `/app/projects/[slug]/references`

- [ ] **5.2** Build components:
  - `CharacterEditor.vue`, `CharacterAvatar.vue`, `CharacterRoster.vue`
  - `StyleGuideEditor.vue`, `StyleSwatch.vue`, `StyleGuideLibrary.vue`
  - `ReferenceSetBuilder.vue`, `ReferenceThumbnail.vue`, `ReferenceSetGallery.vue`

- [ ] **5.3** Create composables:
  - `useCharacters.ts`
  - `useStyleGuides.ts`
  - `useReferences.ts`

- [ ] **5.4** Add CSS:
  - `ui/characters/*`
  - `ui/style-guides/*`
  - `ui/references/*`

- [ ] **5.5** Connect to respective CRUD APIs

#### Deliverables

- ✅ Character roster (add/edit/delete with reference images)
- ✅ Style guide library (visual constraints editor)
- ✅ Reference set builder (continuity anchors)

#### Acceptance Criteria

- [ ] User can create/edit/delete characters
- [ ] User can add reference images to characters
- [ ] User can create style guides with constraints
- [ ] User can create reference sets with items
- [ ] Characters/styles/references are selectable in run form

---

### Phase 6: Project Settings (P1)
**Goal:** Users can configure project storage, secrets, defaults  
**Estimated Time:** 20 hours (2.5 days)  
**Start Date:** 2026-04-13  
**Target Completion:** 2026-04-15

#### Tasks

- [ ] **6.1** Create page: `/app/projects/[slug]/settings`

- [ ] **6.2** Build components:
  - `ProjectStorageConfig.vue` (molecule)
  - `StorageUsageIndicator.vue` (atom)
  - `ProjectSettingsPanel.vue` (organism)

- [ ] **6.3** Create composables:
  - `useStorage.ts` (storage config)
  - `useSecrets.ts` (project secrets)
  - `useBootstrap.ts` (bootstrap import/export)

- [ ] **6.4** Connect to storage config, secrets APIs

- [ ] **6.5** Implement bootstrap import/export

#### Deliverables

- ✅ Storage configuration (local, S3)
- ✅ Secrets management (encrypted API keys)
- ✅ Bootstrap import (from AI prompt)
- ✅ Export project settings

#### Acceptance Criteria

- [ ] User can configure storage (local filesystem, S3)
- [ ] User can view storage usage
- [ ] User can manage project secrets (encrypted)
- [ ] User can import bootstrap from AI prompt
- [ ] User can export project settings

---

### Phase 7: Quick Tools (P2)
**Goal:** Utility mode without project context  
**Estimated Time:** 24 hours (3 days)  
**Start Date:** 2026-04-16  
**Target Completion:** 2026-04-19

#### Tasks

- [ ] **7.1** Create page: `/app/quick-tools`

- [ ] **7.2** Build components:
  - `QuickToolSelector.vue` (molecule)
  - `BgRemoveTool.vue` (molecule)
  - `UpscaleTool.vue` (molecule)
  - `ColorCorrectionTool.vue` (molecule)
  - `QuickToolsPanel.vue` (organism)

- [ ] **7.3** Create composables:
  - `useQuickTools.ts` (quick tools orchestration)

- [ ] **7.4** Add CSS: `ui/quick-tools/tool-card.css`, `ui/quick-tools/tool-workflow.css`

- [ ] **7.5** Connect to utility endpoints

#### Deliverables

- ✅ Tool selector (bg-remove, upscale, color)
- ✅ Single-image workflow for each tool
- ✅ Download result

#### Acceptance Criteria

- [ ] User can select quick tool (no project required)
- [ ] User can upload image for processing
- [ ] User can view processed result
- [ ] User can download result
- [ ] Tools work independently of projects

---

### Phase 8: Advanced Features (P2)
**Goal:** Templates, exports, chat, analytics  
**Estimated Time:** 24 hours (3 days)  
**Start Date:** 2026-04-20  
**Target Completion:** 2026-04-23

#### Tasks

- [ ] **8.1** Create pages:
  - `/app/templates`
  - `/app/exports`
  - `/app/chat`

- [ ] **8.2** Build components:
  - `PromptTemplateSelector.vue`, `PromptTemplateLibrary.vue`
  - `ExportPackageBuilder.vue`, `ExportHistoryPanel.vue`
  - `ChatMessageBubble.vue`

- [ ] **8.3** Create composables:
  - `useTemplates.ts`
  - `useExports.ts`
  - `useChat.ts`
  - `useAnalytics.ts`

- [ ] **8.4** Connect to respective APIs

#### Deliverables

- ✅ Prompt template library (save, load, edit)
- ✅ Export history, package download
- ✅ Chat with agent
- ✅ Quality reports, cost analytics

#### Acceptance Criteria

- [ ] User can save/load prompt templates
- [ ] User can view export history
- [ ] User can download export packages
- [ ] User can chat with agent
- [ ] User can view quality reports and cost analytics

---

## 4. API Integration Strategy

### 4.1 Base URL Configuration

```typescript
// nuxt.config.ts
export default defineNuxtConfig({
  runtimeConfig: {
    public: {
      kromaApiBaseUrl: process.env.KROMA_API_BASE_URL || 'http://127.0.0.1:8788'
    }
  }
})
```

### 4.2 Extended apiFetch Composable

```typescript
// composables/apiFetch.ts (extend existing)
export interface ApiSuccessEnvelope<T> {
  success: true
  data: T
}

export interface ApiErrorEnvelope {
  success: false
  error: {
    error_kind: string
    error_code: string
    message: string
    details?: Record<string, string[]>
  }
}

type FetchInput = Parameters<typeof $fetch>[0]
type FetchOptions = Parameters<typeof $fetch>[1]

export async function apiFetch<T>(input: FetchInput, options?: FetchOptions): Promise<T> {
  const config = useRuntimeConfig()
  const baseURL = config.public.kromaApiBaseUrl
  
  // Prepend API base URL if not already present
  if (typeof input === 'string' && input.startsWith('/api')) {
    input = `${baseURL}${input}`
  }
  
  // Add CSRF headers for mutations (existing logic)
  const method = typeof options?.method === 'string' ? options.method : undefined
  const withCsrf = isMutationMethod(method)
  const enhancedOptions = withCsrf ? withCsrfHeaders(method, options) : options
  
  // Use server-side fetch when appropriate (existing logic)
  const transport = getApiFetchTransport()
  
  try {
    const response = await transport<ApiSuccessEnvelope<T> | T>(input, enhancedOptions)
    return unwrapApiEnvelope<T>(response)
  } catch (error) {
    // Transform error to include error_kind, error_code from backend
    throw normalizeApiError(error)
  }
}

function normalizeApiError(error: unknown): ApiErrorEnvelope {
  // Extract backend error structure
  if (error && typeof error === 'object' && 'data' in error) {
    const data = (error as { data: unknown }).data
    if (data && typeof data === 'object' && 'error_kind' in data) {
      return {
        success: false,
        error: data as { error_kind: string; error_code: string; message: string }
      }
    }
  }
  
  // Fallback for network errors
  return {
    success: false,
    error: {
      error_kind: 'network_error',
      error_code: 'network_error',
      message: 'Network error. Please check your connection.'
    }
  }
}
```

### 4.3 Error Handling Pattern

```typescript
// composables/useRuns.ts
export function useRuns() {
  const toast = useToast()
  
  const errorMessages: Record<string, string> = {
    'project_not_found': 'Project not found',
    'provider_unavailable': 'AI provider is unavailable',
    'validation_failed': 'Invalid run configuration',
    'storage_error': 'Storage configuration error',
    'run_failed': 'Run failed to complete',
    'candidate_not_found': 'Candidate not found'
  }
  
  async function triggerRun(projectSlug: string, config: RunConfig) {
    try {
      const result = await apiFetch<Run>(`/api/projects/${projectSlug}/runs/trigger`, {
        method: 'POST',
        body: config
      })
      return { success: true, data: result }
    } catch (error) {
      const apiError = error as ApiErrorEnvelope
      const message = apiError.error?.message || 
                      errorMessages[apiError.error?.error_code] || 
                      'An unexpected error occurred'
      
      toast.error(message)
      return { success: false, error: message }
    }
  }
  
  return { triggerRun }
}
```

### 4.4 State Management Pattern

```typescript
// stores/projects.ts
export const useProjectsStore = defineStore('projects', () => {
  const projects = ref<ProjectSummary[]>([])
  const activeProject = ref<ProjectInfo | null>(null)
  const isLoading = ref(false)
  const error = ref<string | null>(null)
  
  async function fetchProjects() {
    isLoading.value = true
    error.value = null
    try {
      const response = await apiFetch<{ projects: ProjectSummary[] }>('/api/projects')
      projects.value = response.projects
      return response.projects
    } catch (err) {
      error.value = 'Failed to load projects'
      throw err
    } finally {
      isLoading.value = false
    }
  }
  
  async function fetchProject(slug: string) {
    isLoading.value = true
    error.value = null
    try {
      const response = await apiFetch<ProjectDetail>(`/api/projects/${slug}`)
      activeProject.value = response.project
      return response
    } catch (err) {
      error.value = 'Failed to load project'
      throw err
    } finally {
      isLoading.value = false
    }
  }
  
  async function createProject(input: CreateProjectInput) {
    isLoading.value = true
    error.value = null
    try {
      const response = await apiFetch<Project>(`/api/projects`, {
        method: 'POST',
        body: input
      })
      await fetchProjects() // Refresh list
      return response
    } catch (err) {
      error.value = 'Failed to create project'
      throw err
    } finally {
      isLoading.value = false
    }
  }
  
  return {
    projects,
    activeProject,
    isLoading,
    error,
    fetchProjects,
    fetchProject,
    createProject
  }
})
```

### 4.5 Type Definitions

```typescript
// types/kroma.ts
export interface Project {
  id: string
  slug: string
  name: string
  description: string | null
  created_at: string
  updated_at: string
  owner_id: string
}

export interface ProjectSummary extends Project {
  run_count: number
  asset_count: number
}

export interface ProjectDetail {
  project: Project
  counts: ProjectCounts
  storage: StorageConfig
}

export interface ProjectCounts {
  runs: number
  assets: number
  characters: number
  style_guides: number
  reference_sets: number
}

export interface StorageConfig {
  provider: 'local' | 's3'
  config: LocalStorageConfig | S3StorageConfig
}

export interface LocalStorageConfig {
  root_path: string
}

export interface S3StorageConfig {
  bucket: string
  region: string
  endpoint: string | null
}

export interface Run {
  id: string
  project_slug: string
  status: RunStatus
  prompt: string
  negative_prompt: string | null
  candidates: Candidate[]
  created_at: string
  updated_at: string
  completed_at: string | null
}

export type RunStatus = 'pending' | 'running' | 'completed' | 'failed' | 'cancelled'

export interface Candidate {
  id: string
  run_id: string
  image_url: string
  score: number | null
  is_winner: boolean
  metadata: CandidateMetadata
}

export interface CandidateMetadata {
  seed: number
  steps: number
  guidance_scale: number
  width: number
  height: number
}

export interface Asset {
  id: string
  project_slug: string
  run_id: string | null
  file_path: string
  file_size: number
  mime_type: string
  width: number
  height: number
  metadata: AssetMetadata
  lineage: string[] // parent asset IDs
  created_at: string
}

export interface AssetMetadata {
  prompt: string | null
  negative_prompt: string | null
  seed: number | null
  model: string | null
  tags: string[]
}

export interface Character {
  id: string
  project_slug: string
  name: string
  description: string | null
  reference_images: string[]
  created_at: string
  updated_at: string
}

export interface StyleGuide {
  id: string
  project_slug: string
  name: string
  description: string | null
  constraints: StyleConstraint[]
  reference_images: string[]
  created_at: string
  updated_at: string
}

export interface StyleConstraint {
  type: 'color' | 'composition' | 'lighting' | 'mood' | 'other'
  description: string
  weight: number
}

export interface ReferenceSet {
  id: string
  project_slug: string
  name: string
  description: string | null
  items: ReferenceItem[]
  created_at: string
  updated_at: string
}

export interface ReferenceItem {
  id: string
  reference_set_id: string
  asset_id: string | null
  external_url: string | null
  description: string | null
  tags: string[]
}

export interface ProviderAccount {
  id: string
  name: string
  provider_type: ProviderType
  api_key_encrypted: string
  health_status: HealthStatus
  created_at: string
  updated_at: string
}

export type ProviderType = 'openai' | 'stability' | 'anthropic' | 'midjourney' | 'other'

export interface HealthStatus {
  status: 'healthy' | 'unhealthy' | 'unknown'
  last_checked: string | null
  error_message: string | null
}

export interface CreateProjectInput {
  name: string
  description?: string
  storage_provider?: 'local' | 's3'
  storage_config?: LocalStorageConfig | S3StorageConfig
}

export interface RunConfig {
  prompt: string
  negative_prompt?: string
  model?: string
  width?: number
  height?: number
  steps?: number
  guidance_scale?: number
  seed?: number
  candidate_count?: number
  reference_set_ids?: string[]
  character_ids?: string[]
  style_guide_ids?: string[]
}
```

---

## 5. Development Workflow

### 5.1 Setup Commands

```bash
# 1. Configure for App mode
# Edit front-end-puppet-master/app/puppet-master.config.ts:
#   entities: { website: false, app: true }
#   pmMode: 'build'

# 2. Set API base URL
# Create .env in front-end-puppet-master:
KROMA_API_BASE_URL=http://127.0.0.1:8788

# 3. Start backend (in app/)
npm run backend:rust

# 4. Start frontend (in front-end-puppet-master/)
npm run dev

# 5. Access app
# http://localhost:3000/app
```

### 5.2 Component Creation Template

```vue
<!-- components/atoms/ProjectCard.vue -->
<script setup lang="ts">
import type { ProjectSummary } from '~/types'

const props = defineProps<{
  project: ProjectSummary
}>()

const emit = defineEmits<{
  select: [slug: string]
  edit: [slug: string]
  delete: [slug: string]
}>()

const formattedDate = computed(() => 
  new Date(props.project.created_at).toLocaleDateString()
)
</script>

<template>
  <!-- Uses global classes from ui/projects/project-card.css -->
  <div class="project-card" @click="emit('select', project.slug)">
    <div class="project-card__header">
      <h3 class="project-card__title">{{ project.name }}</h3>
      <button class="project-card__action" @click.stop="emit('edit', project.slug)" aria-label="Edit project">
        <Icon name="lucide:edit" class="icon-md" />
      </button>
    </div>
    <p class="project-card__description">{{ project.description || 'No description' }}</p>
    <div class="project-card__meta">
      <span class="project-card__date">Created: {{ formattedDate }}</span>
      <span class="project-card__count">{{ project.run_count }} runs</span>
    </div>
  </div>
</template>

<!-- No scoped styles - uses ui/projects/project-card.css -->
```

```css
/* assets/css/ui/projects/project-card.css */
.project-card {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
  padding: var(--space-4);
  background: var(--l-bg-elevated);
  border: 1px solid var(--l-border);
  border-radius: var(--radius-lg);
  cursor: pointer;
  transition: 
    transform var(--transition-fast) var(--ease-smooth),
    border-color var(--transition-fast) var(--ease-smooth);
}

.project-card:hover {
  transform: translateY(-2px);
  border-color: var(--i-brand);
}

.project-card__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.project-card__title {
  font-family: var(--font-heading);
  font-size: var(--text-lg);
  font-weight: var(--font-semibold);
  color: var(--t-primary);
  margin: 0;
}

.project-card__action {
  padding: var(--space-1);
  background: transparent;
  border: none;
  cursor: pointer;
  color: var(--t-secondary);
  border-radius: var(--radius-sm);
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background-color var(--transition-fast);
}

.project-card__action:hover {
  background: var(--l-bg-sunken);
  color: var(--t-primary);
}

.project-card__description {
  font-family: var(--font-body);
  font-size: var(--text-sm);
  color: var(--t-secondary);
  margin: 0;
  line-height: var(--leading-relaxed);
}

.project-card__meta {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding-top: var(--space-2);
  border-top: 1px solid var(--l-border);
}

.project-card__date,
.project-card__count {
  font-family: var(--font-ui);
  font-size: var(--text-xs);
  color: var(--t-muted);
}
```

### 5.3 Git Workflow

```bash
# Feature branch naming
git checkout -b feature/phase-1-app-shell
git checkout -b feature/phase-1-projects
git checkout -b feature/phase-2-providers
git checkout -b feature/phase-3-runs

# Commit message format
feat(app): add project list page with CRUD operations
feat(runs): implement run trigger form with validation
fix(assets): resolve asset thumbnail loading issue
style(css): update project card hover states
test(api): add integration tests for projects API
```

---

## 6. Timeline Summary

### Phase Schedule

| Phase | Start Date | End Date | Duration | Priority |
|-------|------------|----------|----------|----------|
| **Phase 1:** App Shell & Projects | 2026-03-23 | 2026-03-26 | 3 days | P0 |
| **Phase 2:** Provider Setup | 2026-03-27 | 2026-03-29 | 2 days | P0 |
| **Phase 3:** Run Workflow | 2026-03-30 | 2026-04-03 | 4 days | P0 |
| **Phase 4:** Asset Management | 2026-04-04 | 2026-04-07 | 3 days | P0 |
| **Phase 5:** Continuity Features | 2026-04-08 | 2026-04-12 | 4 days | P1 |
| **Phase 6:** Project Settings | 2026-04-13 | 2026-04-15 | 2 days | P1 |
| **Phase 7:** Quick Tools | 2026-04-16 | 2026-04-19 | 3 days | P2 |
| **Phase 8:** Advanced Features | 2026-04-20 | 2026-04-23 | 3 days | P2 |

### Total Estimates

| Metric | Value |
|--------|-------|
| **Total Pages** | 15 new pages |
| **Total Components** | 47 new components (15 atoms, 20 molecules, 12 organisms) |
| **Total Composables** | 15 new composables |
| **Total Stores** | 4 new stores |
| **Total CSS Files** | 20+ new CSS files |
| **Total Development Time** | 267 hours (~6.7 weeks full-time) |
| **MVP (Phases 1-4)** | 96 hours (~2.4 weeks full-time) |

---

## 7. Success Metrics

### MVP Complete (Phases 1-4)

- [ ] User can create and manage projects
- [ ] User can configure AI provider accounts
- [ ] User can trigger generation runs
- [ ] User can review and approve candidates
- [ ] User can manage assets with filters
- [ ] All data comes from Rust backend API
- [ ] Zero Python/script dependencies in frontend flow
- [ ] User can complete journey `J01 → J04` (Create Project → Lock Style Baseline)

### Full Feature Set (Phases 1-8)

- [ ] All MVP features complete
- [ ] User can manage characters, style guides, references
- [ ] User can configure project settings (storage, secrets)
- [ ] User can use quick tools without project context
- [ ] User can save/load prompt templates
- [ ] User can export production packages
- [ ] User can chat with agent
- [ ] User can view quality reports and analytics

---

## 8. Risks & Mitigations

### Technical Risks

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Backend API changes break frontend | High | Low | Contract freeze in place; versioning strategy |
| CSS conflicts with existing PM styles | Medium | Medium | Use namespace prefixes for Kroma-specific CSS |
| Performance issues with large asset galleries | Medium | Medium | Implement pagination, lazy loading, virtualization |
| Real-time run status updates complex | Medium | Medium | Start with polling, upgrade to WebSocket later |

### Scope Risks

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Feature creep during development | High | High | Strict adherence to journey steps (J00-J08) |
| Perfectionism in UI polish | Medium | High | MVP-first approach; polish in Phase 8 |
| Integration complexity underestimated | Medium | Medium | Build integration tests alongside features |

---

## 9. Next Steps (Immediate Actions)

### Week 1: Phase 1 - App Shell & Projects

**Day 1 (2026-03-23):**
- [ ] Update `puppet-master.config.ts` for App mode
- [ ] Create `app/layouts/app.vue`
- [ ] Build `AppShell.vue` organism
- [ ] Add CSS: `layout/app-sidebar.css`, `layout/app-header.css`

**Day 2 (2026-03-24):**
- [ ] Create `/app` dashboard page
- [ ] Build `ProjectDashboard.vue` organism
- [ ] Create `useProjects.ts` composable
- [ ] Create `projects.ts` store

**Day 3 (2026-03-25):**
- [ ] Create `/app/projects` list page
- [ ] Build `ProjectCard.vue` atom
- [ ] Add CSS: `ui/projects/project-card.css`, `ui/projects/project-grid.css`
- [ ] Connect to backend APIs (list, create, detail)

**Day 4 (2026-03-26):**
- [ ] Implement project edit/delete
- [ ] Add project detail page tabs
- [ ] Testing and bug fixes
- [ ] Documentation updates

---

## Appendix A: Journey Step Mapping

This plan implements the following journey steps from `USER_FLOW_JOURNEY_MAP.md`:

| Journey Step | Frontend Pages | Frontend Components | Status |
|--------------|----------------|---------------------|--------|
| `J01` Create or Select Project | `/app/projects`, `/app/projects/[slug]` | ProjectCard, ProjectHeader, ProjectDashboard | Phase 1 |
| `J02` Build Continuity References | `/app/projects/[slug]/characters`, `/style-guides`, `/references` | CharacterEditor, StyleGuideEditor, ReferenceSetBuilder | Phase 5 |
| `J03` Bootstrap Story Settings | `/app/projects/[slug]/settings` | ProjectSettingsPanel, BootstrapImport | Phase 6 |
| `J04` Lock Style Baseline | `/app/projects/[slug]/runs`, `/runs/[id]` | RunTriggerForm, RunCandidateViewer, RunReviewWorkflow | Phase 3 |
| `J05` Controlled Variation | `/app/projects/[slug]/runs/[id]` | RunCandidateViewer, WorkflowStepIndicator | Phase 3 |
| `J06` Character Identity Stage | `/app/projects/[slug]/characters` | CharacterRoster, CharacterEditor | Phase 5 |
| `J07` Local Post-Process Chain | `/app/quick-tools` | QuickToolSelector, BgRemoveTool, UpscaleTool | Phase 7 |
| `J08` Review, Curate, and Export | `/app/exports`, `/app/projects/[slug]/assets` | AssetManager, ExportPackageBuilder | Phase 4, 8 |

---

## Appendix B: Backend API Reference

Full API documentation available in `openapi/backend-api.openapi.yaml`.

### Key Endpoints

```yaml
# Projects
GET    /api/projects                      # List projects
POST   /api/projects                      # Create project
GET    /api/projects/{slug}               # Get project detail
PUT    /api/projects/{slug}               # Update project
DELETE /api/projects/{slug}               # Delete project

# Storage
GET    /api/projects/{slug}/storage       # Get storage config
PUT    /api/projects/{slug}/storage/local # Update local storage
PUT    /api/projects/{slug}/storage/s3    # Update S3 storage

# Runs
GET    /api/projects/{slug}/runs          # List runs
POST   /api/projects/{slug}/runs/trigger  # Trigger new run
GET    /api/projects/{slug}/runs/{runId}  # Get run detail
GET    /api/projects/{slug}/runs/{runId}/jobs  # Get run jobs

# Assets
GET    /api/projects/{slug}/assets        # List assets
POST   /api/projects/{slug}/assets        # Upload asset
GET    /api/projects/{slug}/assets/{id}   # Get asset detail
DELETE /api/projects/{slug}/assets/{id}   # Delete asset

# Characters
GET    /api/characters?project_slug={slug}  # List characters
POST   /api/characters                      # Create character
PUT    /api/characters/{id}                 # Update character
DELETE /api/characters/{id}                 # Delete character

# Style Guides
GET    /api/style-guides?project_slug={slug}  # List style guides
POST   /api/style-guides                      # Create style guide
PUT    /api/style-guides/{id}                 # Update style guide
DELETE /api/style-guides/{id}                 # Delete style guide

# Reference Sets
GET    /api/reference-sets?project_slug={slug}  # List reference sets
POST   /api/reference-sets                      # Create reference set
PUT    /api/reference-sets/{id}                 # Update reference set
DELETE /api/reference-sets/{id}                 # Delete reference set

# Provider Accounts
GET    /api/provider-accounts              # List providers
POST   /api/provider-accounts              # Create provider
PUT    /api/provider-accounts/{id}         # Update provider
DELETE /api/provider-accounts/{id}         # Delete provider

# Secrets
GET    /api/projects/{slug}/secrets        # List secrets
POST   /api/projects/{slug}/secrets        # Create/update secret
PUT    /api/projects/{slug}/secrets/rotate # Rotate encryption key
```

---

**Document End**

---

*This document should be updated as phases are completed or priorities change. Last updated: 2026-03-23 14:30 (UTC)*
