<script setup lang="ts">
/**
 * Change Password Modal
 *
 * Modal dialog for changing the current user's password.
 * Uses native <dialog> element for proper accessibility.
 *
 * Uses global CSS: .modal, .modal-header, .modal-body, .modal-footer, .btn, .form-group, .input
 */
import IconX from '~icons/tabler/x'
import IconLock from '~icons/tabler/lock'
import IconEye from '~icons/tabler/eye'
import IconEyeOff from '~icons/tabler/eye-off'

const props = defineProps<{
  modelValue: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
}>()

const { t } = useI18n()
const { toast } = useToast()

const dialogRef = ref<HTMLDialogElement | null>(null)
const isSubmitting = ref(false)

// Form state
const currentPassword = ref('')
const newPassword = ref('')
const confirmPassword = ref('')
const showCurrentPassword = ref(false)
const showNewPassword = ref(false)
const showConfirmPassword = ref(false)
const errorMessage = ref('')

// Validation
const isValid = computed(() => {
  return (
    currentPassword.value.length > 0 &&
    newPassword.value.length >= 8 &&
    confirmPassword.value === newPassword.value
  )
})

const passwordMismatch = computed(() => {
  return confirmPassword.value.length > 0 && confirmPassword.value !== newPassword.value
})

const passwordTooShort = computed(() => {
  return newPassword.value.length > 0 && newPassword.value.length < 8
})

// Open/close dialog when modelValue changes
watch(
  () => props.modelValue,
  isOpen => {
    if (isOpen) {
      dialogRef.value?.showModal()
      resetForm()
    } else {
      dialogRef.value?.close()
    }
  }
)

function resetForm() {
  currentPassword.value = ''
  newPassword.value = ''
  confirmPassword.value = ''
  showCurrentPassword.value = false
  showNewPassword.value = false
  showConfirmPassword.value = false
  errorMessage.value = ''
}

function closeModal() {
  emit('update:modelValue', false)
}

// Handle ESC key
function handleCancel(e: Event) {
  e.preventDefault()
  closeModal()
}

// Handle backdrop click
function handleBackdropClick(e: MouseEvent) {
  const rect = dialogRef.value?.getBoundingClientRect()
  if (!rect) return

  const isOutside =
    e.clientX < rect.left ||
    e.clientX > rect.right ||
    e.clientY < rect.top ||
    e.clientY > rect.bottom

  if (isOutside) {
    closeModal()
  }
}

async function handleSubmit() {
  if (!isValid.value || isSubmitting.value) return

  isSubmitting.value = true
  errorMessage.value = ''

  try {
    await apiFetch<{ updated?: boolean }>('/api/user/change-password', {
      method: 'PUT',
      body: {
        currentPassword: currentPassword.value,
        newPassword: newPassword.value
      }
    })

    toast.success(t('auth.passwordChanged'))
    closeModal()
  } catch (error: unknown) {
    const err = error as { data?: { message?: string } }
    errorMessage.value = err.data?.message || t('common.error')
  } finally {
    isSubmitting.value = false
  }
}
</script>

<template>
  <dialog
    ref="dialogRef"
    class="modal modal-sm"
    aria-labelledby="change-password-title"
    @cancel="handleCancel"
    @click="handleBackdropClick"
  >
    <form @submit.prevent="handleSubmit">
      <div class="modal-header">
        <h2 id="change-password-title" class="modal-title">{{ t('auth.changePassword') }}</h2>
        <button type="button" class="modal-close" :aria-label="t('common.close')" @click="closeModal">
          <IconX />
        </button>
      </div>

      <div class="modal-body">
        <!-- Error message -->
        <div v-if="errorMessage" class="form-error mb-4">
          {{ errorMessage }}
        </div>

        <!-- Current Password -->
        <div class="form-group">
          <label class="form-label" for="current-password">{{ t('auth.currentPassword') }}</label>
          <div class="input-with-icon input-with-action">
            <IconLock class="input-icon" aria-hidden="true" />
            <input
              id="current-password"
              v-model="currentPassword"
              :type="showCurrentPassword ? 'text' : 'password'"
              class="input"
              autocomplete="current-password"
              required
            />
            <button
              type="button"
              class="input-action"
              :aria-label="showCurrentPassword ? 'Hide password' : 'Show password'"
              @click="showCurrentPassword = !showCurrentPassword"
            >
              <IconEyeOff v-if="showCurrentPassword" />
              <IconEye v-else />
            </button>
          </div>
        </div>

        <!-- New Password -->
        <div class="form-group">
          <label class="form-label" for="new-password">{{ t('auth.newPassword') }}</label>
          <div class="input-with-icon input-with-action">
            <IconLock class="input-icon" aria-hidden="true" />
            <input
              id="new-password"
              v-model="newPassword"
              :type="showNewPassword ? 'text' : 'password'"
              class="input"
              :class="{ error: passwordTooShort }"
              autocomplete="new-password"
              minlength="8"
              required
            />
            <button
              type="button"
              class="input-action"
              :aria-label="showNewPassword ? 'Hide password' : 'Show password'"
              @click="showNewPassword = !showNewPassword"
            >
              <IconEyeOff v-if="showNewPassword" />
              <IconEye v-else />
            </button>
          </div>
          <span v-if="passwordTooShort" class="form-hint text-danger">
            {{ t('auth.passwordMinLength') }}
          </span>
        </div>

        <!-- Confirm Password -->
        <div class="form-group">
          <label class="form-label" for="confirm-password">{{ t('auth.confirmPassword') }}</label>
          <div class="input-with-icon input-with-action">
            <IconLock class="input-icon" aria-hidden="true" />
            <input
              id="confirm-password"
              v-model="confirmPassword"
              :type="showConfirmPassword ? 'text' : 'password'"
              class="input"
              :class="{ error: passwordMismatch }"
              autocomplete="new-password"
              required
            />
            <button
              type="button"
              class="input-action"
              :aria-label="showConfirmPassword ? 'Hide password' : 'Show password'"
              @click="showConfirmPassword = !showConfirmPassword"
            >
              <IconEyeOff v-if="showConfirmPassword" />
              <IconEye v-else />
            </button>
          </div>
          <span v-if="passwordMismatch" class="form-hint text-danger">
            {{ t('auth.passwordMismatch') }}
          </span>
        </div>
      </div>

      <div class="modal-footer">
        <button type="button" class="btn btn-secondary" @click="closeModal">
          {{ t('common.cancel') }}
        </button>
        <button type="submit" class="btn btn-primary" :disabled="!isValid || isSubmitting">
          {{ isSubmitting ? t('common.loading') : t('common.save') }}
        </button>
      </div>
    </form>
  </dialog>
</template>

<!--
  Uses global CSS:
  - ui/overlays/modal.css: .modal, .modal-sm, .modal-header, .modal-title, .modal-close, .modal-body, .modal-footer
  - ui/buttons.css: .btn, .btn-primary, .btn-secondary
  - ui/forms/inputs.css: .form-group, .form-label, .input, .input-with-icon, .input-with-action, .form-hint, .form-error
-->
