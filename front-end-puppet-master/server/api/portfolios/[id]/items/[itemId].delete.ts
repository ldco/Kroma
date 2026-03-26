/**
 * DELETE /api/portfolios/[id]/items/[itemId]
 *
 * Deletes a portfolio item.
 * Requires admin authentication.
 */
import { eq, and } from 'drizzle-orm'
import { useDatabase, schema } from '../../../../database/client'
import { requireAdminSectionAccess } from '../../../../utils/admin-rbac'

export default defineEventHandler(async event => {
  await requireAdminSectionAccess(event.context.user, 'portfolios')

  const portfolioId = getRouterParam(event, 'id')
  const itemId = getRouterParam(event, 'itemId')

  // Validate IDs
  if (!portfolioId || !/^\d+$/.test(portfolioId)) {
    throw createError({
      statusCode: 400,
      statusMessage: 'Invalid portfolio ID'
    })
  }

  if (!itemId || !/^\d+$/.test(itemId)) {
    throw createError({
      statusCode: 400,
      statusMessage: 'Invalid item ID'
    })
  }

  const pId = parseInt(portfolioId)
  const iId = parseInt(itemId)
  const db = useDatabase()

  // Check if item exists and belongs to portfolio
  const existing = db
    .select()
    .from(schema.portfolioItems)
    .where(and(eq(schema.portfolioItems.id, iId), eq(schema.portfolioItems.portfolioId, pId)))
    .get()

  if (!existing) {
    throw createError({
      statusCode: 404,
      statusMessage: 'Item not found'
    })
  }

  // Delete item
  db.delete(schema.portfolioItems).where(eq(schema.portfolioItems.id, iId)).run()

  return {
    success: true,
    data: {
      deleted: {
        id: iId,
        itemType: existing.itemType,
        slug: existing.slug
      }
    }
  }
})
