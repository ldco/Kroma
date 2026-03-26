/**
 * Create Role API Endpoint
 *
 * POST /api/admin/roles
 * Body: { name, slug, description?, permissions, color?, icon? }
 * Returns: { success: true, role: Role }
 *
 * Level is auto-calculated from permissions:
 * - roles access → level 90
 * - users access → level 50
 * - neither → level 25
 *
 * Requires: roles page access (master only)
 */
import { eq } from 'drizzle-orm'
import { useDatabase, schema } from '../../../database/client'
import { requirePermission, clearRolePermissionsCache, calculateLevel } from '../../../utils/permissions'
import { createRoleSchema } from '../../../utils/validation'
import { audit } from '../../../utils/audit'
import { safeJsonParse } from '../../../utils/json'

export default defineEventHandler(async event => {
  const currentUser = event.context.user

  // Require roles page access
  await requirePermission(currentUser, 'roles')

  const body = await readBody(event)

  // Validate input
  const result = createRoleSchema.safeParse(body)
  if (!result.success) {
    throw createError({
      statusCode: 400,
      message: result.error.issues[0]?.message || 'Invalid input'
    })
  }

  const { name, slug, description, permissions, color, icon } = result.data

  const db = useDatabase()

  // Check name uniqueness
  const existingByName = db.select().from(schema.roles).where(eq(schema.roles.name, name)).get()
  if (existingByName) {
    throw createError({
      statusCode: 409,
      message: 'A role with this name already exists'
    })
  }

  // Check slug uniqueness
  const existingBySlug = db.select().from(schema.roles).where(eq(schema.roles.slug, slug)).get()
  if (existingBySlug) {
    throw createError({
      statusCode: 409,
      message: 'A role with this slug already exists'
    })
  }

  // Calculate level based on permissions
  const level = calculateLevel(permissions)

  // Create role
  const newRole = db
    .insert(schema.roles)
    .values({
      name,
      slug,
      description: description || null,
      permissions: JSON.stringify(permissions),
      level,
      isBuiltIn: false,
      color: color || 'secondary',
      icon: icon || 'pencil'
    })
    .returning()
    .get()

  // Clear cache
  clearRolePermissionsCache()

  // Audit log
  await audit.roleCreate(event, currentUser!.id, newRole.id, newRole.name, newRole.slug)

  const role = {
    ...newRole,
    permissions: safeJsonParse(newRole.permissions, {}),
    userCount: 0
  }

  return {
    success: true,
    data: {
      role
    },
    role
  }
})
