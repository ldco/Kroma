# Changelog

All notable changes to Puppet Master will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.3.0] - 2026-03-07

### Added

#### Strict API Response Envelope (PM-RCV-002)
- Standardized success envelope: `{ success: true, data: { ... } }`
- 96+ API endpoints migrated to strict format
- API envelope audit script (`npm run audit:api-envelope`)
- `apiFetch` composable for automatic envelope unwrapping (`app/composables/apiFetch.ts`)

#### Centralized RBAC Enforcement (PM-RCV-003)
- Section-level authorization for all `/api/admin/*` endpoints
- Route alias support (stats, logs, audit-logs → health section)
- Content admin section grouping (blog, portfolios, team, etc.)
- Enhanced auth middleware with automatic section enforcement

#### Secret Settings Storage (PM-RCV-004)
- AES-256-GCM encryption for sensitive values
- PBKDF2 key derivation (100,000 iterations)
- Production fail-fast without `PM_SETTINGS_ENCRYPTION_KEY`
- Encrypted secret format: `enc:v1:{iv}:{authTag}:{cipher}`

#### S3 Visibility Contract (PM-RCV-005)
- Public mode: Direct S3 URLs via `S3_PUBLIC_URL`
- Private mode: Signed proxy URLs via `/api/media/s3/*`
- HMAC-SHA256 signature verification
- Timing-safe signature comparison

#### Testing Infrastructure (PM-RCV-012)
- Focused test scripts: `npm run test:file -- <path>`
- API test scripts: `npm run test:api:file -- <path>`
- Improved contributor iteration loops

### Changed

#### API Response Format
- All successful API responses now wrap data in `{ success: true, data: ... }` envelope
- Error response format unchanged (backward compatible)
- Client composables (`apiFetch`) auto-handle envelope unwrapping

#### RBAC Behavior
- Authenticated users receive `403` on unauthorized section paths
- Admin navigation requires section-aware permission checks
- UI should redirect before render for unauthorized deep links

### Security

- Encrypted settings storage prevents plaintext sensitive data in database
- S3 private mode prevents direct bucket access (signed proxy URLs)
- RBAC enforcement prevents privilege escalation via direct API calls
- Production encryption key requirement prevents weak defaults
- Environment variables accepted: `PM_SETTINGS_ENCRYPTION_KEY` or `SETTINGS_ENCRYPTION_KEY` (alias)
- S3 signing key: `S3_PROXY_SIGNING_KEY` (optional, falls back to `S3_SECRET_KEY`)

### Documentation

- Release 1.3.0 comprehensive notes (`docs/releases/1.3.0.md`)
- Migration guide with before/after examples
- Contract change matrix for all endpoint types
- Environment variable reference for new features

### Fixed

- API envelope consistency across all endpoint families
- Section-level permission gaps in admin routes
- S3 URL exposure for private bucket configurations

### Breaking Changes

**API Response Format:**
- Client code must access response data via `response.data` instead of top-level
- Example: `const { data: { items } } = await $fetch('/api/admin/contacts')`

**RBAC Enforcement:**
- Users may newly receive `403` on previously accessible paths
- Admin UI should implement section-aware navigation guards

### Upgrade Actions

1. Set `PM_SETTINGS_ENCRYPTION_KEY` in production (32+ characters)
2. Update API client code for envelope format (or use `apiFetch` composable)
3. Verify RBAC behavior for all user roles
4. Run `npm run audit:api-envelope` to validate compliance
5. Run full test suite: `npm run test:run`

---

## [1.3.1] - 2026-03-10

### Fixed

#### Documentation Corrections (No Code Changes Required)

**Empty-State API Response Envelope:**
- Fixed 5 API endpoints returning raw arrays on empty results instead of envelope format
- Affected endpoints: `/api/blog/posts`, `/api/faq`, `/api/features`, `/api/team`, `/api/testimonials`
- Now consistent: all empty responses return `{ success: true, data: [] }`

**Command and Composable References:**
- Fixed `useApiFetch` → `apiFetch` (correct composable name)
- Fixed `node scripts/audit-api-envelope.mjs` → `npm run audit:api-envelope`
- Removed inaccurate "Automated CI validation for envelope compliance" claim

**Security Wording Alignment:**
- Clarified `S3_PROXY_SIGNING_KEY` is optional (falls back to `S3_SECRET_KEY`)
- Documented `SETTINGS_ENCRYPTION_KEY` as valid legacy alias for `PM_SETTINGS_ENCRYPTION_KEY`

