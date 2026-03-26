# /pm-migrate — Import Existing Code into Puppet Master

**You are the migration tool.** Follow these instructions to:
1. DECOMPOSE the entire imported project into pieces
2. MAP every piece to its Puppet Master equivalent or strategy
3. ASK the user what they want for each major area
4. CREATE a comprehensive migration plan with full mappings

This is for **Brownfield** (existing code) migrations only. For new projects, use `/pm-init`.

This is NOT automated scripting — YOU (Claude) perform the analysis, ask questions, and guide the migration.

## Usage

```
/pm-migrate                 # Full migration workflow
/pm-migrate --analyze       # Analysis + mapping only (no changes)
/pm-migrate --resume        # Continue from saved state
```

---

## The Migration Philosophy

**Every project is broken into 7 domains:**

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        PROJECT DECOMPOSITION                                 │
├─────────────────────────────────────────────────────────────────────────────┤
│  1. FRONTEND      Pages, Components, Layouts, Composables/Hooks             │
│  2. BACKEND       API routes, Server logic, Middleware, Utilities           │
│  3. DATABASE      Schema, Models, Migrations, Seed data                     │
│  4. STYLES        Colors, Typography, Spacing, Component styles             │
│  5. AUTH          Users, Sessions, Roles, Protected routes, Providers       │
│  6. I18N          Locales, Translations, RTL, Date/Number formats           │
│  7. ASSETS        Images, Fonts, Icons, Documents, Other files              │
└─────────────────────────────────────────────────────────────────────────────┘
```

**For EVERY item in each domain, determine an ACTION:**

```
┌─────────────┬───────────────────────────────────────────────────────────────┐
│ Action      │ Meaning                                                       │
├─────────────┼───────────────────────────────────────────────────────────────┤
│ PM_EXISTS   │ Puppet Master already has this — use PM's version             │
│ PM_NATIVE   │ Use PM's native/showcase implementation                       │
│ CREATE      │ Build new in PM based on import                               │
│ REWRITE     │ Port logic from import to PM patterns                         │
│ PROXY       │ Keep external, PM proxies to it                               │
│ KEEP        │ Keep original system running alongside                        │
│ COPY        │ Direct copy (assets, static files)                            │
│ CONVERT     │ Transform format (e.g., Tailwind → PM CSS)                    │
│ MERGE       │ Combine with existing PM component                            │
│ SKIP        │ Not needed, PM handles differently                            │
└─────────────┴───────────────────────────────────────────────────────────────┘
```

---

## Two-Level Architecture Understanding

**CRITICAL: Before analyzing, understand PM's architecture.**

See `docs/PM-ARCHITECTURE.md` for full details.

### Level 1: System Entities (What)

The project being migrated may have:

| Entity | Purpose | PM Equivalent |
|--------|---------|---------------|
| **Website** | Public pages | Website UX (default layout) |
| **App** | User features | App UX (sidebar/minimal layout) |
| **Admin** | Management interface | App UX with admin sections |

**Identify which entities the import has!**

### Level 2: UX Paradigms (How)

PM has TWO UX paradigms, not three:

```
Website UX               App UX
─────────────           ─────────────────────────
• Horizontal header     • Sidebar (desktop)
• Page-based            • Bottom nav (mobile)
• For visitors          • For ALL logged-in users
                        • Both users AND admins
