<script setup lang="ts">
/**
 * Two-Factor Authentication Setup Component
 *
 * Displays QR code for authenticator app setup and handles 2FA enablement.
 * Shows backup codes after successful setup.
 */
import IconShieldLock from '~icons/tabler/shield-lock'
import IconCopy from '~icons/tabler/copy'
import IconCheck from '~icons/tabler/check'
import IconRefresh from '~icons/tabler/refresh'
import IconAlertTriangle from '~icons/tabler/alert-triangle'

const emit = defineEmits<{
  enabled: []
  cancelled: []
}>()

const { t } = useI18n()

// State
const step = ref<'setup' | 'verify' | 'backup'>('setup')
const isLoading = ref(false)
const error = ref('')
const qrCode = ref('')
const uri = ref('')
const backupCodes = ref<string[]>([])
const verificationCode = ref('')
const copiedIndex = ref<number | null>(null)

// Start 2FA setup
async function startSetup() {
  isLoading.value = true
  error.value = ''

  try {
    const response = await apiFetch<{
      qrCode: string
      uri: string
      backupCodes: string[]
    }>('/api/user/2fa/setup', {
      method: 'POST'
    })

    qrCode.value = response.qrCode
    uri.value = response.uri
    backupCodes.value = response.backupCodes
    step.value = 'verify'
  } catch (err: unknown) {
    const e = err as { data?: { message?: string }; message?: string }
    error.value = e?.data?.message || e?.message || 'Failed to start 2FA setup'
  } finally {
    isLoading.value = false
  }
}

// Verify and enable 2FA
async function verifyAndEnable() {
  if (!verificationCode.value || verificationCode.value.length !== 6) {
    error.value = 'Please enter a 6-digit verification code'
    return
  }

  isLoading.value = true
  error.value = ''

  try {
    await apiFetch<{ message: string }>('/api/user/2fa/enable', {
      method: 'POST',
      body: { code: verificationCode.value }
    })

    step.value = 'backup'
  } catch (err: unknown) {
    const e = err as { data?: { message?: string }; message?: string }
    error.value = e?.data?.message || e?.message || 'Invalid verification code'
    verificationCode.value = ''
  } finally {
    isLoading.value = false
  }
}

// Copy backup code to clipboard
async function copyBackupCode(code: string, index: number) {
  try {
    await navigator.clipboard.writeText(code)
    copiedIndex.value = index
    setTimeout(() => {
      copiedIndex.value = null
    }, 2000)
  } catch {
    // Fallback for older browsers
    const textArea = document.createElement('textarea')
    textArea.value = code
    document.body.appendChild(textArea)
    textArea.select()
    document.execCommand('copy')
    document.body.removeChild(textArea)
    copiedIndex.value = index
    setTimeout(() => {
      copiedIndex.value = null
    }, 2000)
  }
}

// Copy all backup codes
async function copyAllBackupCodes() {
  const allCodes = backupCodes.value.join('\n')
  try {
    await navigator.clipboard.writeText(allCodes)
    copiedIndex.value = -1
    setTimeout(() => {
      copiedIndex.value = null
    }, 2000)
  } catch {
    // Fallback
    const textArea = document.createElement('textarea')
    textArea.value = allCodes
    document.body.appendChild(textArea)
    textArea.select()
    document.execCommand('copy')
    document.body.removeChild(textArea)
    copiedIndex.value = -1
    setTimeout(() => {
      copiedIndex.value = null
    }, 2000)
  }
}

// Complete setup
function completeSetup() {
  emit('enabled')
}

// Cancel setup
function cancel() {
  emit('cancelled')
}

// Start setup on mount
onMounted(() => {
  startSetup()
})
</script>

