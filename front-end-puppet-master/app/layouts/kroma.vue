<script setup lang="ts">
/**
 * Kroma App Layout
 *
 * Responsive navigation for Kroma Tauri app:
 * - Phone (< 640px): Bottom navigation bar
 * - Tablet/Desktop (≥ 640px): Vertical sidebar
 *
 * Navigation items:
 * - Dashboard (/app)
 * - Projects (/app/projects)
 * - Quick Tools (/app/quick-tools)
 * - Settings (/app/settings)
 *
 * Note: Provider management is project-scoped and accessed via
 * /app/projects/[slug]/providers from within a project context.
 */
import type { Component } from 'vue'
import IconDashboard from '~icons/tabler/dashboard'
import IconFolder from '~icons/tabler/folder'
import IconTools from '~icons/tabler/tools'
import IconSettings from '~icons/tabler/settings'
import IconLogout from '~icons/tabler/logout'
import IconKey from '~icons/tabler/key'
import IconMoon from '~icons/tabler/moon'
import IconSun from '~icons/tabler/sun'
import { bootstrapKromaToken } from '~/composables/useKromaApi'

const route = useRoute()
const localePath = useLocalePath()
const { user, logout, isLoading } = useAuth()
const { t } = useI18n()
const kromaAuth = useKromaAuthStore()

// Initialize Kroma auth from localStorage on mount (non-blocking)
onMounted(() => {
  // Just trigger bootstrap in background - don't await
  bootstrapKromaToken().catch(console.error)
})

// User menu state
const userMenuOpen = ref(false)

// Navigation items (providers removed - now project-scoped)
const navItems = [
  { to: '/app', label: 'app.dashboard', icon: IconDashboard },
  { to: '/app/projects', label: 'app.projects', icon: IconFolder },
  { to: '/app/quick-tools', label: 'app.quickTools', icon: IconTools },
  { to: '/app/settings', label: 'app.settings', icon: IconSettings }
]

// Get user initials for avatar
const userInitials = computed(() => {
  if (!user.value) return '?'
  if (user.value.name) {
    return user.value.name
      .split(' ')
      .map(n => n[0])
      .join('')
      .toUpperCase()
      .slice(0, 2)
  }
  return user.value.email?.[0]?.toUpperCase() ?? '?'
})

function toggleUserMenu() {
  userMenuOpen.value = !userMenuOpen.value
}

async function handleLogout() {
  await logout()
}

function openChangePasswordModal() {
  userMenuOpen.value = false
  // TODO: Open change password modal
}

// Close user menu on click outside
function handleClickOutside(e: MouseEvent) {
  const target = e.target as HTMLElement
  if (!target.closest('.sidebar-user-wrapper') && !target.closest('.sidebar-user-menu')) {
    userMenuOpen.value = false
  }
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
})

// Theme toggle
const colorMode = useColorMode()
const isDark = computed(() => colorMode.preference === 'dark')

function toggleTheme() {
  colorMode.preference = isDark.value ? 'light' : 'dark'
}
</script>

