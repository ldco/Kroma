/**
 * PUT /api/admin/blog/tags/:id
 *
 * Update a blog tag.
 */
import { eq, and, ne } from 'drizzle-orm'
import { useDatabase, schema } from '../../../../database/client'

export default defineEventHandler(async event => {
  const db = useDatabase()
  const id = parseInt(event.context.params?.id || '')

  if (isNaN(id)) {
    throw createError({ statusCode: 400, message: 'Invalid ID' })
  }

  const body = await readBody(event)
  const { slug, name } = body

  if (!slug || !name) {
    throw createError({ statusCode: 400, message: 'Slug and name are required' })
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

  // Check slug uniqueness (excluding current)
  const slugExists = db
    .select()
    .from(schema.blogTags)
    .where(
      and(
        eq(schema.blogTags.slug, slug),
        ne(schema.blogTags.id, id)
      )
    )
    .get()

  if (slugExists) {
    throw createError({ statusCode: 409, message: 'Slug already exists' })
  }

  // Update tag
  db.update(schema.blogTags)
    .set({
      slug,
      name,
      updatedAt: new Date().toISOString()
    })
    .where(eq(schema.blogTags.id, id))
    .run()

  return {
    success: true,
    data: null
  }
})
