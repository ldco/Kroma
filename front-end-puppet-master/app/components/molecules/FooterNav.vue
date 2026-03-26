<script setup lang="ts">
/**
 * FooterNav Molecule
 *
 * Config-driven horizontal footer navigation links.
 * Uses skeleton classes from footer.css (.footer-nav-inline)
 */
import config from '~/puppet-master.config'
import { buildWebsiteNavLinks } from '~/types/config'

const { t } = useI18n()
const localePath = useLocalePath()

// SSR-safe config access with defensive defaults
const isOnepager = computed(() => config?.features?.onepager ?? false)

// Generate nav links
const navLinks = computed(() =>
  buildWebsiteNavLinks(
    config.sections,
    isOnepager.value ? { onepager: true } : { onepager: false, localePath }
  ).map(link => ({
    id: link.id,
    label: t(`nav.${link.id}`),
    href: link.href
  }))
)
</script>

<template>
  <!-- Horizontal inline navigation -->
  <nav v-if="navLinks.length > 0" class="footer-nav-inline">
    <!-- Anchor links: use plain <a> to avoid Vue Router warnings -->
    <template v-if="isOnepager">
      <a v-for="link in navLinks" :key="link.id" :href="link.href">
        {{ link.label }}
      </a>
    </template>
    <!-- Route links: use NuxtLink for client-side navigation -->
    <template v-else>
      <NuxtLink v-for="link in navLinks" :key="link.id" :to="link.href">
        {{ link.label }}
      </NuxtLink>
    </template>
  </nav>
</template>
