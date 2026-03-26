# Brownfield Migration Entrypoint

Owner: PM Core Team  
Last updated: 2026-02-20

## When to use this doc

Use this document when moving an existing product into Puppet Master or upgrading an existing Puppet Master project across releases.

## First 30 minutes

1. Inventory current project contracts:
   - public routes
   - admin routes and roles
   - database schema/migrations
   - CSS utility usage
2. Read current and target release docs in `docs/releases/`.
3. Create migration branch and run baseline checks:
   - `npm run lint`
   - `npm run test:run`
   - `npm run build`
4. Apply required release upgrade actions in order.
5. Re-run checks and capture any contract deltas.

## Do not break these rules

1. Never skip release-by-release upgrade actions.
2. Keep API response envelope/auth checks consistent on touched endpoints.
3. Do not introduce scoped styles while migrating legacy UI.
4. Keep sanitization rules on all `v-html` render paths.
5. Record local adoption status in release sync docs before deployment.

## Read next

- `docs/releases/README.md`
- latest target release note in `docs/releases/`
- `docs/reference/api-reference.md`
- `docs/reference/configuration.md`
- `docs/architecture/pm-system-overview.md`