### Documentation

- Added comprehensive correction ledger (`docs/releases/1.3.1.md`)
- Updated release index with 1.3.1 as latest (`docs/releases/README.md`)
- Fixed composable/command references in `docs/releases/1.3.0.md`
- Fixed security wording in `docs/releases/1.3.0.md`

### Upgrade Notes

**No breaking changes.** This is a documentation correction patch. Existing 1.3.0 deployments do not require code changes unless:
- Client code handles empty API responses (update to expect envelope format)
- Scripts reference the audit command directly (use `npm run audit:api-envelope`)

---

## [Unreleased]

### Added

#### Two-Factor Authentication (2FA)
- TOTP-based 2FA with QR code setup via `/api/user/2fa/setup`
- Backup codes for account recovery (10 codes per user)
- Rate limiting on all 2FA endpoints:
  - Setup: 10 attempts/hour per user
  - Enable: 5 attempts/15min per user
  - Verify: 5 attempts/15min per IP
  - Disable: 3 attempts/15min per user
- Database migration for 2FA schema (`user_2fa` table, `two_factor_enabled` column)
- Encrypted TOTP secret storage with AES-256

#### API Versioning
- URL path versioning support (`/api/v1/*` routes)
- Accept header versioning (`application/vnd.pm.v1+json`)
- Deprecation headers for sunset versions (`X-API-Deprecated`, `Sunset`)
- Middleware-based URL rewriting to existing handlers
- Version info exported for use in handlers

#### Backup & Recovery
- AES-256-CBC encryption for database backups
- PBKDF2 key derivation (100,000 iterations)
- `--encrypt` flag for manual encryption in backup script
- Auto-encryption when uploading to S3 with `BACKUP_ENCRYPTION_KEY`
- Decryption support in restore script for `.gz.enc` files
- Verification of encrypted backups before restore

#### CI/CD Pipeline
- GitHub Actions workflow (`.github/workflows/ci.yml`) with:
  - Lint & Type Check (ESLint + Nuxi typecheck)
  - Test with Coverage (Codecov/Coveralls integration)
  - Security Scan (npm audit)
  - Build validation with artifact upload
  - Docker image build and caching
  - Visual Regression Tests (Playwright on PRs)
  - Lighthouse CI for performance auditing
  - Production deployment via Kamal (on master push)

#### Error Tracking
- Sentry server plugin (`server/plugins/sentry.ts`):
  - Automatic request context
  - User context from session
  - Sensitive data filtering (passwords, tokens, cookies)
  - 4xx error filtering
  - HTTP, console, and rejection integrations
- Sentry client plugin (`app/plugins/sentry.client.ts`):
  - Vue-specific integrations
  - Browser tracing with Vue Router
  - Component lifecycle tracking
  - Extension URL denial
  - Error filtering (ResizeObserver, network issues)

#### Testing Infrastructure
- Visual regression tests with Playwright (`e2e-playwright/`)
- Accessibility tests with axe-core (WCAG 2.1 AA compliance)
- Responsive design tests across 6 viewports
- Component hover/focus state testing
- 404 page testing

#### Documentation
- Comprehensive CONTRIBUTING.md with:
  - Code of Conduct
  - Development workflow
  - Coding standards (TypeScript, Vue, API, CSS)
  - Commit guidelines (Conventional Commits)
  - Testing guidelines
- Architecture analysis document
- CSS Component Map

### Changed
- Enhanced password validation with strength scoring
- Database backup scripts now support both encrypted and unencrypted formats
- API version middleware now rewrites URLs for transparent versioning
- Improved rate limiter with client IP extraction for proxied environments

### Security
- Password policy enforcement on admin user update flows
- Rate limiting prevents TOTP brute-force attacks (1M combinations)
- Encrypted backups at rest for S3 storage
- CSRF protection on all mutation endpoints
- Sensitive data redaction in Sentry events

---

## [1.1.0] - 2026-02-20

### Exact Ledger (Feature-by-Feature, Fix-by-Fix)

Format used: `Before -> After -> Upgrade impact`.

#### Roles and Permissions APIs

- `RLS-001` Files: `server/api/admin/roles.get.ts`, `server/api/admin/roles.post.ts`  
  Before: Duplicate top-level route handlers existed alongside nested handlers for the same endpoint family.  
  After: Duplicate top-level handlers were removed; canonical handlers are `server/api/admin/roles/index.*.ts`.  
  Upgrade impact: If you customized deleted top-level files, migrate overrides to `index.*` handlers.