```

**"Admin panel" is NOT a separate UX — it's App UX with admin sections visible!**

### Common Migration Mistakes

1. **Creating separate "admin layout"** — Wrong! Use App UX, filter sections by role
2. **Confusing layout with role** — Layout = where nav goes, Role = what nav shows
3. **Missing admin in website-app** — If migrating SaaS, ask about content management

---

## PHASE 1: Verify Code Exists in Import Folder

**Check if import folder has code to migrate:**

```
Glob: import/**/*
```

**Check for code indicators:** package.json, src/, pages/, components/, app/, nuxt.config.*, etc.

**If no code found**, display:
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
                         📁 NO CODE TO MIGRATE
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/pm-migrate is for importing EXISTING code into Puppet Master.

If you want to:

  START A NEW PROJECT
  ───────────────────
  Run /pm-init instead (optionally fill ./import/PROJECT.md first)

  MIGRATE EXISTING CODE
  ─────────────────────
  Copy your existing project to the import folder:
    cp -r ~/your-project/* ./import/
  Then run /pm-migrate again.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

**Stop here if no code.**

**If only PROJECT.md exists** (no code):
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
                         🌱 GREENFIELD DETECTED
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

You have a PROJECT.md specification but no code.
This is a Greenfield project — run /pm-init instead.

/pm-init will analyze your PROJECT.md and create an implementation plan.

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

**Stop here — redirect to /pm-init.**

---

## PHASE 2: Brownfield Project Decomposition

**You must analyze EVERYTHING. Read files, understand the ENTIRE project.**

### 2.1 Project Overview

**Read these files:**
- `import/package.json` — Name, dependencies, scripts
- `import/README.md` — Project description
- Framework config files (nuxt/next/svelte/vite config)
- `import/.env.example` or `import/.env.local` — Environment variables

**Document:**
- Project name and description
- Framework and version
- TypeScript or JavaScript
- Package manager
- All dependencies (categorize: framework, UI, data, utils, dev)

### 2.2 Frontend Decomposition

#### 2.2.1 Pages/Routes

**Find all pages:**
```
Glob: import/pages/**/*
Glob: import/app/**/*
Glob: import/src/routes/**/*
Glob: import/src/pages/**/*
```

**For EACH page, document:**
- Path (URL pattern)
- File location
- Dynamic segments ([id], [slug], etc.)
- Data fetching method (SSR, SSG, client-side, ISR)
- Layout used
- Components used
- Meta/SEO handling

#### 2.2.2 Components

**Find all components:**
```
Glob: import/**/*.tsx (React)
Glob: import/**/*.jsx (React)
Glob: import/**/*.vue (Vue)
Glob: import/**/*.svelte (Svelte)
```

**For EACH component, document and CLASSIFY:**

| Category | Examples | PM Equivalent Location |
|----------|----------|------------------------|
| Atoms | Button, Input, Icon, Badge, Link, Avatar | `components/atoms/` |
| Molecules | Card, NavItem, FormField, SearchBar, MenuItem | `components/molecules/` |
| Organisms | Header, Footer, Sidebar, Modal, Navbar, Form | `components/organisms/` |
| Sections | HeroSection, PricingTable, ContactSection | `components/sections/` |
| Templates | DefaultLayout, AdminLayout, AuthLayout | `layouts/` |

**Read each component to understand:**
- Props interface
- State management (hooks, stores, refs)
- Event handling
- Styling approach
- Child components used
- External dependencies

#### 2.2.3 Layouts

**Find layouts:**
```
Glob: import/layouts/**/*
Glob: import/app/layout.*
Glob: import/src/layouts/**/*
```

**Document each layout and its purpose.**

#### 2.2.4 Composables / Hooks / Utilities

**Find shared logic:**
```
Glob: import/composables/**/*
Glob: import/hooks/**/*
Glob: import/src/hooks/**/*
Glob: import/lib/**/*
Glob: import/utils/**/*
```

**Document each and determine if PM has equivalent.**

### 2.3 Backend Decomposition

#### 2.3.1 API Routes

**Find API routes:**
```
Glob: import/api/**/*
Glob: import/pages/api/**/*
Glob: import/server/**/*
Glob: import/src/api/**/*
Glob: import/app/api/**/*
```

**For EACH endpoint, document:**
- HTTP method(s)
- Path pattern
- Request validation
- Response shape
- Database queries
- External API calls
- Authentication required?
- Authorization (roles)?

#### 2.3.2 Server Middleware

**Find middleware:**
```
Glob: import/middleware/**/*
Glob: import/server/middleware/**/*
```

**Document what each middleware does.**

#### 2.3.3 Server Utilities

**Find server-side utilities:**
```
Grep: "server" in import/lib/
Grep: "server" in import/utils/
```

### 2.4 Database Decomposition

#### 2.4.1 Schema

**Find schema files:**
```
Glob: import/prisma/schema.prisma
Glob: import/drizzle/**/*
Glob: import/src/db/**/*
Glob: import/models/**/*
```

**For EACH model/table, document:**
- Table name
- Fields with types
- Relations
- Indexes
- PM equivalent table (if exists)

#### 2.4.2 Migrations

**Find migrations:**
```
Glob: import/prisma/migrations/**/*
Glob: import/migrations/**/*
Glob: import/drizzle/migrations/**/*
```

#### 2.4.3 Seed Data

**Find seed files:**
```
Glob: import/prisma/seed.*
Glob: import/seeds/**/*
Glob: import/data/**/*
```

### 2.5 Styles Decomposition

#### 2.5.1 Styling System Detection

**Check for styling approaches:**

1. **Tailwind CSS**
   ```
   Glob: import/tailwind.config.*
   Grep: "tailwindcss" in import/package.json
   ```

2. **CSS Modules**
   ```
   Glob: import/**/*.module.css
   Glob: import/**/*.module.scss
   ```

3. **SCSS/SASS**
   ```
   Glob: import/**/*.scss
   Glob: import/**/*.sass
   ```

4. **Styled Components / Emotion**
   ```
   Grep: "styled-components" in import/package.json
   Grep: "@emotion" in import/package.json
   ```

5. **Plain CSS**
   ```
   Glob: import/**/*.css
   ```

#### 2.5.2 Design Tokens

**Extract design tokens:**

**Colors:**
- Find color definitions (CSS vars, Tailwind config, theme file)
- List all colors with their usage

**Typography:**
- Font families
- Font sizes scale
- Line heights
- Font weights

**Spacing:**
- Spacing scale
- Container widths
- Breakpoints

**Other:**
- Border radius values
- Shadow definitions
- Z-index scale
- Transition/animation values

#### 2.5.3 Component Styles

**For EACH component, note styling approach:**
- Inline styles
- CSS classes
- Scoped styles
- Global styles
- CSS-in-JS

### 2.6 Authentication Decomposition

#### 2.6.1 Auth System Detection

**Check package.json for:**
- `next-auth` / `@auth/*`
- `passport`
- `lucia`
- `clerk`
- `supabase`
- `firebase`
- Custom JWT implementation

**Find auth files:**
```
Glob: import/**/*auth*
Grep: "session" in import/
Grep: "jwt" in import/
Grep: "login" in import/
Grep: "password" in import/
```

#### 2.6.2 User Model

**Document user schema:**
- Fields (email, password, name, etc.)
- Role field (if any)
- Profile fields
- Timestamps

#### 2.6.3 Auth Flow

**Document:**
- Login method (email/password, OAuth, magic link)
- OAuth providers (Google, GitHub, etc.)
- Session storage (cookie, JWT, database)
- Token refresh strategy
- Password reset flow

#### 2.6.4 Authorization

**Document:**
- Role definitions
- Permission system
- Protected routes
- API authorization

### 2.7 i18n Decomposition

#### 2.7.1 i18n Detection

**Check package.json for:**
- `next-intl`
- `vue-i18n`
- `react-i18next`
- `svelte-i18n`

**Find translation files:**
```
Glob: import/locales/**/*
Glob: import/i18n/**/*
Glob: import/messages/**/*
Glob: import/lang/**/*
Glob: import/translations/**/*
```

#### 2.7.2 Translation Structure

**Document:**
- Supported locales
- Default locale
- File format (JSON, YAML, JS)
- Key naming convention
- Pluralization approach
- Date/number formatting

#### 2.7.3 RTL Support

**Check for RTL:**
- RTL locales (ar, he, fa, ur)
- RTL-specific styles
- Direction switching logic

### 2.8 Assets Decomposition

#### 2.8.1 Images

```
Glob: import/public/images/**/*
Glob: import/public/img/**/*
Glob: import/assets/images/**/*
Glob: import/static/images/**/*
```

**Document: count, formats, optimization status**

#### 2.8.2 Fonts

```
Glob: import/public/fonts/**/*
Glob: import/assets/fonts/**/*
```

**Document: font families, formats, loading strategy**

#### 2.8.3 Icons

**Check for icon system:**
- Icon component library (lucide, heroicons, etc.)
- SVG sprites
- Individual SVG files
- Icon fonts

#### 2.8.4 Other Assets

```
Glob: import/public/**/*
```

**Document: favicons, manifests, robots.txt, sitemap, documents**

---

## PHASE 3: Create Complete Mapping Tables

**After analysis, create COMPREHENSIVE mapping tables for EVERY item found.**

### 3.1 Frontend Mapping

#### Pages Mapping

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                                    PAGES MAPPING                                        │
├──────────────────────┬──────────────────────┬──────────┬──────────┬────────────────────┤
│ Import Path          │ PM Path              │ Action   │ Priority │ Notes              │
├──────────────────────┼──────────────────────┼──────────┼──────────┼────────────────────┤
│ pages/index.tsx      │ pages/index.vue      │ CREATE   │ High     │ Homepage           │
│ pages/about.tsx      │ pages/about.vue      │ CREATE   │ Medium   │ Static page        │
│ pages/blog/index.tsx │ pages/blog/index.vue │ PM_EXISTS│ High     │ Use PM blog module │
│ pages/blog/[slug].tsx│ pages/blog/[slug].vue│ PM_EXISTS│ High     │ Use PM blog module │
│ pages/admin/...      │ pages/admin/...      │ PM_EXISTS│ High     │ Use PM admin       │
│ pages/login.tsx      │ pages/admin/login.vue│ PM_EXISTS│ High     │ Use PM auth        │
│ pages/dashboard.tsx  │ pages/admin/index.vue│ REWRITE  │ High     │ Adapt to PM admin  │
│ ...                  │ ...                  │ ...      │ ...      │ ...                │
└──────────────────────┴──────────────────────┴──────────┴──────────┴────────────────────┘
```

