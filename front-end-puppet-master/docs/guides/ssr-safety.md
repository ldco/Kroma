# SSR Safety Guide

**Date**: March 8, 2026  
**Status**: Production-tested  
**Related**: Annex B: March 8, 2026 — SSR Safety & Hydration Mismatch Fixes

---

## Overview

This guide covers SSR safety patterns for Puppet Master applications. Following these patterns prevents:

- Hydration class mismatches
- "Cannot read properties of undefined" errors during HMR
- Vue Router `history.state` warnings
- Runtime crashes during server-side rendering

---

## Problem 1: Direct Config Access During HMR

### The Issue

Direct access to `config.features.onepager` during Hot Module Replacement (HMR) causes undefined errors when the config import is temporarily unavailable during module reloading.

### ❌ Unsafe Pattern

```vue
<script setup lang="ts">
import config from '~/puppet-master.config'

function handleLogoClick(event: MouseEvent) {
  if (!config.features.onepager) return  // ❌ Crashes if config is undefined
}
</script>

<template>
  <a v-if="config.features.onepager" ...>  <!-- ❌ SSR crash -->
    ...
  </a>
</template>
```

### ✅ Safe Pattern

```vue
<script setup lang="ts">
import config from '~/puppet-master.config'

// Computed config values for SSR safety
const isOnepager = computed(() => config?.features?.onepager ?? false)

function handleLogoClick(event: MouseEvent) {
  if (!isOnepager.value) return  // ✅ Safe computed ref
}
</script>

<template>
  <a v-if="isOnepager" ...>  <!-- ✅ Safe computed ref -->
    ...
  </a>
</template>
```

### Using the `useSafeConfig()` Composable

For new components, use the `useSafeConfig()` composable:

```vue
<script setup lang="ts">
const { isOnepager, logoHorizontal, defaultLocale } = useSafeConfig()
</script>

<template>
  <img :src="logoHorizontal('dark')" :alt="'Logo'" />
</template>
```

---

## Problem 2: Hydration Class Mismatches

### The Issue

Server renders navigation links without knowing the current scroll position (scrollspy state), but client immediately applies `active` class based on scrollspy detection, causing class mismatch.

### ❌ Unsafe Pattern

```vue
<script setup lang="ts">
const activeSection = ref('home') // Scrollspy state
</script>

<template>
  <AtomsNavLink
    :is-active="activeSection === 'home'"  <!-- ❌ SSR/client mismatch -->
  />
</template>
```

### ✅ Safe Pattern

```vue
<script setup lang="ts">
const { isHydrated } = useHydration()
const activeSection = ref('home')
</script>

<template>
  <AtomsNavLink
    :is-active="isHydrated && activeSection === 'home'"  <!-- ✅ Hydration-safe -->
  />
</template>
```

### Using the `useHydration()` Composable

```vue
<script setup lang="ts">
const { isHydrated } = useHydration()

// Defer client-only state until hydration
const isActive = computed(() => isHydrated.value && someClientState)
</script>
```

---

## Problem 3: Vue Router `history.state` Warning

### The Warning

```
[Vue Router warn]: history.state seems to have been manually replaced 
without preserving the necessary values.
```

### Context

This warning appears when using `@nuxtjs/i18n`'s `setLocale()` function, which internally calls `history.replaceState()` without preserving existing state.

### Resolution

**The warning is cosmetic** — it doesn't break functionality. Keep using `setLocale()`:

```vue
<script setup lang="ts">
const { setLocale } = useI18n()

async function changeLanguage(code: string) {
  await setLocale(code)  // ✅ Warning is cosmetic, functionality works
}
</script>
```

**Do NOT** use workarounds like:
- ❌ `switchLocalePath` + `navigateTo` (causes 500 errors)
- ❌ Manual `history.replaceState` (doesn't prevent warning)

---

## Available Composables

### `useHydration()`

**Location**: `app/composables/useHydration.ts`

Tracks component hydration state to prevent SSR/client mismatches.

```typescript
const { isHydrated } = useHydration()
```

**Use cases**:
- Deferring active state on navigation links
- Client-only class bindings
- Scroll-dependent rendering

---

### `useSafeConfig()`

**Location**: `app/composables/useSafeConfig.ts`

Provides SSR-safe reactive access to build-time configuration.

```typescript
const {
  // Booleans
  isOnepager,
  isMultiLang,
  hasThemeToggle,
  hasWebsite,
  hasApp,
  hasAdmin,
  
  // Logo
  logoPath,
  logoHorizontal,
  
  // Locale
  defaultLocale,
  locales,
  
  // Theme
  defaultTheme,
  
  // Advanced
  getConfig
} = useSafeConfig()
```

**Use cases**:
- Conditional rendering based on features
- Logo path generation
- Locale-dependent logic

---

## ESLint Rule: SSR Safety

A custom ESLint rule warns on direct config access in Vue components:

```
'ssr-safety/no-direct-config-access': 'warn'
```

**Triggers when**: Importing `~/puppet-master.config` directly in a `.vue` file.

**Fix**: Use `useSafeConfig()` composable instead.

---

## Testing Checklist

Before deploying, verify:

- [ ] **Logo renders on SSR** - No crashes during server-side rendering
- [ ] **No hydration mismatches** - Console shows no class mismatch warnings
- [ ] **Active nav state works** - Correct section highlighted after hydration
- [ ] **Locale switching works** - Language changes preserve scroll position
- [ ] **HMR doesn't crash** - Hot reload works without undefined errors
- [ ] **Mobile menu closes correctly** - Navigation timing preserved

---

## Migration Guide

### Updating Existing Components

1. **Find direct config access**:
   ```bash
   grep -r "config\.features\." app/components --include="*.vue"
   ```

2. **Replace with computed + defensive defaults**:
   ```diff
   - const isOnepager = config.features.onepager
   + const isOnepager = computed(() => config?.features?.onepager ?? false)
   ```

3. **Or use `useSafeConfig()`**:
   ```diff
   - import config from '~/puppet-master.config'
   + const { isOnepager } = useSafeConfig()
   ```

4. **Add hydration markers for active states**:
   ```diff
   + const { isHydrated } = useHydration()
   
   - :is-active="activeSection === id"
   + :is-active="isHydrated && activeSection === id"
   ```

---

## Files Modified (Annex B)

| File | Changes | Lines Changed |
|------|---------|---------------|
| `app/components/atoms/Logo.vue` | Added `isOnepager` computed, defensive defaults | ~10 |
| `app/components/molecules/NavLinks.vue` | Added `isHydrated` tracking, deferred active state | ~5 |
| `app/composables/useHydration.ts` | New composable | ~25 |
| `app/composables/useSafeConfig.ts` | New composable | ~60 |
| `eslint-plugin-ssr-safety.js` | Custom ESLint plugin | ~60 |
| `eslint.config.js` | Added SSR safety rule | ~5 |

---

## Future Improvements

1. **Framework-level fix** - Provide SSR-safe config composable at framework level
2. **i18n module update** - `@nuxtjs/i18n` could preserve history state internally
3. **Hydration marker composable** - Already created as `useHydration()`
4. **ESLint rule** - Already implemented

---

## References

- [Vue 3 SSR Hydration](https://vuejs.org/guide/scaling-up/ssr.html#hydration-mismatches)
- [Nuxt SSR Best Practices](https://nuxt.com/docs/guide/going-further/ssr)
- [Annex B: March 8, 2026](./releases/annex-b-ssr-safety.md)