- `RLS-002` File: `server/api/admin/roles/index.get.ts`  
  Before: Response payload returned `{ roles }`.  
  After: Response payload now returns `{ success: true, roles }`.  
  Upgrade impact: Strict API clients should accept `success` in addition to `roles`.

- `RLS-003` File: `server/api/admin/roles/index.post.ts`  
  Before: `audit.roleCreate(...)` was called with incomplete arguments.  
  After: Audit call now passes full payload: `event, actorUserId, roleId, roleName, roleSlug`.  
  Upgrade impact: Role creation audit entries now contain complete metadata.

#### FAQ Module Integrity and Sanitization

- `FAQ-001` File: `server/api/admin/faq/index.post.ts`  
  Before: Create flow did not guarantee required slug generation/uniqueness; empty question/answer strings were allowed; writes were unsanitized.  
  After: Added slug generation and uniqueness checks, `min(1)` validation for question/answer, escaping for category/question, sanitization for answer.  
  Upgrade impact: Safer persisted FAQ content and no missing-slug insert failures.

- `FAQ-002` File: `server/api/admin/faq/[id].put.ts`  
  Before: Update flow attempted to write non-existent `updatedAt` on FAQ table; translation writes were unsanitized; empty translation values were allowed.  
  After: Removed invalid `updatedAt` write, added sanitize/escape for writes, tightened validation with `min(1)`.  
  Upgrade impact: Prevents DB write errors and blocks unsafe/empty translation content.

- `FAQ-003` File: `server/api/admin/faq/[id]/translations.put.ts`  
  Before: Endpoint used ad-hoc validation with no strict schema and no "at least one of question/answer" requirement.  
  After: Added Zod schema, refinement requiring at least one updatable field, and sanitize/escape write path.  
  Upgrade impact: Invalid payloads now fail with structured 400 validation errors.

#### Settings, Audit, and Response Accuracy

- `SET-001` File: `server/api/admin/settings.put.ts`  
  Before: Settings key allowlist was hardcoded and could drift from config/UI definitions.  
  After: Allowed keys are now derived from `app/puppet-master.config.ts` (`config.settings`).  
  Upgrade impact: Custom keys must exist in config or writes are rejected.

- `AUD-001` File: `server/api/admin/audit-logs.get.ts`  
  Before: Filter conditions were not consistently applied to both result rows and pagination totals.  
  After: Unified `whereClause` is applied to both count and list queries; response now includes `success: true`.  
  Upgrade impact: Pagination totals now match filtered result sets.

#### Setup and Workflow Selection

- `SETUP-001` File: `server/api/setup/config.post.ts`  
  Before: Setup schema did not include `aiWorkflow`; write path did not validate saved workflow value.  
  After: Added optional `aiWorkflow` (`claude|qwen|codex`) and persisted-value verification with rollback on mismatch.  
  Upgrade impact: Workflow choice is now explicit and validated at setup time.

- `SETUP-002` File: `server/api/setup/config.post.ts`  
  Before: Project brief output path was coupled to `.claude-data`.  
  After: Brief path now resolves by selected workflow (`.claude-data`, `.qwen-data`, `.codex-data`).  
  Upgrade impact: Multi-workflow setups now store metadata in workflow-specific directories.

- `SETUP-003` File: `server/api/setup/config.post.ts`  
  Before: Workflow data directory creation was not guaranteed before writing setup artifacts.  
  After: Added `ensureWorkflowDataDir(...)` and explicit setup failure on init errors.  
  Upgrade impact: Reduced silent setup failures.

- `SETUP-004` Files: `server/api/setup/config.post.ts`, `server/api/setup/import-zip.post.ts`  
  Before: Cleanup blocks used empty `catch {}` patterns.  
  After: Replaced with explicit lint-safe cleanup handling comments.  
  Upgrade impact: No contract change; improved maintainability and lint compliance.

- `SETUP-005` File: `server/api/setup/config.get.ts`  
  Before: Setup summary did not return `aiWorkflow`; brief path resolution was not workflow-aware and path root was incorrect for workflow data reads.  
  After: Returns `aiWorkflow` and reads brief from `process.cwd()/.<workflow>-data/project-brief.md`.  
  Upgrade impact: Setup UI and workflow-aware tooling can reliably load active workflow metadata.

#### Workflow Utility and CLI Commands

- `WFLOW-001` File: `server/utils/workflow-paths.ts` (new)  
  Before: No centralized utility for typed workflow path/directory resolution and validation.  
  After: Added workflow helpers for config/data dirs, current workflow detection, validation, and data-dir bootstrap.  
  Upgrade impact: Setup and workflow-aware APIs/scripts use a single consistent source of truth.

