/**
 * POST /api/admin/features
 *
 * Create a new feature.
 * Content stored in centralized translations table.
 * Supports multiple locales in one request.
 */
import { eq } from 'drizzle-orm'
import { z } from 'zod'
import { useDatabase, schema } from '../../../database/client'

const createFeatureSchema = z.object({
  slug: z.string().min(1).max(100).regex(/^[a-z0-9-]+$/),
  imageUrl: z.string().max(500).nullish(),
  hoverImageUrl: z.string().max(500).nullish(),
  order: z.number().int().min(0).default(0),
  published: z.boolean().default(true),
  translations: z.record(z.object({
    title: z.string().max(200).nullable().optional(),
    description: z.string().max(1000).nullable().optional()
  })).optional()
})

export default defineEventHandler(async event => {
  const db = useDatabase()
  const body = await readBody(event)

  const result = createFeatureSchema.safeParse(body)
  if (!result.success) {
    throw createError({
      statusCode: 400,
      message: 'Validation failed',
      data: result.error.flatten()
    })
  }

  const { translations, ...data } = result.data

  // Check slug uniqueness
  const existing = db
    .select()
    .from(schema.features)
    .where(eq(schema.features.slug, data.slug))
    .get()

  if (existing) {
    throw createError({ statusCode: 409, message: 'Slug already exists' })
  }

  // Insert feature metadata only
  const insertResult = db
    .insert(schema.features)
    .values({
      slug: data.slug,
      imageUrl: data.imageUrl || null,
      hoverImageUrl: data.hoverImageUrl || null,
      order: data.order,
      published: data.published
    })
    .run()

  const featureId = Number(insertResult.lastInsertRowid)

  // Store translations in centralized table
  if (translations) {
    const translationValues = []

    for (const [locale, trans] of Object.entries(translations)) {
      if (trans.title) {
        translationValues.push({
          locale,
          key: `feature.${featureId}.title`,
          value: trans.title
        })
      }

      if (trans.description) {
        translationValues.push({
          locale,
          key: `feature.${featureId}.description`,
          value: trans.description
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
      id: featureId
    }
  }
})
