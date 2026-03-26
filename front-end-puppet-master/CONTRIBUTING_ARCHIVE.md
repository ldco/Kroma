# Framework Gap Log Archive (Entries 1-30)

**Archived:** 2026-03-01  
**Reason:** Release 1.3.0 completed - archiving historical entries for maintainability  
**Active Entries:** 31+ (see CONTRIBUTING.md Section 13)

---

## Table of Contents

| Entry | Date | Area | Summary |
|-------|------|------|---------|
| 1 | 2026-02-18 | Test infrastructure | Auth session + CSRF context in API tests |
| 2 | 2026-02-19 | Local DB bootstrap | Stale SQLite states |
| 3 | 2026-02-19 | Mobile navigation | Header actions visibility |
| 4 | 2026-02-19 | Footer branding | Attribution visibility |
| 5 | 2026-02-19 | Layout containment | Fixed header behavior |
| 6 | 2026-02-20 | Scrollytelling | Parallax runtime |
| 7 | 2026-02-20 | Private-teaching | Group-size normalization |
| 8 | 2026-02-20 | FAQ rendering | Rich HTML sanitization |
| 9 | 2026-02-20 | Test bootstrap | Portability fix |
| 10 | 2026-02-20 | Style system | Typography + spacing |
| 11 | 2026-02-20 | Style system | Table/action layout |
| 12 | 2026-02-20 | Action controls | Admin UX consistency |
| 13 | 2026-02-20 | Documentation | Entrypoint architecture |
| 14 | 2026-02-20 | Selection UX | Segmented controls |
| 15 | 2026-02-20 | Sanitization | Fallback correctness |
| 16 | 2026-02-20 | Release sync | Ledger protocol |
| 17 | 2026-02-20 | Layout policy | Chrome width defaults |
| 18 | 2026-02-20 | Admin nav | Layout semantics |
| 19 | 2026-02-20 | Onepager | Scrollytelling surface |
| 20 | 2026-02-21 | Route resolution | List/detail shadow |
| 21 | 2026-02-25 | Settings form | Save UX contract |
| 22 | 2026-02-25 | Contact notifications | Telegram secret-source |
| 23 | 2026-02-25 | Settings auditing | Sensitive data policy |
| 24 | 2026-02-25 | Settings visibility | Regression guards |
| 25 | 2026-02-25 | Contribution trace | PM-2026-0225-001 |
| 26 | 2026-02-25 | Password settings | Encryption-at-rest |
| 27 | 2026-02-25 | Admin API | Authorization consistency |
| 28 | 2026-02-25 | Contributor DX | Targeted verification |
| 29 | 2026-02-25 | Admin API | Global authorization |
| 30 | 2026-02-25 | Utility classes | Contract automation |

---

## Archived Entries (1-30)

### Entry 1
**Date:** 2026-02-18  
**Area:** Test infrastructure  
**Gap:** Auth session + CSRF context was hard to propagate in API tests.  
**Framework change:** Enhanced shared test helper cookie/session handling.  
**Files:** `tests/utils/helpers.ts`  
**Compatibility impact:** Backward-compatible.

---

### Entry 2
**Date:** 2026-02-19  
**Area:** Local DB bootstrap  
**Gap:** Stale local SQLite states caused false framework/debug signals.  
**Framework change:** Standardized isolated `DATABASE_URL` + seed workflow in contributor docs.  
**Files:** `CONTRIBUTING.md`  
**Compatibility impact:** Workflow/documentation only.

---

### Entry 3
**Date:** 2026-02-19  
**Area:** Mobile navigation / header actions visibility  
**Gap:** Framework mobile rule hides `.header-actions` globally under `--below-desktop`, which also hides `HeaderActions` inside the mobile drawer (`.mobile-nav-settings`) unless explicitly overridden.  
**Framework change:** Scope mobile hide rule to top header context (or ship framework default override for `.mobile-nav-settings .header-actions`) so theme/language/login controls are visible in mobile drawer out of the box.  
**Files:** `app/assets/css/skeleton/header.css`, `app/assets/css/skeleton/mobile-nav.css`  
**Compatibility impact:** Backward-compatible visual fix; restores expected mobile drawer controls.

---

### Entry 4
**Date:** 2026-02-19  
**Area:** Footer branding attribution  
**Gap:** `features.footerMadeWith` can be left disabled during cleanup/starter tuning, making framework attribution disappear unintentionally.  
**Framework change:** Keep framework-validation default with visible attribution and document explicit opt-out rules if a product intentionally disables it.  
**Files:** `app/puppet-master.config.ts`, `CONTRIBUTING.md`  
**Compatibility impact:** Backward-compatible config/documentation guidance.

