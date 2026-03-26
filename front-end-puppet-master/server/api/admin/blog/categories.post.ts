/**
 * POST /api/admin/blog/categories
 *
 * Create a new blog category.
 * Content stored in centralized translations table with key pattern: blog.category.{id}.{field}
 */
import { eq } from 'drizzle-orm'
import { z } from 'zod'
import { useDatabase, schema } from '../../../database/client'
import { escapeHtml } from '../../../utils/sanitize'

const createCategorySchema = z.object({
  slug: z.string().min(1).max(100).regex(/^[a-z0-9-]+$/),
  order: z.number().int().min(0).default(0),
  translations: z.record(z.object({
    name: z.string().min(1).max(100),
    description: z.string().max(500).optional().nullable()
  })).optional()
})

export default defineEventHandler(async event => {
  const db = useDatabase()
  const body = await readBody(event)

  const result = createCategorySchema.safeParse(body)
  if (!result.success) {
    throw createError({
      statusCode: 400,
      message: 'Validation failed',
      data: result.error.flatten()
    })
  }

  const { translations, ...data } = result.data

  // Check if slug exists
  const existing = db
    .select()
    .from(schema.blogCategories)
    .where(eq(schema.blogCategories.slug, data.slug))
    .get()

  if (existing) {
    throw createError({ statusCode: 409, message: 'Slug already exists' })
  }

  // Insert category
  const [category] = db
    .insert(schema.blogCategories)
    .values({ slug: data.slug, order: data.order })
    .returning()

  if (!category) {
    throw createError({ statusCode: 500, message: 'Failed to create category' })
  }

  // Insert translations into centralized table
  if (translations) {
    const translationValues = []

    for (const [locale, trans] of Object.entries(translations)) {
      translationValues.push({
        locale,
        key: `blog.category.${category.id}.name`,
        value: escapeHtml(trans.name)
      })

      if (trans.description) {
        translationValues.push({
          locale,
          key: `blog.category.${category.id}.description`,
          value: escapeHtml(trans.description)
        })
      }
    }

    if (translationValues.length > 0) {
      db.insert(schema.translations)
        .values(translationValues)
        .run()
    }
  }

  return {
    success: true,
    data: {
      category
    },
    category
  }
})
