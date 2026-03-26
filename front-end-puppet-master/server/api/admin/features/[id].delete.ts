/**
 * DELETE /api/admin/features/:id
 *
 * Delete a feature and its translations.
 */
import { eq, like } from 'drizzle-orm'
import { useDatabase, schema } from '../../../database/client'

export default defineEventHandler(async event => {
  const db = useDatabase()
  const id = parseInt(event.context.params?.id || '')

  if (isNaN(id)) {
    throw createError({ statusCode: 400, message: 'Invalid ID' })
  }

  // Check if feature exists
  const existing = db
    .select()
    .from(schema.features)
    .where(eq(schema.features.id, id))
    .get()

  if (!existing) {
    throw createError({ statusCode: 404, message: 'Feature not found' })
  }

  // Delete translations from centralized table
  db.delete(schema.translations)
    .where(like(schema.translations.key, `feature.${id}.%`))
    .run()

  // Delete feature
  db.delete(schema.features)
    .where(eq(schema.features.id, id))
    .run()

  return {
    success: true,
    data: {}
  }
})
