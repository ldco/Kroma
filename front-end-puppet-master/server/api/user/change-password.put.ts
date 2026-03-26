/**
 * Change Password API Endpoint
 *
 * PUT /api/user/change-password
 * Body: { currentPassword: string, newPassword: string }
 * Returns: { success: true }
 *
 * Security:
 * - Requires authenticated user (session)
 * - Verifies current password before allowing change
 * - Password hashed with scrypt
 * - Enforces shared password policy (same as admin user flows)
 * - Audit logged
 */
import { z } from 'zod'
import { eq } from 'drizzle-orm'
import { useDatabase, schema } from '../../database/client'
import { verifyPassword, hashPassword, validatePassword, DEFAULT_PASSWORD_POLICY } from '../../utils/password'
import { audit } from '../../utils/audit'

// Validation schema - newPassword validated against shared policy
const changePasswordSchema = z.object({
  currentPassword: z.string().min(1, 'Current password is required'),
  newPassword: z.string().min(1, 'New password is required')
})

export default defineEventHandler(async event => {
  // Get authenticated user from context (set by auth middleware)
  const sessionUser = event.context.user as { id: number } | undefined

  if (!sessionUser?.id) {
    throw createError({
      statusCode: 401,
      message: 'Not authenticated'
    })
  }

  const body = await readBody(event)

  // Validate input
  const result = changePasswordSchema.safeParse(body)
  if (!result.success) {
    throw createError({
      statusCode: 400,
      message: result.error.issues[0]?.message || 'Validation failed'
    })
  }

  const { currentPassword, newPassword } = result.data
  const db = useDatabase()

  // Comment 4: Enforce shared password policy (same as admin user flows)
  const passwordValidation = validatePassword(newPassword, DEFAULT_PASSWORD_POLICY)
  if (!passwordValidation.valid) {
    throw createError({
      statusCode: 400,
      message: passwordValidation.errors[0] || 'Password does not meet policy requirements',
      data: { errors: passwordValidation.errors, strength: passwordValidation.strength }
    })
  }

  // Get user with current password hash
  const user = db
    .select({ id: schema.users.id, passwordHash: schema.users.passwordHash })
    .from(schema.users)
    .where(eq(schema.users.id, sessionUser.id))
    .get()

  if (!user) {
    throw createError({
      statusCode: 404,
      message: 'User not found'
    })
  }

  // Verify current password
  if (!verifyPassword(currentPassword, user.passwordHash)) {
    throw createError({
      statusCode: 401,
      message: 'Current password is incorrect'
    })
  }

  // Hash and update new password
  const newPasswordHash = hashPassword(newPassword)

  db.update(schema.users)
    .set({
      passwordHash: newPasswordHash,
      updatedAt: new Date()
    })
    .where(eq(schema.users.id, user.id))
    .run()

  // Audit log the password change
  await audit.passwordChange(event, sessionUser.id, user.id)

  return {
    success: true,
    data: {
      updated: true
    },
    updated: true
  }
})