#### Components Mapping

```
┌─────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                         COMPONENTS MAPPING                                               │
├─────────────────────────────┬────────────────────────────────┬───────────┬──────────┬────────┬──────────┤
│ Import Component            │ PM Equivalent                  │ Level     │ Action   │ Priority│ Notes   │
├─────────────────────────────┼────────────────────────────────┼───────────┼──────────┼────────┼──────────┤
│ components/Button.tsx       │ CSS .btn classes               │ atom      │ PM_NATIVE│ -      │ Use PM   │
│ components/Input.tsx        │ CSS form styling               │ atom      │ PM_NATIVE│ -      │ Use PM   │
│ components/Card.tsx         │ molecules/ContentCard.vue      │ molecule  │ CREATE   │ Medium │ New file │
│ components/Header.tsx       │ organisms/TheHeader.vue        │ organism  │ PM_EXISTS│ High   │ Adapt    │
│ components/Footer.tsx       │ organisms/TheFooter.vue        │ organism  │ PM_EXISTS│ High   │ Adapt    │
│ components/Sidebar.tsx      │ organisms/AdminSidebar.vue     │ organism  │ PM_EXISTS│ Medium │ Admin    │
│ components/Hero.tsx         │ sections/SectionHero.vue       │ section   │ PM_EXISTS│ High   │ Adapt    │
│ components/Pricing.tsx      │ sections/SectionPricing.vue    │ section   │ PM_EXISTS│ Medium │ Adapt    │
│ components/ContactForm.tsx  │ sections/SectionContact.vue    │ section   │ PM_EXISTS│ Medium │ Adapt    │
│ components/BlogList.tsx     │ organisms/BlogPostsGrid.vue    │ organism  │ PM_EXISTS│ Medium │ Adapt    │
│ components/TeamGrid.tsx     │ sections/SectionTeam.vue       │ section   │ PM_EXISTS│ Medium │ Adapt    │
│ components/CustomWidget.tsx │ molecules/CustomWidget.vue     │ molecule  │ CREATE   │ Low    │ New file │
│ ...                         │ ...                            │ ...       │ ...      │ ...    │ ...      │
└─────────────────────────────┴────────────────────────────────┴───────────┴──────────┴────────┴──────────┘
```

