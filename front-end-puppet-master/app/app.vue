<template>
  <!--
    RTL is handled via dir attribute on <html> element.
    Color mode is handled by @nuxtjs/color-mode which adds class to <html>.
    No need to add colorMode class here - it causes hydration mismatch!
  -->
  <NuxtLayout>
    <NuxtPage />
  </NuxtLayout>
</template>

<script setup lang="ts">
import { isRtlLanguage } from '~/utils/rtl'

const { locale } = useI18n()

const isRtl = computed(() => isRtlLanguage(locale.value))

// Set html attributes for:
// 1. lang - for CSS language-based styling and accessibility
// 2. dir - for RTL support (scrollbar, logical properties, text-align)
useHead({
  htmlAttrs: {
    lang: () => locale.value,
    dir: () => (isRtl.value ? 'rtl' : 'ltr')
  }
})
</script>
