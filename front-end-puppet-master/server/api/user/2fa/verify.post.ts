/**
 * 2FA Verify Endpoint
 *
 * POST /api/user/2fa/verify
 * Body: { code: string }
 * Verifies a TOTP code or backup code during login.
 *
 * This endpoint is called after password verification when 2FA is required.
 * It requires a pending 2FA session (set during login).
 *
 * Returns: { success: true, user: { id, email, name, role }, csrfToken: string }
 *
 * Security:
 * - Requires pending 2FA session
 * - Rate limited: 5 attempts per 15 minutes
 * - Supports both TOTP and backup codes
 * - Backup codes are consumed on use
 * - Uses distributed ephemeral store for cross-process safety
 */
import { eq } from 'drizzle-orm'
import { useDatabase, schema } from '../../../database/client'
import { verify2faCode } from '../../../utils/totp'
import { generateSessionId } from '../../../utils/password'
import { generateCsrfToken, setCsrfCookie } from '../../../utils/csrf'
import { twoFactorVerifySchema } from '../../../utils/validation'
import { audit } from '../../../utils/audit'
import { twoFactorVerifyRateLimiter, getClientIp } from '../../../utils/rateLimit'
import { pending2faVerifications } from '../../../utils/ephemeral-store'
import config from '../../../../app/puppet-master.config'

// Maximum verification attempts before session expires
const MAX_ATTEMPTS = 5

export default defineEventHandler(async event => {
  // Check if 2FA is enabled in config
  if (!config.has2FA) {
    throw createError({
      statusCode: 403,
      message: 'Two-factor authentication is not enabled for this project'
    })
  }

  // Rate limit by client IP (async for distributed consistency)
  const clientIp = getClientIp(event)
  const rateLimitAllowed = await twoFactorVerifyRateLimiter.checkRateLimitAsync(clientIp)
  if (!rateLimitAllowed) {
    throw createError({
      statusCode: 429,
      message: 'Too many verification attempts. Please try again later.'
    })
  }

  // Get the pending 2FA token from cookie
  const pendingToken = getCookie(event, 'pm-2fa-pending')

  if (!pendingToken) {
    throw createError({
      statusCode: 400,
      message: 'No pending two-factor verification. Please log in again.'
    })
  }

  // Get pending verification from distributed store
  const pendingSession = await pending2faVerifications.get(pendingToken)
  if (!pendingSession) {
    deleteCookie(event, 'pm-2fa-pending', { path: '/' })
    throw createError({
      statusCode: 400,
      message: 'Verification session expired. Please log in again.'
    })
  }

  // Check if session has expired
  if (Date.now() > pendingSession.expiresAt) {
    await pending2faVerifications.delete(pendingToken)
    deleteCookie(event, 'pm-2fa-pending', { path: '/' })
    throw createError({
      statusCode: 400,
      message: 'Verification session expired. Please log in again.'
    })
  }

  // Check attempts
  if (pendingSession.attempts >= MAX_ATTEMPTS) {
    await pending2faVerifications.delete(pendingToken)
    deleteCookie(event, 'pm-2fa-pending', { path: '/' })
    throw createError({
      statusCode: 429,
      message: 'Too many failed attempts. Please log in again.'
    })
  }

  const body = await readBody(event)

  // Validate input
  const result = twoFactorVerifySchema.safeParse(body)
  if (!result.success) {
    throw createError({
      statusCode: 400,
      message: 'Invalid code format'
    })
  }

  const { code } = result.data
  const db = useDatabase()

  // Get user's 2FA data
  const user2fa = db
    .select()
    .from(schema.user2fa)
    .where(eq(schema.user2fa.userId, pendingSession.userId))
    .get()

  if (!user2fa) {
    // 2FA was disabled between login and verification - shouldn't happen
    await pending2faVerifications.delete(pendingToken)
    deleteCookie(event, 'pm-2fa-pending', { path: '/' })
    throw createError({
      statusCode: 400,
      message: 'Two-factor authentication is no longer enabled. Please log in again.'
    })
  }

  // Parse backup codes
  const hashedBackupCodes: string[] = JSON.parse(user2fa.backupCodes)

  // Verify the code
  const verifyResult = verify2faCode(
    code,
    user2fa.secret,
    hashedBackupCodes,
    pendingSession.email
  )

  if (!verifyResult.success) {
    // Increment attempts
    pendingSession.attempts++
    await pending2faVerifications.set(pendingToken, pendingSession)
    await audit.twoFactorFailed(event, pendingSession.userId, 'invalid_code')

    const remainingAttempts = MAX_ATTEMPTS - pendingSession.attempts
    throw createError({
      statusCode: 401,
      message: `Invalid verification code. ${remainingAttempts} attempts remaining.`,
      data: { attemptsRemaining: remainingAttempts }
    })
  }

  // If backup code was used, mark it as consumed
  if (verifyResult.method === 'backup' && verifyResult.backupCodeIndex !== undefined) {
    hashedBackupCodes.splice(verifyResult.backupCodeIndex, 1)
    db.update(schema.user2fa)
      .set({
        backupCodes: JSON.stringify(hashedBackupCodes),
        backupCodesRemaining: hashedBackupCodes.length,
        updatedAt: new Date()
      })
      .where(eq(schema.user2fa.userId, pendingSession.userId))
      .run()
  }

  // Log successful 2FA verification
  await audit.twoFactorVerified(event, pendingSession.userId, verifyResult.method!)

  // Clean up pending session
  await pending2faVerifications.delete(pendingToken)
  deleteCookie(event, 'pm-2fa-pending', { path: '/' })

  // Get full user data
  const user = db
    .select({
      id: schema.users.id,
      email: schema.users.email,
      name: schema.users.name,
      role: schema.users.role
    })
    .from(schema.users)
    .where(eq(schema.users.id, pendingSession.userId))
    .get()

  if (!user) {
    throw createError({
      statusCode: 404,
      message: 'User not found'
    })
  }

  // Delete any existing session (session fixation prevention)
  const oldSessionId = getCookie(event, 'pm-session')
  if (oldSessionId) {
    db.delete(schema.sessions).where(eq(schema.sessions.id, oldSessionId)).run()
  }

  // Create session (same logic as login)
  const sessionId = generateSessionId()
  const expiresAt = new Date()

  if (pendingSession.rememberMe) {
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

  // Set session cookie
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

  // Log successful login
  await audit.login(event, user.id)

  const userData = {
    id: user.id,
    email: user.email,
    name: user.name,
    role: user.role
  }
  const backupCodesRemaining =
    verifyResult.method === 'backup' ? hashedBackupCodes.length : undefined

  return {
    success: true,
    data: {
      user: userData,
      csrfToken,
      backupCodesRemaining
    },
    user: userData,
    csrfToken,
    // Warn if backup codes are running low
    backupCodesRemaining
  }
})
