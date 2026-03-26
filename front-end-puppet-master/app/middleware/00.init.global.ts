/**
 * Init Check Middleware
 *
 * Controls access to /init based on pmMode:
 * - unconfigured: Redirect to /init for configuration
 * - build/develop: Block /init, redirect to home
 *
 * Note: Mode selection (build/develop) happens in CLI or Claude chat,
 * NOT in the browser. The /init page is only for BUILD mode configuration.
 *
 * pmMode priority: PM_MODE env var > puppet-master.config.ts > 'unconfigured'
 */
export default defineNuxtRouteMiddleware(to => {
  // Get pmMode from runtimeConfig (which reads PM_MODE env var first, then config)
  const { pmMode } = useRuntimeConfig().public

  // When CONFIGURED: block /init route
  if (pmMode !== 'unconfigured' && to.path === '/init') {
    // Already configured, redirect away from init
    return navigateTo('/', { replace: true })
  }

  // When UNCONFIGURED: redirect to /init
  if (pmMode === 'unconfigured' && to.path !== '/init') {
    // Skip API routes and assets
    if (to.path.startsWith('/api') || to.path.startsWith('/_nuxt')) {
      return
    }
    // Allow skip via query param (for CLI users who want to bypass)
    if (to.query.skip === 'true') {
      return
    }
    return navigateTo('/init', { replace: true })
  }
})
