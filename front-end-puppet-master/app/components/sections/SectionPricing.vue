<script setup lang="ts">
/**
 * Pricing Section
 *
 * Displays pricing tiers from module config.
 * Self-contained: fetches title from i18n if not provided.
 *
 * CSS: ui/content/pricing.css
 * Classes: .pricing-grid, .pricing-card, .pricing-card--featured, .pricing-header, .pricing-price, .pricing-features
 */

const { t, te } = useI18n()

defineProps<{
  /** Section title (or use #title slot) */
  title?: string
}>()

// Self-contained: fetch title from i18n if not provided
const sectionTitle = computed(() => {
  if (te('pricing.title')) return t('pricing.title')
  if (te('nav.pricing')) return t('nav.pricing')
  return ''
})
</script>

<template>
  <!--
    Uses global classes from:
    - layout/sections.css (.section)
    - typography/base.css (.section-title, .section-title--center)
    - ui/content/pricing.css (pricing styles)
  -->
  <section id="pricing" class="section">
    <div class="container">
      <h2 v-if="title || sectionTitle || $slots.title" v-reveal class="section-title section-title--center">
        <slot name="title">{{ title ?? sectionTitle }}</slot>
      </h2>
      <div class="section-body">
        <OrganismsPricingTiers />
      </div>
    </div>
  </section>
</template>

<!-- No scoped styles needed - all styles come from global CSS -->
