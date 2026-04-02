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
 *
 * Route Guard Precedence (deterministic order):
 * 1. This init guard runs first (00. prefix ensures load order)
 * 2. app-mode.global.ts runs second
 * 3. When unconfigured: /init and setup routes are explicitly allowed
 * 4. Setup routes: /init, /pm-init, /api/setup/*
 */
import config from '~/puppet-master.config'

// Setup routes that must be accessible when unconfigured
const SETUP_ROUTES = ['/init', '/pm-init']
const SETUP_API_PREFIXES = ['/api/setup/', '/api/i18n/']

/**
 * Check if path is a setup route that should be allowed when unconfigured
 */
function isSetupRoute(path: string): boolean {
  // Exact match for setup pages
  if (SETUP_ROUTES.includes(path)) {
    return true
  }
  // Check API prefixes
  for (const prefix of SETUP_API_PREFIXES) {
    if (path.startsWith(prefix)) {
      return true
    }
  }
  return false
}

/**
 * Normalize path by removing locale prefix for route matching
 * Supports all configured locales from i18n config
 */
function normalizePathForMatching(path: string): string {
  // Get configured locales from runtime config or fallback to config
  const { locale: currentLocale, locales: configuredLocales } = useI18n()
  const localeCodes = configuredLocales.value.map((l: any) => l.code)
  
  // Check if path starts with a locale prefix
  const segments = path.split('/').filter(Boolean)
  if (segments.length > 0 && localeCodes.includes(segments[0])) {
    // Remove locale prefix and reconstruct path
    return '/' + segments.slice(1).join('/')
  }
  return path
}

export default defineNuxtRouteMiddleware(to => {
  // Get pmMode from runtimeConfig (which reads PM_MODE env var first, then config)
  const { pmMode } = useRuntimeConfig().public

  // Normalize path for matching (handles locale-prefixed routes)
  const normalizedPath = normalizePathForMatching(to.path)

  // When CONFIGURED: block /init route
  if (pmMode !== 'unconfigured' && normalizedPath === '/init') {
    // Already configured, redirect away from init
    return navigateTo('/', { replace: true })
  }

  // When UNCONFIGURED: redirect to /init (but allow setup routes)
  if (pmMode === 'unconfigured' && normalizedPath !== '/init') {
    // Skip API routes and assets
    if (normalizedPath.startsWith('/api') || normalizedPath.startsWith('/_nuxt')) {
      return
    }
    // Allow setup routes (explicit exception for unconfigured state)
    if (isSetupRoute(normalizedPath)) {
      return
    }
    // Allow skip via query param (for CLI users who want to bypass)
    if (to.query.skip === 'true') {
      return
    }
    return navigateTo('/init', { replace: true })
  }
})
