/**
 * DELETE /api/admin/blog/categories/:id
 *
 * Delete a blog category and all translations.
 * Removes translations from centralized table with key pattern: blog.category.{id}.%
 */
import { eq, like } from 'drizzle-orm'
import { useDatabase, schema } from '../../../../database/client'

export default defineEventHandler(async event => {
  const db = useDatabase()
  const id = parseInt(event.context.params?.id || '')

  if (isNaN(id)) {
    throw createError({ statusCode: 400, message: 'Invalid ID' })
  }

  // Check if category exists
  const existing = db
    .select()
    .from(schema.blogCategories)
    .where(eq(schema.blogCategories.id, id))
    .get()

  if (!existing) {
    throw createError({ statusCode: 404, message: 'Category not found' })
  }

  // Delete translations from centralized table
  db.delete(schema.translations)
    .where(like(schema.translations.key, `blog.category.${id}.%`))
    .run()

  // Delete category (posts will have categoryId set to null due to foreign key)
  db.delete(schema.blogCategories)
    .where(eq(schema.blogCategories.id, id))
    .run()

  return {
    success: true,
    data: null
  }
})