---

### Entry 5
**Date:** 2026-02-19  
**Area:** Layout containment vs fixed header  
**Gap:** Adding `contain: layout style` to root layout wrappers (`.layout` / `.layout-admin`) can make the website header appear non-fixed (scrolls with content) in some engines.  
**Framework change:** Keep containment only on `.main` (container-query host); avoid root wrapper containment for website/app root layout containers.  
**Files:** `app/assets/css/layout/page.css`, `CONTRIBUTING.md`  
**Compatibility impact:** Backward-compatible CSS behavior fix; restores expected fixed header semantics.

---

### Entry 6
**Date:** 2026-02-20  
**Area:** Onepager storytelling/parallax capability  
**Gap:** Framework had reveal/scrollspy primitives but no reusable parallax/scrollytelling runtime for section-level storytelling.  
**Framework change:** Add baseline engine:
  - composable runtime: `useScrollytelling()`
  - directives: `v-parallax`, `v-scrolly-scene`
  - CSS hooks: `--pm-scroll-progress`, `--pm-scene-progress`, `--pm-parallax-x`, `--pm-parallax-y`
  - reduced-motion safe behavior by default  
**Files:** `app/composables/useScrollytelling.ts`, `app/plugins/scrollytelling.ts`, `app/assets/css/animations/scrollytelling.css`, `app/layouts/default.vue`  
**Compatibility impact:** Additive and backward-compatible; opt-in via data attributes/directives.

---

### Entry 7
**Date:** 2026-02-20  
**Area:** Private-teacher domain limits / drift prevention  
**Gap:** Max-group constraints were centralized as constants, but normalization logic still drifted between app and server implementations.  
**Framework change:** Move normalization into shared domain helper (`normalizePrivateGroupSize`) and reuse from both runtime layers.  
**Files:** `shared/domain/private-teaching.ts`, `server/utils/group-size.ts`, `app/pages/admin/classrooms.vue`  
**Compatibility impact:** Backward-compatible behavior hardening; reduces future config drift risk.

---

### Entry 8
**Date:** 2026-02-20  
**Area:** Rich HTML rendering safety (FAQ)  
**Gap:** FAQ answers were rendered with `v-html` without explicit sanitization path in the public API/UI contract.  
**Framework change:** Sanitize FAQ answer payload in public API and keep explicit safe-render marker in component code.  
**Files:** `server/api/faq/index.get.ts`, `app/components/sections/SectionFaq.vue`, `tests/server/utils/sanitize.test.ts`  
**Compatibility impact:** Backward-compatible security hardening; may strip unsafe markup from legacy content.

---

### Entry 9
**Date:** 2026-02-20  
**Area:** Test bootstrap portability  
**Gap:** Test DB seed step used `tsx` CLI directly and failed in constrained environments (`EPERM` IPC pipe), blocking quality gates.  
**Framework change:** Standardize script execution via `node --import tsx ...` for seed/init workflows.  
**Files:** `package.json`  
**Compatibility impact:** Backward-compatible developer experience fix; improves CI/sandbox reliability.

---

### Entry 10
**Date:** 2026-02-20  
**Area:** Style system contract (typography + spacing)  
**Gap:** Runtime used `heading-md/heading-lg` classes without CSS definitions, causing fallback to default `h*` sizing; spacing between grid/action chains was implicit and inconsistent.  
**Framework change:**
  - Add heading utility contract in global CSS (`.heading-xs` ... `.heading-xl`)
  - Define missing `--font-black` token
  - Add explicit adjacency spacing rules for known regression chains
  - Add style contract tests to prevent silent regressions  
**Files:** `app/assets/css/common/text.css`, `app/assets/css/typography/variables.css`, `app/assets/css/ui/forms/inputs.css`, `app/assets/css/layout/sections.css`, `tests/server/styles/style-system-contract.test.ts`  
**Compatibility impact:** Backward-compatible visual stabilization; headings and spacing become deterministic across admin/public screens.

---

### Entry 11
**Date:** 2026-02-20  
**Area:** Style system resilience (table/action layout + button contrast)  
**Gap:** Admin tables could clip action controls near right border (`actions-col` too narrow + fixed table layout), and branded primary buttons could visually blend with background surfaces.  
**Framework change:**
  - Switched data-table default to readable layout (`table-layout: auto`)
  - Added edge-safe table cell padding and minimum width for action columns
  - Aligned action cells to stable no-wrap rows on desktop (mobile wraps in card mode)
  - Introduced contrast-aware primary/secondary button defaults (`light-dark` + explicit border)
  - Added regression checks in style system contract tests  
