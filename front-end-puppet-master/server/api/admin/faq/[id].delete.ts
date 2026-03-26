/**
 * DELETE /api/admin/faq/:id
 *
 * Delete a FAQ item and its translations.
 */
import { eq, like } from 'drizzle-orm'
import { useDatabase, schema } from '../../../database/client'

export default defineEventHandler(async event => {
  const db = useDatabase()
  const id = parseInt(event.context.params?.id || '')

  if (isNaN(id)) {
    throw createError({ statusCode: 400, message: 'Invalid ID' })
  }

  // Check if FAQ item exists
  const existing = db
    .select()
    .from(schema.faqItems)
    .where(eq(schema.faqItems.id, id))
    .get()

  if (!existing) {
    throw createError({ statusCode: 404, message: 'FAQ item not found' })
  }

  // Delete translations from centralized table
  db.delete(schema.translations)
    .where(like(schema.translations.key, `faq.${id}.%`))
    .run()

  // Delete FAQ item
  db.delete(schema.faqItems)
    .where(eq(schema.faqItems.id, id))
    .run()

  return {
    success: true,
    data: null
  }
})
