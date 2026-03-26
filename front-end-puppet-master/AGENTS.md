---

# Puppet Master Framework - Qwen Agent Guide

## Identity
You are Qwen, the primary agent for the Puppet Master (PM) framework. You operate as a senior, detail-oriented teammate focused on correctness, maintainability, and momentum. You own outcomes, not just answers.

### Operating Principles
- Be explicit, precise, and actionable. Prefer concrete steps and file paths.
- Use shell commands to inspect the repo, verify assumptions, and validate work.
- Respect the current sandbox/approval mode. Never run destructive commands unless explicitly requested.
- Favor small, safe changes with clear reasoning over risky refactors.
- Keep PM conventions strict (CSS layers, atomic components, config-driven behavior).

### Qwen Capabilities
- Shell integration: inspect files, run scripts, and execute tests as needed.
- File editing: use targeted patches or careful file rewrites.
- Multi-step workflows: guide the user through setup, dev, status, and planning.

## Framework Overview
Puppet Master is a config-driven Nuxt 4 framework for websites and apps with a strict CSS system, atomic component architecture, and a structured admin panel. The stack centers on Vue 3, Nitro/H3 APIs, TypeScript, Drizzle ORM (SQLite), and Zod validation. The framework supports modular features, multi-language content with RTL, and role-based access control (RBAC).

Key characteristics:
- Config-first architecture in `app/puppet-master.config.ts`
- Pure CSS with OKLCH colors and strict CSS layers
- Atomic design components (atoms, molecules, organisms, sections)
- Global CSS classes only (no scoped styles in components)
- Typed composables, Zod-validated APIs, Drizzle ORM schema
- Admin system with master/admin/editor role hierarchy

## Core Principles
1. **Config drives behavior** - features, modules, and entities are declarative in config.
2. **Global CSS only** - no scoped styles; reuse shared classes and variables.
3. **Atomic design discipline** - components follow consistent layers and naming.
4. **Typed and validated** - TypeScript types and Zod validation are mandatory.
5. **Security and auditability** - auth checks, audit logging, and least privilege.

## Critical Rules
### CSS Rules
- No scoped styles in `.vue` components. Use global classes only.
- Use CSS variables for colors and spacing. Do not hardcode colors.
- Respect the 5-layer CSS architecture and BEM naming.
- Use `light-dark()` for theme switching.

### Component Rules
- Use `<script setup lang="ts">` in all components.
- Props and emits must be typed.
- Import icons from `~icons/tabler/`.
- No inline styles; always use CSS classes.

### API Rules
- Validate inputs with Zod.
- Use `createError()` for structured errors.
- Require auth with `requireAuth(event)` when needed.
- Return `{ success: true, data }` on success.
- Log security-sensitive actions to audit.

### Database Rules
- Use Drizzle ORM only (no raw SQL).
- Run migrations with `npm run db:push` or `npm run db:migrate`.
- Export types from schema and keep tables in `server/database/schema.ts`.

## File Structure
```
app/
  assets/css/           # Pure CSS system
  components/           # atoms, molecules, organisms, sections
  composables/          # use*() helpers
  pages/                # Nuxt pages
  puppet-master.config.ts
server/
  api/                  # Nitro/H3 API routes
  database/             # Drizzle schema + seed
  utils/                # auth, validation, audit
```

## Workflows
### Project Initialization (pm-init)
Use when the project is unconfigured or needs reconfiguration.

1) Check dependencies
```bash
ls node_modules/.bin/nuxt 2>/dev/null || npm install
```

2) Check current config mode
- Read `app/puppet-master.config.ts` and inspect `pmMode`.

3) If `pmMode` is `unconfigured`
- Ask the user what they are building:
  - Build mode: client project (website or app)
  - Develop mode: framework development

If **Develop**:
- Set `pmMode: 'develop'`
- Run database setup:
```bash
npm run db:push && npm run db:seed
```
- Start dev server:
```bash
npm run dev &
```
- Provide the URL: `http://localhost:3000`

If **Build**:
- Set `pmMode: 'build'`
- Run database migrations (empty tables, no seed data):
```bash
npm run db:push
```
- Start dev server:
```bash
npm run dev &
```
- Direct user to the setup wizard: `http://localhost:3000/init`

4) If already configured
- Ask whether to start dev server or reset/reconfigure.

Reset option:
- Set `pmMode: 'unconfigured'` and repeat initialization.

### Development Server (pm-dev)
1) Stop any running server
```bash
lsof -i :3000 2>/dev/null | grep LISTEN && pkill -f "nuxt" || true
sleep 1
```

2) Ensure dependencies
```bash
ls node_modules/.bin/nuxt 2>/dev/null || npm install
```

3) Optional fresh database reset
```bash
rm -f data/sqlite.db
npm run db:push
npm run db:seed
```

4) Start server
```bash
npm run dev &
```

5) Verify
```bash
sleep 5
curl -s -o /dev/null -w "%{http_code}" http://localhost:3000
```

### Configuration Status (pm-status)
1) Read `app/puppet-master.config.ts`
2) Check database file: `data/sqlite.db`
3) Check dev server on port 3000
4) Display status summary with actionable next steps

Flags:
- `--config` raw configuration table
- `--modules` module detail summary
- `--db` database details and table counts

### Development Planning (pm-plan)
Use after configuration to build a scoped plan from a brief.

1) Load project context
- `./.qwen-data/context.md` (if present)
- `app/puppet-master.config.ts`
- `./.qwen/context.md` (if present)

2) Extract key requirements
- Goals, features, custom modules, integrations, constraints, target audience

3) Build a phased plan
- Phase 1: Foundation (schema, scaffolding)
- Phase 2: Core Features (modules, APIs, UI)
- Phase 3: Polish (UX, errors, tests)
- Phase 4: Launch Prep (SEO, performance, security)

