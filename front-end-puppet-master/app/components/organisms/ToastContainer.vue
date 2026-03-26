<script setup lang="ts">
/**
 * Toast Container Component
 *
 * Renders all active toast notifications.
 * Mount once in app.vue or admin layout - it reads from useToast() state.
 *
 * Uses global CSS: .toast-container, .toast, .toast-*, etc.
 */
import IconCheck from '~icons/tabler/check'
import IconX from '~icons/tabler/x'
import IconAlertTriangle from '~icons/tabler/alert-triangle'
import IconInfoCircle from '~icons/tabler/info-circle'

const { toasts, removeToast } = useToast()

const iconMap = {
  success: IconCheck,
  error: IconX,
  warning: IconAlertTriangle,
  info: IconInfoCircle
}
</script>

<template>
  <Teleport to="body">
    <div class="toast-container" role="region" aria-label="Notifications" aria-live="polite">
      <TransitionGroup name="toast">
        <div
          v-for="toast in toasts"
          :key="toast.id"
          class="toast"
          :class="`toast-${toast.type}`"
          role="alert"
        >
          <component :is="iconMap[toast.type]" class="toast-icon" />
          <span class="toast-content">{{ toast.message }}</span>
          <button
            type="button"
            class="toast-close"
            aria-label="Dismiss"
            @click="removeToast(toast.id)"
          >
            <IconX />
          </button>
        </div>
      </TransitionGroup>
    </div>
  </Teleport>
</template>

<!--
  Uses global CSS:
  - ui/overlays/toast.css: .toast-container, .toast, .toast-success, .toast-error,
    .toast-warning, .toast-info, .toast-icon, .toast-content, .toast-close,
    .toast-enter-active, .toast-leave-active, .toast-enter-from, .toast-leave-to
-->