**Files:** `app/assets/css/ui/content/data-table.css`, `app/assets/css/layout/page.css`, `app/assets/css/ui/forms/buttons.css`, `tests/server/styles/style-system-contract.test.ts`  
**Compatibility impact:** Backward-compatible UX hardening; improves out-of-box readability and action accessibility across admin pages.

---

### Entry 12
**Date:** 2026-02-20  
**Area:** Action controls consistency (admin UX)  
**Gap:** Mixed destructive action patterns (text `Delete` vs trash icon), mixed hover affordances for icon buttons, and row-action wrapping that moved delete controls onto a new line.  
**Framework change:**
  - Standardize destructive table action to icon pattern (`btn btn-icon btn-ghost btn-danger` + `title`/`aria-label`)
  - Enforce desktop no-wrap action rows in table action columns (mobile card mode may wrap)
  - Ensure icon ghost actions have explicit color/background hover feedback
  - Add style contract checks for action-row stability and icon hover affordance  
**Files:** `app/assets/css/ui/content/data-table.css`, `app/assets/css/ui/forms/buttons.css`, `app/pages/admin/assignments.vue`, `app/pages/admin/classrooms.vue`, `app/pages/admin/contacts.vue`, `tests/server/styles/style-system-contract.test.ts`, `CONTRIBUTING.md`  
**Compatibility impact:** Backward-compatible UX consistency hardening; improves scanability and reduces accidental action misses.

---

### Entry 13
**Date:** 2026-02-20  
**Area:** Framework onboarding/documentation architecture  
**Gap:** No role-specific single entrypoint docs; framework understanding is fragmented, forcing developers/agents to piece together rules from many files.  
**Framework change:**
  - Require and maintain 3 canonical entrypoint docs:
    - `docs/entrypoints/framework-development.md`
    - `docs/entrypoints/greenfield-build.md`
    - `docs/entrypoints/brownfield-migration.md`
  - Each entrypoint must include scope, first-step flow, hard rules, and links to canonical deep docs
  - Architecture/process PRs must update the relevant entrypoint in the same PR  
**Files:** `CONTRIBUTING.md` (policy + structure); target framework repo docs listed above  
**Compatibility impact:** Documentation/process change only; high impact on onboarding speed and regression prevention.

---

### Entry 14
**Date:** 2026-02-20  
**Area:** Selection UX pattern (framework-wide)  
**Gap:** Long dropdowns with many compact variants are cognitively heavy and error-prone for high-frequency workflows across domains.  
**Framework change:**
  - Standardize short finite choices to segmented controls
  - Support linked base/modifier segmented pattern for composed values
  - Enforce invalid-state prevention in UI (disable/reset dependent options)
  - Ship reusable segmented control styles in form layer for reuse across modules
  - Keep existing API contracts stable via UI/domain mapping at boundaries  
**Files:** `app/assets/css/ui/forms/inputs.css` (reusable pattern), `app/pages/admin/progress.vue` (reference implementation), `app/utils/grade-scale.ts`, `server/utils/grade-scale.ts`, `server/utils/validation.ts` (domain mapping example)  
**Compatibility impact:** Backward-compatible UX upgrade pattern; no API contract break required.

---

### Entry 15
**Date:** 2026-02-20  
**Area:** Sanitization fallback correctness  
**Gap:** Regex fallback sanitizer removed only `on*=` token and could leave quoted payload in markup (example: `onclick="..."` -> `"..."` residue), producing malformed HTML and incomplete XSS cleanup.  
**Framework change:** Update fallback event-handler regex to remove full attribute expression (`on*="..."`, `on*='...'`, unquoted values).  
**Files:** `server/utils/sanitize.ts`, `tests/server/utils/sanitize.test.ts`  
**Compatibility impact:** Backward-compatible security hardening; only unsafe/malformed inline-handler markup is removed more strictly.

---

### Entry 16
**Date:** 2026-02-20  
**Area:** Framework release sync protocol  
**Gap:** Framework updates were previously applied ad-hoc, making it hard to distinguish "already fixed upstream" vs "project-local workaround".  
**Framework change:** Require release-ledger-based synchronization notes with explicit local adoption status and validation outcomes.  
**Files:** `CONTRIBUTING.md`, `docs/architecture/1.1.0.md` (pattern reference)  
**Compatibility impact:** Process/documentation change only; improves upgrade safety and reduces duplicated fixes.