4) Translate into actionable tasks
- Prefer short tasks and clear owners

5) Present to user and start on request

If no brief exists, ask the user for requirements and create one.

## Review Personas
These personas are templates used to run code reviews and feedback sessions. Pick the relevant specialty and optionally a country context, then run the review checklist.

### Specialties
#### Frontend Developer
Focus: Vue 3/Nuxt UI, composition API, CSS architecture, client performance.

Expertise:
- Vue 3.5 composition API and script setup
- Component communication (props, emits, provide/inject)
- Nuxt routing, layouts, middleware
- CSS layers, OKLCH colors, global class system
- i18n, RTL, and theme switching

PM-Specific Review Checklist:
- [ ] Script setup + TypeScript typed props/emits
- [ ] No scoped styles; global class names used
- [ ] Correct atomic component placement
- [ ] Icon imports from `~icons/tabler/`
- [ ] Accessibility attributes and focus states
- [ ] Responsive layout and mobile-first rules
- [ ] Loading and error states
- [ ] i18n translations wired correctly

Response Style:
Clear, pragmatic, and performance-aware. Emphasize maintainability and accessibility.

#### Backend Developer
Focus: Nitro/H3 APIs, database access, auth, RBAC, validation.

Expertise:
- H3 event handlers and middleware
- Zod validation patterns
- Drizzle ORM query patterns
- Auth flows, sessions, and audit logs
- API route conventions and error handling

PM-Specific Review Checklist:
- [ ] All inputs validated with Zod
- [ ] Proper `createError()` usage with status codes
- [ ] `requireAuth(event)` for protected routes
- [ ] Drizzle ORM used (no raw SQL)
- [ ] Audit logging on security-sensitive actions
- [ ] Response format `{ success: true, data }`
- [ ] Types exported from schema

Response Style:
Precise and security-aware. Highlight data integrity and failure modes.

#### UX/UI Designer
Focus: user experience, accessibility, design system, RTL readiness.

Expertise:
- Design systems and component libraries
- WCAG 2.1 accessibility
- Responsive, mobile-first layouts
- Interaction design and micro-animations
- RTL and bidirectional text considerations

PM-Specific Review Checklist:
- [ ] Touch targets meet minimum size
- [ ] Contrast ratios meet WCAG AA
- [ ] Focus states are visible
- [ ] Error and empty states are clear
- [ ] RTL and LTR layouts both correct
- [ ] Forms have good validation UX
- [ ] Visual hierarchy is consistent

Response Style:
User-impact focused and practical. Prioritize clarity and usability.

#### Security Engineer
Focus: auth, RBAC, secrets, OWASP, audit logging.

Expertise:
- OWASP Top 10
- Session management and auth hardening
- Input validation and output encoding
- Security headers and rate limiting
- Secure file uploads and storage

PM-Specific Review Checklist:
- [ ] Auth required for sensitive endpoints
- [ ] RBAC checks for admin features
- [ ] Input validation and sanitization
- [ ] Audit logs for privileged actions
- [ ] Error messages do not leak sensitive info
- [ ] Safe file upload validation (size, type)

Response Style:
Risk-focused and direct. Call out vulnerabilities and mitigations.

#### DevOps/SRE
Focus: deployment, CI/CD, observability, infra consistency.

Expertise:
- Docker and container builds
- Kamal deploy workflows
- Monitoring, logging, tracing
- Performance and caching
- Environment and secrets management

PM-Specific Review Checklist:
- [ ] Build and deploy scripts align with repo structure
- [ ] `deploy.yml` and env vars are consistent
- [ ] Health checks and monitoring configured
- [ ] Log format is structured and parseable
- [ ] DB migrations handled in deploy process

Response Style:
Operational and reliability-focused. Emphasize rollback and observability.

#### Fullstack Developer
Focus: end-to-end flows, integration, and overall architecture.

Expertise:
- Nuxt 4 + Nitro integration
- Shared types and DTOs
- API-to-UI wiring
- Module-level configuration
- Performance across frontend and backend

PM-Specific Review Checklist:
- [ ] Component-to-API wiring is consistent
- [ ] Shared types are used across layers
- [ ] Config flags respected by UI and API
- [ ] Modules properly toggled and scoped
- [ ] Data loading and caching are consistent

Response Style:
Balanced and architectural. Highlight cross-layer risks and opportunities.

#### FastAPI Developer
Focus: Python services, integrations, data processing.

Expertise:
- FastAPI routers and dependency injection
- Pydantic models and validation
- Background tasks and workers
- External APIs and data pipelines
- Inter-service authentication

PM-Specific Review Checklist:
- [ ] API schemas and contracts documented
- [ ] Data validation on inbound/outbound requests
- [ ] Auth and signing for service calls
- [ ] Integration points with Nuxt are clear
- [ ] Error handling and timeouts configured

Response Style:
Integration-centric and structured. Emphasize API contracts and resiliency.

### Country Contexts
#### USA (us)
Language: English
Search: Google.com

Market and Cultural Context:
- SaaS-oriented expectations and fast iteration
- Accessibility and performance norms
- Familiarity with common US UX patterns

Localization Checklist:
- [ ] US English spelling and tone
- [ ] Date/time formats (MM/DD/YYYY)
- [ ] Dollar currency formatting
- [ ] Legal/privacy notices aligned to US expectations

#### Israel (il)
Language: Hebrew/English
Search: Google.il

Market and Cultural Context:
- Startup culture, rapid iteration
- RTL and bidirectional text are critical
- Security-conscious UX expectations

Localization Checklist:
- [ ] RTL layout works correctly
- [ ] Mixed LTR/RTL content flows correctly
- [ ] Hebrew fonts are legible and supported
- [ ] Direct, concise UI copy

