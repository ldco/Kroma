/**
 * POST /api/admin/blog/tags
 *
 * Create a new blog tag.
 */
import { eq } from 'drizzle-orm'
import { z } from 'zod'
import { useDatabase, schema } from '../../../database/client'
import { escapeHtml } from '../../../utils/sanitize'

const createTagSchema = z.object({
  slug: z.string().min(1).max(50).regex(/^[a-z0-9-]+$/),
  name: z.string().min(1).max(50)
})

export default defineEventHandler(async event => {
  const db = useDatabase()
  const body = await readBody(event)

  const result = createTagSchema.safeParse(body)
  if (!result.success) {
    throw createError({
      statusCode: 400,
      message: 'Validation failed',
      data: result.error.flatten()
    })
  }

  const data = result.data

  // Check if slug exists
  const existing = db
    .select()
    .from(schema.blogTags)
    .where(eq(schema.blogTags.slug, data.slug))
    .get()

  if (existing) {
    throw createError({ statusCode: 409, message: 'Slug already exists' })
  }

  // Insert tag
  const [tag] = db
    .insert(schema.blogTags)
    .values({
      slug: data.slug,
      name: escapeHtml(data.name)
    })
    .returning()

  if (!tag) {
    throw createError({ statusCode: 500, message: 'Failed to create tag' })
  }

  return {
    success: true,
    data: {
      tag
    },
    tag
  }
})
