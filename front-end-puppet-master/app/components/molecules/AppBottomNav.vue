<script setup lang="ts">
/**
 * AppBottomNav - Mobile App-Style Bottom Navigation
 *
 * Used in app mode (admin panel) for phones (< 600px).
 * Material Design 3 inspired with PWA-ready safe area handling.
 *
 * MAX_VISIBLE_ITEMS: 5 (Material Design recommendation)
 * When items exceed 5, shows 4 items + "More" button with overflow menu.
 *
 * Navigation is config-driven via config.admin in puppet-master.config.ts
 * Uses getAdminSectionsForRole() to filter sections by user's role.
 *
 * Uses global CSS:
 * - skeleton/bottom-nav.css: .bottom-nav, .bottom-nav-item, .bottom-nav-label
 * - ui/content/badges.css: .badge-dot
 */
import type { Component } from 'vue'
import config from '~/puppet-master.config'
import IconSettings from '~icons/tabler/settings'
import IconPhoto from '~icons/tabler/photo'
import IconMail from '~icons/tabler/mail'
import IconLanguage from '~icons/tabler/language'
import IconUsers from '~icons/tabler/users'
import IconHeartbeat from '~icons/tabler/heartbeat'
import IconCreditCard from '~icons/tabler/credit-card'
import IconDotsVertical from '~icons/tabler/dots-vertical'
import IconArticle from '~icons/tabler/article'
import IconUsersGroup from '~icons/tabler/users-group'
import IconSparkles from '~icons/tabler/sparkles'
import IconQuote from '~icons/tabler/quote'
import IconBuilding from '~icons/tabler/building'
import IconHelpCircle from '~icons/tabler/help-circle'
import IconShield from '~icons/tabler/shield'
import IconList from '~icons/tabler/list'
import IconLayout from '~icons/tabler/layout'
import IconDatabase from '~icons/tabler/database'

// Icon mapping from config icon names to components
const iconMap: Record<string, Component> = {
  settings: IconSettings,
  photo: IconPhoto,
  mail: IconMail,
  language: IconLanguage,
  users: IconUsers,
  heartbeat: IconHeartbeat,
  'credit-card': IconCreditCard,
  article: IconArticle,
  'users-group': IconUsersGroup,
  sparkles: IconSparkles,
  quote: IconQuote,
  building: IconBuilding,
  'help-circle': IconHelpCircle,
  shield: IconShield,
  list: IconList,
  layout: IconLayout,
  database: IconDatabase
}

const { t } = useI18n()
const localePath = useLocalePath()
const { user } = useAuth()
const { unreadCount } = useUnreadCount()

// SSR-safe config access with defensive defaults
const adminConfig = computed(() => config?.admin ?? { sections: [] })

// Maximum items to show directly (Material Design: 3-5, we use 5)
const MAX_VISIBLE_ITEMS = 5

// State for overflow menu
const showMoreMenu = ref(false)

// Get admin sections filtered by user's role
const adminSections = computed(() => {
  if (!user.value) return []
  return config.getAdminSectionsForRole(user.value.role)
})

// Config-driven navigation items - filtered by role access
// Uses short single-word labels (admin.navXxx) for compact bottom nav display
const allNavItems = computed(() => {
  return adminSections.value.map(section => ({
    to: localePath(`/admin/${section.id}`),
    // Use short nav labels for bottom nav (admin.navSettings, admin.navPortfolio, etc.)
    label: `admin.nav${section.label.charAt(0).toUpperCase() + section.label.slice(1)}`,
    icon: iconMap[section.icon] || IconSettings,
    badge: section.badge
  }))
})

// Split items into visible and overflow
const hasOverflow = computed(() => allNavItems.value.length > MAX_VISIBLE_ITEMS)
const visibleItems = computed(() =>
  hasOverflow.value ? allNavItems.value.slice(0, MAX_VISIBLE_ITEMS - 1) : allNavItems.value
)
const overflowItems = computed(() =>
  hasOverflow.value ? allNavItems.value.slice(MAX_VISIBLE_ITEMS - 1) : []
)

// Check if any overflow item has a badge with count
const overflowHasBadge = computed(
  () => overflowItems.value.some(item => item.badge) && unreadCount.value > 0
)

function toggleMoreMenu() {
  showMoreMenu.value = !showMoreMenu.value
}

function closeMoreMenu() {
  showMoreMenu.value = false
}
</script>

<template>
  <nav class="bottom-nav" aria-label="Main navigation">
    <!-- Visible navigation items -->
    <NuxtLink v-for="item in visibleItems" :key="item.to" :to="item.to" class="bottom-nav-item">
      <span class="relative">
        <component :is="item.icon" class="bottom-nav-icon" />
        <span v-if="item.badge && unreadCount > 0" class="badge-dot">
          {{ unreadCount > 9 ? '9+' : unreadCount }}
        </span>
      </span>
      <span class="bottom-nav-label">{{ t(item.label) }}</span>
    </NuxtLink>

    <!-- More button (only if overflow exists) -->
    <div v-if="hasOverflow" class="bottom-nav-more-wrapper">
      <button
        type="button"
        class="bottom-nav-item"
        :class="{ 'is-active': showMoreMenu }"
        :aria-label="t('common.more')"
        :aria-expanded="showMoreMenu"
        @click="toggleMoreMenu"
      >
        <span class="relative">
          <IconDotsVertical class="bottom-nav-icon" />
          <span v-if="overflowHasBadge" class="badge-dot badge-dot-small"></span>
        </span>
        <span class="bottom-nav-label">{{ t('common.more') }}</span>
      </button>

      <!-- Overflow menu (bottom sheet style) -->
      <Teleport to="body">
        <Transition name="fade">
          <div v-if="showMoreMenu" class="bottom-nav-more-backdrop" @click="closeMoreMenu" />
        </Transition>
        <Transition name="slide-up">
          <div v-if="showMoreMenu" class="bottom-nav-more-menu">
            <NuxtLink
              v-for="item in overflowItems"
              :key="item.to"
              :to="item.to"
              class="bottom-nav-more-item"
              @click="closeMoreMenu"
            >
              <component :is="item.icon" class="bottom-nav-more-icon" />
              <span class="bottom-nav-more-label">{{ t(item.label) }}</span>
              <span v-if="item.badge && unreadCount > 0" class="badge-dot">
                {{ unreadCount > 9 ? '9+' : unreadCount }}
              </span>
            </NuxtLink>
          </div>
        </Transition>
      </Teleport>
    </div>
  </nav>
</template>

<!--
  Uses global CSS classes from skeleton/bottom-nav.css
  No scoped styles needed - following PuppetMaster CSS architecture
-->