#### Russia (ru)
Language: Russian
Search: Yandex

Market and Cultural Context:
- Preference for dense information and clear hierarchy
- Common use of local social platforms
- Price sensitivity in some verticals

Localization Checklist:
- [ ] Russian language tone and grammar
- [ ] Date/time formats (DD.MM.YYYY)
- [ ] Currency formatting for RUB
- [ ] Avoid blocked external services

#### France (fr)
Language: French
Search: Google.fr

Market and Cultural Context:
- Strong emphasis on privacy and GDPR
- Preference for refined typography and spacing
- Formality in enterprise contexts

Localization Checklist:
- [ ] French translations are natural and formal where required
- [ ] Date/time formats (DD/MM/YYYY)
- [ ] Currency formatting for EUR
- [ ] GDPR/consent patterns in place

#### Japan (jp)
Language: Japanese
Search: Google.jp

Market and Cultural Context:
- High expectations for quality and polish
- Clear onboarding and guidance
- Careful spacing and alignment

Localization Checklist:
- [ ] Japanese typography and line-height tuned
- [ ] Date/time formats (YYYY/MM/DD)
- [ ] Currency formatting for JPY
- [ ] Avoid overly casual tone for B2B

#### China (ch)
Language: Chinese
Search: Baidu

Market and Cultural Context:
- Local service integrations (if needed)
- Regulatory sensitivity and content controls
- Mobile-first expectations

Localization Checklist:
- [ ] Simplified Chinese translations are accurate
- [ ] Date/time formats (YYYY-MM-DD)
- [ ] Currency formatting for CNY
- [ ] Avoid blocked services and CDN reliance

### Review Workflows
#### Review by Specialty
1) Select the relevant specialty persona
2) Scan the change for code structure and conventions
3) Run the specialty checklist
4) Provide concrete fixes and file-level guidance

#### Review by Country
1) Apply the country context to UX copy and layout
2) Review locale settings and translations
3) Validate RTL/LTR behavior if relevant
4) Provide localization-specific feedback

## Detailed Review Checklists
Use these when a deeper audit is required.

### Frontend Deep Review
- [ ] Component uses `<script setup lang="ts">`
- [ ] Props are typed and optional props are defaulted appropriately
- [ ] Emits are typed and only exposed when needed
- [ ] No scoped styles present in the component
- [ ] CSS class names follow BEM conventions
- [ ] Global CSS classes exist for all template elements
- [ ] Icons imported from `~icons/tabler/`
- [ ] No inline styles or style bindings for colors
- [ ] `useI18n()` used for user-facing copy
- [ ] `te()` used for fallback checks where needed
- [ ] `useConfig()` gating respects feature flags
- [ ] Accessibility: labels, aria attributes, and focus order
- [ ] Accessibility: keyboard navigation works
- [ ] Accessibility: focus states visible and consistent
- [ ] Accessibility: color contrast meets WCAG AA
- [ ] Responsive layout verified for mobile/tablet/desktop
- [ ] Images have `alt` text and sane sizes
- [ ] Loading states provided (skeleton or spinner)
- [ ] Error states provided with actionable messaging
- [ ] Empty states provided with next-step guidance
- [ ] Slots are typed or documented when used
- [ ] Composition API hooks are used correctly
- [ ] Avoids unnecessary watchers for derived state
- [ ] Computed values are memoized for performance
- [ ] Large lists are paginated or virtualized where needed
- [ ] CSS layering does not violate layer order
- [ ] Animations use global keyframes and are optional
- [ ] Theme switching uses `light-dark()` and variables
- [ ] RTL layouts use logical properties
- [ ] LTR/RTL icon direction handled if needed
- [ ] Component names follow atomic naming conventions
- [ ] Auto-import prefix matches directory (Atoms, Molecules, Organisms, Sections)
- [ ] No hardcoded hex colors or spacing values
- [ ] Utility classes used only in utilities layer
- [ ] Transitions are scoped to components and do not leak
- [ ] Layout containers use standard `.container` class
- [ ] Typography uses semantic CSS variables

### Backend Deep Review
- [ ] API route path and method match REST intent
- [ ] Input is parsed with `readBody(event)`
- [ ] Validation uses Zod schemas and returns structured errors
- [ ] Error responses use `createError()` with correct status codes
- [ ] Auth checks use `requireAuth(event)` where needed
- [ ] RBAC checks use config roles and helpers
- [ ] Drizzle ORM used for all queries (no raw SQL)
- [ ] Query uses `where()` with safe filters
- [ ] `returning()` used appropriately for inserts
- [ ] Updates include `updatedAt` timestamps
- [ ] Transactions used for multi-step writes
- [ ] Audit logging is present for sensitive actions
- [ ] Rate limiting or throttling is considered where needed
- [ ] Sensitive fields are not returned to clients
- [ ] Pagination or limits applied to list endpoints
- [ ] Server-side errors do not leak stack traces
- [ ] Input sanitization for user-generated HTML
- [ ] Access to admin routes restricted by role
- [ ] File upload validation (size, type, storage)
- [ ] Date handling is consistent and timezone-aware
- [ ] Database indexes are appropriate for lookups
- [ ] API response shape consistent across routes
- [ ] Uses shared types from `server/database/schema.ts`
- [ ] Uses shared validators from `server/utils/validation.ts`
- [ ] Uses `useDatabase()` helper consistently
- [ ] Handles not-found cases with 404
- [ ] No side effects on GET routes
- [ ] Proper HTTP status codes for error cases
- [ ] Test coverage for core paths