<template>
  <div class="two-factor-setup">
    <!-- Setup Step: Show QR Code -->
    <template v-if="step === 'setup' || step === 'verify'">
      <div class="setup-header">
        <IconShieldLock class="setup-icon" />
        <h3>{{ t('auth.twoFactor.setupTitle', 'Set up Two-Factor Authentication') }}</h3>
        <p class="text-muted">
          {{ t('auth.twoFactor.setupDescription', 'Scan the QR code with your authenticator app (like Google Authenticator, Authy, or 1Password)') }}
        </p>
      </div>

      <!-- Loading state -->
      <div v-if="isLoading && !qrCode" class="setup-loading">
        <IconRefresh class="spin" />
        <span>{{ t('common.loading', 'Loading...') }}</span>
      </div>

      <!-- QR Code -->
      <div v-else-if="qrCode" class="qr-section">
        <div class="qr-code">
          <img :src="qrCode" alt="2FA QR Code" />
        </div>

        <details class="manual-entry">
          <summary>{{ t('auth.twoFactor.cantScan', "Can't scan? Enter code manually") }}</summary>
          <code class="manual-code">{{ uri }}</code>
        </details>
      </div>

      <!-- Verification Form -->
      <div v-if="step === 'verify'" class="verify-section">
        <label class="form-label" for="verification-code">
          {{ t('auth.twoFactor.enterCode', 'Enter the 6-digit code from your app') }}
        </label>
        <div class="code-input-wrapper">
          <input
            id="verification-code"
            v-model="verificationCode"
            type="text"
            inputmode="numeric"
            pattern="[0-9]*"
            maxlength="6"
            class="input code-input"
            :placeholder="t('auth.twoFactor.codePlaceholder', '000000')"
            autocomplete="one-time-code"
            @keyup.enter="verifyAndEnable"
          />
        </div>

        <!-- Error message -->
        <div v-if="error" class="error-message">
          <IconAlertTriangle class="error-icon" />
          {{ error }}
        </div>

        <!-- Actions -->
        <div class="setup-actions">
          <button type="button" class="btn btn-ghost" @click="cancel">
            {{ t('common.cancel', 'Cancel') }}
          </button>
          <button
            type="button"
            class="btn btn-primary"
            :disabled="isLoading || verificationCode.length !== 6"
            @click="verifyAndEnable"
          >
            {{ isLoading ? t('common.verifying', 'Verifying...') : t('auth.twoFactor.verify', 'Verify & Enable') }}
          </button>
        </div>
      </div>
    </template>

    <!-- Backup Codes Step -->
    <template v-else-if="step === 'backup'">
      <div class="setup-header success">
        <IconCheck class="setup-icon success-icon" />
        <h3>{{ t('auth.twoFactor.enabled', 'Two-Factor Authentication Enabled!') }}</h3>
        <p class="text-muted">
          {{ t('auth.twoFactor.backupDescription', 'Save these backup codes in a safe place. You can use them to sign in if you lose access to your authenticator app.') }}
        </p>
      </div>

      <div class="backup-warning">
        <IconAlertTriangle />
        <span>{{ t('auth.twoFactor.backupWarning', 'These codes will only be shown once!') }}</span>
      </div>

      <div class="backup-codes">
        <div
          v-for="(code, index) in backupCodes"
          :key="index"
          class="backup-code"
          @click="copyBackupCode(code, index)"
        >
          <code>{{ code }}</code>
          <IconCheck v-if="copiedIndex === index" class="copied-icon" />
          <IconCopy v-else class="copy-icon" />
        </div>
      </div>

      <button
        type="button"
        class="btn btn-secondary btn-full"
        @click="copyAllBackupCodes"
      >
        <IconCopy v-if="copiedIndex !== -1" />
        <IconCheck v-else />
        {{ copiedIndex === -1 ? t('auth.twoFactor.copied', 'Copied!') : t('auth.twoFactor.copyAll', 'Copy All Codes') }}
      </button>

      <div class="setup-actions">
        <button
          type="button"
          class="btn btn-primary btn-full"
          @click="completeSetup"
        >
          {{ t('auth.twoFactor.done', "I've saved my backup codes") }}
        </button>
      </div>
    </template>
  </div>
</template>