---

### Entry 17
**Date:** 2026-02-20  
**Area:** Layout policy (website chrome width defaults)  
**Gap:** Mixed header/footer widths and edge-to-edge inner chrome reduced visual consistency; no explicit framework default for website header/footer width strategy.  
**Framework change:**
  - Enforce paired chrome width mode for header+footer (`contained` or `full`)
  - Set default website mode to `contained` (constrained + centered header/footer inner content)
  - Keep `full` as an explicit opt-in for dashboard-like or immersion-first layouts  
**Reasons:**
  - Paired mode removes contradictory layouts and reduces CSS exception debt
  - Constrained default improves scanability/reach on ultrawide desktops
  - Keeping `full` opt-in preserves flexibility for brand-first/full-bleed experiences  
**External references (reviewed 2026-02-20):**
  - USWDS release notes: header/footer max-width tokens default to desktop container and explicitly allow full-page width override (`none`)
  - Bootstrap 5.3 docs: navbar/container architecture recommends full-width chrome with configurable inner container strategy
  - Baymard research: readable content performs best with constrained line length (50-75 chars)  
**Files:** `app/puppet-master.config.ts`, `app/components/organisms/TheHeader.vue`, `app/components/organisms/TheFooter.vue`, `app/assets/css/skeleton/header.css`, `CONTRIBUTING.md`  
**Compatibility impact:** Backward-compatible default-policy clarification; existing projects can keep/choose `full` explicitly.

---

### Entry 18
**Date:** 2026-02-20  
**Area:** Admin navigation layout semantics  
**Gap:** Horizontal admin header navigation was left-biased, while product UX expects centered nav in horizontal mode and top-anchored nav in vertical mode.  
**Framework change:**
  - Center navigation cluster in horizontal admin header
  - Keep logo/actions pinned to edges
  - Preserve top-anchored behavior for vertical sidebar navigation  
**Reasons:**
  - Centered horizontal nav improves orientation and icon discoverability
  - Edge-pinned logo/actions keep stable anchors and reduce accidental pointer travel
  - Top-anchored sidebar matches established app navigation mental model  
**Files:** `app/assets/css/layout/admin-header.css`, `app/layouts/admin.vue`, `CONTRIBUTING.md`  
**Compatibility impact:** Backward-compatible UX/layout correction; no API or schema impact.

---

### Entry 19
**Date:** 2026-02-20  
**Area:** Onepager scrollytelling surface width behavior  
**Gap:** Scrollytelling scenes inherited constrained container width, limiting use of full viewport storytelling space.  
**Framework change:**
  - Keep scrollytelling runtime global for onepager (single top-to-bottom scroll timeline)
  - Standardize reusable section wrapper class `scene-full-bleed`
  - Auto-apply `scene-full-bleed` via `SectionRenderer` so all rendered sections get full-bleed wrapper behavior without manual markup
  - Allow scene wrapper to be full-bleed in onepager mode
  - Keep inner readable surface constrained and centered for text/content balance
  - Treat scene directives/rules as optional local amplifiers, not isolated "point effects"  
**Reasons:**
  - Global timeline preserves narrative continuity across the entire scroll journey
  - Full-bleed wrappers enable art direction/background choreography on wide screens
  - Constrained inner surfaces protect readability and interaction ergonomics
  - Scene-level rules remain additive, so base behavior stays predictable and reusable
  - Class-level standardization reduces template drift and maintenance cost  
**Files:** `app/components/organisms/SectionRenderer.vue`, `app/assets/css/layout/sections.css`, `CONTRIBUTING.md`  
**Compatibility impact:** Backward-compatible visual behavior upgrade; improves storytelling flexibility without sacrificing readability.

---

### Entry 20
**Date:** 2026-02-21  
**Area:** Nuxt list/detail route resolution + i18n test contract sync  
**Gap:** Flat list routes (example `assignments.vue`) can shadow sibling detail routes (`assignments/[id].vue`), and path-scanning tests can keep stale pre-migration file references.  
**Framework change:**
  - Standardize shared-prefix list/detail pages to folder layout (`index.vue` + `[id].vue`)
  - Add explicit contributor rule + troubleshooting notes for route-shadow regressions
  - Align i18n page-scan test paths with migrated route files  
