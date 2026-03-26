/**
 * DELETE /api/admin/pricing/:id
 * Delete a pricing tier and all translations
 * Removes translations from centralized table with key patterns:
 *   - pricing.tier.{id}.%
 *   - pricing.feature.{featureId}.%
 */
import { eq, like } from 'drizzle-orm'
import { useDatabase, schema } from '../../../database/client'

export default defineEventHandler(async event => {
  const db = useDatabase()
  const id = parseInt(getRouterParam(event, 'id') || '')

  if (isNaN(id)) {
    throw createError({ statusCode: 400, message: 'Invalid tier ID' })
  }

  // Check if tier exists
  const [existing] = await db
    .select()
    .from(schema.pricingTiers)
    .where(eq(schema.pricingTiers.id, id))

  if (!existing) {
    throw createError({ statusCode: 404, message: 'Tier not found' })
  }

  // Get existing feature IDs to delete their translations
  const existingFeatures = await db
    .select()
    .from(schema.pricingFeatures)
    .where(eq(schema.pricingFeatures.tierId, id))

  // Delete feature translations from centralized table
  for (const f of existingFeatures) {
    await db.delete(schema.translations)
      .where(like(schema.translations.key, `pricing.feature.${f.id}.%`))
  }

  // Delete tier translations from centralized table
  await db.delete(schema.translations)
    .where(like(schema.translations.key, `pricing.tier.${id}.%`))

  // Delete tier (features cascade automatically)
  await db
    .delete(schema.pricingTiers)
    .where(eq(schema.pricingTiers.id, id))

  return {
    success: true,
    data: {}
  }
})
