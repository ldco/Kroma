/**
 * Delete User API Endpoint
 *
 * DELETE /api/admin/users/:id
 * Returns: { success: true }
 *
 * Requires: master role only (user management is restricted to master)
 */
import { eq } from 'drizzle-orm'
import { useDatabase, schema } from '../../../database/client'
import { getUserPermissions } from '../../../utils/permissions'
import { audit } from '../../../utils/audit'

export default defineEventHandler(async event => {
  const id = getRouterParam(event, 'id')
  if (!id || !/^\d+$/.test(id)) {
    throw createError({
      statusCode: 400,
      message: 'Valid user ID required'
    })
  }

  const userId = parseInt(id)
  const currentUser = event.context.user

  // GAP-002/Comment 2: Master-only access for user management
  const permissions = await getUserPermissions(currentUser)
  if (permissions.users !== true) {
    throw createError({
      statusCode: 403,
      message: 'Access denied: User management requires master role'
    })
  }

  // Prevent self-deletion
  if (currentUser?.id === userId) {
    throw createError({
      statusCode: 400,
      message: 'You cannot delete your own account'
    })
  }

  const db = useDatabase()

  // Get target user
  const targetUser = db.select().from(schema.users).where(eq(schema.users.id, userId)).get()

  if (!targetUser) {
    throw createError({
      statusCode: 404,
      message: 'User not found'
    })
  }

  // Audit log before deletion (need email for log)
  await audit.userDelete(event, currentUser!.id, userId, targetUser.email)

  // Delete user (sessions will cascade delete)
  db.delete(schema.users).where(eq(schema.users.id, userId)).run()

  return {
    success: true,
    data: null
  }
})
