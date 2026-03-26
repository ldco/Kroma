/**
 * DELETE /api/admin/blog/posts/:id
 *
 * Delete a blog post and all related data.
 * Removes translations from centralized table with key pattern: blog.post.{id}.%
 */
import { eq, like } from 'drizzle-orm'
import { useDatabase, schema } from '../../../../database/client'

export default defineEventHandler(async event => {
  const db = useDatabase()
  const id = parseInt(event.context.params?.id || '')

  if (isNaN(id)) {
    throw createError({ statusCode: 400, message: 'Invalid ID' })
  }

  // Check if post exists
  const existing = db
    .select()
    .from(schema.blogPosts)
    .where(eq(schema.blogPosts.id, id))
    .get()

  if (!existing) {
    throw createError({ statusCode: 404, message: 'Post not found' })
  }

  // Delete translations from centralized table
  db.delete(schema.translations)
    .where(like(schema.translations.key, `blog.post.${id}.%`))
    .run()

  // Delete post tags
  db.delete(schema.blogPostTags)
    .where(eq(schema.blogPostTags.postId, id))
    .run()

  // Delete post media
  db.delete(schema.blogMedia)
    .where(eq(schema.blogMedia.postId, id))
    .run()

  // Delete post
  db.delete(schema.blogPosts)
    .where(eq(schema.blogPosts.id, id))
    .run()

  return {
    success: true,
    data: null
  }
})