**Files:** `app/pages/admin/assignments/index.vue`, `app/pages/admin/exercises/index.vue`, `app/pages/public-exercises/index.vue`, `tests/server/i18n/student-room-pages.test.ts`, `CONTRIBUTING.md`  
**Compatibility impact:** Backward-compatible routing reliability hardening; prevents shadowed detail views and stale test path regressions.

---

### Entry 21
**Date:** 2026-02-25  
**Area:** Admin settings form UX contract (save semantics + layout consistency)  
**Gap:** Settings save flow could fail silently from user perspective (top scroll/no feedback), and SEO group mixed narrow/wide input widths reduced scanability.  
**Framework change:**
  - Define mandatory dirty-state save behavior for admin settings forms
  - Disable save when no diff and while in-flight
  - Send changed-only payloads for settings updates
  - Enforce toast-based success/error feedback path
  - Codify consistent full-width field rhythm for content-heavy settings groups (SEO/meta baseline)  
**Files:** `app/pages/admin/settings.vue`, `CONTRIBUTING.md`  
**Compatibility impact:** Backward-compatible UX hardening; no API contract break.

---

### Entry 22
**Date:** 2026-02-25  
**Area:** Contact notifications secret-source contract (Telegram)  
**Gap:** Telegram bot token in `.env` required deploy-time secret updates for routine admin operations and mixed operational ownership (developer-only env edit for a settings-level behavior).  
**Framework change:**
  - Move Telegram bot token to admin settings as secret key (`contact.telegramBotToken`, `type: password`, `public: false`)
  - Keep `TELEGRAM_CHAT_ID` in env/runtime as infrastructure-owned value
  - Document split ownership contract clearly across setup/deploy docs  
**Files:** `app/puppet-master.config.ts`, `server/utils/telegram.ts`, `server/utils/env.ts`, `nuxt.config.ts`, `deploy.yml`, `.env.example`, `README.md`, `docs/reference/configuration.md`, `docs/guides/getting-started.md`, `docs/operations/deployment.md`  
**Compatibility impact:** Configuration contract change; existing projects must migrate `TELEGRAM_BOT_TOKEN` from env into admin settings.

---

### Entry 23
**Date:** 2026-02-25  
**Area:** Sensitive settings auditing policy  
**Gap:** Sensitive settings updates lacked a standardized, secret-safe audit detail contract for security review and incident tracing.  
**Framework change:**
  - Expand settings-update audit details for sensitive keys
  - Include policy-level metadata only (for example `telegramBotTokenConfigured`, registration policy flags)
  - Prohibit raw secret values in audit payloads  
**Files:** `server/api/admin/settings.put.ts`, `tests/api/admin-settings.test.ts`  
**Compatibility impact:** Backward-compatible security hardening; improves auditability without exposing secrets.

---

### Entry 24
**Date:** 2026-02-25  
**Area:** Settings visibility contract + regression guards  
**Gap:** No explicit regression contract proving secret settings remain admin-only while public settings endpoint stays sanitized.  
**Framework change:**
  - Codify visibility rule: secret settings may be available in admin settings endpoint, never in public settings endpoint
  - Add regression tests for admin/public split on Telegram token setting  
**Files:** `tests/api/admin-settings.test.ts`, `server/api/admin/settings.get.ts`, `server/api/settings/index.get.ts`, `server/utils/site-settings.ts`  
**Compatibility impact:** Backward-compatible contract enforcement; prevents accidental secret leakage in future refactors.

---

### Entry 25
**Date:** 2026-02-25  
**Area:** Contribution traceability record (`PM-2026-0225-001`)  
**Gap:** Telegram secret-source migration initially lived in a standalone contribution note file (`.pm-contribution.md`), creating duplicated documentation sources.  
**Framework change:**
  - Consolidate contribution metadata and outcomes into canonical `CONTRIBUTING.md` Framework Gap Log
  - Keep one source of truth for framework-level change history
  - Capture migration impact + validation evidence in the same log stream  
**Files:** `CONTRIBUTING.md`  
**Compatibility impact:** Documentation/process cleanup only.

**Validation evidence for record `PM-2026-0225-001`:**
1. `npm run lint`
2. `npm run build`
3. `npm run test:run`
4. `DATABASE_URL=./data/sqlite.test.db npx vitest run tests/api/admin-settings.test.ts --maxWorkers=1 --hookTimeout=180000`

---

