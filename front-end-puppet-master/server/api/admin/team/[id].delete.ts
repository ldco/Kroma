/**
 * DELETE /api/admin/team/:id
 *
 * Delete a team member and all translations.
 */
import { eq, like } from 'drizzle-orm'
import { useDatabase, schema } from '../../../database/client'

export default defineEventHandler(async event => {
  const db = useDatabase()
  const id = parseInt(event.context.params?.id || '')

  if (isNaN(id)) {
    throw createError({
      statusCode: 400,
      message: 'Invalid ID'
    })
  }

  // Check if member exists
  const existing = db
    .select()
    .from(schema.teamMembers)
    .where(eq(schema.teamMembers.id, id))
    .get()

  if (!existing) {
    throw createError({
      statusCode: 404,
      message: 'Team member not found'
    })
  }

  // Delete translations from centralized table
  db.delete(schema.translations)
    .where(like(schema.translations.key, `team.${id}.%`))
    .run()

  // Delete member
  db.delete(schema.teamMembers)
    .where(eq(schema.teamMembers.id, id))
    .run()

  return {
    success: true,
    data: {}
  }
})
