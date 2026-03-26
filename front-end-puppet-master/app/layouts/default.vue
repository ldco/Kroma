<!--
  Default Layout

  Uses atomic design components:
  - OrganismsTheHeader: Complete header with nav, logo, actions
  - OrganismsTheFooter: Complete footer with social links
  - AtomsBackToTop: Fixed scroll-to-top button

  For simpler layouts, see blank.vue
  For admin, see admin.vue
-->

<script setup lang="ts">
/**
 * Default Layout
 *
 * Uses Holy Grail grid from layout/page.css
 * Classes: .layout, .main
 *
 * Adds feature classes to layout for CSS targeting:
 * - .interactive-header: when header shrinks on scroll
 * - .onepager: when in one-page mode with anchor navigation
 */
import config from '~/puppet-master.config'

const { t } = useI18n()

// SSR-safe config access with defensive defaults
const showBackToTop = computed(() => config?.features?.backToTop ?? false)
const isOnepager = computed(() => config?.features?.onepager ?? false)
const interactiveHeader = computed(() => config?.features?.interactiveHeader ?? false)

// Comment 1: Unwrap isOnepager computed ref before passing to composables
// A ComputedRef object is always truthy, so we must use .value
useReveal({ enabled: isOnepager.value })

// Onepager scrollytelling runtime (global scroll timeline + optional local scenes)
useScrollytelling({ enabled: isOnepager.value })

// Feature classes for CSS targeting
const layoutClasses = computed(() => ({
  layout: true,
  'interactive-header': interactiveHeader,
  onepager: isOnepager
}))
</script>

<template>
  <div :class="layoutClasses">
    <!-- Skip to content link (WCAG 2.4.1) - first focusable element -->
    <a href="#main-content" class="skip-link">{{ t('nav.skipToContent') }}</a>

    <!-- Header with responsive navigation -->
    <OrganismsTheHeader />

    <!-- Main content slot -->
    <main id="main-content" class="main">
      <slot />
    </main>

    <!-- Footer with social links and copyright -->
    <OrganismsTheFooter rich />

    <!-- Back to top button (fixed, appears on scroll) -->
    <AtomsBackToTop v-if="showBackToTop" />
  </div>
</template>