### Entry 26
**Date:** 2026-02-25  
**Area:** Password setting storage/read policy hardening  
**Gap:** Secret settings contract did not enforce encryption-at-rest and could not guarantee consistent secure persistence semantics across future password-type keys.  
**Framework change:**
  - Add framework-level secret serialization contract for `type: password` settings using encrypted-at-rest storage format (`enc:v1:` envelope)
  - Keep API/UI contracts stable by decrypting on read in server runtime helpers
  - Preserve admin visibility policy (master raw value, admin masked value) without exposing encrypted payloads to consumers
  - Keep audit payloads secret-safe (no raw secret values)  
**Files:** `server/utils/secrets.ts`, `server/utils/site-settings.ts`, `server/utils/env.ts`, `server/database/seed.ts`, `server/api/auth/register.post.ts`, `server/utils/telegram.ts`, `tests/api/admin-settings.test.ts`  
**Compatibility impact:** Compatible runtime read path for legacy plaintext rows; seed now migrates password settings to encrypted format.

---

### Entry 27
**Date:** 2026-02-25  
**Area:** Admin API authorization contract consistency  
**Gap:** Settings admin endpoints relied on middleware-only admin enforcement while most admin handlers use explicit in-handler role guards (`requireAdmin`/`requireMaster`), creating mixed enforcement style.  
**Framework change:**
  - Enforce explicit `requireAdmin(...)` in both `GET /api/admin/settings` and `PUT /api/admin/settings`
  - Add regression coverage proving authenticated non-admin (`editor`) receives `403` on settings admin endpoints  
**Files:** `server/api/admin/settings.get.ts`, `server/api/admin/settings.put.ts`, `tests/api/protected-routes.test.ts`  
**Compatibility impact:** No functional change for valid admin/master callers; explicit contract prevents drift if middleware scope changes.

---

### Entry 28
**Date:** 2026-02-25  
**Area:** Contributor DX for targeted verification loops  
**Gap:** Contributors needed ad-hoc `vitest` invocations for single-file runs, leading to inconsistent local command patterns for API vs non-API checks.  
**Framework change:**
  - Add focused scripts:
    - `npm run test:file -- <path...>`
    - `npm run test:api:file -- <path...>`
  - Document targeted script usage in contribution testing expectations  
**Files:** `package.json`, `CONTRIBUTING.md`  
**Compatibility impact:** No runtime impact; faster contributor validation loops.

---

### Entry 29
**Date:** 2026-02-25  
**Area:** Admin API authorization contract consistency (global)  
**Gap:** Many admin route handlers relied only on middleware-level protection, while others enforced explicit in-handler role guards.  
**Framework change:**
  - Standardized explicit `requireAdmin(event.context.user?.role)` guard on all remaining `server/api/admin/*` handlers that lacked `requireAdmin`/`requireMaster`
  - Preserved stricter existing role gates (for example master-only routes) and existing permission checks  
**Files:** `server/api/admin/**/*.ts`  
**Compatibility impact:** No change for authenticated admin/master flows; explicit guard now prevents policy drift if middleware behavior changes.

---

### Entry 30
**Date:** 2026-02-25  
**Area:** Utility class contract automation  
**Gap:** Undefined utility classes in Vue templates were only partially covered by targeted tests.  
**Framework change:**
  - Added automated utility-class audit script (`audit:utility-classes`)
  - Enabled CI fail gate for undefined utility classes (`audit:utility-classes:ci`)
  - Added missing utility classes/aliases to satisfy the contract (`gap-sm`, `input-sm`, `text-error`, `text-warning`, `textarea`)  
**Files:** `scripts/audit-utility-classes.mjs`, `.github/workflows/ci.yml`, `package.json`, `app/assets/css/common/flexbox.css`, `app/assets/css/common/text.css`, `app/assets/css/ui/forms/inputs.css`  
**Compatibility impact:** Backward-compatible; improves CSS contract safety and catches future utility drift automatically.

---

## Categories Summary (Entries 1-30)

| Category | Count | Entry Numbers |
|----------|-------|---------------|
| Security | 6 | 8, 15, 22, 23, 24, 26 |
| Test Infrastructure | 5 | 1, 9, 10, 11, 12 |
| Documentation | 4 | 4, 13, 16, 25 |
| CSS/Styling | 7 | 3, 5, 10, 11, 12, 17, 18 |
| UX/UI | 5 | 14, 19, 20, 21, 29 |
| Authorization/RBAC | 4 | 27, 28, 29, 30 |
| Configuration | 3 | 2, 17, 22 |
| Architecture | 2 | 6, 7 |

---

**Note:** Entries 31+ remain active in `CONTRIBUTING.md` Section 13 (Framework Gap Log).
