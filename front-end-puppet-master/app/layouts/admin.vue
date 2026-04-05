<script setup lang="ts">
/**
 * Admin Layout - App Visual Mode
 *
 * Responsive navigation following Material Design 3:
 * - Phone (< 600px): Bottom navigation bar
 * - Tablet portrait (600-839px): Navigation rail (narrow sidebar)
 * - Tablet landscape / Desktop (≥ 840px): Full sidebar
 *
 * Navigation is config-driven via config.admin in puppet-master.config.ts
 * Uses getAdminSectionsForRole() to filter sections by user's role.
 *
 * Uses existing CSS system:
 * - layout/page.css: .layout-admin responsive styles
 * - layout/kroma-sidebar.css: .sidebar-nav, .sidebar-nav-link, .sidebar-icon-btn
 * - skeleton/bottom-nav.css: .bottom-nav, .bottom-nav-item
 *
 * Auth is handled by middleware (app/middleware/auth.ts)
 */
import type { Component } from 'vue'
import config from '~/puppet-master.config'
import IconSettings from '~icons/tabler/settings'
import IconPhoto from '~icons/tabler/photo'
import IconMail from '~icons/tabler/mail'
import IconLanguage from '~icons/tabler/language'
import IconUsers from '~icons/tabler/users'
import IconHeartbeat from '~icons/tabler/heartbeat'
import IconLogout from '~icons/tabler/logout'
import IconKey from '~icons/tabler/key'
import IconCreditCard from '~icons/tabler/credit-card'
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

// SSR-safe config access with defensive defaults
const adminConfig = computed(() => config?.admin ?? { sections: [] })

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

const route = useRoute()
const { t } = useI18n()
const localePath = useLocalePath()
const { user, logout, isLoading, hasRole } = useAuth()
const { shortLogo } = useLogo()

// User menu panel state (desktop sidebar)
const userMenuOpen = ref(false)
// Mobile user menu state
const mobileUserMenuOpen = ref(false)
// Change password modal state
const changePasswordModalOpen = ref(false)

function toggleUserMenu() {
  userMenuOpen.value = !userMenuOpen.value
}

function toggleMobileUserMenu() {
  mobileUserMenuOpen.value = !mobileUserMenuOpen.value
}

// Get admin sections filtered by user's role
const adminSections = computed(() => {
  if (!user.value) return []
  return config.getAdminSectionsForRole(user.value.role)
})

// Get current page title based on route name (works with any locale prefix)
const currentPageTitle = computed(() => {
  const name = route.name?.toString() ?? ''
  // Try to match from config first
  for (const section of adminSections.value) {
    if (name.startsWith(`admin-${section.id}`)) {
      return t(`admin.${section.label}`)
    }
  }
  // Fallback for dashboard
  if (name.startsWith('admin')) return t('admin.dashboard')
  return t('admin.title')
})

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

// Config-driven admin links - filtered by role access
const adminLinks = computed(() => {
  return adminSections.value.map(section => ({
    to: localePath(`/admin/${section.id}`),
    label: `admin.${section.label}`,
    icon: iconMap[section.icon] || IconSettings,
    badge: section.badge
  }))
})

async function handleLogout() {
  await logout()
}

function openChangePasswordModal() {
  userMenuOpen.value = false
  mobileUserMenuOpen.value = false
  changePasswordModalOpen.value = true
}

// Theme toggle and language switcher now use shared components
// with View Transitions API animations (ThemeToggle, LangSwitcher)

// Close panels on click outside
function handleClickOutside(e: MouseEvent) {
  const target = e.target as HTMLElement

  // Close user menu if click is outside ANY user wrapper (sidebar OR horizontal header)
  const inSidebarUser = target.closest('.sidebar-user-wrapper')
  const inHeaderUser = target.closest('.admin-header-user-wrapper')
  if (!inSidebarUser && !inHeaderUser) {
    userMenuOpen.value = false
  }

  // Close mobile user menu
  if (!target.closest('.mobile-user-wrapper')) {
    mobileUserMenuOpen.value = false
  }
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
})
</script>

