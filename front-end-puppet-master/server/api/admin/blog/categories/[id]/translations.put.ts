/**
 * PUT /api/admin/blog/categories/:id/translations
 *
 * Update translations for a blog category.
 * Uses centralized translations table with key pattern: blog.category.{id}.{field}
 */
import { eq, and } from 'drizzle-orm'
import { z } from 'zod'
import { useDatabase, schema } from '../../../../../database/client'
import { escapeHtml } from '../../../../../utils/sanitize'

const updateTranslationSchema = z.object({
  locale: z.string().min(2).max(5),
  name: z.string().min(1).max(100).optional(),
  description: z.string().max(500).optional().nullable()
})

export default defineEventHandler(async event => {
  const db = useDatabase()
  const id = parseInt(event.context.params?.id || '')

  if (isNaN(id)) {
    throw createError({ statusCode: 400, message: 'Invalid ID' })
  }

  const body = await readBody(event)
  const result = updateTranslationSchema.safeParse(body)

  if (!result.success) {
    throw createError({
      statusCode: 400,
      message: 'Validation failed',
      data: result.error.flatten()
    })
  }

  const { locale, name, description } = result.data

  // Check if category exists
  const existing = db
    .select()
    .from(schema.blogCategories)
    .where(eq(schema.blogCategories.id, id))
    .get()

  if (!existing) {
    throw createError({ statusCode: 404, message: 'Category not found' })
  }

  // Upsert name
  if (name !== undefined) {
    const nameKey = `blog.category.${id}.name`
    const existingName = db
      .select()
      .from(schema.translations)
      .where(and(eq(schema.translations.key, nameKey), eq(schema.translations.locale, locale)))
      .get()

    if (existingName) {
      db.update(schema.translations)
        .set({ value: escapeHtml(name), updatedAt: new Date() })
        .where(eq(schema.translations.id, existingName.id))
        .run()
    } else {
      db.insert(schema.translations)
        .values({ locale, key: nameKey, value: escapeHtml(name) })
        .run()
    }
  }

  // Upsert description
  if (description !== undefined) {
    const descKey = `blog.category.${id}.description`
    const existingDesc = db
      .select()
      .from(schema.translations)
      .where(and(eq(schema.translations.key, descKey), eq(schema.translations.locale, locale)))
      .get()

    if (existingDesc) {
      db.update(schema.translations)
        .set({ value: description ? escapeHtml(description) : '', updatedAt: new Date() })
        .where(eq(schema.translations.id, existingDesc.id))
        .run()
    } else if (description) {
      db.insert(schema.translations)
        .values({ locale, key: descKey, value: escapeHtml(description) })
        .run()
    }
  }

  return {
    success: true,
    data: null
  }
})
