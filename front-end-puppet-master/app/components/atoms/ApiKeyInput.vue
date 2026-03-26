<script setup lang="ts">
/**
 * API Key Input Atom
 *
 * Secure password-style input for API keys.
 * Features:
 * - Show/hide toggle
 * - Strength indicator (optional)
 * - Copy to clipboard (optional)
 */
const props = defineProps<{
  modelValue: string
  placeholder?: string
  showStrength?: boolean
  showCopy?: boolean
  label?: string
  required?: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const showKey = ref(false)
const copied = ref(false)

const inputType = computed(() => (showKey.value ? 'text' : 'password'))

const strength = computed(() => {
  const key = props.modelValue
  if (!key) return 0
  if (key.length < 20) return 1
  if (key.length < 40) return 2
  return 3
})

const strengthLabel = computed(() => {
  switch (strength.value) {
    case 1:
      return 'Weak'
    case 2:
      return 'Medium'
    case 3:
      return 'Strong'
    default:
      return ''
  }
})

const strengthClass = computed(() => {
  switch (strength.value) {
    case 1:
      return 'strength-bar--weak'
    case 2:
      return 'strength-bar--medium'
    case 3:
      return 'strength-bar--strong'
    default:
      return ''
  }
})

function toggleShow() {
  showKey.value = !showKey.value
}

async function copyToClipboard() {
  if (!props.modelValue) return

  try {
    await navigator.clipboard.writeText(props.modelValue)
    copied.value = true
    setTimeout(() => {
      copied.value = false
    }, 2000)
  } catch (error) {
    console.error('Failed to copy to clipboard:', error)
  }
}
</script>

<template>
  <!-- Uses global classes from ui/forms/inputs.css -->
  <div class="api-key-input">
    <label v-if="label" class="api-key-input__label">
      {{ label }}
      <span v-if="required" class="required">*</span>
    </label>

    <div class="api-key-input__wrapper">
      <input
        :type="inputType"
        :value="modelValue"
        :placeholder="placeholder"
        class="api-key-input__field"
        @input="emit('update:modelValue', ($event.target as HTMLInputElement).value)"
      />

      <div class="api-key-input__actions">
        <button
          v-if="showCopy"
          type="button"
          class="api-key-input__action"
          @click="copyToClipboard"
          :aria-label="copied ? 'Copied' : 'Copy to clipboard'"
          :title="copied ? 'Copied' : 'Copy to clipboard'"
        >
          <Icon v-if="!copied" name="lucide:copy" class="icon-sm" />
          <Icon v-else name="lucide:check" class="icon-sm" />
        </button>

        <button
          type="button"
          class="api-key-input__action"
          @click="toggleShow"
          :aria-label="showKey ? 'Hide API key' : 'Show API key'"
          :title="showKey ? 'Hide API key' : 'Show API key'"
        >
          <Icon v-if="showKey" name="lucide:eye-off" class="icon-sm" />
          <Icon v-else name="lucide:eye" class="icon-sm" />
        </button>
      </div>
    </div>

    <div v-if="showStrength && modelValue" class="api-key-input__strength">
      <div class="strength-bars">
        <div class="strength-bar" :class="strengthClass" :style="{ width: strength * 33.33 + '%' }" />
      </div>
      <span class="strength-label">{{ strengthLabel }}</span>
    </div>

    <p class="api-key-input__hint">
      <Icon name="lucide:info" class="icon-xs" />
      Your API key will be encrypted and stored securely
    </p>
  </div>
</template>

<!-- No scoped styles - uses ui/forms/inputs.css -->
