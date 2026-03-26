/**
 * Update Role API Endpoint
 *
 * PUT /api/admin/roles/:id
 * Body: { name?, slug?, description?, permissions?, color?, icon? }
 * Returns: { success: true, role: Role }
 *
 * Level is auto-calculated from permissions when permissions change:
 * - roles access → level 90
 * - users access → level 50
 * - neither → level 25
 *
 * Requires: roles page access (master only)
 * Cannot modify master role
 */
import { eq, ne, and } from 'drizzle-orm'
import { useDatabase, schema } from '../../../database/client'
import { requirePermission, clearRolePermissionsCache, calculateLevel } from '../../../utils/permissions'
import { updateRoleSchema } from '../../../utils/validation'
import { audit } from '../../../utils/audit'
import { safeJsonParse } from '../../../utils/json'

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

  const body = await readBody(event)

  // Validate input
  const result = updateRoleSchema.safeParse(body)
  if (!result.success) {
    throw createError({
      statusCode: 400,
      message: result.error.issues[0]?.message || 'Invalid input'
    })
  }

  const { name, slug, description, permissions, color, icon } = result.data

  const db = useDatabase()

  // Get existing role
  const existingRole = db.select().from(schema.roles).where(eq(schema.roles.id, roleId)).get()

  if (!existingRole) {
    throw createError({
      statusCode: 404,
      message: 'Role not found'
    })
  }

  // Cannot modify master role
  if (existingRole.slug === 'master') {
    throw createError({
      statusCode: 403,
      message: 'Cannot modify master role'
    })
  }

  // Check name uniqueness if changing
  if (name && name !== existingRole.name) {
    const existingByName = db
      .select()
      .from(schema.roles)
      .where(and(eq(schema.roles.name, name), ne(schema.roles.id, roleId)))
      .get()
    if (existingByName) {
      throw createError({
        statusCode: 409,
        message: 'A role with this name already exists'
      })
    }
  }

  // Check slug uniqueness if changing
  if (slug && slug !== existingRole.slug) {
    const existingBySlug = db
      .select()
      .from(schema.roles)
      .where(and(eq(schema.roles.slug, slug), ne(schema.roles.id, roleId)))
      .get()
    if (existingBySlug) {
      throw createError({
        statusCode: 409,
        message: 'A role with this slug already exists'
      })
    }
  }

  // Build update object
  const updates: Record<string, unknown> = {
    updatedAt: new Date()
  }

  if (name !== undefined) updates.name = name
  if (slug !== undefined) updates.slug = slug
  if (description !== undefined) updates.description = description
  if (permissions !== undefined) {
    updates.permissions = JSON.stringify(permissions)
    // Auto-calculate level when permissions change
    updates.level = calculateLevel(permissions)
  }
  if (color !== undefined) updates.color = color
  if (icon !== undefined) updates.icon = icon

  // Update role
  db.update(schema.roles).set(updates).where(eq(schema.roles.id, roleId)).run()

  // Clear cache
  clearRolePermissionsCache()

  // Get updated role
  const updatedRole = db.select().from(schema.roles).where(eq(schema.roles.id, roleId)).get()

  // Audit log
  const changesForLog = { ...updates }
  delete changesForLog.updatedAt
  if (Object.keys(changesForLog).length > 0) {
    await audit.roleUpdate(event, currentUser!.id, roleId, updatedRole!.name, changesForLog)
  }

  const role = {
    ...updatedRole,
    permissions: safeJsonParse(updatedRole!.permissions, {})
  }

  return {
    success: true,
    data: {
      role
    },
    role
  }
})
