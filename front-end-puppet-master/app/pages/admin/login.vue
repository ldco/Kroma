<script setup lang="ts">
/**
 * Admin Login Page
 *
 * Used in:
 * - app-only mode: Main entry point (/ redirects here)
 * - website-admin mode: Hidden admin access at /admin/login
 *
 * Reuses the same login form logic, different layout context.
 * Supports two-factor authentication (2FA) flow.
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
  layout: 'blank',
  pageTransition: false
})

const { t } = useI18n()

useHead({
  title: () => `${t('auth.login')} | Admin`
})

const { login, isLoading, isAuthenticated, requires2fa, cancel2fa } = useAuth()
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
const errorMessage = ref('')

// Redirect if already authenticated
onMounted(async () => {
  const { checkSession } = useAuth()
  await checkSession()
  if (isAuthenticated.value) {
    navigateTo('/admin')
  }
})

async function handleSubmit() {
  errorMessage.value = ''

  const result = await login({
    email: email.value,
    password: password.value,
    rememberMe: rememberMe.value
  })

  if (result.success) {
    // Check if 2FA is required
    if (result.requires2fa) {
      // Stay on page, TwoFactorVerify component will be shown
      return
    }
    navigateTo('/admin')
  } else {
    errorMessage.value = result.error || t('auth.loginFailed')
  }
}

// Handle 2FA verification success
function on2faVerified() {
  navigateTo('/admin')
}

// Handle 2FA cancellation
function on2faCancelled() {
  cancel2fa()
  // Reset form
  password.value = ''
  errorMessage.value = ''
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

      <!-- 2FA Verification -->
      <template v-if="requires2fa">
        <MoleculesTwoFactorVerify
          @verified="on2faVerified"
          @cancelled="on2faCancelled"
        />
      </template>

      <!-- Login Form -->
      <template v-else>
        <!-- Title -->
        <h1 class="auth-title">Admin {{ t('auth.login') }}</h1>

        <!-- Error Message -->
        <div
          v-if="errorMessage"
          class="form-error text-center mb-4"
        >
          {{ errorMessage }}
        </div>

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

          <!-- Remember me -->
          <label class="checkbox">
            <input v-model="rememberMe" type="checkbox" />
            <span>{{ t('auth.rememberMe') }}</span>
          </label>

          <!-- Submit -->
          <button type="submit" class="btn btn-primary btn-full" :disabled="isLoading">
            <IconLogin v-if="!isLoading" aria-hidden="true" />
            <span>{{ isLoading ? t('common.loading') : t('auth.login') }}</span>
          </button>
        </form>

        <!-- Back to site link (only for website-admin mode) -->
        <div class="auth-footer">
          <NuxtLink to="/" class="auth-link">← Back to website</NuxtLink>
        </div>
      </template>
    </div>
  </div>
</template>
