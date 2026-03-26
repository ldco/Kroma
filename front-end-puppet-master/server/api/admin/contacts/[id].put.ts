/**
 * PUT /api/admin/contacts/:id
 *
 * Updates a contact submission (mark as read/unread).
 * Requires admin authentication.
 */
import { eq } from 'drizzle-orm'
import { z } from 'zod'
import { useDatabase, schema } from '../../../database/client'

const updateSchema = z.object({
  read: z.boolean()
})

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

  const body = await readBody(event)
  const result = updateSchema.safeParse(body)

  if (!result.success) {
    throw createError({
      statusCode: 400,
      statusMessage: 'Validation failed',
      data: result.error.flatten()
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

  // Update read status
  const updated = db
    .update(schema.contactSubmissions)
    .set({ read: result.data.read })
    .where(eq(schema.contactSubmissions.id, parseInt(id)))
    .returning()
    .get()

  return {
    success: true,
    data: {
      item: updated
    }
  }
})
