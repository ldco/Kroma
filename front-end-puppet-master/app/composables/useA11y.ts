/**
 * Accessibility Testing Composable (HIGH-05)
 *
 * Provides runtime accessibility checking in development mode.
 * Uses axe-core for WCAG 2.1 AA compliance testing.
 *
 * Usage:
 * // In a component or page
 * const { checkA11y, violations } = useA11y()
 * onMounted(() => checkA11y())
 *
 * Installation:
 * npm install -D axe-core
 */
import { ref, onMounted } from 'vue'

interface A11yViolation {
  id: string
  impact: 'minor' | 'moderate' | 'serious' | 'critical'
  description: string
  help: string
  helpUrl: string
  nodes: {
    html: string
    target: string[]
    failureSummary: string
  }[]
}

export function useA11y(options?: { runOnMount?: boolean; element?: HTMLElement | string }) {
  const violations = ref<A11yViolation[]>([])
  const isChecking = ref(false)
  const lastCheckedAt = ref<Date | null>(null)

  /**
   * Run accessibility check on the specified element
   */
  async function checkA11y(element?: HTMLElement | string) {
    // Only run in development
    if (import.meta.env.PROD) {
      console.warn('[A11y] Accessibility checks are disabled in production')
      return
    }

    // Dynamically import axe-core (only in development)
    let axe
    try {
      axe = await import('axe-core')
    } catch {
      console.warn('[A11y] axe-core not installed. Run: npm install -D axe-core')
      return
    }

    isChecking.value = true
    violations.value = []

    try {
      const target = element || options?.element || document.body
      const context = typeof target === 'string' ? document.querySelector(target) : target

      if (!context) {
        console.warn('[A11y] Target element not found')
        return
      }

      const results = await axe.default.run(context as HTMLElement, {
        runOnly: {
          type: 'tag',
          values: ['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa']
        }
      })

      violations.value = results.violations.map(v => ({
        id: v.id,
        impact: v.impact as A11yViolation['impact'],
        description: v.description,
        help: v.help,
        helpUrl: v.helpUrl,
        nodes: v.nodes.map(n => ({
          html: n.html,
          target: n.target as string[],
          failureSummary: n.failureSummary || ''
        }))
      }))

      lastCheckedAt.value = new Date()

      // Log results in development
      if (violations.value.length > 0) {
        console.group(`[A11y] Found ${violations.value.length} accessibility issues`)
        violations.value.forEach(v => {
          const impactColors = {
            critical: 'color: #d93025; font-weight: bold',
            serious: 'color: #ea8600',
            moderate: 'color: #f9ab00',
            minor: 'color: #188038'
          }
          console.groupCollapsed(`%c[${v.impact.toUpperCase()}] ${v.help}`, impactColors[v.impact])
          console.log('Description:', v.description)
          console.log('Help:', v.helpUrl)
          v.nodes.forEach((n, i) => {
            console.log(`Node ${i + 1}:`, n.target.join(', '))
            console.log('HTML:', n.html)
            console.log('Fix:', n.failureSummary)
          })
          console.groupEnd()
        })
        console.groupEnd()
      } else {
        console.log('[A11y] No accessibility issues found!')
      }

      return results
    } catch (error) {
      console.error('[A11y] Error running accessibility check:', error)
    } finally {
      isChecking.value = false
    }
  }

  // Run on mount if requested
  if (options?.runOnMount) {
    onMounted(() => {
      // Delay check to ensure DOM is ready
      setTimeout(() => checkA11y(), 500)
    })
  }

  return {
    violations,
    isChecking,
    lastCheckedAt,
    checkA11y
  }
}

/**
 * Accessibility overlay component for development
 * Shows a floating panel with violation count
 */
export function useA11yOverlay() {
  const { violations, checkA11y, isChecking } = useA11y()
  const isVisible = ref(false)

  function toggle() {
    isVisible.value = !isVisible.value
  }

  function show() {
    isVisible.value = true
    checkA11y()
  }

  function hide() {
    isVisible.value = false
  }

  return {
    violations,
    isChecking,
    isVisible,
    toggle,
    show,
    hide,
    refresh: checkA11y
  }
}
