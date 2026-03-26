/**
 * PUT /api/admin/features/:id
 *
 * Update a feature.
 * Content stored in centralized translations table.
 * Supports multiple locales in one request.
 */
import { eq, and, ne } from 'drizzle-orm'
import { z } from 'zod'
import { useDatabase, schema } from '../../../database/client'

const updateFeatureSchema = z.object({
  slug: z.string().min(1).max(100).regex(/^[a-z0-9-]+$/).optional(),
  imageUrl: z.string().max(500).nullish(),
  hoverImageUrl: z.string().max(500).nullish(),
  order: z.number().int().min(0).optional(),
  published: z.boolean().optional(),
  translations: z.record(z.object({
    title: z.string().max(200).nullable().optional(),
    description: z.string().max(1000).nullable().optional()
  })).optional()
})

export default defineEventHandler(async event => {
  const db = useDatabase()
  const id = parseInt(event.context.params?.id || '')

  if (isNaN(id)) {
    throw createError({ statusCode: 400, message: 'Invalid ID' })
  }

  const body = await readBody(event)

  const result = updateFeatureSchema.safeParse(body)
  if (!result.success) {
    throw createError({
      statusCode: 400,
      message: 'Validation failed',
      data: result.error.flatten()
    })
  }

  // Check if feature exists
  const existing = db
    .select()
    .from(schema.features)
    .where(eq(schema.features.id, id))
    .get()

  if (!existing) {
    throw createError({ statusCode: 404, message: 'Feature not found' })
  }

  const { translations, ...data } = result.data

  // Check slug uniqueness (excluding current)
  if (data.slug && data.slug !== existing.slug) {
    const slugExists = db
      .select()
      .from(schema.features)
      .where(and(eq(schema.features.slug, data.slug), ne(schema.features.id, id)))
      .get()

    if (slugExists) {
      throw createError({ statusCode: 409, message: 'Slug already exists' })
    }
  }

  // Build update values
  const updateValues: Record<string, unknown> = {}

  if (data.slug !== undefined) updateValues.slug = data.slug
  if (data.imageUrl !== undefined) updateValues.imageUrl = data.imageUrl
  if (data.hoverImageUrl !== undefined) updateValues.hoverImageUrl = data.hoverImageUrl
  if (data.order !== undefined) updateValues.order = data.order
  if (data.published !== undefined) updateValues.published = data.published

  if (Object.keys(updateValues).length > 0) {
    db.update(schema.features)
      .set(updateValues)
      .where(eq(schema.features.id, id))
      .run()
  }

  // Update translations in centralized table
  if (translations) {
    for (const [locale, trans] of Object.entries(translations)) {
      const titleKey = `feature.${id}.title`
      const descKey = `feature.${id}.description`

      // Upsert title
      const existingTitle = db
        .select()
        .from(schema.translations)
        .where(and(eq(schema.translations.key, titleKey), eq(schema.translations.locale, locale)))
        .get()

      if (existingTitle) {
        db.update(schema.translations)
          .set({ value: trans.title || '', updatedAt: new Date() })
          .where(eq(schema.translations.id, existingTitle.id))
          .run()
      } else if (trans.title) {
        db.insert(schema.translations)
          .values({ locale, key: titleKey, value: trans.title })
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
          .set({ value: trans.description || '', updatedAt: new Date() })
          .where(eq(schema.translations.id, existingDesc.id))
          .run()
      } else if (trans.description) {
        db.insert(schema.translations)
          .values({ locale, key: descKey, value: trans.description })
          .run()
      }
    }
  }

  return {
    success: true,
    data: {}
  }
})