#### Composables/Hooks Mapping

```
┌──────────────────────────────────────────────────────────────────────────────────────────────┐
│                                    COMPOSABLES MAPPING                                        │
├─────────────────────────────┬────────────────────────────────┬──────────┬────────────────────┤
│ Import Hook                 │ PM Equivalent                  │ Action   │ Notes              │
├─────────────────────────────┼────────────────────────────────┼──────────┼────────────────────┤
│ useAuth()                   │ useAuth()                      │ PM_EXISTS│ PM has this        │
│ useToast()                  │ useToast()                     │ PM_EXISTS│ PM has this        │
│ useLocalStorage()           │ useLocalStorage()              │ PM_EXISTS│ VueUse             │
│ useFetch()                  │ useFetch() / $fetch            │ PM_EXISTS│ Nuxt built-in      │
│ useMediaQuery()             │ useMediaQuery()                │ PM_EXISTS│ VueUse             │
│ useCustomLogic()            │ composables/useCustomLogic.ts  │ CREATE   │ Port logic         │
│ ...                         │ ...                            │ ...      │ ...                │
└─────────────────────────────┴────────────────────────────────┴──────────┴────────────────────┘
```

### 3.2 Backend Mapping

#### API Endpoints Mapping

```
┌──────────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                           API ENDPOINTS MAPPING                                               │
├─────────────────────────┬──────────┬───────────────────────────┬──────────┬──────────┬───────────────────────┤
│ Import Endpoint         │ Methods  │ PM Equivalent             │ Action   │ Priority │ Notes                 │
├─────────────────────────┼──────────┼───────────────────────────┼──────────┼──────────┼───────────────────────┤
│ /api/auth/login         │ POST     │ /api/auth/login           │ PM_EXISTS│ High     │ Use PM auth           │
│ /api/auth/logout        │ POST     │ /api/auth/logout          │ PM_EXISTS│ High     │ Use PM auth           │
│ /api/auth/session       │ GET      │ /api/auth/session         │ PM_EXISTS│ High     │ Use PM auth           │
│ /api/users              │ GET,POST │ /api/admin/users          │ PM_EXISTS│ Medium   │ Use PM admin API      │
│ /api/posts              │ GET,POST │ /api/blog/posts           │ PM_EXISTS│ Medium   │ Use PM blog API       │
│ /api/posts/[id]         │ GET,PUT,D│ /api/blog/[slug]          │ PM_EXISTS│ Medium   │ Use PM blog API       │
│ /api/contact            │ POST     │ /api/contact/submit       │ PM_EXISTS│ Medium   │ Use PM contact        │
│ /api/upload             │ POST     │ /api/upload/image         │ PM_EXISTS│ Low      │ Use PM upload         │
│ /api/custom/endpoint    │ GET,POST │ /api/custom/endpoint      │ REWRITE  │ Medium   │ Port to Nitro         │
│ /api/external/data      │ GET      │ /api/proxy/external/data  │ PROXY    │ Low      │ Proxy to external     │
│ ...                     │ ...      │ ...                       │ ...      │ ...      │ ...                   │
└─────────────────────────┴──────────┴───────────────────────────┴──────────┴──────────┴───────────────────────┘
```

