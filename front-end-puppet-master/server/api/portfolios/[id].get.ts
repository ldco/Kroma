/**
 * GET /api/portfolios/[id]
 *
 * Returns a single portfolio with its items.
 * Public endpoint - returns only if published unless admin.
 *
 * Path params:
 *   - id: number | string - Portfolio ID or slug
 *
 * Query params:
 *   - includeItems: boolean - Include portfolio items (default: true)
 */
import { eq, and, desc } from 'drizzle-orm'
import { useDatabase, schema } from '../../database/client'
import { safeJsonParse } from '../../utils/json'
import { hasAdminSectionAccess } from '../../utils/admin-rbac'

export default defineEventHandler(async event => {
  const db = useDatabase()
  const idOrSlug = getRouterParam(event, 'id')
  const query = getQuery(event)
  const user = event.context.user

  if (!idOrSlug) {
    throw createError({
      statusCode: 400,
      statusMessage: 'Portfolio ID or slug required'
    })
  }

  // Check if ID (numeric) or slug (alphanumeric with dashes)
  const isNumericId = /^\d+$/.test(idOrSlug)

  // Get portfolio by ID or slug
  const portfolio = isNumericId
    ? db
        .select()
        .from(schema.portfolios)
        .where(eq(schema.portfolios.id, parseInt(idOrSlug)))
        .get()
    : db
        .select()
        .from(schema.portfolios)
        .where(eq(schema.portfolios.slug, idOrSlug))
        .get()

  if (!portfolio) {
    throw createError({
      statusCode: 404,
      statusMessage: 'Portfolio not found'
    })
  }

  const canReadUnpublished = user ? await hasAdminSectionAccess(user, 'portfolios') : false

  // Check if published or authorized portfolios editor/admin
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

  // Get items if requested (default: true)
  const includeItems = query.includeItems !== 'false'
  let items: Array<Record<string, unknown>> = []

  if (includeItems) {
    const itemConditions = [eq(schema.portfolioItems.portfolioId, portfolio.id)]

    // Non-admin views only include published items.
    if (!canReadUnpublished) {
      itemConditions.push(eq(schema.portfolioItems.published, true))
    }

    items = db
      .select()
      .from(schema.portfolioItems)
      .where(itemConditions.length === 1 ? itemConditions[0] : and(...itemConditions))
      .orderBy(desc(schema.portfolioItems.order), desc(schema.portfolioItems.createdAt))
      .all()

    // Parse tags JSON for case study items
    items = items.map(item => ({
      ...item,
      tags: safeJsonParse(item.tags as string, [])
    }))
  }

  return {
    success: true,
    data: {
      ...portfolio,
      items
    }
  }
})
