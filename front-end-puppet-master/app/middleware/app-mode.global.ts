/**
 * App Mode Middleware
 *
 * Handles routing based on the configured entity structure:
 *
 * - App only (no website):  Redirect / → /admin/login (no public website)
 * - Website + App:          Normal routing, /login available
 * - Website + Admin:        Normal routing, admin at /admin (hidden)
 * - Website only:           Block /admin/* routes (no admin)
 *
 * This middleware runs on every route change (global).
 *
 * Route Guard Precedence:
 * 1. 00.init.global.ts runs first (init/setup checks)
 * 2. This app-mode guard runs second (entity-based routing)
 * 3. Locale-prefixed paths are normalized before matching
 * 4. When pmMode is 'unconfigured', setup routes are always allowed (no redirect loop)
 */
import config from '~/puppet-master.config'

// Setup routes that must always be accessible when pmMode is 'unconfigured'
// This prevents redirect loops during initial setup / first-run scenario
const SETUP_ROUTES = ['/init', '/pm-init']

/**
 * Check if path is a setup route that should be allowed during unconfigured state
 */
function isSetupRoute(path: string): boolean {
  return SETUP_ROUTES.includes(path)
}

/**
 * Normalize path by removing locale prefix for route matching
 * Supports all configured locales from i18n config
 */
function normalizePathForMatching(path: string): string {
  // Get configured locales from i18n
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
  const { entities, admin } = config

  // Get pmMode to check if app is in unconfigured (setup) state
  const { pmMode } = useRuntimeConfig().public

  // Normalize path for matching (handles locale-prefixed routes like /en/app, /fr/admin)
  const normalizedPath = normalizePathForMatching(to.path)

  // ═══════════════════════════════════════════════════════════════════════════
  // UNCONFIGURED MODE: Always allow setup routes (prevent redirect loops)
  // This check runs BEFORE entity-based routing to ensure /init is never
  // redirected to /admin/login during first-run setup
  // ═══════════════════════════════════════════════════════════════════════════
  if (pmMode === 'unconfigured' && isSetupRoute(normalizedPath)) {
    return // Allow access to setup routes
  }

  // ═══════════════════════════════════════════════════════════════════════════
  // APP-ONLY: No public website, redirect root to admin login
  // (entities.website: false, entities.app: true)
  // ═══════════════════════════════════════════════════════════════════════════
  if (!entities.website && entities.app) {
    // Allow admin routes (including locale-prefixed)
    if (normalizedPath.startsWith('/admin')) {
      return // continue to admin
    }

    // Allow app routes (user features, including locale-prefixed)
    if (normalizedPath.startsWith('/app')) {
      return // continue to app
    }

    // Redirect everything else to admin login
    return navigateTo('/admin/login', { replace: true })
  }

  // ═══════════════════════════════════════════════════════════════════════════
  // WEBSITE-ONLY: No admin, block admin routes
  // (entities.website: true, entities.app: false, admin.enabled: false)
  // ═══════════════════════════════════════════════════════════════════════════
  if (entities.website && !entities.app && !admin.enabled) {
    if (normalizedPath.startsWith('/admin')) {
      // Redirect admin attempts to home
      return navigateTo('/', { replace: true })
    }
  }

  // ═══════════════════════════════════════════════════════════════════════════
  // WEBSITE + APP: Website with visible login button
  // (entities.website: true, entities.app: true)
  // ═══════════════════════════════════════════════════════════════════════════
  // No special routing needed - /login page will be available

  // ═══════════════════════════════════════════════════════════════════════════
  // WEBSITE + ADMIN: Website with hidden admin (default)
  // (entities.website: true, admin.enabled: true)
  // ═══════════════════════════════════════════════════════════════════════════
  // No special routing needed - admin at /admin works as expected
})
