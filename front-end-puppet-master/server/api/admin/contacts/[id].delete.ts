/**
 * DELETE /api/admin/contacts/:id
 *
 * Deletes a contact submission.
 * Requires admin authentication.
 */
import { eq } from 'drizzle-orm'
import { useDatabase, schema } from '../../../database/client'

export default defineEventHandler(async event => {
  // Check authentication
  const session = event.context.session
  if (!session?.userId) {
    throw createError({
      statusCode: 401,
      statusMessage: 'Unauthorized'
    })
  }

  const id = getRouterParam(event, 'id')
  if (!id || !/^\d+$/.test(id)) {
    throw createError({
      statusCode: 400,
      statusMessage: 'Valid ID required'
    })
  }

  const db = useDatabase()

  // Check if submission exists
  const existing = db
    .select()
    .from(schema.contactSubmissions)
    .where(eq(schema.contactSubmissions.id, parseInt(id)))
    .get()

  if (!existing) {
    throw createError({
      statusCode: 404,
      statusMessage: 'Contact submission not found'
    })
  }

  // Delete submission
  db.delete(schema.contactSubmissions)
    .where(eq(schema.contactSubmissions.id, parseInt(id)))
    .run()

  return {
    success: true,
    data: {
      deleted: { id: parseInt(id) }
    }
  }
})
