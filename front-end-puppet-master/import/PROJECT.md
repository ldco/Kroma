# Project Specification

<!--
Fill out this document to describe the project you want to build.
Then run /pm-migrate to analyze requirements and create an implementation plan.

Delete sections that don't apply. Add details where needed.
The more specific you are, the better the analysis.
-->

## Overview

**Project Name**:
**Client/Company**:
**Description**:
<!-- What is this project? What problem does it solve? -->

**Target Users**:
<!-- Who will use this? Customers? Employees? Both? -->

## Project Type

<!--
PM builds ONE thing at a time. Choose what you're building:

If you need BOTH a marketing site AND an app product,
deploy them separately:
  - PM instance #1 → Website (marketing.example.com)
  - PM instance #2 → App (app.example.com)
-->

### What are you building? (choose one)

- [ ] **Website** — Marketing site, landing pages, company info
      Public pages for visitors (unauthenticated)

- [ ] **App** — Product, dashboard, user features
      Application for logged-in users (/ redirects to /login)

### Admin Panel

- [ ] **Admin Panel** — Management interface at /admin
      Secure area for authorized users to manage project data

### Website (if Website enabled)

**Layout mode:**
- [ ] **Onepager** — All sections on one scrolling page (#anchor nav)
- [ ] **SPA** — Each section is its own route (/about, /portfolio)

**Static sections (no data, just UI):**
- [ ] Hero — Main banner/headline
- [ ] About — Company/personal info

**Content features (each = section + data + admin):**

| Feature | Section | Admin | Data tables |
|---------|---------|-------|-------------|
| [ ] Blog | /blog, /blog/[slug] | /admin/blog | posts, categories, tags |
| [ ] Portfolio | /portfolio, /portfolio/[slug] | /admin/portfolio | items, categories |
| [ ] Team | #team or /team | /admin/team | members |
| [ ] Testimonials | #testimonials | /admin/testimonials | testimonials |
| [ ] FAQ | #faq | /admin/faq | items, categories |
| [ ] Clients | #clients | /admin/clients | clients |
| [ ] Pricing | #pricing or /pricing | /admin/pricing | tiers, features |
| [ ] Features | #features | /admin/features | feature_cards |
| [ ] Contact | /contact | /admin/contacts | submissions |

**Custom features:**

| Feature name | What it shows | Data fields | Admin? |
|--------------|---------------|-------------|--------|
| ___ | ___ | ___ | [ ] |
| ___ | ___ | ___ | [ ] |

**Static pages (no data):**
- [ ] Terms of Service
- [ ] Privacy Policy
- [ ] Other: _______________

### App (if App enabled)

**Pre-built pages:**
- [ ] Dashboard — User home/overview
- [ ] Profile / Settings — User account

**Custom features:**

| Feature name | What it shows | Data fields | Admin? |
|--------------|---------------|-------------|--------|
| ___ | ___ | ___ | [ ] |
| ___ | ___ | ___ | [ ] |
| ___ | ___ | ___ | [ ] |

### System Admin (always available with Admin Panel)

- [x] Users — User accounts management
- [x] Roles — Role permissions (master only)
- [x] Translations — UI text (i18n)
- [x] Settings — Site configuration

## Features

### Core Features
- [ ] **Multilingual** — Languages: _______________
- [ ] **Dark mode** — Light/dark theme toggle
- [ ] **Contact form** — With email/Telegram notifications

### User Roles Needed

<!-- Which roles do you need? -->
- [x] **Master** — Full system access (developer/agency)
- [ ] **Admin** — Content + user management (client owner)
- [ ] **Editor** — Content only (content team)
- [ ] **User** — App access only (only if building App)

## Design & Branding

### Colors
<!-- Provide hex codes or describe -->
- Brand (primary):
- Accent:
- Black (text):
- White (background):

### Typography
<!--
PM uses 3 font slots. For each, choose source:
- Google Fonts: Just provide font name (e.g., "Inter")
- Self-hosted: Place files in public/fonts/, use @font-face
-->

**Accent font** (headings, hero text):
- Font name:
- Source: [ ] Google Fonts [ ] Self-hosted

**Text font** (body, paragraphs):
- Font name:
- Source: [ ] Google Fonts [ ] Self-hosted

**Mono font** (code, technical):
- Font name:
- Source: [ ] Google Fonts [ ] Self-hosted

### Icons
<!-- Choose icon library for the project -->
- [ ] **Tabler Icons** — 5400+ icons, MIT license (PM default)
- [ ] **Lucide** — Fork of Feather, consistent style
- [ ] **Heroicons** — By Tailwind team, solid/outline
- [ ] **Phosphor** — Flexible weights, 6 styles
- [ ] **Iconoir** — 1500+ minimal icons

### Design References
<!-- NOT assets - these guide the design direction -->
- [ ] Brand guidelines document
- [ ] Figma/Sketch mockups
- [ ] Reference website: _______________

## Assets

### Asset Locations

| Asset Type | Location | Naming |
|------------|----------|--------|
| **Logos** | `public/logos/` | `{shape}_{theme}_{lang}.svg` |
| Hero/banner | `public/images/hero/` | — |
| Team photos | `public/images/team/` | — |
| Portfolio | `public/images/portfolio/` | — |
| Other images | `public/images/` | — |
| Custom icons | `public/icons/` | — |
| Illustrations | `public/images/illustrations/` | — |
| Self-hosted fonts | `public/fonts/` | — |

**Logo naming convention:**
- Shape: `horizontal` (header) or `circle` (compact)
- Theme: `dark` (for light bg) or `light` (for dark bg)
- Lang: `en`, `ru`, `he`, etc.
- Example: `horizontal_dark_en.svg`, `circle_light_ru.svg`

## Existing Content

<!-- Do you have content to migrate? -->
- [ ] Starting fresh — no existing data
- [ ] Text content (copy, descriptions)
- [ ] Blog posts from existing site
- [ ] Database/spreadsheet to migrate

## Integrations

### Pre-built (PM provides)
<!-- Just configure via .env -->
- [ ] **SMTP Email** — Contact confirmations, notifications
- [ ] **Telegram Bot** — Admin notifications for contact form
- [ ] **S3 Storage** — AWS S3, Cloudflare R2, MinIO compatible

### Needs Integration (not pre-built)
<!-- Will require custom implementation -->
- [ ] **Analytics**: Google Analytics / Plausible / Yandex Metrica
- [ ] **Payment**: Stripe / PayPal
- [ ] **OAuth**: Google / GitHub / other social login
- [ ] **Other**: _______________

## Deployment

<!--
PM provides: Kamal + Ansible + Traefik (with Let's Encrypt SSL)
- Ansible: Provisions fresh Ubuntu VPS
- Kamal: Containerized deployment
- Traefik: Reverse proxy with automatic SSL
-->

### Hosting
- [ ] **VPS (Recommended)** — Kamal + Ansible (PM provides)
- [ ] **Other** — Manual setup required: _______________

### Domain
- Domain: _______________
<!-- SSL is automatic via Traefik + Let's Encrypt -->

## Notes

<!-- Anything else? Timeline, budget, special requirements, constraints -->




---

<!--
After filling this out, run:

/pm-migrate

Claude will analyze your requirements against PM capabilities,
identify what PM provides vs what needs building,
and create an implementation plan.
-->