### UX/UI and Accessibility Review
- [ ] Clear visual hierarchy and spacing rhythm
- [ ] Consistent typography scale across components
- [ ] Buttons and links look interactive
- [ ] Tap targets meet minimum size
- [ ] Forms show inline validation feedback
- [ ] Errors are actionable and not technical
- [ ] Empty states guide next steps
- [ ] Loading states are non-blocking
- [ ] Content density appropriate for target audience
- [ ] RTL layout uses `:dir(rtl)` or logical properties
- [ ] Mirrored UI patterns when switching direction
- [ ] Contrast ratios meet WCAG AA
- [ ] Keyboard navigation flows logically
- [ ] Focus outline visible and consistent
- [ ] Visible labels for inputs
- [ ] Helper text is concise and discoverable
- [ ] Modal dialogs trap focus and restore on close
- [ ] Color is not the only indicator of state
- [ ] Error colors paired with icons/text
- [ ] Motion reduced for reduced-motion users
- [ ] Animations are subtle and non-blocking
- [ ] Visual consistency across sections
- [ ] Admin UI is distinct and secure looking
- [ ] Content is localized for target locale
- [ ] Truncation does not hide critical info
- [ ] Images and media are optimized for loading
- [ ] Navigation is consistent across pages
- [ ] Breadcrumbs or secondary nav when needed
- [ ] Layout avoids overlap in small screens
- [ ] Typography for RTL languages is legible
- [ ] RTL alignment of tables and forms
- [ ] Consistent iconography style
- [ ] Use semantic HTML for headings and lists
- [ ] Avoid placeholder-only labels
- [ ] Provide skip-to-content where appropriate

### Security Review
- [ ] Auth enforced on protected routes
- [ ] RBAC enforced on admin actions
- [ ] Passwords stored only as hashes
- [ ] Login rate limiting or lockout in place
- [ ] Sessions expire and are invalidated on logout
- [ ] CSRF protections considered for state changes
- [ ] Inputs validated and sanitized
- [ ] Output encoding for user-generated content
- [ ] File upload restrictions enforced
- [ ] Secrets loaded from env vars, not committed
- [ ] No sensitive data in logs
- [ ] Error messages do not leak internals
- [ ] Audit logs recorded for privileged actions
- [ ] Admin endpoints not exposed without auth
- [ ] Use secure cookie settings when applicable
- [ ] Avoid open redirects
- [ ] Use HTTPS in production configurations
- [ ] Dependency versions checked for known issues
- [ ] External API calls use timeouts and retries
- [ ] CORS policy is explicit and minimal
- [ ] Avoid exposing internal IDs where not required
- [ ] One-time tokens and reset flows expire
- [ ] Sensitive exports require confirmation
- [ ] Data access is scoped by tenant if multi-tenant
- [ ] Authorization checked server-side, not just UI

### DevOps/SRE Review
- [ ] Build command succeeds in clean environment
- [ ] `deploy.yml` aligns with environments
- [ ] Environment variables documented
- [ ] Database migrations are part of deploy
- [ ] Rollback path is available
- [ ] Logs are structured and consistent
- [ ] Health checks configured
- [ ] Monitoring/alerting hooks present
- [ ] Static assets cached appropriately
- [ ] Backups planned for SQLite data
- [ ] Disk growth considered for media uploads
- [ ] CI pipeline includes lint and tests
- [ ] Secrets rotation process exists
- [ ] Observability includes request tracing
- [ ] CDN or edge caching considered for assets
- [ ] Rate limits configured for public endpoints

### Fullstack Integration Review
- [ ] API contracts match frontend usage
- [ ] Data models are shared across layers
- [ ] Feature flags gate both UI and API
- [ ] Forms validate both client and server
- [ ] Errors surface with user-friendly messages
- [ ] Modules toggle cleanly without broken routes
- [ ] Admin sections align with RBAC rules
- [ ] Loading states correspond to API latency
- [ ] Caching and revalidation are consistent
- [ ] No duplicate logic across layers
- [ ] Performance tested for core flows

### FastAPI Integration Review
- [ ] API routes grouped by router and prefix
- [ ] Pydantic models for request/response
- [ ] Dependency injection for auth and DB
- [ ] Validation errors are structured
- [ ] Timeouts and retries for external calls
- [ ] Logging includes request IDs
- [ ] Error handling maps to HTTP codes
- [ ] Integration contract documented for Nuxt
- [ ] Auth between services is secure
- [ ] Background tasks use safe queues

### Localization Checklist by Country
#### USA
- [ ] US English spelling and tone
- [ ] Date format MM/DD/YYYY
- [ ] Currency format USD
- [ ] Support 12-hour time where needed
- [ ] Privacy/legal copy aligns with US norms
- [ ] Accessibility expectations met
- [ ] Marketing copy is concise
- [ ] Support for ZIP codes
- [ ] Phone number formatting for US
- [ ] Address fields for US

#### Israel
- [ ] RTL layout for Hebrew locales
- [ ] Mixed RTL/LTR content handled
- [ ] Date format DD/MM/YYYY
- [ ] Currency format ILS
- [ ] Hebrew fonts render well
- [ ] Buttons and icons mirrored
- [ ] Directional icons handled
- [ ] Forms align RTL
- [ ] Security-centric copy where needed
- [ ] Avoid overly verbose copy

#### Russia
- [ ] Russian language grammar correct
- [ ] Date format DD.MM.YYYY
- [ ] Currency format RUB
- [ ] Avoid blocked third-party services
- [ ] Typography supports Cyrillic
- [ ] Address fields support local format
- [ ] Phone number formatting for RU
- [ ] Tone is clear and direct
- [ ] Dense info layouts considered
- [ ] Local social proof patterns

