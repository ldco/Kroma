/**
 * Toast Notification Types
 */

export type ToastType = 'success' | 'error' | 'warning' | 'info'

export interface Toast {
  id: number
  type: ToastType
  message: string
  duration: number
}

export interface ToastOptions {
  /** Duration in ms before auto-dismiss (default: 4000, 0 = no auto-dismiss) */
  duration?: number
}
