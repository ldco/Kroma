/**
 * DELETE /api/portfolios/[id]
 *
 * Deletes a portfolio and all its items (cascade).
 * Requires admin authentication.
 */
import { eq } from 'drizzle-orm'
import { useDatabase, schema } from '../../database/client'
import { requireAdminSectionAccess } from '../../utils/admin-rbac'

export default defineEventHandler(async event => {
  await requireAdminSectionAccess(event.context.user, 'portfolios')

  const id = getRouterParam(event, 'id')

  // Validate ID
  if (!id || !/^\d+$/.test(id)) {
    throw createError({
      statusCode: 400,
      statusMessage: 'Invalid portfolio ID'
    })
  }

  const portfolioId = parseInt(id)
  const db = useDatabase()

  // Check if portfolio exists
  const existing = db
    .select()
    .from(schema.portfolios)
    .where(eq(schema.portfolios.id, portfolioId))
    .get()

  if (!existing) {
    throw createError({
      statusCode: 404,
      statusMessage: 'Portfolio not found'
    })
  }

  // Delete portfolio (items cascade automatically due to foreign key)
  db.delete(schema.portfolios).where(eq(schema.portfolios.id, portfolioId)).run()

  return {
    success: true,
    data: {
      deleted: {
        id: portfolioId,
        slug: existing.slug
      }
    }
  }
})
