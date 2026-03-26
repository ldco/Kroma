/**
 * 2FA Status Endpoint
 *
 * GET /api/user/2fa/status
 * Returns the current 2FA status for the authenticated user.
 *
 * Returns: { enabled: boolean, backupCodesRemaining?: number }
 *
 * Security:
 * - Requires authenticated session
 */
import { eq } from 'drizzle-orm'
import { useDatabase, schema } from '../../../database/client'
import config from '../../../../app/puppet-master.config'

export default defineEventHandler(async event => {
  // Check if 2FA is enabled in config - if not, return disabled status
  if (!config.has2FA) {
    return {
      success: true,
      data: {
        enabled: false,
        available: false
      },
      enabled: false,
      available: false
    }
  }

  // Require authentication
  if (!event.context.session?.userId) {
    throw createError({
      statusCode: 401,
      message: 'Authentication required'
    })
  }

  const userId = event.context.session.userId
  const db = useDatabase()

  // Get user's 2FA status
  const user = db
    .select({
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

  // If 2FA is not enabled for user, return simple response
  if (!user.twoFactorEnabled) {
    return {
      success: true,
      data: {
        enabled: false,
        available: true // 2FA is available but not enabled for this user
      },
      enabled: false,
      available: true // 2FA is available but not enabled for this user
    }
  }

  // Get backup codes remaining
  const user2fa = db
    .select({
      backupCodesRemaining: schema.user2fa.backupCodesRemaining
    })
    .from(schema.user2fa)
    .where(eq(schema.user2fa.userId, userId))
    .get()

  return {
    success: true,
    data: {
      enabled: true,
      available: true,
      backupCodesRemaining: user2fa?.backupCodesRemaining ?? 0
    },
    enabled: true,
    available: true,
    backupCodesRemaining: user2fa?.backupCodesRemaining ?? 0
  }
})
