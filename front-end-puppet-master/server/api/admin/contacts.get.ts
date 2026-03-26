/**
 * GET /api/admin/contacts
 *
 * Returns paginated contact form submissions.
 * Requires admin authentication.
 *
 * Query params:
 *   - unread: boolean - Only show unread submissions
 *   - page: number - Page number (default: 1)
 *   - limit: number - Items per page (default: 20, max: 100)
 */
import { eq, desc, sql } from 'drizzle-orm'
import { useDatabase, schema } from '../../database/client'
import {
  parsePaginationParams,
  paginationClauses,
  buildPaginationMeta
} from '../../utils/pagination'

export default defineEventHandler(async event => {
  // Check authentication
  const session = event.context.session
  if (!session?.userId) {
    throw createError({
      statusCode: 401,
      statusMessage: 'Unauthorized'
    })
  }

  const db = useDatabase()
  const query = getQuery(event)
  const params = parsePaginationParams(query as Record<string, unknown>)
  const { limitClause, offsetClause } = paginationClauses(params.page!, params.limit!)
  const unreadOnly = query.unread === 'true'

  // Get counts in a single query (total and unread)
  const countsResult = db
    .select({
      total: sql<number>`count(*)`,
      unread: sql<number>`sum(case when read = 0 then 1 else 0 end)`
    })
    .from(schema.contactSubmissions)
    .get()

  const total = unreadOnly ? (countsResult?.unread ?? 0) : (countsResult?.total ?? 0)
  const unreadCount = countsResult?.unread ?? 0

  // Build query with DB-level pagination
  let items
  if (unreadOnly) {
    items = db
      .select()
      .from(schema.contactSubmissions)
      .where(eq(schema.contactSubmissions.read, false))
      .orderBy(desc(schema.contactSubmissions.createdAt))
      .limit(limitClause)
      .offset(offsetClause)
      .all()
  } else {
    items = db
      .select()
      .from(schema.contactSubmissions)
      .orderBy(desc(schema.contactSubmissions.createdAt))
      .limit(limitClause)
      .offset(offsetClause)
      .all()
  }

  const meta = buildPaginationMeta(total, params.page!, params.limit!, items)

  return {
    success: true,
    data: {
      items,
      meta,
      unreadCount
    }
  }
})