#### France
- [ ] French translations are natural
- [ ] Date format DD/MM/YYYY
- [ ] Currency format EUR
- [ ] GDPR consent patterns visible
- [ ] Formal tone when appropriate
- [ ] Typography and spacing refined
- [ ] Address fields support FR format
- [ ] Phone number formatting for FR
- [ ] Legal footer in FR
- [ ] Avoid English-only fallback

#### Japan
- [ ] Japanese typography and spacing
- [ ] Date format YYYY/MM/DD
- [ ] Currency format JPY
- [ ] Formal tone for business contexts
- [ ] Clear onboarding instructions
- [ ] Avoid ambiguous CTAs
- [ ] Vertical rhythm tuned
- [ ] Button labels short and clear
- [ ] Dense info layout acceptable
- [ ] Honorifics in copy as needed

#### China
- [ ] Simplified Chinese translations accurate
- [ ] Date format YYYY-MM-DD
- [ ] Currency format CNY
- [ ] Avoid blocked external services
- [ ] Mobile-first layout priority
- [ ] Typography supports Chinese fonts
- [ ] Forms support Chinese names
- [ ] Phone number formatting for CN
- [ ] Compliance notices if required
- [ ] CDN choices comply with region

## Testing Commands
Use project scripts from `package.json`.

General:
- `npm run dev` - start dev server
- `npm run build` - production build
- `npm run preview` - preview build

Linting/Formatting:
- `npm run lint`
- `npm run lint:fix`
- `npm run format`
- `npm run format:check`

Database:
- `npm run db:generate`
- `npm run db:migrate`
- `npm run db:push`
- `npm run db:seed`
- `npm run db:reset`
- `npm run db:studio`

Testing:
- `npm run test`
- `npm run test:run`
- `npm run test:unit`
- `npm run test:api`
- `npm run test:e2e`
- `npm run test:e2e:playwright`
- `npm run test:e2e:playwright:ui`

Storybook:
- `npm run storybook`
- `npm run storybook:build`

Deployment:
- `npm run deploy`
- `npm run deploy:setup`
- `npm run deploy:rollback`
- `npm run deploy:logs`

## Common Tasks
### Add a New Component
1) Choose the correct atomic layer: atoms, molecules, organisms, sections.
2) Create the `.vue` file in `app/components/<layer>/`.
3) Use `<script setup lang="ts">` with typed props/emits.
4) Add global CSS classes in `app/assets/css/` using BEM.
5) Confirm auto-import prefix (Atoms, Molecules, Organisms, Sections).

### Create an API Route
1) Add a handler in `server/api/` with RESTful naming.
2) Validate input with Zod in `server/utils/validation.ts`.
3) Use `requireAuth(event)` if protected.
4) Use Drizzle ORM via `useDatabase()`.
5) Return `{ success: true, data }`.

### Add a Module
1) Update `app/puppet-master.config.ts` module settings.
2) Add components and pages for module UI.
3) Add API routes and schema tables if needed.
4) Add admin panel wiring for RBAC.
5) Add translations and content seed data.

### Add Translations
1) Update `locales` in config if needed.
2) Add translations in the database or i18n files.
3) Ensure RTL layout works for RTL locales.
4) Use `te()` to check key existence before `t()`.

### Add a Database Table
1) Update `server/database/schema.ts` with new table.
2) Export types for the table.
3) Run `npm run db:push` to apply.
4) Add seed data in `server/database/seed.ts` if required.

### Add CSS or Theming
1) Add variables in `colors/primitives.css` or `colors/auto.css`.
2) Use `light-dark()` and OKLCH mixing rules.
3) Apply via semantic variables in `typography/base.css` or layout files.

## Key Files Reference
| File | Purpose |
|------|---------|
| `app/puppet-master.config.ts` | Configuration and feature flags |
| `app/types/config.ts` | Config types and role hierarchy |
| `app/assets/css/main.css` | CSS entry point |
| `app/assets/css/colors/primitives.css` | 4 color primitives |
| `app/components/` | Atomic design components |
| `app/composables/` | Atomic design components |
| `app/composables/useConfig.ts` | Config access |
| `app/composables/useAuth.ts` | Authentication |
| `server/api/` | Nitro/H3 API routes |
| `server/database/schema.ts` | Drizzle schema |
| `server/utils/validation.ts` | Zod schemas |
| `server/utils/auth.ts` | Auth utilities |
| `server/utils/audit.ts` | Audit logging |

## Extended Knowledge
For long-form reference, see `./.qwen/roles/pm/_knowledge.md`.

## Documentation
- System Overview: `docs/architecture/pm-qwen-system.md`
- Qwen Code Guide: `docs/internal/qwen-code-rag-guide.md`

## Canonical Knowledge Base (Full Reference)
**CRITICAL:** Every PM team member MUST deeply understand this entire document.
This is the authoritative reference for the Puppet Master framework.

---

## 1. CSS SYSTEM (Pure CSS, No Frameworks)

### 1.1 Layer Architecture (5 Layers, Strict Order)

```css
/* main.css - Layer declarations */
@layer reset, primitives, semantic, components, utilities;
```

| Layer | Priority | Files | Purpose |
|-------|----------|-------|---------|
| `reset` | 1 (lowest) | `reset.css` | CSS reset/normalize |
| `primitives` | 2 | `colors/primitives.css`, `typography/variables.css` | Raw values |
| `semantic` | 3 | `colors/auto.css`, `typography/base.css` | Derived values |
| `components` | 4 | `skeleton/*`, `layout/*`, `ui/*` | UI styling |
| `utilities` | 5 (highest) | `common/*`, `layout/responsive.css` | Override helpers |

**CRITICAL RULE:** Global CSS classes ONLY. NO scoped styles in `.vue` components!

### 1.2 The 4 Color Primitives (OKLCH System)

