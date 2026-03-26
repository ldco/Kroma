<script setup lang="ts">
/**
 * Logo Atom
 *
 * Displays the site logo with automatic theme/language switching.
 * Uses the useLogo composable for dynamic logo selection.
 *
 * Uses ClientOnly to avoid SSR hydration mismatch since
 * logo src depends on colorMode which can differ between server and client.
 */
import config from '~/puppet-master.config'

const { headerLogo } = useLogo()
const localePath = useLocalePath()

defineProps<{
  /** Link destination (default: home) */
  to?: string
  /** Alt text for accessibility */
  alt?: string
  /** Additional CSS classes */
  class?: string
}>()

// SSR-safe computed config values with defensive defaults
const isOnepager = computed(() => config?.features?.onepager ?? false)

// SSR fallback: use the default theme logo with defensive defaults
const ssrFallbackLogo = computed(() => {
  const theme = config?.defaultTheme === 'dark' ? 'light' : 'dark'
  const basePath = config?.logo?.basePath || '/logos'
  const defaultLocale = config?.defaultLocale || 'en'
  return `${basePath}/horizontal_${theme}_${defaultLocale}.svg`
})

// Locale-aware home path (keeps current language when clicking logo)
const homePath = computed(() => localePath('/'))
</script>

<template>
  <!-- Uses global classes from skeleton/header.css (.logo-link, .logo-img) -->
  <NuxtLink :to="to ?? homePath" class="logo-link" :class="$props.class">
    <ClientOnly>
      <img :src="headerLogo" :alt="alt ?? 'Logo'" class="logo-img" width="200" height="40" />
      <template #fallback>
        <img :src="ssrFallbackLogo" :alt="alt ?? 'Logo'" class="logo-img" width="200" height="40" />
      </template>
    </ClientOnly>
  </NuxtLink>
</template>

<!-- No scoped styles needed - all styles come from skeleton/header.css -->