<template>
  <div class="layout-admin" :class="{ 'layout-admin--horizontal': !config.features.appVerticalNav }">
    <!-- ═══════════════════════════════════════════════════════════════════════════
         HORIZONTAL HEADER (when appVerticalNav: false)
         ═══════════════════════════════════════════════════════════════════════════ -->
    <header v-if="!config.features.appVerticalNav" class="admin-header-horizontal">
      <div class="admin-header-inner">
        <!-- Logo -->
        <NuxtLink :to="localePath('/admin')" class="admin-header-logo">
          <ClientOnly>
            <img :src="shortLogo" alt="Logo" class="logo-img" width="36" height="36" />
            <template #fallback>
              <img
                :src="`${config.logo.basePath}/circle_${config.defaultTheme === 'dark' ? 'light' : 'dark'}_${config.defaultLocale}.svg`"
                alt="Logo"
                class="logo-img"
                width="36"
                height="36"
              />
            </template>
          </ClientOnly>
        </NuxtLink>

        <!-- Navigation links (icon-only with tooltips) -->
        <nav class="admin-header-nav">
          <NuxtLink
            v-for="link in adminLinks"
            :key="link.to"
            :to="link.to"
            class="admin-header-link"
          >
            <component :is="link.icon" class="admin-header-link-icon" />
            <span class="admin-header-tooltip">{{ t(link.label) }}</span>
          </NuxtLink>
        </nav>

        <!-- Actions (theme, lang, user) -->
        <div class="admin-header-actions">
          <AtomsThemeToggle class="admin-header-btn" />
          <AtomsLangSwitcher direction="down" />

          <!-- User avatar with dropdown -->
          <div class="admin-header-user-wrapper">
            <button
              type="button"
              class="admin-header-user-avatar"
              @click="toggleUserMenu"
              :aria-label="t('admin.userMenu')"
            >
              <span class="avatar-initials">{{ userInitials }}</span>
            </button>

            <!-- User dropdown menu -->
            <div v-if="userMenuOpen" class="admin-header-user-menu">
              <div class="user-menu-info">
                <span v-if="user?.name" class="user-menu-name">{{ user.name }}</span>
                <span class="user-menu-email">{{ user?.email }}</span>
                <span class="user-menu-role">{{ user?.role }}</span>
              </div>
              <button type="button" class="user-menu-action" @click="openChangePasswordModal">
                <IconKey />
                <span>{{ t('auth.changePassword') }}</span>
              </button>
              <button
                type="button"
                class="user-menu-logout"
                @click="handleLogout"
                :disabled="isLoading"
              >
                <IconLogout />
                <span>{{ t('auth.logout') }}</span>
              </button>
            </div>
          </div>
        </div>
      </div>
    </header>

    <!-- ═══════════════════════════════════════════════════════════════════════════
         VERTICAL SIDEBAR (when appVerticalNav: true)
         ═══════════════════════════════════════════════════════════════════════════ -->
    <aside v-if="config.features.appVerticalNav" class="admin-sidebar">
      <!-- Logo (short/circle version) - ClientOnly to avoid SSR hydration mismatch -->
      <div class="sidebar-header">
        <NuxtLink :to="localePath('/admin')">
          <ClientOnly>
            <img :src="shortLogo" alt="Logo" class="logo-img" width="40" height="40" />
            <template #fallback>
              <img
                :src="`${config.logo.basePath}/circle_${config.defaultTheme === 'dark' ? 'light' : 'dark'}_${config.defaultLocale}.svg`"
                alt="Logo"
                class="logo-img"
                width="40"
                height="40"
              />
            </template>
          </ClientOnly>
        </NuxtLink>
      </div>

      <!-- Nav links with tooltips -->
      <nav class="sidebar-nav">
        <NuxtLink v-for="link in adminLinks" :key="link.to" :to="link.to" class="sidebar-nav-link">
          <component :is="link.icon" />
          <span class="sidebar-tooltip">{{ t(link.label) }}</span>
        </NuxtLink>
      </nav>

      <!-- Footer actions -->
      <div class="sidebar-footer">
        <!-- Theme toggle - uses shared component with View Transitions animation -->
        <AtomsThemeToggle class="sidebar-icon-btn" />

        <!-- Language switcher - reusable component -->
        <AtomsLangSwitcher direction="side" />

        <!-- User avatar with popover menu (at bottom per UX best practice) -->
        <div class="sidebar-user-wrapper">
          <button
            type="button"
            class="sidebar-user-avatar"
            @click="toggleUserMenu"
            :aria-label="t('admin.userMenu')"
          >
            <span class="avatar-initials">{{ userInitials }}</span>
          </button>
          <div v-if="userMenuOpen" class="sidebar-user-menu">
            <div class="user-menu-info">
              <span v-if="user?.name" class="user-menu-name">{{ user.name }}</span>
              <span class="user-menu-email">{{ user?.email }}</span>
              <span class="user-menu-role">{{ user?.role }}</span>
            </div>
            <button type="button" class="user-menu-action" @click="openChangePasswordModal">
              <IconKey />
              <span>{{ t('auth.changePassword') }}</span>
            </button>
            <button
              type="button"
              class="user-menu-logout"
              @click="handleLogout"
              :disabled="isLoading"
            >
              <IconLogout />
              <span>{{ t('auth.logout') }}</span>
            </button>
          </div>
        </div>
      </div>
    </aside>

    <!-- Global UI Components -->
    <OrganismsConfirmDialog />
    <OrganismsToastContainer />
    <OrganismsChangePasswordModal v-model="changePasswordModalOpen" />

    <!-- Main Content -->
    <main class="admin-main">
      <!-- Mobile header (phones only < 600px) - shows page title + user avatar -->
      <!-- Shown for BOTH vertical nav and horizontal header modes on phones (CSS handles visibility) -->
      <header class="admin-header mobile-only">
        <span class="admin-title">{{ currentPageTitle }}</span>

        <!-- Mobile user avatar with menu -->
        <div class="mobile-user-wrapper">
          <button
            type="button"
            class="mobile-user-avatar"
            @click="toggleMobileUserMenu"
            :aria-label="t('admin.userMenu')"
          >
            <span class="avatar-initials">{{ userInitials }}</span>
          </button>

          <!-- Mobile user menu dropdown -->
          <div v-if="mobileUserMenuOpen" class="mobile-user-menu">
            <!-- User info -->
            <div class="mobile-user-info">
              <span v-if="user?.name" class="mobile-user-name">{{ user.name }}</span>
              <span class="mobile-user-email">{{ user?.email }}</span>
              <span class="mobile-user-role">{{ user?.role }}</span>
            </div>

            <div class="mobile-menu-divider"></div>

            <!-- Theme & Language - uses shared components for consistency -->
            <div class="mobile-menu-toggles">
              <AtomsThemeToggle />
              <AtomsLangSwitcher direction="inline" />
            </div>

            <div class="mobile-menu-divider"></div>

            <!-- Change Password -->
            <button type="button" class="mobile-menu-item" @click="openChangePasswordModal">
              <IconKey />
              <span>{{ t('auth.changePassword') }}</span>
            </button>

            <!-- Logout -->
            <button
              type="button"
              class="mobile-menu-item mobile-menu-logout"
              @click="handleLogout"
              :disabled="isLoading"
            >
              <IconLogout />
              <span>{{ t('auth.logout') }}</span>
            </button>
          </div>
        </div>
      </header>

      <div class="admin-content">
        <slot />
      </div>
    </main>

    <!-- Bottom Navigation (phones only < 600px) -->
    <!-- Shown for BOTH vertical nav and horizontal header modes on phones (CSS handles visibility) -->
    <!-- ClientOnly to avoid hydration mismatch: canManageUsers is false on server, true on client -->
    <ClientOnly>
      <MoleculesAppBottomNav />
    </ClientOnly>
  </div>
</template>

<!--
  Uses global CSS classes:
  - layout/page.css: .layout-admin responsive styles (phone/tablet/desktop)
  - layout/admin-header.css: .admin-header-horizontal (horizontal nav mode)
  - layout/kroma-sidebar.css: .sidebar-nav, .sidebar-nav-link, .sidebar-icon-btn, .sidebar-tooltip
  - skeleton/bottom-nav.css: .bottom-nav, .bottom-nav-item (phone only)
  - ui/buttons.css: .btn, .btn-icon, .btn-ghost
  - ui/content/badges.css: .badge-dot
  - layout/responsive.css: .mobile-only
-->
