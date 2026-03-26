/**
 * GET /api/portfolios/[id]/items
 *
 * Returns items in a portfolio.
 * Public endpoint - returns only published items unless admin.
 *
 * Path params:
 *   - id: number - Portfolio ID
 *
 * Query params:
 *   - all: boolean - Include unpublished (admin only)
 *   - type: 'image' | 'video' | 'link' | 'case_study' - Filter by item type
 *   - limit: number - Limit results
 *   - offset: number - Pagination offset
 */
import { eq, desc, and } from 'drizzle-orm'
import { useDatabase, schema } from '../../../../database/client'
import { safeJsonParse } from '../../../../utils/json'
import { hasAdminSectionAccess } from '../../../../utils/admin-rbac'

export default defineEventHandler(async event => {
  const db = useDatabase()
  const portfolioId = getRouterParam(event, 'id')
  const query = getQuery(event)
  const user = event.context.user

  // Validate portfolio ID
  if (!portfolioId || !/^\d+$/.test(portfolioId)) {
    throw createError({
      statusCode: 400,
      statusMessage: 'Invalid portfolio ID'
    })
  }

  const id = parseInt(portfolioId)

  // Check if portfolio exists
  const portfolio = db
    .select()
    .from(schema.portfolios)
    .where(eq(schema.portfolios.id, id))
    .get()

  if (!portfolio) {
    throw createError({
      statusCode: 404,
      statusMessage: 'Portfolio not found'
    })
  }

  const canReadUnpublished = user ? await hasAdminSectionAccess(user, 'portfolios') : false

  // Check if portfolio is accessible
  if (!portfolio.published && !canReadUnpublished) {
    if (user) {
      throw createError({
        statusCode: 403,
        statusMessage: 'Access denied: unpublished portfolios require portfolios section access'
      })
    }
    throw createError({
      statusCode: 404,
      statusMessage: 'Portfolio not found'
    })
  }

  // Build query conditions
  const conditions = [eq(schema.portfolioItems.portfolioId, id)]

  // Non-admin only sees published items
  const requestedAll = query.all === 'true'
  let includeAll = false
  if (requestedAll) {
    if (!user || !canReadUnpublished) {
      throw createError({
        statusCode: 403,
        statusMessage: 'Access denied: unpublished items require portfolios section access'
      })
    }
    includeAll = true
  }
  if (!includeAll) {
    conditions.push(eq(schema.portfolioItems.published, true))
  }

  // Filter by item type
  const validTypes = ['image', 'video', 'link', 'case_study'] as const
  type ItemType = (typeof validTypes)[number]
  if (query.type && validTypes.includes(query.type as ItemType)) {
    conditions.push(eq(schema.portfolioItems.itemType, query.type as ItemType))
  }

  // Parse pagination params
  const limit = query.limit ? parseInt(query.limit as string) : undefined
  const offset = query.offset ? parseInt(query.offset as string) : 0

  // Execute query
  const whereClause = conditions.length === 1 ? conditions[0] : and(...conditions)

  const baseQuery = db
    .select()
    .from(schema.portfolioItems)
    .where(whereClause)
    .orderBy(desc(schema.portfolioItems.order), desc(schema.portfolioItems.createdAt))

  let items
  if (limit) {
    items = baseQuery.limit(limit).offset(offset).all()
  } else {
    items = baseQuery.all()
  }

  // Parse tags JSON for case study items
  return {
    success: true,
    data: items.map(item => ({
      ...item,
      tags: safeJsonParse(item.tags, [])
    }))
  }
})
