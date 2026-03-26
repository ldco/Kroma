/**
 * 2FA Disable Endpoint
 *
 * POST /api/user/2fa/disable
 * Body: { password: string }
 * Disables 2FA for the user after password confirmation.
 *
 * Returns: { success: true, message: string }
 *
 * Security:
 * - Requires authenticated session
 * - Requires password confirmation
 * - Audit logged for security
 */
import { eq } from 'drizzle-orm'
import { useDatabase, schema } from '../../../database/client'
import { verifyPassword } from '../../../utils/password'
import { twoFactorDisableSchema } from '../../../utils/validation'
import { audit } from '../../../utils/audit'
import { twoFactorDisableRateLimiter } from '../../../utils/rateLimit'
import config from '../../../../app/puppet-master.config'

export default defineEventHandler(async event => {
  // Check if 2FA is enabled in config
  if (!config.has2FA) {
    throw createError({
      statusCode: 403,
      message: 'Two-factor authentication is not enabled for this project'
    })
  }

  // Require authentication
  if (!event.context.session?.userId) {
    throw createError({
      statusCode: 401,
      message: 'Authentication required'
    })
  }

  const userId = event.context.session.userId

  // Rate limit by user ID
  if (!twoFactorDisableRateLimiter.checkRateLimit(String(userId))) {
    throw createError({
      statusCode: 429,
      message: 'Too many disable attempts. Please try again later.'
    })
  }

  const body = await readBody(event)

  // Validate input
  const result = twoFactorDisableSchema.safeParse(body)
  if (!result.success) {
    throw createError({
      statusCode: 400,
      message: 'Password is required'
    })
  }

  const { password } = result.data
  const db = useDatabase()

  // Get user with password hash
  const user = db
    .select({
      id: schema.users.id,
      passwordHash: schema.users.passwordHash,
      twoFactorEnabled: schema.users.twoFactorEnabled
    })
    .from(schema.users)
    .where(eq(schema.users.id, userId))
    .get()

  if (!user) {
    throw createError({
      statusCode: 404,
      message: 'User not found'
    })
  }

  if (!user.twoFactorEnabled) {
    throw createError({
      statusCode: 400,
      message: 'Two-factor authentication is not enabled'
    })
  }

  // Verify password
  if (!verifyPassword(password, user.passwordHash)) {
    throw createError({
      statusCode: 401,
      message: 'Incorrect password'
    })
  }

  // Delete 2FA record
  db.delete(schema.user2fa).where(eq(schema.user2fa.userId, userId)).run()

  // Disable 2FA on user record
  db.update(schema.users)
    .set({
      twoFactorEnabled: false,
      updatedAt: new Date()
    })
    .where(eq(schema.users.id, userId))
    .run()

  // Log audit event
  await audit.twoFactorDisabled(event, userId, userId, 'User disabled 2FA')

  return {
    success: true,
    data: {
      message: 'Two-factor authentication has been disabled.'
    },
    message: 'Two-factor authentication has been disabled.'
  }
})