### 3.3 Database Mapping

#### Schema Mapping

```
┌──────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                          DATABASE SCHEMA MAPPING                                          │
├─────────────────────┬─────────────────────┬──────────┬────────────────────────────────────────────────────┤
│ Import Table        │ PM Table            │ Action   │ Field Mapping                                      │
├─────────────────────┼─────────────────────┼──────────┼────────────────────────────────────────────────────┤
│ User                │ users               │ PM_EXISTS│ id→id, email→email, password→passwordHash,        │
│                     │                     │          │ role→role, name→(split to firstName/lastName)     │
├─────────────────────┼─────────────────────┼──────────┼────────────────────────────────────────────────────┤
│ Post                │ blog_posts          │ PM_EXISTS│ id→id, title→title, content→content,              │
│                     │                     │          │ slug→slug, authorId→authorId, publishedAt→...     │
├─────────────────────┼─────────────────────┼──────────┼────────────────────────────────────────────────────┤
│ Category            │ blog_categories     │ PM_EXISTS│ id→id, name→name, slug→slug                       │
├─────────────────────┼─────────────────────┼──────────┼────────────────────────────────────────────────────┤
│ Product             │ (none)              │ CREATE   │ New table needed - define schema                   │
├─────────────────────┼─────────────────────┼──────────┼────────────────────────────────────────────────────┤
│ Order               │ (none)              │ CREATE   │ New table needed - define schema                   │
├─────────────────────┼─────────────────────┼──────────┼────────────────────────────────────────────────────┤
│ Settings            │ settings            │ PM_EXISTS│ Key-value store, map keys                          │
├─────────────────────┼─────────────────────┼──────────┼────────────────────────────────────────────────────┤
│ ...                 │ ...                 │ ...      │ ...                                                │
└─────────────────────┴─────────────────────┴──────────┴────────────────────────────────────────────────────┘
```

### 3.4 Styles Mapping

#### Color Mapping

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                                    COLOR MAPPING                                        │
├─────────────────────────┬─────────────────────────┬──────────┬─────────────────────────┤
│ Import Color            │ PM Token                │ Action   │ Notes                   │
├─────────────────────────┼─────────────────────────┼──────────┼─────────────────────────┤
│ --primary: #3B82F6      │ --color-brand           │ CONVERT  │ Map to brand color      │
│ --secondary: #10B981    │ --color-accent          │ CONVERT  │ Map to accent           │
│ --background: #FFFFFF   │ --color-white           │ PM_NATIVE│ Use PM primitive        │
│ --text: #1F2937         │ --color-black           │ PM_NATIVE│ Use PM primitive        │
│ --error: #EF4444        │ --color-danger          │ PM_NATIVE│ Use PM semantic         │
│ --success: #22C55E      │ --color-success         │ PM_NATIVE│ Use PM semantic         │
│ --gray-100: #F3F4F6     │ (auto-calculated)       │ PM_NATIVE│ PM calculates grays     │
│ ...                     │ ...                     │ ...      │ ...                     │
└─────────────────────────┴─────────────────────────┴──────────┴─────────────────────────┘
```

#### Typography Mapping

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                                  TYPOGRAPHY MAPPING                                     │
├─────────────────────────┬─────────────────────────┬──────────┬─────────────────────────┤
│ Import Font             │ PM Equivalent           │ Action   │ Notes                   │
├─────────────────────────┼─────────────────────────┼──────────┼─────────────────────────┤
│ font-family: Inter      │ --font-sans             │ CONVERT  │ Change font in config   │
│ font-family: Roboto Mono│ --font-mono             │ PM_NATIVE│ PM has mono font        │
│ text-sm (14px)          │ .text-sm                │ PM_NATIVE│ Use PM class            │
│ text-lg (18px)          │ .text-lg                │ PM_NATIVE│ Use PM class            │
│ font-bold               │ .font-bold              │ PM_NATIVE│ Use PM class            │
│ ...                     │ ...                     │ ...      │ ...                     │
└─────────────────────────┴─────────────────────────┴──────────┴─────────────────────────┘
```