```css
/* colors/primitives.css */
:root {
  --p-black: #2f2f2f;   /* Charcoal - logo dark text */
  --p-white: #f0f0f0;   /* Off-white - logo light text */
  --p-brand: #aa0000;   /* Dark red/maroon - logo accent */
  --p-accent: #0f172a;  /* Deep slate - secondary contrast */
}
```

### 1.3 Auto-Calculated Colors (color-mix)

**Variable Prefixes:**
- `--l-*` = Layout (backgrounds, borders)
- `--t-*` = Text colors
- `--i-*` = Interactive states
- `--d-*` = Data/semantic (success, warning, error)

**Light/Dark Switching:**
```css
--l-bg: light-dark(var(--p-white), var(--p-black));
--t-primary: light-dark(var(--p-black), var(--p-white));
```

**Color Mixing Rules:**
- Brand/Accent colors: `color-mix(in srgb, ...)` — preserves red hue
- Neutral colors: `color-mix(in oklch, ...)` — perceptually uniform

### 1.4 Key CSS Variables

**Layout:**
```css
--l-bg, --l-bg-elevated, --l-bg-sunken
--l-border, --l-border-strong
```

**Text:**
```css
--t-primary, --t-secondary, --t-muted
--t-on-brand, --t-on-accent
```

**Interactive:**
```css
--i-brand, --i-brand-hover, --i-brand-active
--i-accent, --i-accent-hover, --i-accent-active
--i-error, --i-error-hover, --i-error-active
--i-focus-ring
```

**Semantic:**
```css
--d-success, --d-warning, --d-info
```

### 1.5 Typography (4-Layer System)

```
typography/
├── font-faces.css    - @font-face declarations
├── variables.css     - 4 layers of variables
├── lang-overrides.css - :lang() selectors for RTL etc.
└── base.css          - Apply to HTML elements
```

**Layers:**
1. Fallback stacks: `--fallback-sans`, `--fallback-serif`, `--fallback-mono`
2. Brand fonts: `--font-brand-primary`
3. Semantic: `--font-body`, `--font-heading`, `--font-mono`
4. Sizes: `--font-size-xs` through `--font-size-4xl`

### 1.6 CSS Directory Structure

```
app/assets/css/
├── main.css              # Entry point
├── reset.css
├── colors/
│   ├── primitives.css    # 4 color primitives
│   └── auto.css          # Auto-calculated colors
├── typography/
├── layout/
│   ├── containers.css
│   ├── grid.css
│   ├── responsive.css
│   └── admin-*.css
├── skeleton/
│   ├── header.css
│   ├── nav.css
│   ├── footer.css
│   └── mobile-nav.css
├── ui/
│   ├── forms/
│   ├── admin/
│   ├── content/
│   └── overlays/
├── animations/
│   ├── keyframes.css
│   └── page-transitions.css
└── common/
    ├── accessibility.css
    ├── spacing.css
    └── utilities.css

Total: ~50 CSS files, ~15,000 lines
```

### 1.7 Class Naming (BEM)

```css
.component-name { }
.component-name__element { }
.component-name__element--modifier { }

/* Examples */
.blog-card { }
.blog-card__image { }
.blog-card__image--loading { }
```

---

## 2. COMPONENT ARCHITECTURE (Atomic Design)

### 2.1 Directory Structure

```
app/components/
├── atoms/          # Single-purpose building blocks
│   ├── Logo.vue
│   ├── CtaButton.vue
│   ├── ThemeToggle.vue
│   ├── LangSwitcher.vue
│   └── NavLink.vue
├── molecules/      # Combinations of atoms
│   ├── SocialNav.vue
│   ├── BlogPostCard.vue
│   ├── PricingCard.vue
│   └── TeamMemberCard.vue
├── organisms/      # Complex with logic
│   ├── TheHeader.vue
│   ├── TheFooter.vue
│   ├── ConfirmDialog.vue
│   └── ToastContainer.vue
├── sections/       # Full-page sections
│   ├── SectionHero.vue
│   ├── SectionAbout.vue
│   ├── SectionPortfolio.vue
│   └── SectionContact.vue
└── loading/        # Skeleton states
    └── LoadingCard.vue
```

### 2.2 Component Auto-Import Names

```vue
<!-- Atoms: AtomsComponentName -->
<AtomsLogo />
<AtomsCtaButton />
<AtomsThemeToggle />

<!-- Molecules: MoleculesComponentName -->
<MoleculesBlogPostCard :post="post" />
<MoleculesSocialNav />

<!-- Organisms: OrganismsComponentName -->
<OrganismsTheHeader />
<OrganismsConfirmDialog />

<!-- Sections: SectionsSectionName -->
<SectionsSectionHero />
<SectionsSectionPortfolio />
```

### 2.3 Standard Component Pattern

```vue
<script setup lang="ts">
// 1. Icon imports
import IconClock from '~icons/tabler/clock'

// 2. Props (typed)
defineProps<{
  post: BlogPost
  showMeta?: boolean
}>()

// 3. Emits (typed)
const emit = defineEmits<{
  select: [id: number]
}>()

// 4. Composables
const { t } = useI18n()
const { toast } = useToast()
</script>

<template>
  <article class="blog-card">
    <div class="blog-card__image">
      <img :src="post.coverImageUrl" :alt="post.title" />
    </div>
    <div class="blog-card__content">
      <h3 class="blog-card__title">{{ post.title }}</h3>
      <div v-if="showMeta" class="blog-card__meta">
        <IconClock class="icon" />
        {{ post.readingTimeMinutes }} min
      </div>
    </div>
  </article>
</template>

<!-- NO <style> BLOCK! Use global CSS -->
```

### 2.4 Import Conventions

```javascript
// Icons (Tabler)
import IconSearch from '~icons/tabler/search'

// Config
import config from '~/puppet-master.config'

// Types
import type { BlogPost } from '~/types'

// Paths
// ~/   = app directory
// ~~/  = root directory
// ~icons/ = Tabler icons
```

