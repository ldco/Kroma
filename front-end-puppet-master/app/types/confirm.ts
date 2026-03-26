/**
 * Confirm Dialog Types
 */

export interface ConfirmOptions {
  /** Dialog title (default: 'Confirm') */
  title?: string
  /** Confirm button text (default: 'Confirm') */
  confirmText?: string
  /** Cancel button text (default: 'Cancel') */
  cancelText?: string
  /** Confirm button variant: 'primary' | 'danger' (default: 'primary') */
  variant?: 'primary' | 'danger'
}

export interface ConfirmState {
  isOpen: boolean
  message: string
  options: ConfirmOptions
  resolve: ((value: boolean) => void) | null
}
