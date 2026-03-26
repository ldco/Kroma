/**
 * DELETE /api/admin/clients/:id
 *
 * Delete a client.
 */
import { eq } from 'drizzle-orm'
import { useDatabase, schema } from '../../../database/client'

export default defineEventHandler(async event => {
  const db = useDatabase()
  const id = parseInt(event.context.params?.id || '')

  if (isNaN(id)) {
    throw createError({ statusCode: 400, message: 'Invalid ID' })
  }

  // Check if client exists
  const existing = db
    .select()
    .from(schema.clients)
    .where(eq(schema.clients.id, id))
    .get()

  if (!existing) {
    throw createError({ statusCode: 404, message: 'Client not found' })
  }

  // Delete client
  db.delete(schema.clients)
    .where(eq(schema.clients.id, id))
    .run()

  return {
    success: true,
    data: null
  }
})
