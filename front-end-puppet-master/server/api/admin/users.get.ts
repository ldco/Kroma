/**
 * List Users API Endpoint
 *
 * GET /api/admin/users
 * Query params: page, limit
 * Returns: { users: User[], meta: PaginationMeta }
 *
 * Requires: master role only (user management is restricted to master)
 */
import { desc, sql, ne } from 'drizzle-orm'
import { useDatabase, schema } from '../../database/client'
import {
  parsePaginationParams,
  paginationClauses,
  buildPaginationMeta
} from '../../utils/pagination'
import { getUserPermissions } from '../../utils/permissions'

export default defineEventHandler(async event => {
  const db = useDatabase()
  const currentUser = event.context.user
  const query = getQuery(event)
  const params = parsePaginationParams(query as Record<string, unknown>)
  const { limitClause, offsetClause } = paginationClauses(params.page!, params.limit!)

  // GAP-002/Comment 2: Master-only access for user management
  const permissions = await getUserPermissions(currentUser)
  if (permissions.users !== true) {
    throw createError({
      statusCode: 403,
      message: 'Access denied: User management requires master role'
    })
  }

  // Get paginated users (excluding password hash)
  const users = db
    .select({
      id: schema.users.id,
      email: schema.users.email,
      name: schema.users.name,
      role: schema.users.role,
      createdAt: schema.users.createdAt,
      updatedAt: schema.users.updatedAt
    })
    .from(schema.users)
    .orderBy(desc(schema.users.createdAt))
    .limit(limitClause)
    .offset(offsetClause)
    .all()

  // Get total count for pagination
  const countResult = db
    .select({ count: sql<number>`count(*)` })
    .from(schema.users)
    .get()

  const total = countResult?.count ?? 0
  const meta = buildPaginationMeta(total, params.page!, params.limit!, users)

  return {
    success: true,
    data: { users, meta },
    users,
    meta
  }
})