- `WFLOW-002` Files: `scripts/switch-workflow.ts` (new), `package.json`  
  Before: No official command to switch workflow and bootstrap folders.  
  After: Added `npm run workflow:switch -- <claude|qwen|codex>`.  
  Upgrade impact: Safer workflow switching without manual config edits.

- `WFLOW-003` Files: `scripts/workflow-info.ts` (new), `package.json`  
  Before: No official command to inspect workflow state.  
  After: Added `npm run workflow:info` for current workflow and directory diagnostics.  
  Upgrade impact: Faster onboarding/troubleshooting for multi-agent workflows.

#### Config, Types, and Init UI

- `CFG-001` File: `app/puppet-master.config.ts`  
  Before: No top-level `aiWorkflow` config field.  
  After: Added typed `aiWorkflow` field (`'claude' | 'qwen' | 'codex'`).  
  Upgrade impact: Framework workflow selection is now config-driven.

- `CFG-002` File: `app/types/config.ts`  
  Before: No shared `AiWorkflow` type.  
  After: Added exported `AiWorkflow` type for cross-layer typing.  
  Upgrade impact: Consistent workflow typing across app/server/scripts.

- `INIT-001` Files: `app/pages/init.vue`, `app/assets/css/ui/content/init.css`  
  Before: Init wizard had no workflow selector.  
  After: Added Claude/Qwen/Codex selection cards, payload wiring to setup API, and UI styles.  
  Upgrade impact: Workflow is now selected during setup rather than inferred.

#### CSS Rule Compliance and UI Consistency

- `CSS-001` Files: `app/components/atoms/AppImage.vue`, `app/assets/css/ui/content/app-image.css`, `app/assets/css/ui/content/index.css`  
  Before: `AppImage` used a local `<style>` block.  
  After: Styles moved to global CSS and imported via content index.  
  Upgrade impact: Aligns with PM global CSS-only policy.

- `CSS-002` Files: `app/pages/admin/roles.vue`, `app/assets/css/ui/admin/pages.css`  
  Before: Roles page used local styles and non-standard tokens (`--c-success`, `--border-color`, `--bg-subtle`).  
  After: Styles moved to global admin CSS namespace and normalized to semantic tokens (`--d-success`, `--l-border`, `--l-bg-sunken`).  
  Upgrade impact: Better cross-theme consistency and policy compliance.

- `CSS-003` Files: `app/pages/admin/login.vue`, `app/components/organisms/ChangePasswordModal.vue`  
  Before: Inline style attributes controlled error/help presentation.  
  After: Inline styles replaced with semantic utility classes (`text-center`, `mb-4`, `text-danger`).  
  Upgrade impact: Easier override and consistent styling behavior.

- `CSS-004` Files: `app/components/loading/LoadingCard.vue`, `app/components/loading/LoadingText.vue`, `app/assets/css/ui/content/loading-placeholders.css`  
  Before: Loading spacing relied on inline style attributes/bindings.  
  After: Spacing moved to CSS classes (`.placeholder-card__title-placeholder`) and existing container gap.  
  Upgrade impact: No expected behavioral change; improved style maintainability.

#### Tooling and Lint Reliability

- `TOOL-001` File: `vitest.config.ts`  
  Before: Default Vitest config required `html` reporter (can fail if optional packages are missing).  
  After: Default reporter set to `['default']`.  
  Upgrade impact: Reliable test startup in minimal environments.

- `TOOL-002` File: `server/utils/password.ts`  
  Before: Special-character checks used duplicated escape-heavy regex patterns.  
  After: Replaced with reusable `specialCharRegex = /[^A-Za-z0-9]/`.  
  Upgrade impact: Same policy intent, cleaner implementation.

### API Contract Notes

- `GET /api/admin/roles` now includes `success: true`.
- `GET /api/admin/audit-logs` now includes `success: true` and accurate filtered totals.
- `PUT /api/admin/faq/:id/translations` now enforces strict Zod validation and rejects empty payloads.
- `POST /api/setup/config` now accepts optional `aiWorkflow` and returns effective `aiWorkflow`.
- `GET /api/setup/config` now includes `aiWorkflow`.

### Documentation

- Added release index: `docs/releases/README.md`.
- Added detailed release guide: `docs/releases/1.1.0.md`.

---

## [1.0.0] - 2025-01-15

### Added

