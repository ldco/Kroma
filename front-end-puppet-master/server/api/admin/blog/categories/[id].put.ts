/**
 * PUT /api/admin/blog/categories/:id
 *
 * Update a blog category.
 * Content stored in centralized translations table with key pattern: blog.category.{id}.{field}
 */
import { eq, and, ne } from 'drizzle-orm'
import { z } from 'zod'
import { useDatabase, schema } from '../../../../database/client'
import { escapeHtml } from '../../../../utils/sanitize'
import config from '~~/app/puppet-master.config'

const updateCategorySchema = z.object({
  slug: z.string().min(1).max(100).regex(/^[a-z0-9-]+$/).optional(),
  order: z.number().int().min(0).optional(),
  name: z.string().min(1).max(100).optional(),
  description: z.string().max(500).optional().nullable(),
  translations: z.record(z.object({
    name: z.string().min(1).max(100),
    description: z.string().max(500).optional().nullable()
  })).optional()
})

export default defineEventHandler(async event => {
  const db = useDatabase()
  const id = parseInt(event.context.params?.id || '')

  if (isNaN(id)) {
    throw createError({ statusCode: 400, message: 'Invalid ID' })
  }

  const body = await readBody(event)
  const result = updateCategorySchema.safeParse(body)

  if (!result.success) {
    throw createError({
      statusCode: 400,
      message: 'Validation failed',
      data: result.error.flatten()
    })
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

  const { translations, name, description, ...data } = result.data

  // Check slug uniqueness (excluding current)
  if (data.slug && data.slug !== existing.slug) {
    const slugExists = db
      .select()
      .from(schema.blogCategories)
      .where(and(eq(schema.blogCategories.slug, data.slug), ne(schema.blogCategories.id, id)))
      .get()

    if (slugExists) {
      throw createError({ statusCode: 409, message: 'Slug already exists' })
    }
  }

  // Build update values
  const updateValues: Record<string, unknown> = { updatedAt: new Date() }
  if (data.slug !== undefined) updateValues.slug = data.slug
  if (data.order !== undefined) updateValues.order = data.order

  // Update category
  db.update(schema.blogCategories)
    .set(updateValues)
    .where(eq(schema.blogCategories.id, id))
    .run()

  // Handle simple name/description update for default locale
  const defaultLocale = config.defaultLocale || 'en'

  if (name !== undefined) {
    const nameKey = `blog.category.${id}.name`
    const existingName = db
      .select()
      .from(schema.translations)
      .where(and(eq(schema.translations.key, nameKey), eq(schema.translations.locale, defaultLocale)))
      .get()

    if (existingName) {
      db.update(schema.translations)
        .set({ value: escapeHtml(name), updatedAt: new Date() })
        .where(eq(schema.translations.id, existingName.id))
        .run()
    } else {
      db.insert(schema.translations)
        .values({ locale: defaultLocale, key: nameKey, value: escapeHtml(name) })
        .run()
    }
  }

  if (description !== undefined) {
    const descKey = `blog.category.${id}.description`
    const existingDesc = db
      .select()
      .from(schema.translations)
      .where(and(eq(schema.translations.key, descKey), eq(schema.translations.locale, defaultLocale)))
      .get()

    if (existingDesc) {
      db.update(schema.translations)
        .set({ value: description ? escapeHtml(description) : '', updatedAt: new Date() })
        .where(eq(schema.translations.id, existingDesc.id))
        .run()
    } else if (description) {
      db.insert(schema.translations)
        .values({ locale: defaultLocale, key: descKey, value: escapeHtml(description) })
        .run()
    }
  }

  // Handle multi-locale translations
  if (translations) {
    for (const [locale, trans] of Object.entries(translations)) {
      const nameKey = `blog.category.${id}.name`
      const descKey = `blog.category.${id}.description`

      // Upsert name
      const existingName = db
        .select()
        .from(schema.translations)
        .where(and(eq(schema.translations.key, nameKey), eq(schema.translations.locale, locale)))
        .get()

      if (existingName) {
        db.update(schema.translations)
          .set({ value: escapeHtml(trans.name), updatedAt: new Date() })
          .where(eq(schema.translations.id, existingName.id))
          .run()
      } else {
        db.insert(schema.translations)
          .values({ locale, key: nameKey, value: escapeHtml(trans.name) })
          .run()
      }

      // Upsert description
      const existingDesc = db
        .select()
        .from(schema.translations)
        .where(and(eq(schema.translations.key, descKey), eq(schema.translations.locale, locale)))
        .get()

      if (existingDesc) {
        db.update(schema.translations)
          .set({ value: trans.description ? escapeHtml(trans.description) : '', updatedAt: new Date() })
          .where(eq(schema.translations.id, existingDesc.id))
          .run()
      } else if (trans.description) {
        db.insert(schema.translations)
          .values({ locale, key: descKey, value: escapeHtml(trans.description) })
          .run()
      }
    }
  }

  return {
    success: true,
    data: null
  }
})