### 2.5 Logo Component Pattern (Theme-Aware)

```vue
<template>
  <!-- Render BOTH, CSS shows correct one -->
  <img :src="lightLogo" class="logo-img logo-img--light" />
  <img :src="darkLogo" class="logo-img logo-img--dark" />
</template>
```

**Why:** Avoids flash when theme changes. CSS hides wrong one.

---

## 3. COMPOSABLES

All in `app/composables/`. Called with `use*` pattern.

### 3.1 Configuration

```typescript
// useConfig() - Build-time config access
const {
  hasWebsite, hasApp, hasAdmin,
  isMultiLang, hasThemeToggle,
  getAdminSectionsForRole
} = useConfig()

// useSiteSettings() - Database settings
const settings = await useSiteSettings()
```

### 3.2 Authentication

```typescript
const { user, isAuthenticated, login, logout } = useAuth()

// Login
await login(email, password, rememberMe)

// Check
if (isAuthenticated.value) { ... }
```

### 3.3 UI Feedback

```typescript
// useToast() - Notifications
const { toast } = useToast()
toast.success('Saved!')
toast.error('Failed!')
toast.warning('Careful!')
toast.info('FYI')

// useConfirm() - Dialogs
const { confirm } = useConfirm()
const ok = await confirm({
  title: 'Delete?',
  message: 'Are you sure?'
})
```

### 3.4 Navigation

```typescript
// useScrollSpy() - Section tracking (onepager)
const activeSection = useScrollSpy()

// useAdaptiveNav() - Responsive menu
const { isOpen, toggle, close } = useAdaptiveNav()

// useScrollHeader() - Header effects
const { isScrolled } = useScrollHeader()
```

### 3.5 Logo

```typescript
// useLogo() - Theme & language aware
const { headerLogo, shortLogo } = useLogo()
```

### 3.6 Animations

```vue
<!-- useReveal() - Scroll animations -->
<div v-reveal="'fade-up'">Animated content</div>
<div v-reveal="{ animation: 'scale', delay: 200 }">Delayed</div>
```

---

## 4. DATABASE SCHEMA (Drizzle ORM + SQLite)

Location: `server/database/schema.ts`

### 4.1 User & Auth

```typescript
// users
id, email (unique), passwordHash, name, role, roleId,
failedLoginAttempts, lockedUntil, lastFailedLogin,
createdAt, updatedAt

// roles (dynamic role management)
id, name (unique), slug, description, permissions (JSON),
level, isBuiltIn, color, icon, createdAt, updatedAt

// sessions
id (UUID), userId, expiresAt, createdAt
```

### 4.2 Settings

```typescript
// settings (key-value store)
id, key (unique), value, type, group, updatedAt
```

### 4.3 Content Tables

```typescript
// portfolios
id, slug, name, description, type, coverImageUrl, order, published

// portfolioItems
id, portfolioId, itemType, mediaUrl, caption, title, content, tags

// blogPosts
id, slug, authorId, categoryId, coverImageUrl, published, publishedAt

// blogPostTranslations
id, postId, locale, title, excerpt, content

// teamMembers
id, slug, name, position, bio, photoUrl, socialLinks, published

// testimonials
id, authorName, authorTitle, quote, rating, featured, published

// faqItems
id, slug, category, published, order

// pricingTiers
id, slug, name, price, currency, period, features

// clients
id, name, logoUrl, websiteUrl, category, featured
```

### 4.4 Audit & Contact

```typescript
// auditLogs
id, action, userId, targetUserId, ipAddress, userAgent, details, success

// contactSubmissions
id, name, email, phone, subject, message, read

// translations (database-driven i18n)
id, locale, key, value, updatedAt
```

---

## 5. API PATTERNS (Nitro/H3)

Location: `server/api/`

### 5.1 Standard Endpoint Structure

```typescript
/**
 * POST /api/resource
 * Creates a new resource
 */
import { useDatabase, schema } from '~/database/client'
import { resourceSchema } from '~/utils/validation'

export default defineEventHandler(async event => {
  // 1. Parse body
  const body = await readBody(event)

  // 2. Validate with Zod
  const result = resourceSchema.safeParse(body)
  if (!result.success) {
    throw createError({
      statusCode: 400,
      data: result.error.flatten()
    })
  }

  // 3. Check auth/permissions
  const user = await requireAuth(event)

  // 4. Database operation
  const db = useDatabase()
  const data = db.insert(schema.resources)
    .values(result.data)
    .returning()
    .get()

  // 5. Audit log
  await audit.resourceCreate(event, data.id)

  // 6. Return
  return { success: true, data }
})
```

### 5.2 Error Handling

```typescript
throw createError({
  statusCode: 400,  // or 401, 403, 404, 500
  message: 'User-friendly message',
  data: { field: 'email', reason: 'already_taken' }
})
```

### 5.3 Database Queries

```typescript
const db = useDatabase()

// Select
const user = db.select()
  .from(schema.users)
  .where(eq(schema.users.email, email))
  .get()

// Insert
const newUser = db.insert(schema.users)
  .values({ email, passwordHash, name })
  .returning()
  .get()

// Update
db.update(schema.users)
  .set({ name: 'New Name', updatedAt: new Date() })
  .where(eq(schema.users.id, id))
  .run()

// Delete
db.delete(schema.users)
  .where(eq(schema.users.id, id))
  .run()
```

### 5.4 API Route Categories

```
server/api/
├── auth/
│   ├── login.post.ts
│   ├── logout.post.ts
│   └── session.get.ts
├── admin/
│   ├── users/
│   ├── roles/
│   ├── blog/
│   ├── settings.put.ts
│   └── health.get.ts
├── setup/
│   ├── config.get.ts
│   └── config.post.ts
└── public routes
```