#### Core Framework
- Nuxt 4 with Vue 3.5 and TypeScript
- Nitro server engine with API routes
- SQLite database with Drizzle ORM
- Config-driven architecture (`puppet-master.config.ts`)

#### Authentication & Authorization
- Session-based authentication with HTTP-only cookies
- Role-based access control (RBAC): user, editor, admin, master
- Account lockout after 5 failed attempts (30min lock)
- Password hashing with scrypt (Node.js native crypto)
- CSRF protection (double-submit cookie pattern)

#### Admin Panel
- Material Design 3 inspired UI
- Responsive navigation:
  - Desktop: Full sidebar
  - Tablet: Navigation rail
  - Mobile: Bottom navigation
- Config-driven module system
- User management with role assignment
- Translation management with cache invalidation
- Settings management
- Health monitoring dashboard

#### Content Modules
- Blog with categories and tags
- Portfolio with media items (images, videos)
- Team members with translations
- Testimonials
- FAQ sections
- Pricing tiers
- Features showcase

#### Internationalization
- Database-driven translations
- Multi-language support (en, ru, he)
- RTL support for Hebrew/Arabic
- Translation caching (5-minute TTL)
- Admin-editable translations

#### CSS Architecture
- 5-layer CSS system:
  1. reset - CSS normalization
  2. primitives - Raw values (colors, fonts)
  3. semantic - Calculated values
  4. components - UI styling
  5. utilities - Override helpers
- OKLCH color space with auto-calculated variations
- Light/dark mode with `light-dark()` function
- No framework dependencies (pure CSS)

#### DevOps
- Docker multi-stage Alpine build (~200MB image)
- Kamal deployment with zero-downtime
- Ansible server provisioning
- Traefik reverse proxy with auto SSL
- Health check endpoint for monitoring

#### Security
- Security headers (CSP, HSTS, X-Frame-Options)
- Rate limiting on login and contact forms
- Input validation with Zod schemas
- Audit logging for security events
- Timing-safe password comparison

### Infrastructure
- SQLite with WAL mode for concurrency
- In-memory and Redis-backed rate limiting
- Translation caching with invalidation
- Optimistic locking for concurrent edits

---

## Version History

| Version | Date       | Description                                |
|---------|------------|--------------------------------------------|
| 1.3.1   | 2026-03-10 | Documentation correction patch for 1.3.0   |
| 1.3.0   | 2026-03-07 | API contract hardening & security release  |
| 1.2.1   | 2026-02-21 | Patch release for v1.2.0 follow-up         |
| 1.2.0   | 2026-02-20 | Stability and security hardening release   |
| 1.1.0   | 2026-02-20 | Stability and security hardening release   |
| 1.0.0   | 2025-01-15 | Initial production release                 |

---

## Migration Notes

### Upgrading to Latest

1. **Read the release notes**:
   - `docs/releases/1.1.0.md`

2. **Database migrations**:
   ```bash
   npm run db:migrate
   ```

3. **2FA Configuration** (if using):
   ```env
   TOTP_ENCRYPTION_KEY=your-32-byte-key
   ```

4. **Backup Encryption** (optional but recommended for S3):
   ```env
   BACKUP_ENCRYPTION_KEY=your-strong-passphrase
   ```

5. **Sentry Configuration** (optional):
   ```env
   # Server-side
   SENTRY_DSN=https://xxx@sentry.io/xxx
   SENTRY_ENVIRONMENT=production

   # Client-side
   NUXT_PUBLIC_SENTRY_DSN=https://xxx@sentry.io/xxx
   NUXT_PUBLIC_SENTRY_ENVIRONMENT=production
   ```

---

## Release Notes Template

```markdown
## [X.Y.Z] - YYYY-MM-DD

### Added
- New features

### Changed
- Changes in existing functionality

### Deprecated
- Features that will be removed in future versions

### Removed
- Features removed in this release

### Fixed
- Bug fixes

### Security
- Security improvements
```

## Versioning Guidelines

- **Major (X)**: Breaking changes to API or configuration
- **Minor (Y)**: New features, backwards compatible
- **Patch (Z)**: Bug fixes and minor improvements

## Links

[Unreleased]: https://github.com/your-org/puppetmaster2/compare/v1.1.0...HEAD
[1.1.0]: https://github.com/your-org/puppetmaster2/compare/v1.0.0...v1.1.0
[1.0.0]: https://github.com/your-org/puppetmaster2/releases/tag/v1.0.0

---

*For detailed documentation, see the [docs/](./docs/) directory.*
