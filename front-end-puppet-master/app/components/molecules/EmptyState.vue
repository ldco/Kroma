<script setup lang="ts">
/**
 * Empty State Component (HIGH-07)
 *
 * Displays a friendly message when content is empty.
 * Provides context and optional action to help users.
 *
 * Usage:
 * <EmptyState
 *   icon="inbox"
 *   title="No messages yet"
 *   description="When customers contact you, their messages will appear here."
 *   actionLabel="View help"
 *   @action="openHelp"
 * />
 */
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{
    icon?: string
    title: string
    description?: string
    actionLabel?: string
    actionHref?: string
    variant?: 'default' | 'compact' | 'large'
    illustration?: 'empty-box' | 'search' | 'error' | 'success' | 'custom'
  }>(),
  {
    variant: 'default',
    illustration: 'empty-box'
  }
)

const emit = defineEmits<{
  action: []
}>()

const iconSvgs = {
  'empty-box': `<svg viewBox="0 0 100 100" fill="none" xmlns="http://www.w3.org/2000/svg">
    <rect x="20" y="35" width="60" height="45" rx="3" stroke="currentColor" stroke-width="2" fill="none"/>
    <path d="M20 45 L50 30 L80 45" stroke="currentColor" stroke-width="2" fill="none"/>
    <line x1="50" y1="30" x2="50" y2="55" stroke="currentColor" stroke-width="2"/>
    <circle cx="50" cy="62" r="8" stroke="currentColor" stroke-width="2" fill="none"/>
    <path d="M47 62 L50 65 L55 59" stroke="currentColor" stroke-width="2" fill="none"/>
  </svg>`,
  search: `<svg viewBox="0 0 100 100" fill="none" xmlns="http://www.w3.org/2000/svg">
    <circle cx="42" cy="42" r="22" stroke="currentColor" stroke-width="2" fill="none"/>
    <line x1="58" y1="58" x2="75" y2="75" stroke="currentColor" stroke-width="3" stroke-linecap="round"/>
    <path d="M35 42 Q42 35 49 42" stroke="currentColor" stroke-width="2" fill="none" stroke-linecap="round"/>
  </svg>`,
  error: `<svg viewBox="0 0 100 100" fill="none" xmlns="http://www.w3.org/2000/svg">
    <circle cx="50" cy="50" r="35" stroke="currentColor" stroke-width="2" fill="none"/>
    <line x1="50" y1="30" x2="50" y2="55" stroke="currentColor" stroke-width="3" stroke-linecap="round"/>
    <circle cx="50" cy="67" r="3" fill="currentColor"/>
  </svg>`,
  success: `<svg viewBox="0 0 100 100" fill="none" xmlns="http://www.w3.org/2000/svg">
    <circle cx="50" cy="50" r="35" stroke="currentColor" stroke-width="2" fill="none"/>
    <path d="M35 50 L45 60 L65 40" stroke="currentColor" stroke-width="3" fill="none" stroke-linecap="round" stroke-linejoin="round"/>
  </svg>`
}

const illustrationSvg = computed(() => {
  return iconSvgs[props.illustration as keyof typeof iconSvgs] || iconSvgs['empty-box']
})

function handleAction() {
  emit('action')
}
</script>

<template>
  <div class="empty-state" :class="[`empty-state--${variant}`]">
    <!-- eslint-disable vue/no-v-html -- Safe: hardcoded internal SVGs only -->
    <div
      v-if="illustration !== 'custom'"
      class="empty-state__illustration"
      v-html="illustrationSvg"
    />
    <!-- eslint-enable vue/no-v-html -->
    <slot v-else name="illustration" />

    <h3 class="empty-state__title">{{ title }}</h3>

    <p v-if="description" class="empty-state__description">
      {{ description }}
    </p>

    <slot name="content" />

    <div v-if="actionLabel" class="empty-state__action">
      <a v-if="actionHref" :href="actionHref" class="btn btn--primary">
        {{ actionLabel }}
      </a>
      <button v-else type="button" class="btn btn--primary" @click="handleAction">
        {{ actionLabel }}
      </button>
    </div>

    <slot name="footer" />
  </div>
</template>
