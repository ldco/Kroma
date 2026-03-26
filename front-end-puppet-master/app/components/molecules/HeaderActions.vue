<script setup lang="ts">
/**
 * HeaderActions Molecule
 *
 * Header action buttons grouped together:
 * - Quick contact buttons (phone + messenger) - optional via prop
 * - Theme toggle (if doubleTheme enabled)
 * - Language switcher (if multiLangs enabled)
 * - Login button (only in website-app mode)
 *
 * Shows only enabled features based on config.
 */
import IconLogin from '~icons/tabler/login'
import config from '~/puppet-master.config'

const props = withDefaults(
  defineProps<{
    /** Show contact buttons (set false in mobile menu where they're in header) */
    showContact?: boolean
    /** Language switcher direction: down (header), side (sidebar), inline (mobile menu) */
    langDirection?: 'down' | 'side' | 'inline'
  }>(),
  {
    showContact: true,
    langDirection: 'down'
  }
)

const { t } = useI18n()

// SSR-safe config access with defensive defaults
const hasContact = computed(() => config?.headerContact?.enabled ?? false)
const hasThemeToggle = computed(() => config?.hasThemeToggle ?? false)
const hasMultiLang = computed(() => config?.isMultiLang ?? false)
const hasLoginButton = computed(() => config?.hasLoginButton ?? false)
</script>

<template>
  <!-- Uses global .header-actions class from skeleton/header.css -->
  <div class="header-actions">
    <!-- Quick contact buttons (left of toggles) - hidden in mobile menu -->
    <MoleculesHeaderContact v-if="showContact && hasContact" />

    <AtomsThemeToggle v-if="hasThemeToggle" />
    <AtomsLangSwitcher v-if="hasMultiLang" :direction="langDirection" />

    <!-- Login button - only in website-app mode -->
    <NuxtLink v-if="hasLoginButton" to="/login" class="header-login-btn">
      <IconLogin aria-hidden="true" />
      <span>{{ t('auth.login') }}</span>
    </NuxtLink>
  </div>
</template>

<!-- No scoped styles needed - uses skeleton/header.css -->
