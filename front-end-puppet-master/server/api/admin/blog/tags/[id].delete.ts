/**
 * DELETE /api/admin/blog/tags/:id
 *
 * Delete a blog tag.
 */
import { eq } from 'drizzle-orm'
import { useDatabase, schema } from '../../../../database/client'

export default defineEventHandler(async event => {
  const db = useDatabase()
  const id = parseInt(event.context.params?.id || '')

  if (isNaN(id)) {
    throw createError({ statusCode: 400, message: 'Invalid ID' })
  }

  // Check if tag exists
  const existing = db
    .select()
    .from(schema.blogTags)
    .where(eq(schema.blogTags.id, id))
    .get()

  if (!existing) {
    throw createError({ statusCode: 404, message: 'Tag not found' })
  }

  // Delete tag (will cascade to blogPostTags)
  db.delete(schema.blogTags)
    .where(eq(schema.blogTags.id, id))
    .run()

  return {
    success: true,
    data: null
  }
})
