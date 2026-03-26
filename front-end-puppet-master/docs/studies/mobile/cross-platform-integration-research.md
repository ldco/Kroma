# Cross-Platform App Integration Research

Date: 2026-02-20  
Scope: Find the best way to transform Puppet Master apps into desktop + mobile apps with minimal rework.

## Executive Decision

Best primary instrument for Puppet Master right now: **Tauri 2**.

Why:

1. It supports **desktop + mobile** with one frontend stack.
2. It works with existing web frontends (Nuxt is officially documented).
3. It includes **Linux desktop** support, unlike first-party React Native paths.

Secondary fallback (if team wants lower mobile setup friction first): **Capacitor for mobile + keep web/PWA for desktop**, then migrate desktop shell to Tauri.

## Puppet Master Constraints

Puppet Master is currently:

- Nuxt 4 + Nitro/H3 APIs
- SQLite + Drizzle
- Config-driven and CSS-architecture strict

Important implication:

- Tauri Nuxt guidance requires static frontend output (`ssr: false`, static preset) because Tauri does not ship a Node runtime in-app.
- Therefore, PM mobile/desktop shells should use:
  - bundled static frontend + remote PM backend API, or
  - hosted URL shell mode for fastest MVP.

## Candidate Comparison

### Electron

Fit:

- Good for desktop wrapping of existing web apps.
- Not suitable for mobile targets.

Verdict:

- **Desktop-only option**, not a unified PM mobile+desktop instrument.

### Capacitor

Fit:

- Very strong mobile-web bridge for existing JavaScript apps.
- Officially centered on iOS/Android/Web.

Risks for PM full cross-platform requirement:

- Desktop is not first-class in official docs.
- Community Electron plugin exists but shows stale release cadence.

Verdict:

- **Good mobile bridge**, not the best single tool for desktop+mobile+Linux together.

### Tauri 2

Fit:

- Officially positions itself as desktop + mobile capable (macOS, Windows, Linux, Android, iOS).
- Works with web frontends; Nuxt-specific integration is documented.

Risks:

- Mobile plugin ecosystem maturity is still uneven (not all plugins equally stable).
- Requires PM frontend/backend separation discipline for app-shell mode.

Verdict:

- **Best unified instrument for PM objective**.

### Flutter

Fit:

- Excellent platform coverage (mobile, desktop, web).

Tradeoff for PM:

- Flutter docs and workflows assume a Flutter codebase.
- For PM (Nuxt/Vue), this usually means major UI rewrite unless using a WebView wrapper.

Verdict:

- Powerful, but **not easiest for transforming existing PM apps**.

### React Native

Fit:

- Strong for mobile.

Tradeoff for PM requirement:

- Desktop support is mainly partner/community out-of-tree.
- Linux desktop appears via community paths, not first-party parity.

Verdict:

- Not ideal for PM’s Linux desktop requirement and low-rewrite objective.

## Recommended PM Strategy

### Recommendation A (Primary): Tauri 2 App Shell Mode

Target:

- One PM app-shell stack for desktop + mobile.

Architecture:

1. PM backend stays deployed as API service (Nitro + DB).
2. PM frontend runs in static app-shell mode for Tauri.
3. Shell talks to API over HTTPS with token/session strategy.

Benefits:

- Maximum reuse of PM frontend code.
- One shell technology across desktop and mobile.
- Linux desktop supported.

### Recommendation B (Fallback): Capacitor-First Mobile

Use when:

- Team needs immediate mobile packaging and already has strong JS mobile plugin familiarity.

Path:

1. Ship mobile via Capacitor quickly.
2. Keep desktop as web/PWA short-term.
3. Converge desktop to Tauri later for unified distribution.

## Integration Plan for Puppet Master

### Phase 0: Framework Prerequisites

1. Introduce explicit app-shell runtime mode:
   - `web`
   - `tauri`
   - `capacitor`
2. Ensure all critical data flows can run with remote API base URL.
3. Separate any server-only assumptions from frontend routes.

### Phase 1: Tauri Desktop Pilot

1. Create `apps/pm-shell-tauri/` workspace.
2. Build PM frontend in static mode for shell.
3. Validate auth, file uploads, notifications, and deep-link handling.
4. Ship Linux + Windows + macOS beta.

### Phase 2: Tauri Mobile Pilot

1. Add Android/iOS targets to same shell project.
2. Validate key native capabilities required by PM clients.
3. Define plugin policy:
   - stable-only plugins for production
   - pinned versions for reproducibility

### Phase 3: Framework Productization

1. Add PM docs command flow:
   - `npm run appshell:init`
   - `npm run appshell:build:<target>`
2. Publish "PM to App" guide with release checklist and store submission flow.
3. Add CI lanes for desktop + mobile shell builds.

## Key Risks and Mitigations

1. Risk: PM server-side assumptions break in static shell mode.  
Mitigation: formalize API-first boundaries and test in shell mode early.

2. Risk: Mobile plugin parity gaps in Tauri ecosystem.  
Mitigation: maintain required-capability matrix and pin plugin versions.

3. Risk: Teams pick community-only desktop bridges with weak maintenance.  
Mitigation: default framework guidance should be Tauri-first for unified targets.

## Decision Matrix (PM-Specific)

Scale: 1 (poor) to 5 (strong)

| Option | Reuse Existing PM Nuxt App | Desktop + Mobile Coverage | Linux Desktop | Unified Stack | PM Recommendation |
|---|---:|---:|---:|---:|---|
| Electron | 5 | 2 | 5 | 2 | No (desktop only) |
| Capacitor | 5 | 3 | 1 | 3 | Partial (mobile-first fallback) |
| Tauri 2 | 4 | 5 | 5 | 5 | **Yes (primary)** |
| Flutter | 1 | 5 | 5 | 4 | No (rewrite-heavy for PM) |
| React Native | 1 | 3 | 2 | 3 | No (Linux desktop not first-party) |

## Sources

Official and primary references used:

1. Tauri homepage (cross-platform claim): https://v2.tauri.app/
2. Tauri Nuxt integration (static/SSR constraints): https://v2.tauri.app/start/frontend/nuxt/
3. Tauri 2 stable and mobile support notes: https://v2.tauri.app/es/blog/tauri-20/
4. Tauri RC note on mobile parity caveats: https://v2.tauri.app/fr/blog/tauri-2-0-0-release-candidate/
5. Electron docs homepage (desktop framework): https://www.electronjs.org/docs/latest/
6. Capacitor docs intro (existing JS project, iOS/Android focus): https://capacitorjs.com/docs
7. Capacitor support policy (supported platforms policy): https://capacitorjs.com/docs/main/reference/support-policy
8. Capacitor community Electron plugin repository: https://github.com/capacitor-community/electron
9. Flutter supported deployment platforms: https://docs.flutter.dev/reference/supported-platforms
10. Flutter desktop support docs: https://docs.flutter.dev/platform-integration/desktop
11. React Native out-of-tree platforms: https://reactnative.dev/docs/0.75/out-of-tree-platforms

## Next Document

Implementation roadmap for PM team:

- `docs/studies/mobile/appshell-tauri-implementation-plan.md`
