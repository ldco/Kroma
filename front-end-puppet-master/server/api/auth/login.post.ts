/**
 * Login API Endpoint
 *
 * POST /api/auth/login
 * Body: { email: string, password: string, rememberMe?: boolean }
 * Returns:
 *   - If 2FA disabled: { success: true, user: { id, email, name, role }, csrfToken: string }
 *   - If 2FA enabled: { success: true, requires2fa: true, message: string }
 *
 * Security:
 * - Rate limited: 5 attempts per 15 minutes per IP
 * - Account lockout: 5 failed attempts = 30 min lockout
 * - Password hashed with scrypt
 * - CSRF token generated on success
 * - 2FA support with TOTP and backup codes
 */
import { eq } from 'drizzle-orm'
import { randomBytes } from 'crypto'
import { useDatabase, schema } from '../../database/client'
import { verifyPassword, generateSessionId } from '../../utils/password'
import { generateCsrfToken, setCsrfCookie } from '../../utils/csrf'
import { loginRateLimiter, getClientIp } from '../../utils/rateLimit'
import {
  checkAccountLocked,
  recordFailedAttempt,
  resetFailedAttempts
} from '../../utils/accountLockout'
import { audit } from '../../utils/audit'
import { loginSchema } from '../../utils/validation'
import { pending2faVerifications } from '../../utils/ephemeral-store'
import config from '../../../app/puppet-master.config'

export default defineEventHandler(async event => {
  // Rate limit check - 5 attempts per 15 minutes per IP
  // Skip if NUXT_PUBLIC_DISABLE_RATE_LIMIT is set (for testing)
  const disableRateLimit = process.env.NUXT_PUBLIC_DISABLE_RATE_LIMIT === 'true'

  if (!disableRateLimit) {
    const clientIp = getClientIp(event)

    if (!loginRateLimiter.checkRateLimit(clientIp)) {
      throw createError({
        statusCode: 429,
        message: 'Too many login attempts. Please try again in 15 minutes.',
        data: {
          retryAfter: 15 * 60, // seconds
          remaining: 0
        }
      })
    }
  }

  const body = await readBody(event)

  // Validate input
  const result = loginSchema.safeParse(body)
  if (!result.success) {
    throw createError({
      statusCode: 400,
      message: 'Invalid email or password format'
    })
  }

  const { email, password, rememberMe } = result.data
  const db = useDatabase()

  // Find user by email (include 2FA status)
  const user = db.select().from(schema.users).where(eq(schema.users.email, email)).get()

  if (!user) {
    // Don't reveal that user doesn't exist - but log it
    await audit.loginFailed(event, email, 'user_not_found')
    throw createError({
      statusCode: 401,
      message: 'Invalid email or password'
    })
  }

  // Check if account is locked
  const lockoutStatus = await checkAccountLocked(user.id)
  if (lockoutStatus.isLocked) {
    await audit.loginFailed(event, email, 'account_locked', user.id)
    throw createError({
      statusCode: 423, // Locked
      message: `Account temporarily locked due to too many failed attempts. Try again in ${lockoutStatus.minutesRemaining} minutes.`,
      data: {
        lockedUntil: lockoutStatus.lockedUntil,
        minutesRemaining: lockoutStatus.minutesRemaining
      }
    })
  }

  // Verify password
  if (!verifyPassword(password, user.passwordHash)) {
    // Record failed attempt
    const lockoutResult = await recordFailedAttempt(user.id)

    if (lockoutResult.isLocked) {
      await audit.accountLocked(event, user.id, 'max_failed_attempts')
      await audit.loginFailed(event, email, 'invalid_password_locked', user.id)
      throw createError({
        statusCode: 423, // Locked
        message: `Account locked due to too many failed attempts. Try again in ${lockoutResult.lockoutMinutes} minutes.`,
        data: {
          attemptsRemaining: 0,
          lockoutMinutes: lockoutResult.lockoutMinutes
        }
      })
    }

    await audit.loginFailed(event, email, 'invalid_password', user.id)
    throw createError({
      statusCode: 401,
      message: 'Invalid email or password',
      data: {
        attemptsRemaining: lockoutResult.attemptsRemaining
      }
    })
  }

  // Password verified - reset failed attempts
  await resetFailedAttempts(user.id)

  // Check if 2FA is enabled (both globally in config AND for this user)
  if (config.has2FA && user.twoFactorEnabled) {
    // Create a pending 2FA verification session
    const pendingToken = randomBytes(32).toString('hex')
    const expiresAt = Date.now() + 5 * 60 * 1000 // 5 minutes (as timestamp)

    await pending2faVerifications.set(pendingToken, {
      userId: user.id,
      email: user.email,
      rememberMe,
      expiresAt,
      attempts: 0
    })

    // Set pending 2FA cookie
    setCookie(event, 'pm-2fa-pending', pendingToken, {
      httpOnly: true,
      secure: process.env.NODE_ENV === 'production',
      sameSite: 'strict',
      path: '/',
      expires: new Date(expiresAt)
    })

    // Generate CSRF token for the 2FA verification request
    // This is required because /api/user/2fa/verify is protected by CSRF middleware
    const csrfToken = generateCsrfToken()
    setCsrfCookie(event, csrfToken)

    return {
      success: true,
      data: {
        requires2fa: true,
        message: 'Two-factor authentication required',
        csrfToken
      },
      requires2fa: true,
      message: 'Two-factor authentication required',
      csrfToken
    }
  }

  // No 2FA - complete login
  await audit.login(event, user.id)

  // Delete any existing session (session fixation prevention)
  const oldSessionId = getCookie(event, 'pm-session')
  if (oldSessionId) {
    db.delete(schema.sessions).where(eq(schema.sessions.id, oldSessionId)).run()
  }

  // Create new session
  const sessionId = generateSessionId()
  const expiresAt = new Date()

  // Session duration: 30 days if rememberMe, otherwise 24 hours
  if (rememberMe) {
    expiresAt.setDate(expiresAt.getDate() + 30)
  } else {
    expiresAt.setHours(expiresAt.getHours() + 24)
  }

  db.insert(schema.sessions)
    .values({
      id: sessionId,
      userId: user.id,
      expiresAt
    })
    .run()

  // Set session cookie (upgraded to strict SameSite for CSRF protection)
  setCookie(event, 'pm-session', sessionId, {
    httpOnly: true,
    secure: process.env.NODE_ENV === 'production',
    sameSite: 'strict',
    path: '/',
    expires: expiresAt
  })

  // Generate and set CSRF token
  const csrfToken = generateCsrfToken()
  setCsrfCookie(event, csrfToken)

  const userData = {
    id: user.id,
    email: user.email,
    name: user.name,
    role: user.role
  }

  return {
    success: true,
    data: {
      user: userData,
      csrfToken
    },
    user: userData,
    csrfToken
  }
})
