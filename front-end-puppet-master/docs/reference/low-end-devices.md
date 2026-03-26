# Low-End Device Optimization Guide

This guide explains how to optimize PuppetMaster sites for low-end devices like Raspberry Pi, budget Android phones, and older hardware.

## Table of Contents

1. [Understanding the Problem](#understanding-the-problem)
2. [Quick Start: Enable Islands](#quick-start-enable-islands)
3. [Component Classification](#component-classification)
4. [Migration Guide](#migration-guide)
5. [Configuration Options](#configuration-options)
6. [CSS Optimization](#css-optimization)
7. [Image Optimization](#image-optimization)
8. [Font Optimization](#font-optimization)
9. [Testing on Low-End Devices](#testing-on-low-end-devices)
10. [Performance Targets](#performance-targets)

---

## Understanding the Problem

### Why Low-End Devices Struggle

| Resource | High-End Device | Low-End Device | Impact |
|----------|-----------------|----------------|--------|
| CPU | 8-core @ 3GHz | 4-core @ 1.5GHz | JS parsing 4-10x slower |
| RAM | 8-16GB | 1-4GB | Page swapping, crashes |
| Network | 5G/WiFi 6 | 3G/4G/WiFi 4 | Slower asset loading |
| GPU | Dedicated | Integrated/None | Animation jank |

### The JavaScript Tax

Vue/Nuxt ships a runtime (~45KB gzipped) that must be:
1. **Downloaded** - Network dependent
2. **Parsed** - CPU intensive
3. **Compiled** - CPU intensive
4. **Executed** - CPU + Memory intensive

On a Raspberry Pi 4, parsing 270KB of JavaScript takes ~800ms.
On a Raspberry Pi 3, it takes ~1.5 seconds.

### The Solution: Islands Architecture

Islands Architecture = **selective hydration**. Only interactive components ship JavaScript to the client. Static content renders as plain HTML.

```
Before (Full Hydration):
┌─────────────────────────────────────┐
│  Everything hydrates (270KB JS)    │
│  Header | Hero | About | Footer    │  ← ALL need JS
└─────────────────────────────────────┘

After (Islands):
┌─────────────────────────────────────┐
│  Only interactive parts (80KB JS)  │
│  Header | Hero | About | Footer    │
│    JS   | HTML | HTML  | HTML      │  ← Most is just HTML
└─────────────────────────────────────┘
```

---

## Quick Start: Enable Islands

### Step 1: Enable in Configuration

```typescript
// nuxt.config.ts
export default defineNuxtConfig({
  experimental: {
    componentIslands: true,
    payloadExtraction: true,  // Smaller hydration data
  }
})
```

### Step 2: Identify Static Components

Components that are **static** (no user interaction):
- Display-only content (text, images)
- Navigation links (CSS hover is fine)
- Social media link lists
- Footer content
- Hero sections (unless animated via JS)

Components that are **interactive** (need hydration):
- Forms with validation
- Theme toggles
- Language switchers
- Mobile menu hamburger
- Carousels/sliders with JS controls
- Any component with `@click`, `v-model`, `ref()`, `watch()`

### Step 3: Convert Static Components

Rename static components with `.server.vue` suffix:

```bash
# Before
components/sections/SectionHero.vue

# After (just rename!)
components/sections/SectionHero.server.vue
```

Or move to a `server/` subdirectory:

```bash
# Alternative: server directory
components/server/SectionHero.vue
```

---

## Component Classification

### PuppetMaster Component Audit

#### Atoms (Simple building blocks)

| Component | Interactive? | Convert to Island? |
|-----------|--------------|-------------------|
| `BaseButton.vue` | Yes (click) | No |
| `BaseIcon.vue` | No | **Yes** → `.server.vue` |
| `BaseLogo.vue` | No | **Yes** → `.server.vue` |
| `BaseImage.vue` | No | **Yes** → `.server.vue` |
| `BaseLink.vue` | No (just `<a>`) | **Yes** → `.server.vue` |
| `BaseInput.vue` | Yes (v-model) | No |
| `BaseTextarea.vue` | Yes (v-model) | No |

#### Molecules (Combined atoms)

| Component | Interactive? | Convert to Island? |
|-----------|--------------|-------------------|
| `SocialNav.vue` | No (just links) | **Yes** → `.server.vue` |
| `NavLinks.vue` | Maybe (active state) | Evaluate |
| `ThemeToggle.vue` | Yes (click) | No |
| `LangSwitcher.vue` | Yes (click) | No |
| `ContactInfo.vue` | No (display) | **Yes** → `.server.vue` |
| `MadeWith.vue` | No (display) | **Yes** → `.server.vue` |

#### Organisms (Complex components)

| Component | Interactive? | Convert to Island? |
|-----------|--------------|-------------------|
| `HeaderNav.vue` | Yes (scroll, hamburger) | No |
| `FooterRich.vue` | No (links only) | **Yes** → `.server.vue` |
| `MobileNav.vue` | Yes (gestures) | No |
| `ContactForm.vue` | Yes (form) | No |

#### Sections (Page sections)

| Component | Interactive? | Convert to Island? |
|-----------|--------------|-------------------|
| `SectionHero.vue` | No (static) | **Yes** → `.server.vue` |
| `SectionAbout.vue` | No (static) | **Yes** → `.server.vue` |
| `SectionServices.vue` | No (static) | **Yes** → `.server.vue` |
| `SectionPortfolio.vue` | Maybe (gallery) | Evaluate |
| `SectionContact.vue` | Yes (form) | No |

---

## Migration Guide

### Converting a Component to Server Island

#### Before: Regular Component

```vue
<!-- components/sections/SectionAbout.vue -->
<script setup lang="ts">
/**
 * About Section
 * Displays company information.
 */
const { t } = useI18n()
const settings = useSiteSettings()
</script>

<template>
  <section id="about" class="section section-about">
    <div class="container">
      <h2 class="section-title">{{ t('about.title') }}</h2>
      <p class="section-content">{{ t('about.content') }}</p>
    </div>
  </section>
</template>
```

#### After: Server Island (Just Rename!)

```vue
<!-- components/sections/SectionAbout.server.vue -->
<script setup lang="ts">
/**
 * About Section (Server Island)
 * Renders on server only - no JavaScript shipped to client.
 */
const { t } = useI18n()  // Works in server components
const settings = useSiteSettings()  // Works in server components
</script>

<template>
  <section id="about" class="section section-about">
    <div class="container">
      <h2 class="section-title">{{ t('about.title') }}</h2>
      <p class="section-content">{{ t('about.content') }}</p>
    </div>
  </section>
</template>
```

**That's it!** The component now renders server-side only.

### What Works in Server Components

```vue
<script setup>
// ✅ Data fetching
const { data } = await useFetch('/api/settings')

// ✅ i18n
const { t, locale } = useI18n()

// ✅ Runtime config
const config = useRuntimeConfig()

// ✅ Composables that don't use browser APIs
const settings = useSiteSettings()

// ✅ Route info
const route = useRoute()

// ✅ Async operations
const items = await $fetch('/api/items')
</script>

<template>
  <!-- ✅ Static content -->
  <div>{{ data.title }}</div>

  <!-- ✅ Links (regular and NuxtLink) -->
  <a href="/about">About</a>
  <NuxtLink to="/contact">Contact</NuxtLink>

  <!-- ✅ CSS interactions (hover, focus, transitions) -->
  <button class="btn">Hover works via CSS!</button>

  <!-- ✅ Conditional rendering -->
  <div v-if="settings.showBanner">Banner</div>

  <!-- ✅ Loops -->
  <ul>
    <li v-for="item in items" :key="item.id">{{ item.name }}</li>
  </ul>
</template>
```

### What Does NOT Work in Server Components

```vue
<script setup>
// ❌ Reactive state
const count = ref(0)  // ERROR
const state = reactive({})  // ERROR

// ❌ Computed properties
const doubled = computed(() => count.value * 2)  // ERROR

// ❌ Watchers
watch(count, () => {})  // ERROR

// ❌ Lifecycle hooks
onMounted(() => {})  // ERROR
onBeforeUnmount(() => {})  // ERROR

// ❌ Browser APIs
const width = window.innerWidth  // ERROR
document.querySelector()  // ERROR

// ❌ Composables using browser APIs
const { x, y } = useMouse()  // ERROR if it uses window
</script>

<template>
  <!-- ❌ Event handlers -->
  <button @click="handleClick">Click</button>  <!-- ERROR -->

  <!-- ❌ v-model -->
  <input v-model="text" />  <!-- ERROR -->

  <!-- ❌ Dynamic classes bound to reactive state -->
  <div :class="{ active: isActive }">  <!-- ERROR if isActive is ref -->
</template>
```

### Splitting Interactive Parts

If a component is mostly static with one interactive part, split it:

```vue
<!-- components/sections/SectionHero.server.vue -->
<template>
  <section class="section-hero">
    <h1>{{ t('hero.title') }}</h1>
    <p>{{ t('hero.subtitle') }}</p>

    <!-- Embed interactive island within server component -->
    <HeroButtons />  <!-- This stays as regular .vue -->
  </section>
</template>
```

```vue
<!-- components/sections/HeroButtons.vue (stays interactive) -->
<script setup>
const handleGetStarted = () => {
  navigateTo('/services')
}
</script>

<template>
  <div class="hero-buttons">
    <button @click="handleGetStarted">Get Started</button>
  </div>
</template>
```

---

## Configuration Options

### nuxt.config.ts Optimization

```typescript
// nuxt.config.ts
export default defineNuxtConfig({
  // Enable Islands
  experimental: {
    componentIslands: true,
    payloadExtraction: true,
    renderJsonPayloads: true,
  },

  // Optimize builds
  vite: {
    build: {
      // Smaller chunks
      chunkSizeWarningLimit: 500,
      rollupOptions: {
        output: {
          // Manual chunk splitting
          manualChunks: {
            'vue-vendor': ['vue', 'vue-router'],
          }
        }
      }
    }
  },

  // Route-level optimization
  routeRules: {
    // Pre-render static pages
    '/': { prerender: true },
    '/about': { prerender: true },
    '/services': { prerender: true },

    // SPA mode for admin (needs full interactivity)
    '/admin/**': { ssr: true },
  },

  // Reduce bundle size
  build: {
    analyze: false,  // Enable to see bundle composition
  }
})
```

### puppet-master.config.ts Low-End Settings

```typescript
// puppet-master.config.ts
const config = {
  features: {
    // Disable heavy features for low-end
    pageTransitions: '',  // Disable transitions
    interactiveHeader: false,  // Simpler header
    hideHeaderOnScroll: false,  // Less JS

    // Keep essential features
    multiLangs: true,
    doubleTheme: true,
  }
}
```

---

## CSS Optimization

### Use CSS Instead of JavaScript

```css
/* ✅ CSS hover effects (work in server components) */
.button {
  transition: transform 0.2s, background-color 0.2s;
}
.button:hover {
  transform: translateY(-2px);
  background-color: var(--color-brand-hover);
}

/* ✅ CSS-only mobile menu (no JS needed) */
.mobile-nav {
  transform: translateX(-100%);
  transition: transform 0.3s;
}
.mobile-nav:target,
.mobile-nav.is-open {
  transform: translateX(0);
}

/* ✅ CSS scroll animations (if supported) */
@supports (animation-timeline: scroll()) {
  .fade-in {
    animation: fadeIn linear;
    animation-timeline: view();
  }
}
```

### Reduce CSS Size with PurgeCSS

Already configured in `nuxt.config.ts`:

```typescript
postcss: {
  plugins: {
    '@fullhuman/postcss-purgecss': {
      content: ['./app/**/*.vue', './app/**/*.ts'],
      safelist: {
        standard: [/^dark/, /^light/, /^page-/, /^router-/],
        variables: [/^--/]
      }
    }
  }
}
```

---

## Image Optimization

### Use Optimized Formats

```vue
<!-- ✅ Use WebP with fallback -->
<picture>
  <source srcset="/images/hero.webp" type="image/webp">
  <img src="/images/hero.jpg" alt="Hero" loading="lazy">
</picture>
```

### Lazy Load Below-Fold Images

```vue
<!-- ✅ Native lazy loading -->
<img src="/images/photo.jpg" loading="lazy" alt="Photo">

<!-- ✅ With dimensions (prevents layout shift) -->
<img
  src="/images/photo.jpg"
  loading="lazy"
  width="800"
  height="600"
  alt="Photo"
>
```

### Optimize SVGs

Run SVGO on all SVG assets:

```bash
npx svgo public/logos/*.svg --multipass
npx svgo public/icons/*.svg --multipass
```

This typically reduces SVG size by 60-70%.

---

## Font Optimization

### Current Implementation

PuppetMaster loads fonts asynchronously via `plugins/fonts.client.ts`:

```typescript
// Non-blocking font loading
export default defineNuxtPlugin(() => {
  const link = document.createElement('link')
  link.rel = 'stylesheet'
  link.href = 'https://fonts.googleapis.com/css2?family=Montserrat:wght@400;500;600;700;900&display=swap'
  link.media = 'print'  // Non-blocking
  link.onload = () => { link.media = 'all' }
  document.head.appendChild(link)
})
```

### Further Optimization: Reduce Font Weights

If you don't need all weights:

```typescript
// Only load weights you actually use
const fontUrl = 'https://fonts.googleapis.com/css2?family=Montserrat:wght@400;700&display=swap'
// Reduced from 5 weights to 2 = ~60% smaller font download
```

### Self-Host for Offline/Low-Latency

```bash
# Download fonts locally
npm install @fontsource/montserrat
```

```typescript
// nuxt.config.ts
css: [
  '@fontsource/montserrat/400.css',
  '@fontsource/montserrat/700.css',
]
```

---

## Testing on Low-End Devices

### Lighthouse with Throttling

```bash
# Mobile simulation (slow 4G, 4x CPU slowdown)
npx lighthouse http://localhost:3000 --preset=mobile

# Custom throttling for Pi-like device
npx lighthouse http://localhost:3000 \
  --throttling.cpuSlowdownMultiplier=6 \
  --throttling.downloadThroughputKbps=1000
```

### Chrome DevTools Throttling

1. Open DevTools → Performance tab
2. Click gear icon → CPU: 6x slowdown
3. Network tab → Slow 3G

### Real Device Testing

#### Raspberry Pi

```bash
# On your dev machine, expose the server
npm run dev -- --host 0.0.0.0

# On Pi's Chromium browser
http://YOUR_DEV_IP:3000
```

#### Android Emulator

Use Android Studio with a low-end device profile:
- Pixel 2 (2017)
- Nexus 5X (2015)

### Performance Budgets

Add to your CI:

```javascript
// lighthouse-budget.json
{
  "resourceSizes": [
    { "resourceType": "script", "budget": 150 },
    { "resourceType": "stylesheet", "budget": 50 },
    { "resourceType": "total", "budget": 500 }
  ],
  "timings": [
    { "metric": "first-contentful-paint", "budget": 2000 },
    { "metric": "interactive", "budget": 4000 }
  ]
}
```

---

## Performance Targets

### Target Metrics by Device Class

| Device Class | FCP | LCP | TTI | JS Budget |
|--------------|-----|-----|-----|-----------|
| High-end (iPhone 15) | <1s | <1.5s | <2s | 300KB |
| Mid-range (Pixel 6a) | <1.5s | <2.5s | <3s | 200KB |
| Low-end (Pi 4) | <2.5s | <4s | <5s | 150KB |
| Very low-end (Pi 3) | <4s | <6s | <8s | 100KB |

### Expected Results After Islands Migration

| Metric | Before | After Islands |
|--------|--------|---------------|
| JS Bundle | ~270KB | ~150KB |
| Lighthouse Mobile | 75-80 | 85-92 |
| FCP (mobile) | 3.5s | 2.5s |
| TTI (mobile) | 5s | 3.5s |
| Pi 4 usable | Sluggish | Smooth |
| Pi 3 usable | Painful | Acceptable |

---

## Checklist for Low-End Optimization

### Before Launch

- [ ] Enable `componentIslands: true` in nuxt.config.ts
- [ ] Convert static sections to `.server.vue`
- [ ] Convert static organisms (Footer, SocialNav) to `.server.vue`
- [ ] Run SVGO on all SVG assets
- [ ] Reduce font weights if possible
- [ ] Enable PurgeCSS for production builds
- [ ] Test with Lighthouse mobile throttling
- [ ] Test on actual low-end device if possible

### Component Conversion Checklist

- [ ] `SectionHero.vue` → `SectionHero.server.vue`
- [ ] `SectionAbout.vue` → `SectionAbout.server.vue`
- [ ] `SectionServices.vue` → `SectionServices.server.vue`
- [ ] `FooterRich.vue` → `FooterRich.server.vue`
- [ ] `SocialNav.vue` → `SocialNav.server.vue`
- [ ] `MadeWith.vue` → `MadeWith.server.vue`
- [ ] `BaseLogo.vue` → `BaseLogo.server.vue`
- [ ] `ContactInfo.vue` → `ContactInfo.server.vue`

### Verification

- [ ] Build completes without errors
- [ ] All pages render correctly
- [ ] Interactive components still work (forms, toggles)
- [ ] Lighthouse mobile score improved
- [ ] Test on slowest target device

---

## Troubleshooting

### "Cannot use X in server component"

The component uses reactive state or browser APIs. Either:
1. Keep it as regular `.vue` (needs hydration)
2. Split into static part (`.server.vue`) + interactive part (`.vue`)

### "Hydration mismatch" errors

Server and client rendered different HTML. Common causes:
- Using `Date.now()` or random values
- Conditional rendering based on browser state
- Fix: Use `<ClientOnly>` wrapper or ensure deterministic rendering

### Server component not updating

Server components don't react to client-side state changes. If you need reactivity:
1. Use `<NuxtIsland>` with `:props` that trigger re-fetch
2. Or keep as regular component

---

## Resources

- [Nuxt Islands Documentation](https://nuxt.com/docs/guide/directory-structure/components#server-components)
- [Web.dev Performance Guide](https://web.dev/performance/)
- [Lighthouse Performance Scoring](https://developer.chrome.com/docs/lighthouse/performance/performance-scoring/)
