<script setup lang="ts">
/**
 * Quick Tools Page - Placeholder
 *
 * Utility tools (bg-remove, upscale, color) without project context.
 */
definePageMeta({
  layout: 'kroma'
})

const { t } = useI18n()
</script>

<template>
  <div class="quick-tools-page">
    <header class="page-header">
      <div class="page-header__content">
        <h1 class="page-header__title">
          {{ t('app.nav.quickTools') }}
        </h1>
        <p class="page-header__subtitle">
          {{ t('app.quickTools.subtitle') }}
        </p>
      </div>
    </header>

    <div class="empty-state">
      <Icon name="lucide:wand" class="empty-state__icon" />
      <h3 class="empty-state__title">{{ t('app.quickTools.comingSoon') }}</h3>
      <p class="empty-state__description">
        {{ t('app.quickTools.comingSoonDescription') }}
      </p>
    </div>
  </div>
</template>
