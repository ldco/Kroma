# PM AppShell (Tauri) Implementation Plan

Date: 2026-02-20  
Status: Planning document for future implementation

## Goal

Enable Puppet Master projects to be transformed into desktop and mobile apps with minimal rework by introducing a first-class **AppShell mode** using Tauri 2.

## Scope

In scope:

1. Tauri 2 app shell for desktop (Linux, Windows, macOS)
2. Tauri 2 app shell for mobile (Android, iOS)
3. Reuse existing PM Nuxt frontend with app-shell build mode
4. Keep PM backend as Nuxt/Nitro API service

Out of scope:

1. Rewriting PM UI in Flutter/React Native
2. Replacing PM backend stack
3. Packaging local Node/Nitro as embedded runtime for mobile

## Core Architecture (Target State)

Runtime split:

1. `web` mode: existing PM full web runtime
2. `appshell` mode: static Nuxt frontend bundle inside Tauri shell

Request flow in appshell mode:

1. Tauri WebView loads static PM frontend
2. Frontend calls remote PM API (`API_BASE_URL`)
3. PM backend (Nuxt/Nitro + DB) remains server-hosted

Why this architecture:

1. Keeps PM backend unchanged
2. Works for desktop and mobile
3. Matches Nuxt + Tauri constraints for shell builds

## Required Framework Additions

### 1. Config Surface

Add app-shell config in `app/puppet-master.config.ts`:

```ts
appShell: {
  enabled: false,
  platform: 'web' as 'web' | 'tauri',
  apiBaseUrl: '',
  offlineMode: false
}
```

Type updates in `app/types/config.ts`:

```ts
export type AppShellPlatform = 'web' | 'tauri'
```

### 2. Runtime Environment Contract

Introduce environment flags:

1. `NUXT_PUBLIC_PM_RUNTIME=web|appshell`
2. `NUXT_PUBLIC_API_BASE_URL=https://api.example.com`
3. `NUXT_PUBLIC_PM_APPSHELL=true|false`

Rules:

1. AppShell mode must not depend on server-side page rendering
2. All critical data access must route through API client layer

### 3. Frontend Data Access Layer

Create a unified API client abstraction:

1. `app/composables/useApiClient.ts`
2. Centralize base URL selection by runtime mode
3. Keep auth token/session transport consistent

### 4. Auth Session Strategy

Default strategy for appshell:

1. API token-based auth with refresh flow
2. Secure storage through Tauri plugin (no plain localStorage for long-lived secrets)
3. Optional cookie mode for trusted same-origin deployments

### 5. Tauri Workspace

Add workspace:

1. `apps/pm-shell-tauri/`
2. `src-tauri/` config and Rust shell
3. Build scripts for desktop and mobile targets

Planned script surface (root `package.json`):

```bash
npm run appshell:init
npm run appshell:dev:desktop
npm run appshell:build:desktop
npm run appshell:dev:android
npm run appshell:build:android
npm run appshell:dev:ios
npm run appshell:build:ios
```

## Milestone Plan

### M0: Readiness and Boundaries

Deliverables:

1. Runtime boundary doc (web vs appshell)
2. API dependency map for frontend routes
3. List of SSR-dependent pages/composables

Acceptance criteria:

1. Every user-critical route classified as SSR-dependent or API-compatible
2. No unknown runtime dependencies

### M1: Framework Runtime Toggle

Deliverables:

1. Config/types additions for app-shell
2. Environment-based runtime selection
3. API base URL routing in one client layer

Acceptance criteria:

1. Web mode behavior unchanged
2. AppShell mode can boot frontend against remote API

### M2: Desktop Pilot (Tauri)

Deliverables:

1. `apps/pm-shell-tauri/` initialized
2. Linux/Windows/macOS dev build scripts
3. Signed desktop build pipeline draft

Acceptance criteria:

1. Login, dashboard, CRUD, uploads, logout work on Linux desktop first
2. No hard dependency on local Node runtime inside shell

### M3: Mobile Pilot (Tauri)

Deliverables:

1. Android/iOS shell setup
2. Secure credential storage integration
3. Deep-link and app lifecycle handling

Acceptance criteria:

1. Same core flows pass on Android
2. iOS build and sign path documented

### M4: Productization

Deliverables:

1. PM docs for project owners ("Convert PM project to app")
2. CI lanes for desktop and mobile builds
3. Release checklist and rollback procedure

Acceptance criteria:

1. A PM project owner can package app builds via documented commands
2. Framework team can cut repeatable app-shell releases

## File-Level Future Work Map

| Area | Planned Files |
|---|---|
| Config & Types | `app/puppet-master.config.ts`, `app/types/config.ts` |
| Runtime flags | `nuxt.config.ts`, `.env.example` |
| API abstraction | `app/composables/useApiClient.ts` |
| Auth storage | `app/composables/useAuth.ts` and appshell auth adapter |
| Shell workspace | `apps/pm-shell-tauri/*` |
| Build scripts | root `package.json`, optional `scripts/appshell/*` |
| Docs | `docs/guides/*`, `docs/releases/*` |

## Compatibility Requirements

Hard requirements before GA:

1. Linux desktop support
2. Android support
3. No breaking changes for existing PM web projects
4. Documented migration path from web-only PM to appshell PM

## Security Requirements

1. Enforce HTTPS API base URL in production appshell builds
2. Store long-lived credentials in secure storage plugin
3. Avoid exposing admin secrets in public runtime config
4. Apply strict CSP and navigation allowlist in shell

## QA Matrix (Minimum)

Core scenarios to pass:

1. Auth login/logout/token refresh
2. Admin CRUD flows
3. File upload flows
4. Offline/network loss handling
5. App update flow and rollback test

Platforms:

1. Linux desktop (primary)
2. Windows desktop
3. macOS desktop
4. Android
5. iOS

## Migration Path for Existing PM Projects

1. Update framework version to appshell-capable release
2. Enable appshell config in `app/puppet-master.config.ts`
3. Set `NUXT_PUBLIC_API_BASE_URL`
4. Run appshell build commands
5. Execute QA matrix before distribution

## Risks and Controls

1. Risk: Hidden SSR dependencies in frontend routes  
Control: M0 dependency map and runtime tests

2. Risk: Mobile plugin instability  
Control: allowlist only stable plugins and pin versions

3. Risk: Auth/session inconsistency between web and appshell  
Control: single auth contract and shared integration tests

## Release Strategy

Suggested release sequence:

1. `1.x` experimental flag release (`appShell.enabled`)
2. Desktop beta release
3. Android beta release
4. iOS beta release
5. Stable GA after matrix completion

## References

1. Cross-platform research: `docs/studies/mobile/cross-platform-integration-research.md`
2. Setup workflows: `docs/guides/setup-workflows.md`
3. Release policy: `docs/releases/README.md`
