<script setup lang="ts">
/**
 * Settings Page - Placeholder
 *
 * App settings.
 */
definePageMeta({
  layout: 'kroma'
})

const { t } = useI18n()
</script>

<template>
  <div class="settings-page">
    <header class="page-header">
      <div class="page-header__content">
        <h1 class="page-header__title">
          {{ t('app.settings') }}
        </h1>
        <p class="page-header__subtitle">
          {{ t('app.settings.subtitle') }}
        </p>
      </div>
    </header>

    <div class="empty-state">
      <Icon name="lucide:settings" class="empty-state__icon" />
      <h3 class="empty-state__title">{{ t('app.settings.comingSoon') }}</h3>
      <p class="empty-state__description">
        {{ t('app.settings.comingSoonDescription') }}
      </p>
    </div>
  </div>
</template>
