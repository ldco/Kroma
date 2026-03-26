<script setup lang="ts">
/**
 * FooterCta Molecule
 *
 * Config-driven CTA button for footer.
 * - Text: from translations (cta.footerButton) - supports i18n
 * - URL: from settings (footer.ctaUrl) - language-independent
 * ONLY shows if URL is set in database.
 */

const { t } = useI18n()
const { settings } = useSiteSettings()

// URL from settings, text from translations
const ctaUrl = computed(() => settings.value?.footer?.ctaUrl)
const ctaText = computed(() => t('cta.footerButton'))

// Only show if URL exists in database
const hasValidCta = computed(() => !!ctaUrl.value)
</script>

<template>
  <div v-if="hasValidCta" class="footer-cta">
    <NuxtLink :href="ctaUrl!" class="footer-cta-button">
      {{ ctaText }}
    </NuxtLink>
  </div>
</template>
