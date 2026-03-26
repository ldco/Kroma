/**
 * Toast Notification Composable
 *
 * Displays temporary notification messages.
 * Replaces native alert() with styled, accessible toasts.
 *
 * Usage:
 *   const { toast } = useToast()
 *   toast.success('Item saved!')
 *   toast.error('Failed to delete')
 *   toast.warning('Connection slow')
 *   toast.info('New message received')
 */
import type { ToastType, Toast, ToastOptions } from '~/types'

// Auto-incrementing ID for toast management
let toastId = 0

// Shared state across all component instances
const toasts = ref<Toast[]>([])

export function useToast() {
  function addToast(type: ToastType, message: string, options: ToastOptions = {}) {
    const id = ++toastId
    const duration = options.duration ?? 4000

    toasts.value.push({ id, type, message, duration })

    // Auto-dismiss
    if (duration > 0) {
      setTimeout(() => {
        removeToast(id)
      }, duration)
    }

    return id
  }

  function removeToast(id: number) {
    const index = toasts.value.findIndex(t => t.id === id)
    if (index > -1) {
      toasts.value.splice(index, 1)
    }
  }

  const toast = {
    success: (message: string, options?: ToastOptions) => addToast('success', message, options),
    error: (message: string, options?: ToastOptions) =>
      addToast('error', message, { duration: 6000, ...options }),
    warning: (message: string, options?: ToastOptions) => addToast('warning', message, options),
    info: (message: string, options?: ToastOptions) => addToast('info', message, options)
  }

  return {
    toast,
    toasts: readonly(toasts),
    removeToast
  }
}
