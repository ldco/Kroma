# Puppet Master Framework Knowledge Base

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
