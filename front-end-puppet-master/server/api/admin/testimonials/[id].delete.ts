/**
 * DELETE /api/admin/testimonials/:id
 *
 * Delete a testimonial.
 */
import { eq } from 'drizzle-orm'
import { useDatabase, schema } from '../../../database/client'

export default defineEventHandler(async event => {
  const db = useDatabase()
  const id = parseInt(event.context.params?.id || '')

  if (isNaN(id)) {
    throw createError({ statusCode: 400, message: 'Invalid ID' })
  }

  // Check if testimonial exists
  const existing = db
    .select()
    .from(schema.testimonials)
    .where(eq(schema.testimonials.id, id))
    .get()

  if (!existing) {
    throw createError({ statusCode: 404, message: 'Testimonial not found' })
  }

  // Delete testimonial (cascades to translations)
  db.delete(schema.testimonials)
    .where(eq(schema.testimonials.id, id))
    .run()

  return {
    success: true,
    data: {}
  }
})
