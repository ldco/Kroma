/**
 * Delete Role API Endpoint
 *
 * DELETE /api/admin/roles/:id
 * Returns: { success: true }
 *
 * Requires: roles page access (master only)
 * Cannot delete built-in roles
 * Cannot delete roles with assigned users
 */
import { eq } from 'drizzle-orm'
import { useDatabase, schema } from '../../../database/client'
import { requirePermission, clearRolePermissionsCache } from '../../../utils/permissions'
import { audit } from '../../../utils/audit'

export default defineEventHandler(async event => {
  const id = getRouterParam(event, 'id')
  if (!id || !/^\d+$/.test(id)) {
    throw createError({
      statusCode: 400,
      message: 'Valid role ID required'
    })
  }

  const roleId = parseInt(id)
  const currentUser = event.context.user

  // Require roles page access (master only)
  await requirePermission(currentUser, 'roles')

  const db = useDatabase()

  // Get existing role
  const existingRole = db.select().from(schema.roles).where(eq(schema.roles.id, roleId)).get()

  if (!existingRole) {
    throw createError({
      statusCode: 404,
      message: 'Role not found'
    })
  }

  // Cannot delete built-in roles
  if (existingRole.isBuiltIn) {
    throw createError({
      statusCode: 403,
      message: 'Cannot delete built-in roles'
    })
  }

  // Check if any users are assigned to this role
  const usersWithRole = db
    .select({ id: schema.users.id })
    .from(schema.users)
    .where(eq(schema.users.roleId, roleId))
    .limit(1)
    .get()

  if (usersWithRole) {
    throw createError({
      statusCode: 409,
      message: 'Cannot delete role with assigned users. Reassign users first.'
    })
  }

  // Delete role
  db.delete(schema.roles).where(eq(schema.roles.id, roleId)).run()

  // Clear cache
  clearRolePermissionsCache()

  // Audit log
  await audit.roleDelete(event, currentUser!.id, roleId, existingRole.name)

  return {
    success: true,
    data: null
  }
})
