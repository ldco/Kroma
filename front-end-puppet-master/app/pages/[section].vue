<!--
  Dynamic Section Page (SPA Mode)

  Single dynamic route that renders section components based on URL.
  Uses SectionRenderer for automatic component resolution.

  Enables seamless switching between onepager and SPA modes:
  - onepager: true  → All sections on index.vue, anchor nav (#about)
  - onepager: false → Each section at its own route (/about)

  Adding a new section:
  1. Add to config.sections: { id: 'history', inNav: true }
  2. Create component: app/components/sections/SectionHistory.vue
  3. Add i18n: nav.history, history.title, etc.
  That's it - SectionRenderer handles the rest automatically.

  NOT handled (have dedicated pages):
  - /          → index.vue (hero + onepager sections)
  - /login     → login.vue
  - /admin/*   → admin/*.vue
-->

<script setup lang="ts">
import config from '~/puppet-master.config'
import { getDynamicWebsiteSectionIds } from '~/types/config'

definePageMeta({
  layout: 'default'
  // Uses global transition from nuxt.config.ts
  // To override for this page, uncomment:
  // pageTransition: { name: 'page-zoom', mode: 'out-in' }
})

const route = useRoute()
const { t, te } = useI18n()

// SSR-safe config access with defensive defaults
const sectionsConfig = computed(() => config?.sections ?? [])

// Get section ID from route param
const sectionId = computed(() => route.params.section as string)

// Valid sections for this dynamic route (excludes home which is index.vue)
const dynamicSections: string[] = getDynamicWebsiteSectionIds(sectionsConfig)

// 404 if section doesn't exist - reactive to route changes
watch(sectionId, (newSection) => {
  if (!dynamicSections.includes(newSection)) {
    throw createError({
      statusCode: 404,
      statusMessage: 'Page Not Found'
    })
  }
}, { immediate: true })

// Page title with fallback to nav label
const pageTitle = computed(() => {
  const seoKey = `seo.${sectionId.value}Title`
  return te(seoKey) ? t(seoKey) : t(`nav.${sectionId.value}`)
})

// Page description (optional - empty if not set)
const pageDescription = computed(() => {
  const seoKey = `seo.${sectionId.value}Description`
  return te(seoKey) ? t(seoKey) : ''
})

// Page meta
useHead({
  title: pageTitle,
  meta: [
    {
      name: 'description',
      content: pageDescription
    }
  ]
})
</script>

<template>
  <div>
    <!-- Dynamic section rendering via SectionRenderer -->
    <OrganismsSectionRenderer :section-id="sectionId" />
  </div>
</template>
