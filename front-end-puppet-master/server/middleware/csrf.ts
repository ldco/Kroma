/**
 * CSRF Validation Middleware
 *
 * Validates CSRF token on state-changing requests (POST, PUT, DELETE, PATCH).
 * Skips validation for:
 * - Safe methods (GET, HEAD, OPTIONS)
 * - Non-API routes
 * - Login endpoint (no session yet)
 * - Logout endpoint (destroying session anyway)
 * - Public endpoints that don't require auth
 * - Setup lifecycle endpoints (unauthenticated initial configuration)
 */
import { validateCsrfToken } from '../utils/csrf'

// Routes that are exempt from CSRF validation
const CSRF_EXEMPT_ROUTES = [
  '/api/auth/login', // No session yet
  '/api/auth/logout', // Destroying session
  '/api/contact/submit', // Public form, has its own rate limiting
  '/api/health', // Health check
  // Setup lifecycle endpoints - unauthenticated initial configuration
  // These are protected by setup-guard.ts (pmMode check) instead
  '/api/setup/config', // POST/DELETE setup config
  '/api/setup/import-zip' // POST/DELETE import zip
]

// Methods that don't change state
const SAFE_METHODS = ['GET', 'HEAD', 'OPTIONS']

export default defineEventHandler(event => {
  const method = event.method.toUpperCase()
  const path = getRequestURL(event).pathname

  // Skip for safe methods
  if (SAFE_METHODS.includes(method)) {
    return
  }

  // Skip for non-API routes
  if (!path.startsWith('/api/')) {
    return
  }

  // Skip for exempt routes
  if (CSRF_EXEMPT_ROUTES.some(route => path.startsWith(route))) {
    return
  }

  // Validate CSRF token
  if (!validateCsrfToken(event)) {
    throw createError({
      statusCode: 403,
      message: 'Invalid or missing CSRF token'
    })
  }
})
