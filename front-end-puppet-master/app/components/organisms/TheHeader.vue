<script setup lang="ts">
/**
 * TheHeader Organism
 *
 * Main site header with responsive behavior.
 * Desktop: Full nav visible (unless items overflow)
 * Mobile: Hamburger menu with overlay navigation
 * Uses Tabler icons via unplugin-icons.
 *
 * Smart adaptive nav: Detects when nav items don't fit
 * and automatically switches to hamburger menu.
 */
import config from '~/puppet-master.config'
import IconX from '~icons/tabler/x'

const { t } = useI18n()
const isMenuOpen = ref(false)
const showMenu = ref(false)

// SSR-safe config access with defensive defaults
const headerFooterWidthMode = computed(() => config?.headerFooterWidthMode ?? 'full')
const hideHeaderOnScroll = computed(() => config?.features?.hideHeaderOnScroll ?? false)
const isOnepager = computed(() => config?.features?.onepager ?? false)
const interactiveHeader = computed(() => config?.features?.interactiveHeader ?? false)
const headerContactEnabled = computed(() => config?.headerContact?.enabled ?? false)

// Adaptive nav - detects overflow and collapses to hamburger
const { headerRef, navRef, isCollapsed } = useAdaptiveNav()

// Scroll behavior - ONLY in onepager mode!
// hideOnScroll respects config.features.hideHeaderOnScroll
const { isScrolled, isHidden } = useScrollHeader({
  threshold: 50,
  hideOnScroll: hideHeaderOnScroll,
  enabled: isOnepager.value && interactiveHeader.value
})

// Animation timing for hamburger
watch(isMenuOpen, open => {
  if (open) {
    setTimeout(() => {
      showMenu.value = true
    }, 200)
  } else {
    showMenu.value = false
  }
})

function closeMenu() {
  isMenuOpen.value = false
}
</script>

<template>
  <header
    ref="headerRef"
    class="header"
    :class="{
      'header--scrolled': isScrolled,
      'header--hidden': isHidden,
      'header--nav-collapsed': isCollapsed,
      'header--contained': headerFooterWidthMode !== 'full',
      'header--full': headerFooterWidthMode === 'full'
    }"
  >
    <div class="header-inner container">
      <!-- Hamburger: shown on mobile OR when nav overflows on desktop -->
      <div class="header-hamburger" :class="{ 'is-visible': isCollapsed }">
        <AtomsHamburgerIcon
          :is-open="isMenuOpen"
          :label="isMenuOpen ? t('nav.closeMenu') : t('nav.openMenu')"
          @toggle="isMenuOpen = !isMenuOpen"
        />
      </div>

      <!-- Logo (center on mobile, left on desktop) -->
      <AtomsLogo class="header-logo" />

      <!-- Desktop: Navigation (center) - hidden when collapsed -->
      <nav ref="navRef" class="header-nav">
        <MoleculesNavLinks />
      </nav>

      <!-- Desktop: Actions (right) -->
      <MoleculesHeaderActions class="header-actions" />

      <!-- Mobile: Contact buttons (right, opposite to hamburger) -->
      <MoleculesHeaderContact v-if="headerContactEnabled" class="mobile-only" />
    </div>

    <!-- Mobile Navigation Overlay - uses skeleton/mobile-nav.css -->
    <Teleport to="body">
      <Transition name="backdrop">
        <div v-if="showMenu" class="mobile-nav-backdrop" @click="closeMenu" />
      </Transition>
      <Transition name="slide-left">
        <nav v-if="showMenu" class="mobile-nav">
          <div class="mobile-nav-header">
            <AtomsLogo />
            <button
              type="button"
              class="mobile-nav-close"
              :aria-label="t('nav.closeMenu')"
              @click="closeMenu"
            >
              <IconX />
            </button>
          </div>
          <div class="mobile-nav-body">
            <MoleculesNavLinks vertical @navigate="closeMenu" />
          </div>
          <div class="mobile-nav-settings">
            <MoleculesHeaderActions :show-contact="false" lang-direction="inline" />
          </div>
        </nav>
      </Transition>
    </Teleport>
  </header>
</template>

<!--
  All styles are in global CSS:
  - skeleton/header.css (header, logo, mobile layout)
  - skeleton/mobile-nav.css (mobile nav overlay)
  - skeleton/nav.css (nav links)
  No scoped styles needed!
-->
