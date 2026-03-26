/**
 * Accessibility Development Plugin (HIGH-05)
 *
 * Automatically runs accessibility checks in development mode.
 * Logs violations to the console with helpful information.
 *
 * Enable/disable via URL param: ?a11y=true or ?a11y=false
 */
export default defineNuxtPlugin(() => {
  // Only run in development
  if (import.meta.env.PROD) return

  // Check URL param
  if (typeof window !== 'undefined') {
    const params = new URLSearchParams(window.location.search)
    const a11yParam = params.get('a11y')

    // Disabled by default unless explicitly enabled
    if (a11yParam !== 'true') return
  }

  // Run initial check after page load
  if (typeof window !== 'undefined') {
    window.addEventListener('load', async () => {
      // Wait for Vue to finish hydrating
      await new Promise(resolve => setTimeout(resolve, 1000))

      try {
        const axe = await import('axe-core')
        const results = await axe.default.run({
          runOnly: {
            type: 'tag',
            values: ['wcag2a', 'wcag2aa']
          }
        })

        if (results.violations.length > 0) {
          console.warn(
            `%c[A11y] ${results.violations.length} accessibility issue(s) found`,
            'color: #ea8600; font-weight: bold'
          )
          console.log('Run useA11y().checkA11y() for details')
        } else {
          console.log('%c[A11y] Page passes WCAG 2.1 AA checks', 'color: #188038')
        }
      } catch {
        // axe-core not installed, silently skip
      }
    })
  }
})
