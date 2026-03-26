# Mobile Framework Research: Flutter vs React Native vs Tauri

Date: 2026-02-20  
Scope: Mobile implementation options for Puppet Master (Android + iOS).

## Executive Summary

For Puppet Master specifically, **Tauri 2 (mobile)** is the best default path when the goal is to reuse the existing Nuxt codebase with the least rewrite.

If the goal shifts to a highly native, fully custom mobile UI stack and the team accepts a rewrite, choose:

1. **Flutter** when you want one codebase with strong platform coverage and tight rendering control.
2. **React Native** when your team is already React/Expo-native and wants that ecosystem.

## What Matters for Puppet Master

PM today is Nuxt 4 + Nitro/H3 + Drizzle/SQLite and a config-driven architecture.  
So the key mobile criteria are:

1. Reuse of existing PM frontend and API flows
2. Migration effort from current Vue/Nuxt implementation
3. Native capability access (camera, biometrics, deep links, etc.)
4. Long-term maintainability and release cadence
5. Risk of ecosystem gaps for required mobile features

## Option 1: Flutter

### Verified facts

1. Flutter supports Android and iOS, with explicit supported version ranges documented (as of Flutter 3.41.2: Android 24-36, iOS 13-26).  
2. Impeller is default on iOS and Android API 29+ and is designed to reduce runtime shader compilation stutter through precompilation.
3. "Add-to-app" is officially supported for Android and iOS (plus web), with documented mobile limitations.

### PM impact

Pros:

1. Strong mobile-first framework with mature rendering/runtime behavior.
2. Good long-term platform breadth.

Cons:

1. Requires a major rewrite from PM Vue/Nuxt UI to Dart/Flutter widgets.
2. Lower direct reuse of existing PM frontend layer.

Use Flutter if:

1. You are ready to build a dedicated mobile product surface, not just package PM web UX.
2. Mobile UI/animation fidelity is a top priority and rewrite cost is acceptable.

## Option 2: React Native

### Verified facts

1. Official setup guide targets Android and iOS.
2. New Architecture became default in 0.76 and is the only runtime from 0.82 onward.
3. 0.84 made Hermes V1 default and continues removal of legacy architecture components.
4. React Native has official "integration with existing apps" guidance for embedding flows into native apps.
5. Non-mobile platforms are explicitly handled through partner/community "out-of-tree" projects.

### PM impact

Pros:

1. Large ecosystem and strong community velocity.
2. JS/TS-friendly if team is React-native skilled.

Cons:

1. PM uses Vue/Nuxt, so UI layer still needs substantial rewrite to React Native components.
2. Linux desktop is not a first-party core path in RN itself.

Use React Native if:

1. Your delivery org already runs React Native/Expo at scale.
2. You accept rewrite effort for a native-mobile-first strategy.

## Option 3: Tauri 2 (Mobile)

### Verified facts

1. Tauri 2 stable positions support for Android and iOS.
2. Tauri’s Nuxt integration guide explicitly requires static output (`ssr: false`) and states Tauri does not support server-based SSR solutions.
3. Tauri plugin development supports Kotlin/Java (Android) and Swift (iOS).
4. Tauri maintainers explicitly noted that not all desktop features/plugins are available on mobile yet, and plugin stability can vary by plugin.

### PM impact

Pros:

1. Highest reuse of PM frontend code because PM is already web-stack based.
2. Fastest path to "mobile app packaging" with minimal product-level rewrite.
3. Plugin path allows native extensions when needed.

Cons:

1. Requires explicit runtime split (appshell static frontend + remote PM API backend).
2. Mobile plugin parity is improving but not uniformly complete.

Use Tauri if:

1. You want fastest PM-to-mobile conversion with lowest frontend rewrite cost.
2. Your priority is shipping PM projects to mobile quickly, then iterating on native depth where needed.

## Decision Matrix (PM Mobile Priority)

Scale: 1 (poor) to 5 (strong)

| Criterion | Flutter | React Native | Tauri 2 |
|---|---:|---:|---:|
| Reuse PM Nuxt frontend | 1 | 1 | 5 |
| Mobile native UX potential | 5 | 4 | 3 |
| Integration speed for PM today | 2 | 2 | 5 |
| Native extension path | 4 | 5 | 4 |
| Ecosystem predictability (mobile) | 5 | 5 | 3 |
| Overall fit for PM short/mid term | 3 | 3 | **5** |

## Recommendation for Puppet Master

### Primary recommendation

1. Adopt **Tauri 2 mobile** as PM’s first mobile implementation path.
2. Keep PM backend as Nuxt/Nitro API service.
3. Build PM app-shell frontend in static mode for mobile packaging.

### Secondary recommendation

1. For teams that need deeply native mobile UX and accept rewrite cost, create a separate **Flutter track**.
2. Use **React Native** only when team capability/existing stack strongly favors RN/Expo.

## Suggested next implementation steps

1. Formalize PM runtime modes: `web` and `appshell`.
2. Introduce explicit `API_BASE_URL` contract for appshell builds.
3. Pilot one PM module end-to-end on Android using Tauri mobile.
4. Validate required native features against Tauri plugin support before GA.

## Sources

Primary sources used:

1. Flutter supported platforms: https://docs.flutter.dev/reference/supported-platforms
2. Flutter Impeller: https://docs.flutter.dev/perf/impeller
3. Flutter add-to-app: https://docs.flutter.dev/add-to-app
4. Flutter upgrade/channels: https://docs.flutter.dev/install/upgrade
5. React Native setup (targets Android/iOS): https://reactnative.dev/docs/set-up-your-environment
6. React Native New Architecture default (0.76): https://reactnative.dev/blog/2024/10/23/release-0.76-new-architecture
7. React Native New Architecture only runtime (0.82): https://reactnative.dev/blog/2025/10/08/react-native-0.82
8. React Native 0.84 (Hermes V1 default, legacy removals): https://reactnative.dev/blog/2026/02/11/react-native-0.84
9. React Native integration with existing apps: https://reactnative.dev/docs/integration-with-existing-apps.html
10. React Native out-of-tree platforms: https://reactnative.dev/docs/0.78/out-of-tree-platforms
11. Tauri Nuxt guide (`ssr: false`): https://v2.tauri.app/start/frontend/nuxt/
12. Tauri 2 stable release (mobile + plugin model caveats): https://v2.tauri.app/blog/tauri-20/
13. Tauri mobile plugin development: https://v2.tauri.app/develop/plugins/develop-mobile/
14. Tauri prerequisites (mobile targets): https://v2.tauri.app/start/prerequisites/
