/**
 * 2FA Setup Endpoint
 *
 * POST /api/user/2fa/setup
 * Generates a new TOTP secret and returns QR code for authenticator app setup.
 * Does NOT enable 2FA - user must verify with /enable endpoint.
 *
 * Returns: { secret: string, qrCode: string, uri: string }
 *
 * Security:
 * - Requires authenticated session
 * - Rate limited: 5 requests per 15 minutes
 * - Returns temp secret (not stored until verified)
 * - Uses distributed ephemeral store for cross-process safety
 */
import { eq } from 'drizzle-orm'
import { useDatabase, schema } from '../../../database/client'
import {
  generateTotpSecret,
  generateTotpQRCode,
  getTotpUri,
  encryptTotpSecret,
  generateBackupCodes
} from '../../../utils/totp'
import { twoFactorSetupRateLimiter, getClientIp } from '../../../utils/rateLimit'
import { pending2FASessions } from '../../../utils/ephemeral-store'
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

  // Rate limit by user ID (async for distributed consistency)
  const userIdStr = String(userId)
  const rateLimitAllowed = await twoFactorSetupRateLimiter.checkRateLimitAsync(userIdStr)
  if (!rateLimitAllowed) {
    throw createError({
      statusCode: 429,
      message: 'Too many setup attempts. Please try again later.'
    })
  }

  const db = useDatabase()

  // Get user to check if 2FA is already enabled
  const user = db
    .select({
      id: schema.users.id,
      email: schema.users.email,
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

  if (user.twoFactorEnabled) {
    throw createError({
      statusCode: 400,
      message: 'Two-factor authentication is already enabled. Disable it first to set up a new device.'
    })
  }

  // Generate new TOTP secret and backup codes
  const secret = generateTotpSecret()
  const encryptedSecret = encryptTotpSecret(secret)
  const backupCodes = generateBackupCodes()

  // Generate QR code and URI
  const qrCode = await generateTotpQRCode(secret, user.email)
  const uri = getTotpUri(secret, user.email)

  // Store pending setup in distributed ephemeral store (TTL: 5 minutes)
  await pending2FASessions.set(userIdStr, {
    secret,
    encryptedSecret,
    backupCodes,
    createdAt: Date.now()
  })

  return {
    success: true,
    data: {
      qrCode,
      uri,
      backupCodes: backupCodes.plain
    },
    // Don't expose the raw secret - only the QR code and manual entry URI
    qrCode,
    uri,
    // Return plain backup codes (only shown once during setup)
    backupCodes: backupCodes.plain
  }
})
