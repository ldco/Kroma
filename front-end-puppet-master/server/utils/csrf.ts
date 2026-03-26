/**
 * CSRF Protection Utilities
 *
 * Implements double-submit cookie pattern for CSRF protection.
 * Token is stored in HTTP-only cookie and must be sent in X-CSRF-Token header.
 */
import { randomBytes, timingSafeEqual } from 'crypto'
import type { H3Event } from 'h3'

const CSRF_COOKIE_NAME = 'pm-csrf'
const CSRF_HEADER_NAME = 'x-csrf-token'
const TOKEN_LENGTH = 32

/**
 * Generate a secure random CSRF token
 */
export function generateCsrfToken(): string {
  return randomBytes(TOKEN_LENGTH).toString('hex')
}

/**
 * Set CSRF token cookie
 * Cookie is HTTP-only to prevent XSS access
 */
export function setCsrfCookie(event: H3Event, token: string): void {
  setCookie(event, CSRF_COOKIE_NAME, token, {
    httpOnly: true,
    secure: process.env.NODE_ENV === 'production',
    sameSite: 'strict',
    path: '/'
  })
}

/**
 * Get CSRF token from cookie
 */
export function getCsrfToken(event: H3Event): string | undefined {
  return getCookie(event, CSRF_COOKIE_NAME)
}

/**
 * Delete CSRF cookie (on logout)
 */
export function deleteCsrfCookie(event: H3Event): void {
  deleteCookie(event, CSRF_COOKIE_NAME, { path: '/' })
}

/**
 * Validate CSRF token from header against cookie
 * Uses timing-safe comparison to prevent timing attacks
 */
export function validateCsrfToken(event: H3Event): boolean {
  const cookieToken = getCookie(event, CSRF_COOKIE_NAME)
  const headerToken = getHeader(event, CSRF_HEADER_NAME)

  // Both must be present
  if (!cookieToken || !headerToken) {
    return false
  }

  // Must be same length (prevents timing attacks on length)
  if (cookieToken.length !== headerToken.length) {
    return false
  }

  try {
    // Timing-safe comparison
    return timingSafeEqual(Buffer.from(cookieToken, 'utf8'), Buffer.from(headerToken, 'utf8'))
  } catch {
    return false
  }
}

/**
 * Get the CSRF header name for client-side use
 */
export function getCsrfHeaderName(): string {
  return CSRF_HEADER_NAME
}

/**
 * Get the CSRF cookie name
 */
export function getCsrfCookieName(): string {
  return CSRF_COOKIE_NAME
}
