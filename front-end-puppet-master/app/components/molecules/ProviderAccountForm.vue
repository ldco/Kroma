<script setup lang="ts">
/**
 * Provider Account Form Molecule
 *
 * Form for creating/editing provider accounts.
 * Supports multiple provider types (OpenAI, Stability, Anthropic, etc.)
 */
import type { ProviderAccount, ProviderType, CreateProviderInput } from '~/types/kroma'

const props = defineProps<{
  provider?: ProviderAccount | null
  mode?: 'create' | 'edit'
}>()

const emit = defineEmits<{
  submit: [data: CreateProviderInput]
  cancel: []
}>()

const mode = computed(() => props.mode || 'create')
const isEdit = computed(() => mode.value === 'edit')

// Form state
const formName = ref(props.provider?.name || '')
const formProviderType = ref<ProviderType>(props.provider?.provider_type || 'openai')
const formApiKey = ref('')

// Provider type options
const providerTypeOptions = [
  { value: 'openai', label: 'OpenAI', description: 'GPT-4, DALL-E 3, etc.' },
  { value: 'stability', label: 'Stability AI', description: 'Stable Diffusion, etc.' },
  { value: 'anthropic', label: 'Anthropic', description: 'Claude models' },
  { value: 'midjourney', label: 'Midjourney', description: 'Midjourney API' },
  { value: 'other', label: 'Other', description: 'Custom provider' }
]

// Validation
const errors = ref<Record<string, string>>({})

function validate() {
  errors.value = {}

  if (!formName.value.trim()) {
    errors.value.name = 'Name is required'
  }

  if (!formApiKey.value && !isEdit.value) {
    errors.value.apiKey = 'API key is required'
  } else if (!isEdit.value && formApiKey.value.length < 20) {
    errors.value.apiKey = 'API key must be at least 20 characters'
  }

  return Object.keys(errors.value).length === 0
}

function handleSubmit() {
  if (!validate()) {
    return
  }

  const input: CreateProviderInput = {
    name: formName.value.trim(),
    provider_type: formProviderType.value,
    api_key: formApiKey.value
  }

  emit('submit', input)
}

function handleCancel() {
  emit('cancel')
}
</script>

<template>
  <!-- Uses global classes from ui/forms/index.css -->
  <form class="provider-account-form" @submit.prevent="handleSubmit">
    <div class="form-group">
      <label for="provider-name" class="form-label">
        Provider Name
        <span class="required">*</span>
      </label>
      <input
        id="provider-name"
        v-model="formName"
        type="text"
        class="form-input"
        :class="{ 'form-input--error': errors.name }"
        placeholder="My OpenAI Account"
        :disabled="isEdit"
      />
      <p v-if="errors.name" class="form-error">{{ errors.name }}</p>
      <p v-if="isEdit" class="form-hint">
        <Icon name="lucide:info" class="icon-xs" />
        Provider name cannot be changed. Create a new provider if you need a different name.
      </p>
    </div>

    <div class="form-group">
      <label class="form-label">
        Provider Type
        <span class="required">*</span>
      </label>
      <div class="provider-type-grid">
        <label
          v-for="option in providerTypeOptions"
          :key="option.value"
          class="provider-type-option"
          :class="{ 'provider-type-option--selected': formProviderType === option.value }"
        >
          <input
            v-model="formProviderType"
            type="radio"
            name="provider-type"
            :value="option.value"
            class="provider-type-option__radio"
          />
          <div class="provider-type-option__content">
            <span class="provider-type-option__label">{{ option.label }}</span>
            <span class="provider-type-option__description">{{ option.description }}</span>
          </div>
        </label>
      </div>
      <p v-if="errors.providerType" class="form-error">{{ errors.providerType }}</p>
    </div>

    <div class="form-group">
      <AtomsApiKeyInput
        v-model="formApiKey"
        label="API Key"
        :required="!isEdit"
        :show-strength="true"
        :show-copy="true"
        placeholder="sk-..."
      />
      <p v-if="errors.apiKey" class="form-error">{{ errors.apiKey }}</p>
    </div>

    <div class="form-actions">
      <button type="button" class="btn btn-ghost" @click="handleCancel">
        Cancel
      </button>
      <button type="submit" class="btn btn-primary">
        {{ isEdit ? 'Update Provider' : 'Add Provider' }}
      </button>
    </div>
  </form>
</template>

<!-- No scoped styles - uses ui/providers/provider-account-form.css -->
