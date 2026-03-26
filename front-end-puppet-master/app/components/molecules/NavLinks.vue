<script setup lang="ts">
/**
 * NavLinks Molecule
 *
 * A group of navigation links.
 * Supports both regular routes and anchor links (for one-pager mode).
 * Uses scrollspy for active state in onepager mode.
 */
import config from '~/puppet-master.config'
import { buildWebsiteNavLinks, getNavWebsiteSectionIds } from '~/types/config'

defineProps<{
  /** Vertical layout (for mobile menu) */
  vertical?: boolean
}>()

const { t } = useI18n()
const localePath = useLocalePath()

const emit = defineEmits<{
  navigate: []
}>()

// Track if component is hydrated (to prevent SSR/client mismatch for active state)
const isHydrated = ref(false)
onMounted(() => {
  isHydrated.value = true
})

// SSR-safe config access with defensive defaults
const sectionsConfig = computed(() => config?.sections ?? [])
const isOnepager = computed(() => config?.features?.onepager ?? false)

// Get section IDs for scrollspy
const sectionIds = computed(() => getNavWebsiteSectionIds(sectionsConfig.value))

// Scrollspy for onepager mode - detects which section is in view
const { activeSection } = isOnepager.value
  ? useScrollSpy(sectionIds.value)
  : { activeSection: ref('') }

// Navigation links - derived from sections config
const navLinks = computed(() => {
  if (isOnepager.value) {
    return buildWebsiteNavLinks(sectionsConfig.value, { onepager: true }).map(link => ({
      id: link.id,
      to: link.href,
      label: t(`nav.${link.id}`),
      isAnchor: link.isAnchor
    }))
  }

  return buildWebsiteNavLinks(sectionsConfig.value, { onepager: false, localePath }).map(link => ({
    id: link.id,
    to: link.href,
    label: t(`nav.${link.id}`),
    isAnchor: link.isAnchor
  }))
})
</script>

<template>
  <!-- Uses global classes from skeleton/nav.css (.nav-links, .nav-links--vertical) -->
  <nav class="nav-links" :class="{ 'nav-links--vertical': vertical }">
    <AtomsNavLink
      v-for="link in navLinks"
      :key="link.to"
      :to="link.to"
      :is-anchor="link.isAnchor"
      :is-active="link.isAnchor && isHydrated && activeSection === link.id"
      @click="emit('navigate')"
    >
      {{ link.label }}
    </AtomsNavLink>
  </nav>
</template>

<!-- No scoped styles needed - uses skeleton/nav.css -->
