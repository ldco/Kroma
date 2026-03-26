/**
 * List Roles API Endpoint
 *
 * GET /api/admin/roles
 * Returns: { roles: Role[] } with user counts
 *
 * Requires: roles page access (master only)
 */
import { sql } from 'drizzle-orm'
import { useDatabase, schema } from '../../../database/client'
import { requirePermission } from '../../../utils/permissions'
import { safeJsonParse } from '../../../utils/json'

export default defineEventHandler(async event => {
  const currentUser = event.context.user

  // Require roles page access
  await requirePermission(currentUser, 'roles')

  const db = useDatabase()

  // Get all roles with user counts
  const rolesData = db
    .select({
      id: schema.roles.id,
      name: schema.roles.name,
      slug: schema.roles.slug,
      description: schema.roles.description,
      permissions: schema.roles.permissions,
      level: schema.roles.level,
      isBuiltIn: schema.roles.isBuiltIn,
      color: schema.roles.color,
      icon: schema.roles.icon,
      createdAt: schema.roles.createdAt,
      updatedAt: schema.roles.updatedAt
    })
    .from(schema.roles)
    .orderBy(schema.roles.level)
    .all()

  // Get user counts per role
  const userCounts = db
    .select({
      roleId: schema.users.roleId,
      count: sql<number>`count(*)`.as('count')
    })
    .from(schema.users)
    .groupBy(schema.users.roleId)
    .all()

  const countMap = new Map(userCounts.map(uc => [uc.roleId, uc.count]))

  // Parse permissions and add user counts
  const roles = rolesData.map(role => ({
    ...role,
    permissions: safeJsonParse(role.permissions, {}),
    userCount: countMap.get(role.id) || 0
  }))

  // Sort by level descending (highest authority first)
  roles.sort((a, b) => b.level - a.level)

  return {
    success: true,
    data: {
      roles
    },
    roles
  }
})
