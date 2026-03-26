# Greenfield Build Entrypoint

Owner: PM Core Team  
Last updated: 2026-02-20

## When to use this doc

Use this document when starting a brand-new project on Puppet Master (new product, no legacy system constraints).

## First 30 minutes

1. Install dependencies and bootstrap database:
   - `npm install`
   - `npm run db:push`
   - `npm run db:seed`
2. Decide runtime shape in `app/puppet-master.config.ts`:
   - `entities.website`
   - `entities.app`
   - `admin.enabled`
3. Set core feature flags and module toggles.
4. Start local runtime: `npm run dev`.
5. Validate baseline:
   - `npm run lint`
   - `npm run build`

## Do not break these rules

1. Keep config-driven behavior (avoid hardcoded nav/feature logic).
2. Keep role access aligned with config-driven admin sections.
3. Use global CSS classes and PM tokens only.
4. Keep footer attribution default enabled unless there is explicit opt-out requirement.
5. Document every framework-level pattern change in release notes.

## Read next

- `docs/guides/getting-started.md`
- `docs/guides/setup-workflows.md`
- `docs/reference/configuration.md`
- `docs/reference/security.md`
- `docs/releases/README.md`
