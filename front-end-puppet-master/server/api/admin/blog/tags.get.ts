/**
 * GET /api/admin/blog/tags
 *
 * Get all blog tags for admin.
 */
import { asc } from 'drizzle-orm'
import { useDatabase, schema } from '../../../database/client'

export default defineEventHandler(async () => {
  const db = useDatabase()

  const tags = db
    .select()
    .from(schema.blogTags)
    .orderBy(asc(schema.blogTags.name))
    .all()

  return {
    success: true,
    data: tags,
    tags
  }
})
