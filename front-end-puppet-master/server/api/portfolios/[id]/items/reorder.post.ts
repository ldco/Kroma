/**
 * POST /api/portfolios/[id]/items/reorder
 *
 * Bulk reorders portfolio items.
 * Requires admin authentication.
 *
 * Body: { items: [{ id: number, order: number }, ...] }
 */
import { z } from 'zod'
import { eq, and, inArray } from 'drizzle-orm'
import { useDatabase, schema, transactionSync } from '../../../../database/client'
import { requireAdminSectionAccess } from '../../../../utils/admin-rbac'

const reorderSchema = z.object({
  items: z.array(
    z.object({
      id: z.number().int(),
      order: z.number().int()
    })
  )
})

export default defineEventHandler(async event => {
  await requireAdminSectionAccess(event.context.user, 'portfolios')

  const portfolioId = getRouterParam(event, 'id')

  // Validate portfolio ID
  if (!portfolioId || !/^\d+$/.test(portfolioId)) {
    throw createError({
      statusCode: 400,
      statusMessage: 'Invalid portfolio ID'
    })
  }

  const pId = parseInt(portfolioId)
  const db = useDatabase()

  // Check if portfolio exists
  const portfolio = db
    .select()
    .from(schema.portfolios)
    .where(eq(schema.portfolios.id, pId))
    .get()

  if (!portfolio) {
    throw createError({
      statusCode: 404,
      statusMessage: 'Portfolio not found'
    })
  }

  // Parse and validate request body
  const body = await readBody(event)
  const result = reorderSchema.safeParse(body)

  if (!result.success) {
    throw createError({
      statusCode: 400,
      statusMessage: 'Validation failed',
      data: result.error.flatten()
    })
  }

  const { items } = result.data

  if (items.length === 0) {
    return {
      success: true,
      data: {
        updated: 0
      }
    }
  }

  // Verify all items belong to this portfolio
  const itemIds = items.map(i => i.id)
  const existingItems = db
    .select({ id: schema.portfolioItems.id })
    .from(schema.portfolioItems)
    .where(
      and(eq(schema.portfolioItems.portfolioId, pId), inArray(schema.portfolioItems.id, itemIds))
    )
    .all()

  const existingIds = new Set(existingItems.map(i => i.id))
  const invalidIds = itemIds.filter(id => !existingIds.has(id))

  if (invalidIds.length > 0) {
    throw createError({
      statusCode: 400,
      statusMessage: `Items not found in this portfolio: ${invalidIds.join(', ')}`
    })
  }

  // Update orders atomically in a transaction
  const now = new Date()
  transactionSync(db => {
    for (const item of items) {
      db.update(schema.portfolioItems)
        .set({ order: item.order, updatedAt: now })
        .where(eq(schema.portfolioItems.id, item.id))
        .run()
    }
  })

  return {
    success: true,
    data: {
      updated: items.length
    }
  }
})
