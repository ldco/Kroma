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
 */
import config from '~/puppet-master.config'

export default defineNuxtRouteMiddleware(to => {
  const { entities, admin } = config

  // ═══════════════════════════════════════════════════════════════════════════
  // APP-ONLY: No public website, redirect root to admin login
  // (entities.website: false, entities.app: true)
  // ═══════════════════════════════════════════════════════════════════════════
  if (!entities.website && entities.app) {
    // Allow admin routes
    if (to.path.startsWith('/admin')) {
      return // continue to admin
    }

    // Allow app routes (user features)
    if (to.path.startsWith('/app')) {
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
    if (to.path.startsWith('/admin')) {
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
