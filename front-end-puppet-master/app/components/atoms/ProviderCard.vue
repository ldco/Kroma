<script setup lang="ts">
/**
 * Provider Card Atom
 *
 * Displays provider account information.
 * Used in provider list.
 */
import type { ProviderAccount } from '~/types/kroma'

const props = defineProps<{
  provider: ProviderAccount
}>()

const emit = defineEmits<{
  edit: [id: string]
  delete: [id: string]
  test: [id: string]
}>()

// Provider type display names
const providerTypeNames: Record<string, string> = {
  openai: 'OpenAI',
  stability: 'Stability AI',
  anthropic: 'Anthropic',
  midjourney: 'Midjourney',
  other: 'Other'
}

// Provider type icons
const providerTypeIcons: Record<string, string> = {
  openai: 'lucide:bot',
  stability: 'lucide:image',
  anthropic: 'lucide:brain',
  midjourney: 'lucide:palette',
  other: 'lucide:key'
}

const providerTypeName = computed(() => {
  return providerTypeNames[props.provider.provider_type] || props.provider.provider_type
})

const providerTypeIcon = computed(() => {
  return providerTypeIcons[props.provider.provider_type] || 'lucide:key'
})

const isHealthy = computed(() => {
  return props.provider.health_status.status === 'healthy'
})

const isUnhealthy = computed(() => {
  return props.provider.health_status.status === 'unhealthy'
})

const lastChecked = computed(() => {
  if (!props.provider.health_status.last_checked) return 'Never'
  return new Date(props.provider.health_status.last_checked).toLocaleDateString()
})
</script>

<template>
  <!-- Uses global classes from ui/providers/provider-card.css -->
  <div class="provider-card" :class="{ 'provider-card--unhealthy': isUnhealthy }">
    <div class="provider-card__header">
      <div class="provider-card__icon">
        <Icon :name="providerTypeIcon" class="icon-lg" />
      </div>
      <div class="provider-card__info">
        <h3 class="provider-card__title">{{ provider.name }}</h3>
        <span class="provider-card__type">{{ providerTypeName }}</span>
      </div>
      <div class="provider-card__status">
        <span
          class="status-badge"
          :class="{
            'status-badge--healthy': isHealthy,
            'status-badge--unhealthy': isUnhealthy,
            'status-badge--unknown': !isHealthy && !isUnhealthy
          }"
        >
          {{ provider.health_status.status }}
        </span>
      </div>
    </div>

    <div class="provider-card__body">
      <div class="provider-card__meta">
        <span class="provider-card__label">API Key:</span>
        <span class="provider-card__value">••••••••{{ provider.api_key_encrypted.slice(-4) }}</span>
      </div>
      <div class="provider-card__meta">
        <span class="provider-card__label">Last checked:</span>
        <span class="provider-card__value">{{ lastChecked }}</span>
      </div>
      <div v-if="provider.health_status.error_message" class="provider-card__error">
        <Icon name="lucide:alert-circle" class="icon-sm" />
        <span>{{ provider.health_status.error_message }}</span>
      </div>
    </div>

    <div class="provider-card__actions">
      <button
        type="button"
        class="provider-card__action"
        @click.stop="emit('test', provider.id)"
        aria-label="Test connection"
      >
        <Icon name="lucide:activity" class="icon-sm" />
        <span>Test</span>
      </button>
      <button
        type="button"
        class="provider-card__action"
        @click.stop="emit('edit', provider.id)"
        aria-label="Edit provider"
      >
        <Icon name="lucide:edit" class="icon-sm" />
        <span>Edit</span>
      </button>
      <button
        type="button"
        class="provider-card__action provider-card__action--danger"
        @click.stop="emit('delete', provider.id)"
        aria-label="Delete provider"
      >
        <Icon name="lucide:trash-2" class="icon-sm" />
        <span>Delete</span>
      </button>
    </div>
  </div>
</template>

<!-- No scoped styles - uses ui/providers/provider-card.css -->
