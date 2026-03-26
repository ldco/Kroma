<script setup lang="ts">
/**
 * Login Page
 *
 * Used in:
 * - website-app mode: Visible at /login
 * - app-only mode: Redirected from / to /admin/login (uses admin login instead)
 * - website-admin mode: Hidden, accessible at /admin/login (uses admin login)
 *
 * This page is only used when mode = 'website-app'
 *
 * Uses ClientOnly for logo to avoid SSR hydration mismatch.
 */
import config from '~/puppet-master.config'
import IconLogin from '~icons/tabler/login'
import IconMail from '~icons/tabler/mail'
import IconLock from '~icons/tabler/lock'
import IconEye from '~icons/tabler/eye'
import IconEyeOff from '~icons/tabler/eye-off'

definePageMeta({
  layout: 'blank'
})

const { t } = useI18n()
const { shortLogo } = useLogo()

// SSR-safe config access with defensive defaults
const logoConfig = computed(() => config?.logo ?? { basePath: '/logos' })
const defaultTheme = computed(() => config?.defaultTheme || 'light')
const defaultLocale = computed(() => config?.defaultLocale || 'en')

// SSR fallback: use the default theme logo
const ssrFallbackLogo = computed(() => {
  const theme = defaultTheme.value === 'dark' ? 'light' : 'dark'
  return `${logoConfig.value.basePath}/circle_${theme}_${defaultLocale.value}.svg`
})

const email = ref('')
const password = ref('')
const showPassword = ref(false)
const rememberMe = ref(false)
const isLoading = ref(false)

async function handleSubmit() {
  isLoading.value = true
  // TODO: Implement actual login logic
  console.log('Login attempt:', { email: email.value, rememberMe: rememberMe.value })
  await new Promise(resolve => setTimeout(resolve, 1000))
  isLoading.value = false
}
</script>

<template>
  <!-- Uses .auth-page, .auth-card, etc. from ui/forms/inputs.css -->
  <div class="auth-page">
    <div class="auth-card">
      <!-- Logo - using circle variant for auth pages - ClientOnly for SSR -->
      <div class="auth-logo">
        <ClientOnly>
          <img :src="shortLogo" alt="Logo" class="auth-logo-img" />
          <template #fallback>
            <img :src="ssrFallbackLogo" alt="Logo" class="auth-logo-img" />
          </template>
        </ClientOnly>
      </div>

      <!-- Title -->
      <h1 class="auth-title">{{ t('auth.login') }}</h1>

      <!-- Form -->
      <form class="auth-form" @submit.prevent="handleSubmit">
        <!-- Email -->
        <div class="form-group">
          <label class="form-label" for="email">{{ t('auth.email') }}</label>
          <div class="input-with-icon">
            <IconMail class="input-icon" aria-hidden="true" />
            <input
              id="email"
              v-model="email"
              type="email"
              class="input"
              autocomplete="email"
              required
            />
          </div>
        </div>

        <!-- Password -->
        <div class="form-group">
          <label class="form-label" for="password">{{ t('auth.password') }}</label>
          <div class="input-with-icon input-with-action">
            <IconLock class="input-icon" aria-hidden="true" />
            <input
              id="password"
              v-model="password"
              :type="showPassword ? 'text' : 'password'"
              class="input"
              autocomplete="current-password"
              required
            />
            <button
              type="button"
              class="input-action"
              :aria-label="showPassword ? 'Hide password' : 'Show password'"
              @click="showPassword = !showPassword"
            >
              <IconEyeOff v-if="showPassword" />
              <IconEye v-else />
            </button>
          </div>
        </div>

        <!-- Remember me + Forgot password -->
        <div class="flex justify-between items-center gap-4">
          <label class="checkbox">
            <input v-model="rememberMe" type="checkbox" />
            <span>{{ t('auth.rememberMe') }}</span>
          </label>
          <NuxtLink to="/forgot-password" class="auth-link">
            {{ t('auth.forgotPassword') }}
          </NuxtLink>
        </div>

        <!-- Submit -->
        <button type="submit" class="btn btn-primary btn-full" :disabled="isLoading">
          <IconLogin v-if="!isLoading" aria-hidden="true" />
          <span>{{ isLoading ? t('common.loading') : t('auth.login') }}</span>
        </button>
      </form>
    </div>
  </div>
</template>
