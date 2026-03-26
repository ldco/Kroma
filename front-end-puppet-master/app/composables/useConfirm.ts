/**
 * Confirm Dialog Composable
 *
 * Promise-based confirm dialog that replaces native browser confirm().
 * Uses native <dialog> element for accessibility and proper focus management.
 *
 * Usage:
 *   const { confirm } = useConfirm()
 *   const confirmed = await confirm('Are you sure?', { title: 'Delete Item' })
 *   if (confirmed) { ... }
 */
import type { ConfirmOptions, ConfirmState } from '~/types'

// Shared state across all component instances
const state = reactive<ConfirmState>({
  isOpen: false,
  message: '',
  options: {},
  resolve: null
})

export function useConfirm() {
  /**
   * Show a confirm dialog and wait for user response
   * @param message - The message to display
   * @param options - Optional configuration
   * @returns Promise that resolves to true (confirmed) or false (cancelled)
   */
  function confirm(message: string, options: ConfirmOptions = {}): Promise<boolean> {
    return new Promise(resolve => {
      state.message = message
      state.options = options
      state.resolve = resolve
      state.isOpen = true
    })
  }

  /**
   * Close the dialog with a result
   */
  function close(result: boolean) {
    if (state.resolve) {
      state.resolve(result)
    }
    state.isOpen = false
    state.message = ''
    state.options = {}
    state.resolve = null
  }

  return {
    // For consuming components
    confirm,
    // For ConfirmDialog component
    state: readonly(state),
    close
  }
}
