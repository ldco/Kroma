# PM Team Role Template

This file documents the structure of PM team roles. Not used directly.

## Specialties (7)

| ID | Specialty | Focus |
|----|-----------|-------|
| `ux` | UX/UI Designer | User experience, accessibility, design systems |
| `fullstack` | Nuxt3 Fullstack | Full PM architecture, Vue + Nitro integration |
| `frontend` | Nuxt3 Frontend | Vue3, components, composables, styling |
| `backend` | Nuxt3 Backend | Nitro, API routes, database, auth |
| `fastapi` | Python FastAPI | External APIs, microservices, data processing |
| `devops` | DevOps/SRE | Docker, deployment, CI/CD, monitoring |
| `security` | Security Engineer | OWASP, auth, encryption, pentesting |

## Countries (6)

| Code | Country | Language | Search Engine | Context |
|------|---------|----------|---------------|---------|
| `ru` | Russia | Russian | Yandex | Russian market, CIS region |
| `us` | USA | English | Google.com | American market, global SaaS |
| `fr` | France | French | Google.fr | French/European market, GDPR |
| `jp` | Japan | Japanese | Google.jp | Japanese market, quality standards |
| `ch` | China | Chinese | Baidu | Chinese market, regulations |
| `il` | Israel | Hebrew | Google.il | Israeli market, RTL, startup culture |

## PM Framework Knowledge (All Roles)

Every PM team member deeply understands:

### Architecture
- Two-level system: Entities (Website/App) + UX Paradigms
- Config-driven via `puppet-master.config.ts`
- Modular entity configuration with RBAC
- SQLite + Drizzle ORM
- Pure CSS (OKLCH, CSS Layers)

### Key Patterns
- Atomic Design components (atoms → sections)
- Global CSS classes only, no scoped styles
- `~/` imports for app, `~~/` for root
- Database-driven i18n with RTL support
- RBAC: master → admin → editor → user

### File Structure
- `app/` - Frontend (pages, components, composables)
- `server/` - Backend (API routes, database, utils)
- `app/puppet-master.config.ts` - All configuration
- `server/database/schema.ts` - Database schema

## Naming Convention

Role files: `{specialty}-{country}.md`
Example: `ux-il.md`, `fullstack-ru.md`, `security-us.md`