<template>
  <div class="layout-kroma">
    <!-- ═══════════════════════════════════════════════════════════════════════════
         VERTICAL SIDEBAR (tablet landscape / desktop ≥ 640px)
         ═══════════════════════════════════════════════════════════════════════════ -->
    <aside class="kroma-sidebar">
      <!-- Logo -->
      <div class="sidebar-header">
        <NuxtLink :to="localePath('/app')" class="sidebar-logo">
          <span class="sidebar-logo-text">Kroma</span>
        </NuxtLink>
      </div>

      <!-- Nav links -->
      <nav class="sidebar-nav">
        <NuxtLink
          v-for="item in navItems"
          :key="item.to"
          :to="localePath(item.to)"
          class="sidebar-nav-link"
          :class="{ 'sidebar-nav-link--active': route.path === item.to }"
        >
          <component :is="item.icon" class="sidebar-nav-icon" />
          <span class="sidebar-nav-label">{{ t(item.label) }}</span>
        </NuxtLink>
      </nav>

      <!-- Footer actions -->
      <div class="sidebar-footer">
        <!-- Theme toggle -->
        <button
          type="button"
          class="sidebar-icon-btn"
          @click="toggleTheme"
          :aria-label="isDark ? t('common.lightMode') : t('common.darkMode')"
        >
          <IconSun v-if="isDark" class="sidebar-icon" />
          <IconMoon v-else class="sidebar-icon" />
        </button>

        <!-- User avatar with popover menu -->
        <div class="sidebar-user-wrapper">
          <button
            type="button"
            class="sidebar-user-avatar"
            @click="toggleUserMenu"
            :aria-label="t('admin.userMenu')"
          >
            <span class="avatar-initials">{{ userInitials }}</span>
          </button>

          <!-- User dropdown menu -->
          <div v-if="userMenuOpen" class="sidebar-user-menu">
            <div class="user-menu-info">
              <span v-if="user?.name" class="user-menu-name">{{ user.name }}</span>
              <span class="user-menu-email">{{ user?.email }}</span>
              <span class="user-menu-role">{{ user?.role }}</span>
            </div>

            <div class="user-menu-divider"></div>

            <button type="button" class="user-menu-action" @click="openChangePasswordModal">
              <IconKey class="user-menu-icon" />
              <span>{{ t('auth.changePassword') }}</span>
            </button>

            <button
              type="button"
              class="user-menu-action user-menu-logout"
              @click="handleLogout"
              :disabled="isLoading"
            >
              <IconLogout class="user-menu-icon" />
              <span>{{ t('auth.logout') }}</span>
            </button>
          </div>
        </div>
      </div>
    </aside>

    <!-- ═══════════════════════════════════════════════════════════════════════════
         MAIN CONTENT AREA
         ═══════════════════════════════════════════════════════════════════════════ -->
    <main class="kroma-main">
      <!-- Mobile header (phones only < 640px) -->
      <header class="kroma-mobile-header">
        <span class="kroma-mobile-title">Kroma</span>

        <!-- Mobile user avatar -->
        <div class="mobile-user-wrapper">
          <button
            type="button"
            class="mobile-user-avatar"
            @click="toggleUserMenu"
            :aria-label="t('admin.userMenu')"
          >
            <span class="avatar-initials">{{ userInitials }}</span>
          </button>

          <!-- Mobile user menu -->
          <div v-if="userMenuOpen" class="mobile-user-menu">
            <div class="mobile-user-info">
              <span v-if="user?.name" class="mobile-user-name">{{ user.name }}</span>
              <span class="mobile-user-email">{{ user?.email }}</span>
              <span class="mobile-user-role">{{ user?.role }}</span>
            </div>

            <div class="mobile-menu-divider"></div>

            <!-- Theme toggle -->
            <button type="button" class="mobile-menu-item" @click="toggleTheme">
              <IconSun v-if="isDark" class="mobile-menu-icon" />
              <IconMoon v-else class="mobile-menu-icon" />
              <span>{{ isDark ? t('common.lightMode') : t('common.darkMode') }}</span>
            </button>

            <div class="mobile-menu-divider"></div>

            <!-- Change Password -->
            <button type="button" class="mobile-menu-item" @click="openChangePasswordModal">
              <IconKey class="mobile-menu-icon" />
              <span>{{ t('auth.changePassword') }}</span>
            </button>

            <!-- Logout -->
            <button
              type="button"
              class="mobile-menu-item mobile-menu-logout"
              @click="handleLogout"
              :disabled="isLoading"
            >
              <IconLogout class="mobile-menu-icon" />
              <span>{{ t('auth.logout') }}</span>
            </button>
          </div>
        </div>
      </header>

      <!-- Page content -->
      <div class="kroma-content">
        <slot />
      </div>
    </main>

    <!-- ═══════════════════════════════════════════════════════════════════════════
         BOTTOM NAVIGATION (phones only < 640px)
         ═══════════════════════════════════════════════════════════════════════════ -->
    <nav class="kroma-bottom-nav">
      <NuxtLink
        v-for="item in navItems"
        :key="item.to"
        :to="localePath(item.to)"
        class="kroma-bottom-nav-item"
        :class="{ 'kroma-bottom-nav-item--active': route.path === item.to }"
      >
        <component :is="item.icon" class="kroma-bottom-nav-icon" />
        <span class="kroma-bottom-nav-label">{{ t(item.label) }}</span>
      </NuxtLink>
    </nav>

    <!-- Global UI Components -->
    <OrganismsConfirmDialog />
    <OrganismsToastContainer />
  </div>
</template>

<!--
  Uses global CSS classes:
  - layout/kroma-sidebar.css: .kroma-sidebar, .sidebar-nav, .sidebar-nav-link
  - layout/kroma-main.css: .kroma-main, .kroma-content
  - layout/kroma-mobile-header.css: .kroma-mobile-header
  - skeleton/bottom-nav.css: .kroma-bottom-nav, .kroma-bottom-nav-item
  - ui/buttons.css: .btn, .btn-icon
  - common/icons.css: .sidebar-icon
-->
