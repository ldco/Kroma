<script setup lang="ts">
/**
 * SocialNav Molecule
 *
 * Config-driven social links navigation.
 * - Reads icon names from config
 * - Gets values from useSiteSettings()
 * - Only renders platforms that have URLs in database
 * - Supports both Tabler icons and custom icons
 */
import config from '~/puppet-master.config'

// Tabler brand icons (regular/stroked - consistent style)
import IconBrandTelegram from '~icons/tabler/brand-telegram'
import IconBrandInstagram from '~icons/tabler/brand-instagram'
import IconBrandWhatsapp from '~icons/tabler/brand-whatsapp'
import IconBrandFacebook from '~icons/tabler/brand-facebook'
import IconBrandYoutube from '~icons/tabler/brand-youtube'
import IconBrandLinkedin from '~icons/tabler/brand-linkedin'
import IconBrandX from '~icons/tabler/brand-x'
import IconBrandGithub from '~icons/tabler/brand-github'
import IconBrandGitlab from '~icons/tabler/brand-gitlab'
import IconBrandVk from '~icons/tabler/brand-vk'
import IconBrandTiktok from '~icons/tabler/brand-tiktok'
import IconBrandPinterest from '~icons/tabler/brand-pinterest'
import IconBrandDribbble from '~icons/tabler/brand-dribbble'
import IconBrandBehance from '~icons/tabler/brand-behance'
import IconBrandDiscord from '~icons/tabler/brand-discord'
import IconBrandMedium from '~icons/tabler/brand-medium'
import IconBrandThreads from '~icons/tabler/brand-threads'
import IconBrandTwitch from '~icons/tabler/brand-twitch'

// Custom icons (for brands not in Tabler)
import IconMax from '~/assets/icons/custom/IconMax.vue'

// Icon map: config icon name → component
const iconMap: Record<string, Component> = {
  // Messaging
  'brand-telegram': IconBrandTelegram,
  'brand-whatsapp': IconBrandWhatsapp,
  'brand-discord': IconBrandDiscord,
  // Social
  'brand-instagram': IconBrandInstagram,
  'brand-facebook': IconBrandFacebook,
  'brand-x': IconBrandX,
  'brand-threads': IconBrandThreads,
  'brand-tiktok': IconBrandTiktok,
  'brand-pinterest': IconBrandPinterest,
  'brand-vk': IconBrandVk,
  // Video
  'brand-youtube': IconBrandYoutube,
  'brand-twitch': IconBrandTwitch,
  // Professional
  'brand-linkedin': IconBrandLinkedin,
  'brand-medium': IconBrandMedium,
  // Dev/Design
  'brand-github': IconBrandGithub,
  'brand-gitlab': IconBrandGitlab,
  'brand-dribbble': IconBrandDribbble,
  'brand-behance': IconBrandBehance,
  // Custom
  'custom-max': IconMax
}

// SSR-safe config access with defensive defaults
const settingsConfig = computed(() => config?.settings ?? [])

// Build platform → icon mapping from config
const platformIcons = computed(() => {
  const map: Record<string, string> = {}
  for (const setting of settingsConfig.value) {
    if (setting.group === 'social' && setting.icon) {
      // Extract platform name from key: 'social.telegram' → 'telegram'
      const platform = setting.key.split('.')[1]
      if (platform) {
        map[platform] = setting.icon
      }
    }
  }
  return map
})

defineProps<{
  /** Vertical layout instead of horizontal */
  vertical?: boolean
  /** Show labels next to icons */
  showLabels?: boolean
  /** Show icons in filled circles */
  circle?: boolean
}>()

// Get social links from settings (only those with values)
const { socialLinks } = useSiteSettings()

// Filter to only platforms we have icons for
const activeSocials = computed(() => {
  return socialLinks.value.filter(link => {
    const iconName = platformIcons.value[link.platform]
    return iconName && iconMap[iconName]
  })
})

// Get icon component for a platform
function getIcon(platform: string): Component | undefined {
  const iconName = platformIcons.value[platform]
  return iconName ? iconMap[iconName] : undefined
}
</script>

<template>
  <nav
    v-if="activeSocials.length > 0"
    class="social-nav"
    :class="{
      'social-nav--vertical': vertical,
      'social-nav--circle': circle
    }"
    aria-label="Social media links"
  >
    <a
      v-for="social in activeSocials"
      :key="social.platform"
      :href="social.url"
      class="social-nav-link"
      :class="`social-nav-link--${social.platform}`"
      :aria-label="social.platform"
      target="_blank"
      rel="noopener noreferrer"
    >
      <component :is="getIcon(social.platform)" class="social-nav-icon" />
      <span v-if="showLabels" class="social-nav-label">{{ social.platform }}</span>
    </a>
  </nav>
</template>
