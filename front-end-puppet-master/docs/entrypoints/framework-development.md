# Framework Development Entrypoint

Owner: PM Core Team  
Last updated: 2026-02-20

## When to use this doc

Use this document when you are developing Puppet Master itself (framework internals, shared architecture, core APIs, CSS contracts, tooling, and docs policy).

## First 30 minutes

1. Read `app/puppet-master.config.ts` to confirm mode/features surface.
2. Read `CONTRIBUTING.md` sections: pre-change checklist, framework rules, framework gap log.
3. Run baseline health:
   - `npm run lint`
   - `npm run test:run`
   - `npm run build`
4. Open critical architecture docs:
   - `docs/architecture/pm-architecture.md`
   - `docs/architecture/pm-system-overview.md`
5. Open release docs:
   - `docs/releases/README.md`
   - latest release file in `docs/releases/`

## Do not break these rules

1. No scoped styles in Vue components; use global PM CSS only.
2. Keep API input validation in Zod and return structured errors.
3. Keep server writes in Drizzle ORM (no raw SQL shortcuts).
4. Keep release docs exact: feature-by-feature and fix-by-fix.
5. Architecture/process PRs must update relevant entrypoint docs in the same PR.

## Read next

- `docs/architecture/pm-architecture.md`
- `docs/reference/configuration.md`
- `docs/reference/api-reference.md`
- `docs/styles/CSS_ARCHITECTURE.md`
- `docs/releases/README.md`