#### Component Styles Mapping (if keeping original styles)

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                              TAILWIND → PM CSS MAPPING                                  │
├─────────────────────────┬─────────────────────────┬──────────┬─────────────────────────┤
│ Tailwind Class          │ PM CSS Equivalent       │ Action   │ Notes                   │
├─────────────────────────┼─────────────────────────┼──────────┼─────────────────────────┤
│ flex                    │ .flex                   │ PM_NATIVE│ PM has this             │
│ items-center            │ .items-center           │ PM_NATIVE│ PM has this             │
│ justify-between         │ .justify-between        │ PM_NATIVE│ PM has this             │
│ bg-primary              │ .bg-brand               │ CONVERT  │ Use PM token            │
│ text-white              │ .text-white             │ PM_NATIVE│ PM has this             │
│ rounded-lg              │ .rounded-lg             │ PM_NATIVE│ PM has this             │
│ shadow-md               │ .shadow-md              │ PM_NATIVE│ PM has this             │
│ p-4                     │ .p-4                    │ PM_NATIVE│ PM has this             │
│ hover:bg-primary-dark   │ .btn:hover              │ CONVERT  │ PM handles hover        │
│ ...                     │ ...                     │ ...      │ ...                     │
└─────────────────────────┴─────────────────────────┴──────────┴─────────────────────────┘
```

### 3.5 Auth Mapping

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                                    AUTH MAPPING                                         │
├─────────────────────────┬─────────────────────────┬──────────┬─────────────────────────┤
│ Import Feature          │ PM Equivalent           │ Action   │ Notes                   │
├─────────────────────────┼─────────────────────────┼──────────┼─────────────────────────┤
│ Email/Password login    │ PM session auth         │ PM_EXISTS│ Built-in                │
│ Google OAuth            │ (not built-in)          │ CREATE   │ Add OAuth provider      │
│ JWT tokens              │ PM sessions             │ PM_NATIVE│ PM uses sessions        │
│ User roles (admin,user) │ PM RBAC (master,admin,ed│ PM_EXISTS│ Map roles               │
│ Protected routes        │ auth middleware         │ PM_EXISTS│ Use PM middleware       │
│ Password reset          │ (basic in PM)           │ REWRITE  │ Enhance if needed       │
│ ...                     │ ...                     │ ...      │ ...                     │
└─────────────────────────┴─────────────────────────┴──────────┴─────────────────────────┘
```

### 3.6 i18n Mapping

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                                    I18N MAPPING                                         │
├─────────────────────────┬─────────────────────────┬──────────┬─────────────────────────┤
│ Import Feature          │ PM Equivalent           │ Action   │ Notes                   │
├─────────────────────────┼─────────────────────────┼──────────┼─────────────────────────┤
│ Locales: en, es, fr     │ PM locales config       │ CONVERT  │ Add to config           │
│ Translation files       │ DB translations         │ CONVERT  │ Import to database      │
│ t('key') function       │ t('key')                │ PM_EXISTS│ Same pattern            │
│ RTL support             │ PM RTL support          │ PM_EXISTS│ Built-in                │
│ Date formatting         │ Intl.DateTimeFormat     │ PM_NATIVE│ Standard API            │
│ ...                     │ ...                     │ ...      │ ...                     │
└─────────────────────────┴─────────────────────────┴──────────┴─────────────────────────┘
```

### 3.7 Assets Mapping

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                                   ASSETS MAPPING                                        │
├─────────────────────────┬─────────────────────────┬──────────┬─────────────────────────┤
│ Import Path             │ PM Path                 │ Action   │ Notes                   │
├─────────────────────────┼─────────────────────────┼──────────┼─────────────────────────┤
│ public/images/*         │ public/images/*         │ COPY     │ Direct copy             │
│ public/fonts/*          │ public/fonts/*          │ COPY     │ Direct copy             │
│ public/favicon.ico      │ public/favicon.ico      │ COPY     │ Direct copy             │
│ public/robots.txt       │ public/robots.txt       │ COPY     │ Review content          │
│ public/sitemap.xml      │ (auto-generated)        │ PM_NATIVE│ PM generates            │
│ assets/icons/*.svg      │ ~icons/tabler/*         │ PM_NATIVE│ Use PM icon system      │
│ ...                     │ ...                     │ ...      │ ...                     │
└─────────────────────────┴─────────────────────────┴──────────┴─────────────────────────┘
```

---

## PHASE 4: Strategy Questions

**After showing mappings, ask KEY DECISIONS:**

### Question 1: Overall Migration Goal

```
What is your migration goal?

○ Full PM Native (Recommended)
  - Use PM's built-in features wherever possible
  - Rewrite custom parts to follow PM patterns
  - Result: Clean, maintainable PM project

○ PM Frontend + Keep Backend
  - PM handles frontend only
  - Keep your existing backend running
  - PM proxies API requests

○ Gradual Migration
  - Move piece by piece over time
  - Both systems run in parallel
  - For very large/complex projects

○ PM + Custom Extensions
  - Use PM core but keep unique business logic
  - Extend PM where needed
```

### Question 2: Backend Decision

```
Based on the analysis, your backend has:
- {X} API endpoints
- {Database type}
- {Auth system}

Backend strategy:

○ Use PM Backend (Recommended if fits)
  - Auth: Use PM's session auth
  - Database: Migrate to PM's SQLite/Drizzle
  - APIs: Use PM's existing APIs where possible
  - Custom: Rewrite unique endpoints as Nitro routes

○ Keep External Backend
  - Your backend keeps running
  - PM proxies all /api/* requests
  - No data migration needed

○ Hybrid
  - Auth: PM handles authentication
  - Content: Proxy to existing backend
  - Best for complex backends

○ Different Backend
  - Build new backend in Nitro
  - Don't migrate old logic
  - Fresh start with PM patterns
```

