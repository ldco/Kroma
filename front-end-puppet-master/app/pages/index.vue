<!--
  Home Page

  In onepager mode: Renders ALL sections for scroll-based navigation.
  In SPA mode: Renders ONLY hero - other sections have their own routes.

  Sections are rendered dynamically from config.sections.
  Each section component fetches its own content from i18n.
-->

<script setup lang="ts">
import config from '~/puppet-master.config'
import { getDynamicWebsiteSectionIds, getWebsiteSectionHref } from '~/types/config'

const { t } = useI18n()
const localePath = useLocalePath()

// SSR-safe config access with defensive defaults
const isOnepager = computed(() => config?.features?.onepager ?? false)

// Get link based on mode (anchor for onepager, route for SPA)
const getSectionLink = (sectionId: string) => {
  return getWebsiteSectionHref(
    sectionId,
    isOnepager.value ? { onepager: true } : { onepager: false, localePath }
  )
}

// Get sections to render (excluding 'home' which is the hero)
const sectionsToRender = computed(() =>
  getDynamicWebsiteSectionIds(config.sections).map(id => ({ id }))
)

// Page meta with translations
useHead({
  title: t('seo.homeTitle'),
  meta: [{ name: 'description', content: t('seo.homeDescription') }]
})
</script>

<template>
  <div>
    <!-- Hero Section (always shown, has special props) -->
    <div :class="{ 'scene-full-bleed': isOnepager }" v-scrolly-scene="isOnepager ? { id: 'home' } : false">
      <SectionsSectionHero
        :subtitle="t('hero.subtitle')"
        :primary-cta="t('hero.primaryCta')"
        :primary-link="getSectionLink('services')"
        :secondary-cta="t('hero.secondaryCta')"
        :secondary-link="getSectionLink('portfolio')"
        parallax
      />
    </div>

    <!-- Other sections rendered dynamically (only in onepager mode) -->
    <template v-if="isOnepager">
      <OrganismsSectionRenderer
        v-for="section in sectionsToRender"
        :key="section.id"
        :section-id="section.id"
      />
    </template>
  </div>
</template>
