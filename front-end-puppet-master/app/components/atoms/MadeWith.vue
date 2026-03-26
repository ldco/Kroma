<script setup lang="ts">
/**
 * MadeWith Atom
 *
 * "Made with Puppet Master" branding with horizontal logo.
 * Links to the Puppet Master website.
 * Centered at the very bottom of footer.
 *
 * Logo Strategy: Render BOTH theme variants, CSS shows correct one.
 * Language is baked into the path via locale (Russian for ru, English for others).
 *
 * IMPORTANT: Logo is stored in /public/puppetmaster/ - a special folder
 * that should NOT be replaced when customizing the site for clients.
 */

const { t, locale } = useI18n()
const puppetMasterUrl = 'https://puppetmaster.dev'

// Get language suffix (ru for Russian, en for all others)
const langSuffix = computed(() => (locale.value === 'ru' ? 'ru' : 'en'))

// Logo paths - 2 variants for theme, language baked in
const basePath = '/puppetmaster'
const lightThemeLogo = computed(() => `${basePath}/horizontal_dark_${langSuffix.value}.svg`)
const darkThemeLogo = computed(() => `${basePath}/horizontal_light_${langSuffix.value}.svg`)
</script>

<template>
  <div class="made-with">
    <span class="made-with-text">{{ t('footer.madeWith') }}</span>
    <a
      :href="puppetMasterUrl"
      target="_blank"
      rel="noopener noreferrer"
      class="made-with-link"
      :aria-label="t('footer.puppetMaster')"
    >
      <!-- 2 theme variants - CSS shows correct one, language is in the path -->
      <img
        :src="lightThemeLogo"
        alt="Puppet Master"
        class="made-with-logo light-logo"
        width="120"
        height="24"
      />
      <img
        :src="darkThemeLogo"
        alt="Puppet Master"
        class="made-with-logo dark-logo"
        width="120"
        height="24"
      />
    </a>
  </div>
</template>