### Question 3: Styles Decision

```
Your project uses: {Tailwind / CSS Modules / etc.}
PM uses: Pure CSS with OKLCH color system

Styling strategy:

○ Full PM Styles (Recommended)
  - Use PM's CSS system completely
  - Map your colors to PM tokens
  - Consistent, themeable, light-dark() support

○ Keep Original + Add PM
  - Install {Tailwind/etc.} in PM
  - Keep your class names
  - Gradually adopt PM patterns

○ Convert to PM
  - Map {Tailwind} classes to PM equivalents
  - Takes more effort initially
  - Cleaner result
```

### Question 4: Data Decision (if database detected)

```
Your database: {Prisma + PostgreSQL / etc.}
PM database: SQLite + Drizzle

Data strategy:

○ Fresh Start
  - Empty database
  - Re-enter content via admin
  - Cleanest approach

○ Migrate Data
  - Export from old DB
  - Transform and import to PM
  - I'll help with migration queries

○ Keep External Database
  - Point PM to your database
  - Write Drizzle schema to match
  - Most complex option
```

### Question 5: Project Type

**PM builds ONE thing at a time. Determine what the import is:**

```
Your project appears to be:
- Public marketing site: {yes/no}
- User-facing application: {yes/no}
- Has admin/CMS: {yes/no}

PM builds ONE thing. Which describes this project better?

○ Website
  Marketing site, landing pages, company info
  → Public pages for visitors (Website UX)

○ App
  Product, dashboard, user features
  → Application for logged-in users (App UX)
  → Root (/) redirects to /login

NOTE: If the import has BOTH a marketing site AND an app product,
      you'll need to split them into separate PM instances.
```

**Then ask about admin panel:**

```
Enable Admin Panel?

○ Yes — Management interface at /admin (Recommended)
○ No — No management interface needed
```

**Map selection to config:**

```typescript
// Website with Admin:
entities: { website: true, app: false },
admin: { enabled: true }

// App with Admin:
entities: { website: false, app: true },
admin: { enabled: true }
```

### Question 6: App UX Preferences

**Only ask if App or Admin entities were selected:**

```
For authenticated users (app and/or admin), which visual style?

○ Sidebar (Recommended for many features)
  Desktop: Vertical sidebar | Mobile: Bottom navigation
  → Best for: 3+ features, admin panels, dashboards

○ Minimal Header
  Horizontal header with minimal navigation
  → Best for: Single-feature apps, simple tools

○ Full Header
  Horizontal header with dropdown menus
  → Best for: Apps with grouped features

NOTE: Both regular users AND admins use the same UX paradigm.
      The layout defines WHERE navigation goes.
      User ROLE defines WHAT navigation shows.
```

---

## PHASE 5: Generate Migration Plan Document

**Create `.qwen-data/migration-plan.md` with ALL mappings and steps:**

```markdown
# Migration Plan: {project name}

Generated: {date}
Source: ./import/ ({framework})
Target: Puppet Master

---

## Project Configuration

| Setting | Value | Notes |
|---------|-------|-------|
| Type    | {Website / App} | What PM is building |
| Admin   | {Enabled / Disabled} | Content management at /admin |

## Strategy Summary

| Domain    | Strategy        | Notes                              |
|-----------|-----------------|-----------------------------------|
| Frontend  | {PM_NATIVE/etc} | {notes}                           |
| Backend   | {PM/PROXY/etc}  | {notes}                           |
| Database  | {PM/KEEP/etc}   | {notes}                           |
| Styles    | {PM/KEEP/etc}   | {notes}                           |
| Auth      | {PM/KEEP/etc}   | {notes}                           |
| i18n      | {PM/CONVERT}    | {notes}                           |
| Assets    | {COPY}          | {notes}                           |

---

## Complete Mappings

### Frontend Mapping
{Insert full pages mapping table}
{Insert full components mapping table}
{Insert full composables mapping table}

### Backend Mapping
{Insert full API endpoints mapping table}

### Database Mapping
{Insert full schema mapping table}

### Styles Mapping
{Insert full color mapping table}
{Insert full typography mapping table}

### Auth Mapping
{Insert full auth mapping table}

### i18n Mapping
{Insert full i18n mapping table}

### Assets Mapping
{Insert full assets mapping table}

---

## Migration Phases

### Phase 1: Configuration
1. Update puppet-master.config.ts
2. Set environment variables
3. Set project type (website OR app) and admin.enabled

### Phase 2: Assets
1. Copy static files
2. Set up fonts
3. Configure icons

### Phase 3: Styles (if converting)
1. Update color tokens in config
2. Review typography settings
3. Test theme switching

### Phase 4: Database (if migrating)
1. Run db:push for schema
2. Run migration queries
3. Verify data integrity

### Phase 5: Backend (if rewriting)
1. Create custom API routes
2. Set up proxy (if hybrid)
3. Test all endpoints

### Phase 6: Frontend
1. Adapt PM sections to your content
2. Create custom components
3. Build custom pages

### Phase 7: Auth (if using PM auth)
1. Migrate users
2. Set up roles
3. Configure protected routes

### Phase 8: i18n (if applicable)
1. Import translations to DB
2. Configure locales
3. Test language switching

### Phase 9: Testing & Validation
{Checklist}

---

## Reference Files

Import files to reference during migration:
{List key files}

---

## Next Steps

1. /pm start — Initialize PM
2. Work through phases
3. Ask Claude for help: "Help me migrate {specific item}"
```