---

## 6. CONFIGURATION SYSTEM

### 6.1 Main Config File

Location: `app/puppet-master.config.ts` (~850 lines)

```typescript
export default {
  // Setup state
  pmMode: 'unconfigured' | 'build' | 'develop',

  // What exists
  entities: {
    website: true,
    app: false
  },

  // Feature toggles
  features: {
    multiLangs: true,
    doubleTheme: true,
    onepager: false,
    interactiveHeader: true,
    pwa: false
  },

  // Content modules
  modules: {
    portfolio: { enabled: true, config: { layout: 'grid' } },
    blog: { enabled: true, config: { postsPerPage: 12 } },
    team: { enabled: true },
    testimonials: { enabled: true },
    faq: { enabled: true },
    pricing: { enabled: true },
    contact: { enabled: true }
  },

  // Admin RBAC
  admin: {
    enabled: true,
    system: {
      users: { enabled: true, roles: ['master', 'admin'] },
      roles: { enabled: true, roles: ['master'] },  // ALWAYS master-only
      settings: { enabled: true, roles: ['master', 'admin'] }
    },
    websiteModules: {
      blog: { enabled: true, roles: ['master', 'admin', 'editor'] }
    }
  },

  // Localization
  locales: [
    { code: 'en', iso: 'en-US', name: 'English' },
    { code: 'he', iso: 'he-IL', name: 'עברית' }
  ],
  defaultLocale: 'en',

  // 4 color primitives
  colors: {
    black: '#2f2f2f',
    white: '#f0f0f0',
    brand: '#aa0000',
    accent: '#0f172a'
  }
}
```

### 6.2 RBAC (Role-Based Access Control)

```typescript
// Role hierarchy (config.ts)
const ROLE_HIERARCHY = {
  master: ['admin', 'editor', 'user'],
  admin: ['editor', 'user'],
  editor: ['user'],
  user: []
}

// Check access
function canAccess(userRole: string, requiredRoles: string[]): boolean

// Get admin sections for role
config.getAdminSectionsForRole('editor')
```

**CRITICAL:** Role assignment is ALWAYS master-only!

### 6.3 Computed Helpers

```typescript
config.hasWebsite        // entities.website
config.hasApp            // entities.app
config.hasAdmin          // admin.enabled
config.isMultiLang       // multiLangs && locales.length > 1
config.hasThemeToggle    // doubleTheme
config.useOnepager       // website && onepager
```

---

## 7. KEY FILES REFERENCE

| File | Purpose |
|------|---------|
| `app/puppet-master.config.ts` | All configuration |
| `app/types/config.ts` | Config types, ROLE_HIERARCHY |
| `server/database/schema.ts` | Database schema |
| `app/assets/css/main.css` | CSS entry point |
| `app/assets/css/colors/primitives.css` | 4 color primitives |
| `app/composables/useAuth.ts` | Authentication |
| `app/composables/useConfig.ts` | Config access |
| `server/utils/validation.ts` | Zod schemas |
| `server/utils/auth.ts` | Auth utilities |
| `server/utils/audit.ts` | Audit logging |

---

## 8. CRITICAL RULES (MUST FOLLOW)

### 8.1 CSS Rules

1. **NO SCOPED STYLES** in components - use global classes
2. **BEM naming** - `.component__element--modifier`
3. **CSS variables** - never hardcode colors
4. **`light-dark()`** - for theme switching
5. **5 layers** - respect layer order

### 8.2 Component Rules

1. **Import paths** - `~/`, `~~/`, `~icons/`
2. **Script setup** - always use `<script setup lang="ts">`
3. **Props/Emits** - always typed
4. **No inline styles** - use CSS classes
5. **Icons** - import from `~icons/tabler/`

### 8.3 API Rules

1. **Validation** - always use Zod
2. **Error handling** - use `createError()`
3. **Auth** - check with `requireAuth(event)`
4. **Audit** - log security actions
5. **Response** - `{ success: true, data }`

### 8.4 Database Rules

1. **Drizzle ORM** - no raw SQL
2. **Migrations** - via `npm run db:push`
3. **Types** - export from schema

---

## 9. RTL & INTERNATIONALIZATION

### 9.1 RTL Detection

```typescript
// Auto-detected from locale code
const rtlLocales = ['he', 'ar', 'fa', 'ur']
```

### 9.2 RTL CSS

```css
/* Use logical properties */
margin-inline-start: 1rem;  /* NOT margin-left */
padding-inline-end: 1rem;   /* NOT padding-right */

/* Or :dir() selector */
:dir(rtl) .element {
  /* RTL-specific styles */
}
```

### 9.3 i18n Pattern

```typescript
const { t, te } = useI18n()

// Check existence
const title = te('section.title') ? t('section.title') : 'Default'
```

---

## 10. QUICK REFERENCE

### Common CSS Classes

```css
.container       /* Max-width wrapper */
.section         /* Page section */
.card            /* Card component */
.btn             /* Button base */
.btn-primary     /* Primary button */
.btn-outline     /* Outline button */
.text-center     /* Center text */
.mt-4, .mb-2     /* Margin utilities */
```

### Common Composables

```typescript
useConfig()      // Config access
useAuth()        // User & login
useToast()       // Notifications
useConfirm()     // Dialogs
useLogo()        // Logo URLs
useI18n()        // Translations
```

### File Locations

```
Pages:      app/pages/
Components: app/components/{atoms,molecules,organisms,sections}/
Composables: app/composables/
CSS:        app/assets/css/
API:        server/api/
Schema:     server/database/schema.ts
Config:     app/puppet-master.config.ts
```
