<script setup lang="ts">
/**
 * Two-Factor Authentication Verification Component
 *
 * Shown during login when 2FA is required.
 * Accepts TOTP codes and backup codes.
 */
import IconShieldLock from '~icons/tabler/shield-lock'
import IconArrowLeft from '~icons/tabler/arrow-left'
import IconAlertTriangle from '~icons/tabler/alert-triangle'

const emit = defineEmits<{
  verified: []
  cancelled: []
}>()

const { t } = useI18n()
const { verify2fa, isLoading, cancel2fa } = useAuth()

// State
const code = ref('')
const error = ref('')
const useBackupCode = ref(false)

// Handle verification
async function handleVerify() {
  if (!code.value) {
    error.value = useBackupCode.value
      ? t('auth.twoFactor.enterBackupCode', 'Please enter a backup code')
      : t('auth.twoFactor.enterCode', 'Please enter a verification code')
    return
  }

  error.value = ''
  const result = await verify2fa(code.value)

  if (result.success) {
    emit('verified')
  } else {
    error.value = result.error || t('auth.twoFactor.invalidCode', 'Invalid code')
    code.value = ''
  }
}

// Handle cancel
function handleCancel() {
  cancel2fa()
  emit('cancelled')
}

// Toggle between TOTP and backup code
function toggleBackupCode() {
  useBackupCode.value = !useBackupCode.value
  code.value = ''
  error.value = ''
}
</script>

<template>
  <div class="two-factor-verify">
    <div class="verify-header">
      <IconShieldLock class="verify-icon" />
      <h3>{{ t('auth.twoFactor.verifyTitle', 'Two-Factor Authentication') }}</h3>
      <p class="text-muted">
        <template v-if="useBackupCode">
          {{ t('auth.twoFactor.enterBackupDescription', 'Enter one of your backup codes') }}
        </template>
        <template v-else>
          {{ t('auth.twoFactor.verifyDescription', 'Enter the 6-digit code from your authenticator app') }}
        </template>
      </p>
    </div>

    <form class="verify-form" @submit.prevent="handleVerify">
      <!-- Code Input -->
      <div class="form-group">
        <label class="form-label" for="2fa-code">
          {{ useBackupCode
            ? t('auth.twoFactor.backupCode', 'Backup Code')
            : t('auth.twoFactor.verificationCode', 'Verification Code')
          }}
        </label>
        <input
          id="2fa-code"
          v-model="code"
          :type="useBackupCode ? 'text' : 'text'"
          :inputmode="useBackupCode ? 'text' : 'numeric'"
          :pattern="useBackupCode ? '[A-Za-z0-9]*' : '[0-9]*'"
          :maxlength="useBackupCode ? 8 : 6"
          class="input code-input"
          :placeholder="useBackupCode
            ? t('auth.twoFactor.backupPlaceholder', 'XXXXXXXX')
            : t('auth.twoFactor.codePlaceholder', '000000')
          "
          autocomplete="one-time-code"
          autofocus
        />
      </div>

      <!-- Error Message -->
      <div v-if="error" class="error-message">
        <IconAlertTriangle class="error-icon" />
        {{ error }}
      </div>

      <!-- Actions -->
      <div class="verify-actions">
        <button
          type="submit"
          class="btn btn-primary btn-full"
          :disabled="isLoading"
        >
          {{ isLoading
            ? t('common.verifying', 'Verifying...')
            : t('auth.twoFactor.verify', 'Verify')
          }}
        </button>
      </div>

      <!-- Toggle Backup Code -->
      <button
        type="button"
        class="toggle-backup-btn"
        @click="toggleBackupCode"
      >
        {{ useBackupCode
          ? t('auth.twoFactor.useAuthenticator', 'Use authenticator app instead')
          : t('auth.twoFactor.useBackup', 'Use a backup code instead')
        }}
      </button>

      <!-- Back to Login -->
      <button
        type="button"
        class="back-btn"
        @click="handleCancel"
      >
        <IconArrowLeft />
        {{ t('auth.twoFactor.backToLogin', 'Back to login') }}
      </button>
    </form>
  </div>
</template>