---

## PHASE 6: Update Configuration

**Based on decisions, update `puppet-master.config.ts`:**

Use Edit tool to set:
- `entities.website` / `entities.app` (boolean flags for what exists)
- `admin.enabled` (boolean for content management)
- `admin.system` / `admin.websiteModules` / `admin.appModules` (enabled modules + roles)
- `features` (multilingual, darkMode, onepager, etc.)
- `modules` (based on what import has - portfolio, blog, team, etc.)
- `dataSource.provider`
- `locales`
- `colors` (brand colors if provided mapping)

---

## PHASE 7: Initial Safe Operations

**Perform non-destructive setup:**

1. Copy assets: `cp -r ./import/public/* ./public/`
2. Create proxy route (if proxy strategy)
3. Save migration state to `.qwen-data/migration.json`

---

## PHASE 8: Summary & Next Steps

**For BROWNFIELD, display comprehensive summary with mapping counts:**

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
                    ✅ MIGRATION ANALYSIS COMPLETE
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Project: {name} → Puppet Master

  Type:   {Website / App}
  Admin:  {Enabled / Disabled}

MAPPING SUMMARY
─────────────────────────────────────────────────────────────────────────────
  Pages:       {X} mapped ({Y} PM_EXISTS, {Z} CREATE)
  Components:  {X} mapped ({Y} PM_EXISTS, {Z} CREATE, {W} SKIP)
  API Routes:  {X} mapped ({Y} PM_EXISTS, {Z} REWRITE, {W} PROXY)
  DB Tables:   {X} mapped ({Y} PM_EXISTS, {Z} CREATE)
  Colors:      {X} mapped to PM tokens
  Translations:{X} keys to migrate

STRATEGY
─────────────────────────────────────────────────────────────────────────────
  Backend:  {strategy}
  Styles:   {strategy}
  Data:     {strategy}

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Plan saved to: .qwen-data/migration-plan.md

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

**For GREENFIELD, display simpler summary:**

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
                    ✅ PROJECT ANALYSIS COMPLETE
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Project: {name}

  Type:   {Website / App}
  Admin:  {Enabled / Disabled}

CAPABILITY SUMMARY
─────────────────────────────────────────────────────────────────────────────
  PM Native:    {X} features ready to use
  To Build:     {Y} custom features
  Integrations: {Z} external services

Configuration updated: puppet-master.config.ts
Plan saved to: .qwen-data/migration-plan.md

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### 8.1 Offer to Start

**Use TodoWrite to present options:**

```
Ready to initialize the project?

Options:
1. Yes, run /pm-dev now (Recommended) - Set up database and start dev server
2. No, I want to review the plan first - You can run /pm-dev later
3. No, I need to make changes first - Edit config or plan, then run /pm-dev
```

**If user selects "Yes":**
- Inform them: "Starting project initialization..."
- Run `/pm-dev` logic (or instruct user to run it)

**If user selects "No":**
- Display next steps:
```
Next Steps:
  1. Review: .qwen-data/migration-plan.md
  2. Adjust: puppet-master.config.ts (if needed)
  3. Start:  /pm-dev

After starting, ask me:
  "Help me build the booking feature"
  "Set up the API proxy"
  "Create the custom dashboard"
```

---

## Ongoing Assistance

After setup, user can request help with ANY mapped item:

- "Migrate component X" → Read import, create PM equivalent
- "Set up proxy for /api/custom" → Create Nitro proxy route
- "Convert the color palette" → Update PM config with colors
- "Import the translations" → Create DB import queries
- "Migrate the User table" → Create Drizzle schema + migration

**Always reference the mapping tables to guide the work.**

---

## Notes

- **You are the tool** — analyze by reading, not scripting
- **Map EVERYTHING** — no item should be unmapped
- **Ask before assuming** — user decides strategy for each domain
- **PM patterns first** — prefer PM solutions over porting foreign patterns
- **Preserve import/** — never modify, always reference