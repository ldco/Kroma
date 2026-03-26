<script setup lang="ts">
/**
 * Section Renderer
 *
 * Dynamically renders section components based on section ID.
 * Uses lazy component imports for SSR compatibility.
 *
 * This enables DRY section management - just add to config.sections
 * and create the component, no need to edit index.vue or [section].vue.
 */

import config from '~/puppet-master.config'

const props = defineProps<{
  /** Section ID from config (e.g., 'about', 'portfolio') */
  sectionId: string
}>()

const { t, te } = useI18n()

// SSR-safe config access with defensive defaults
const isOnepager = computed(() => config?.features?.onepager ?? false)

// Map section IDs to lazy-loaded components
// Nuxt auto-imports these as LazySectionsSection{Name}
const sectionComponents: Record<string, ReturnType<typeof defineAsyncComponent>> = {
  about: defineAsyncComponent(() => import('~/components/sections/SectionAbout.vue')),
  portfolio: defineAsyncComponent(() => import('~/components/sections/SectionPortfolio.vue')),
  pricing: defineAsyncComponent(() => import('~/components/sections/SectionPricing.vue')),
  contact: defineAsyncComponent(() => import('~/components/sections/SectionContact.vue')),
  team: defineAsyncComponent(() => import('~/components/sections/SectionTeam.vue')),
  testimonials: defineAsyncComponent(() => import('~/components/sections/SectionTestimonials.vue')),
  features: defineAsyncComponent(() => import('~/components/sections/SectionFeatures.vue')),
  clients: defineAsyncComponent(() => import('~/components/sections/SectionClients.vue')),
  faq: defineAsyncComponent(() => import('~/components/sections/SectionFaq.vue')),
  blog: defineAsyncComponent(() => import('~/components/sections/SectionBlog.vue'))
}

// Get title from i18n (fallback to nav label)
const sectionTitle = computed(() => {
  const titleKey = `${props.sectionId}.title`
  const navKey = `nav.${props.sectionId}`
  return te(titleKey) ? t(titleKey) : te(navKey) ? t(navKey) : ''
})

// Get the component for this section
const SectionComponent = computed(() => sectionComponents[props.sectionId])
const componentExists = computed(() => !!SectionComponent.value)

const sceneDirectiveValue = computed(() =>
  isOnepager.value ? { id: props.sectionId } : false
)
</script>

<template>
  <div :class="{ 'scene-full-bleed': config.features.onepager }" v-scrolly-scene="sceneDirectiveValue">
    <component
      v-if="componentExists"
      :is="SectionComponent"
      :title="sectionTitle"
    />
    <section v-else :id="sectionId" class="section">
      <div class="container">
        <p class="text-center text-secondary">
          Section component not found: {{ sectionId }}
        </p>
      </div>
    </section>
  </div>
</template>
