/**
 * POST /api/admin/testimonials
 *
 * Create a new testimonial.
 * Quote and authorTitle stored in centralized translations table.
 * Supports multiple locales in one request.
 */
import { z } from 'zod'
import { useDatabase, schema } from '../../../database/client'

const createTestimonialSchema = z.object({
  authorName: z.string().min(1).max(100),
  authorPhoto: z.string().max(500).nullish(),
  rating: z.number().int().min(1).max(5).default(5),
  order: z.number().int().min(0).default(0),
  published: z.boolean().default(true),
  translations: z.record(z.object({
    quote: z.string().max(2000),
    authorTitle: z.string().max(200)
  })).optional()
})

export default defineEventHandler(async event => {
  const db = useDatabase()
  const body = await readBody(event)

  const result = createTestimonialSchema.safeParse(body)
  if (!result.success) {
    throw createError({
      statusCode: 400,
      message: 'Validation failed',
      data: result.error.flatten()
    })
  }

  const { translations, ...data } = result.data

  // Insert testimonial metadata only
  const insertResult = db
    .insert(schema.testimonials)
    .values({
      authorName: data.authorName,
      authorTitle: null, // No longer stored in main table
      authorPhotoUrl: data.authorPhoto || null,
      rating: data.rating,
      order: data.order,
      published: data.published
    })
    .run()

  const testimonialId = Number(insertResult.lastInsertRowid)

  // Store translations in centralized table
  if (translations) {
    const translationValues = []

    for (const [locale, trans] of Object.entries(translations)) {
      if (trans.quote) {
        translationValues.push({
          locale,
          key: `testimonial.${testimonialId}.quote`,
          value: trans.quote
        })
      }

      if (trans.authorTitle) {
        translationValues.push({
          locale,
          key: `testimonial.${testimonialId}.authorTitle`,
          value: trans.authorTitle
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
      id: testimonialId
    }
  }
})
