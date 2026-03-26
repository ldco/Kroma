/**
 * Admin Audit Logs Endpoint
 *
 * GET /api/admin/audit-logs
 *
 * Returns security audit logs from the database.
 * Requires master role.
 *
 * Query params:
 * - limit: Number of entries to return (default: 50, max: 200)
 * - offset: Pagination offset (default: 0)
 * - action: Filter by action type (optional)
 * - userId: Filter by actor user ID (optional)
 */
import { and, desc, eq, count } from 'drizzle-orm'
import { useDatabase, schema } from '../../database/client'
import { requireMaster } from '../../utils/roles'
import { safeJsonParseOrNull } from '../../utils/json'
import type { UserRole, AuditAction } from '../../database/schema'

export default defineEventHandler(async event => {
  // Require master role
  const user = event.context.user as { role: UserRole } | undefined
  requireMaster(user?.role)

  const db = useDatabase()
  const query = getQuery(event)

  // Parse pagination
  let limit = parseInt(String(query.limit || '50'), 10)
  if (isNaN(limit) || limit < 1) limit = 50
  if (limit > 200) limit = 200

  let offset = parseInt(String(query.offset || '0'), 10)
  if (isNaN(offset) || offset < 0) offset = 0

  // Build query conditions
  const conditions = []

  // Filter by action
  if (query.action) {
    conditions.push(eq(schema.auditLogs.action, String(query.action) as AuditAction))
  }

  // Filter by user ID
  if (query.userId) {
    const userId = parseInt(String(query.userId), 10)
    if (!isNaN(userId)) {
      conditions.push(eq(schema.auditLogs.userId, userId))
    }
  }

  const whereClause =
    conditions.length === 0
      ? undefined
      : conditions.length === 1
        ? conditions[0]
        : and(...conditions)

  // Get total count for pagination (filtered when applicable)
  const totalResult = whereClause
    ? db.select({ count: count() }).from(schema.auditLogs).where(whereClause).get()
    : db.select({ count: count() }).from(schema.auditLogs).get()
  const total = totalResult?.count ?? 0

  const logs = (whereClause
    ? db
        .select({
          id: schema.auditLogs.id,
          action: schema.auditLogs.action,
          userId: schema.auditLogs.userId,
          targetUserId: schema.auditLogs.targetUserId,
          ipAddress: schema.auditLogs.ipAddress,
          userAgent: schema.auditLogs.userAgent,
          details: schema.auditLogs.details,
          success: schema.auditLogs.success,
          createdAt: schema.auditLogs.createdAt,
          actorName: schema.users.name,
          actorEmail: schema.users.email
        })
        .from(schema.auditLogs)
        .leftJoin(schema.users, eq(schema.auditLogs.userId, schema.users.id))
        .where(whereClause)
        .orderBy(desc(schema.auditLogs.createdAt))
        .limit(limit)
        .offset(offset)
    : db
        .select({
          id: schema.auditLogs.id,
          action: schema.auditLogs.action,
          userId: schema.auditLogs.userId,
          targetUserId: schema.auditLogs.targetUserId,
          ipAddress: schema.auditLogs.ipAddress,
          userAgent: schema.auditLogs.userAgent,
          details: schema.auditLogs.details,
          success: schema.auditLogs.success,
          createdAt: schema.auditLogs.createdAt,
          actorName: schema.users.name,
          actorEmail: schema.users.email
        })
        .from(schema.auditLogs)
        .leftJoin(schema.users, eq(schema.auditLogs.userId, schema.users.id))
        .orderBy(desc(schema.auditLogs.createdAt))
        .limit(limit)
        .offset(offset)
  ).all()

  // Transform the results
  const transformedLogs = logs.map(log => ({
    id: log.id,
    action: log.action,
    userId: log.userId,
    targetUserId: log.targetUserId,
    actorName: log.actorName,
    actorEmail: log.actorEmail,
    ipAddress: log.ipAddress,
    userAgent: log.userAgent,
    details: safeJsonParseOrNull(log.details),
    success: log.success,
    createdAt: log.createdAt?.toISOString()
  }))

  return {
    success: true,
    data: {
      logs: transformedLogs,
      meta: {
        total,
        limit,
        offset,
        hasMore: offset + logs.length < total
      }
    }
  }
})
