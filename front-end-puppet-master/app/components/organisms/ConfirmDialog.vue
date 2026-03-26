<script setup lang="ts">
/**
 * Confirm Dialog Component
 *
 * Global confirm dialog using native <dialog> element.
 * Provides proper accessibility: focus trapping, ESC to close, ARIA attributes.
 *
 * Mount once in app.vue or admin layout - it reads from useConfirm() state.
 *
 * Uses global CSS: .modal, .modal-header, .modal-body, .modal-footer, .btn
 */
import IconX from '~icons/tabler/x'

const { t } = useI18n()
const { state, close } = useConfirm()

const dialogRef = ref<HTMLDialogElement | null>(null)

// Open/close dialog when state changes
watch(
  () => state.isOpen,
  isOpen => {
    if (isOpen) {
      dialogRef.value?.showModal()
    } else {
      dialogRef.value?.close()
    }
  }
)

// Handle ESC key (native dialog handles this, but we need to call close())
function handleCancel(e: Event) {
  e.preventDefault()
  close(false)
}

// Handle backdrop click
function handleBackdropClick(e: MouseEvent) {
  const rect = dialogRef.value?.getBoundingClientRect()
  if (!rect) return

  // Click is on backdrop if outside dialog content
  const isOutside =
    e.clientX < rect.left ||
    e.clientX > rect.right ||
    e.clientY < rect.top ||
    e.clientY > rect.bottom

  if (isOutside) {
    close(false)
  }
}

const title = computed(() => state.options.title || t('common.confirm'))
const confirmText = computed(() => state.options.confirmText || t('common.confirm'))
const cancelText = computed(() => state.options.cancelText || t('common.cancel'))
const variant = computed(() => state.options.variant || 'primary')
</script>

<template>
  <dialog
    ref="dialogRef"
    class="confirm-dialog modal modal-sm"
    aria-labelledby="confirm-title"
    aria-describedby="confirm-message"
    @cancel="handleCancel"
    @click="handleBackdropClick"
  >
    <div class="modal-header">
      <h2 id="confirm-title" class="modal-title">{{ title }}</h2>
      <button type="button" class="modal-close" :aria-label="cancelText" @click="close(false)">
        <IconX />
      </button>
    </div>

    <div class="modal-body">
      <p id="confirm-message">{{ state.message }}</p>
    </div>

    <div class="modal-footer">
      <button type="button" class="btn btn-secondary" @click="close(false)">
        {{ cancelText }}
      </button>
      <button
        type="button"
        class="btn"
        :class="variant === 'danger' ? 'btn-danger' : 'btn-primary'"
        @click="close(true)"
      >
        {{ confirmText }}
      </button>
    </div>
  </dialog>
</template>

<!--
  Uses global CSS:
  - ui/overlays/modal.css: .modal, .modal-sm, .modal-header, .modal-title, .modal-close, .modal-body, .modal-footer
  - ui/buttons.css: .btn, .btn-primary, .btn-secondary, .btn-danger

  Native <dialog> provides:
  - ::backdrop pseudo-element (styled below)
  - Focus trapping
  - ESC key to close
  - Proper stacking context
-->
